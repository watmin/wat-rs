# BRIEF — envelope step 3a: every runtime error kind is a declared wat record

Excursus 003. Design: `DESIGN-the-error-envelope-and-its-frames.md` (D2), and its § RULING
2026-09-26 (option A). Builder, verbatim: *"Define the new records in wat code and use the existing
tooling to macro rust code from them."*

**Read `wat-rs/CLAUDE.md` in full before anything else.** Nothing else carries its floor and codemod
doctrine to you.

## Why

D2 says the envelope carries structure (`Failure.error <- :wat::core::Error`) and never a string. Step
3b cannot do that yet, because a runtime error has **no wat value to be**:

- `RuntimeErrorKind` (`src/value/signal.rs:376`) has 31 variants.
- Each is written as `#wat.runtime/<Kind> {…}` by a Rust `#[derive(ToEdn)]`.
- Not one of those tags names a declared type. Only three `:wat::runtime::*` records exist
  (`TypeField`, `TypeVariant`, `TypeInfo`, in `wat/runtime-typeinfo.wat`).

The wire already carries the `:wat::core::Error` floor (`:message :location :causes`) plus the
kind-specific fields. See `tests/types/probe_arc234_stone3b_record_assoc__probe4_type_mismatch.edn`.
The shape exists; only its declaration is missing, and so is the value that 3b needs to hold.

This step supplies that value, and ships **no envelope change**. Step 3b makes `LociDiedError` carry
it.

## What to build

1. **Declare one `defrecord :wat::runtime::<Kind>` per `RuntimeErrorKind` variant** in a new stdlib
   file, `wat/runtime-errors.wat`. Load it wherever the stdlib load order requires; follow how
   `wat/kernel/diagnostics.wat` is wired in `src/load/stdlib.rs:47`.
   - Each record opens with the floor, `[message <- :wat::core::String location <- :wat::core::Span
     causes <- (:wat::core::Vector :- [:wat::core::Error])]`, so it satisfies the `:wat::core::Error`
     surface. `wat/core.wat:2182`/`Fault` is the model.
   - The floor is followed by the kind's own fields, named exactly as today's EDN keys (kebab, and
     honouring `#[to_edn(key = …)]`).
   - Every record and every field gets a `;;` doc line. Say what it means, not what it repeats.
   - Type mapping: `String`/`&'static str` → `String`; `usize`/`i64` → `i64`; `Option<String>` →
     `(Option :- [String])`; `Vec<String>` → `(Vector :- [String])`; `crate::span::Span` →
     `:wat::core::Span`.

2. **Nested payloads.** Measure each one, then apply the rule that fits it:
   - `ValueSnapshot` (`got` on `NotCallable`, `TypeMismatch`, `BadCondition`) is written today as an
     **untagged map** `{:type :rendered :provenance}`. That is a record-shaped value without a tag,
     which the builder has ruled out ("record-shaped values are always tagged").
     - Declare `:wat::runtime::ValueSnapshot`. Its `provenance` field is an Option over a declared
       `defenum` of `Provenance` (`src/edn/error.rs:141`: `Literal`, `SymbolBound`, `RuntimeBuilt`,
       with `Unknown` as `None`).
     - The payloads are strings and Spans, so this is a declaration, not a design.
   - **A nested ERROR** (`EvalVerificationFailed.err: HashError`, `MacroExpansionFailed.cause:
     MacroError`) is a cause. It goes in `causes`, not in a kind field.
     - Each such cause becomes a value at the `:wat::core::Error` floor (a `:wat::core::Fault` built
       from its message, location and causes).
     - Its kind-specific shape is **out of scope**. Name that loss in your report; do not paper over
       it.
     - If a record would then have no kind field left, it keeps only the floor. That is fine.
   - `ReteCeiling` is data, not an error. Declare a `defenum` for it, provided its payloads are
     scalar. If they are not, STOP and report what you measured.

3. **Derive the Rust side from the `.wat`, with the existing tooling.** No human types a record field
   name into Rust.
   - Registration: one `::wat_source_derive::wat_record_from!(env, "wat/runtime-errors.wat",
     ":wat::runtime::<Kind>")` per record, beside `src/types.rs:2289` (`Span`). For the enums, use
     `wat_enum_register_from!`.
   - Field names: `wat_field_names_from!` consts, the pattern at `src/freeze.rs:143`.
   - Conversion: add a new `RuntimeError::to_record(&self) -> Value` that builds the record's struct
     Value.
     - It is an **exhaustive `match` with no `_` arm**, so a new Rust variant without a record fails
       to compile.
     - It fills fields through the generated name consts.
     - `message` is today's `Display` text, `location` is the error's span, and `causes` is per
       rule 2.
   - If the existing macros truly cannot express something, extend `crates/wat-source-derive`. Do
     not add a hand-written second copy. Say which macro and why.

4. **Keep the Rust `ToEdn` derive on `RuntimeErrorKind`, unchanged.** It still writes the wire until
   3b. Do not touch `LociDiedError`, `src/process/died.rs`, or any golden.

## Open question: measure it, don't guess it

`AssertionFailed { message, actual, expected }` has a kind field called `message`, which is also the
floor's name. What does today's EDN emit: one `:message` or two? Report what you find.
- If the two coincide (the kind's message *is* the floor message), the record declares `message`
  once.
- If they differ, STOP and report. Do not invent a rename.

## Gates to add (each must be mutation-proven: break it, see RED, restore)

- **G1, the declaration is the list.** Add a test that builds one instance of every variant and calls
  `to_record`.
  - The set of produced record type names must **equal** the set of `defrecord :wat::runtime::*`
    names that the test reads from `wat/runtime-errors.wat`, excluding `ValueSnapshot` and the
    enums.
  - Mutation: add a stray defrecord to the file. G1 must go RED.
  - Removing a record, by contrast, fails the *build* (the name const disappears). Say so; that
    removal is not a mutation G1 must catch.
- **G2, the record agrees with today's wire.** For every variant, compare `to_record` rendered to EDN
  with the existing `ToEdn` output, after removing `:frames`/`:frames-elided`.
  - They must be equal, with exactly two named exceptions: `:got` is now tagged
    `#wat.runtime/ValueSnapshot`, and nested errors moved to `:causes`.
  - The comparison is structural (parsed EDN), not a comparison of text.
  - Mutation: swap two fields' values in one arm of `to_record`. G2 must go RED.
- **G3, round trip.** For every variant, reading the record's EDN (`:wat::edn::read`, or its Rust
  equivalent) returns a Value equal to the one written.
  - This proves the tags resolve to registered types.
  - Mutation: remove one `wat_record_from!` call. G3 must go RED for that kind. If the build fails
    instead, report that and say what proved the tag lookup.

## Scope fence

- **IN:** `wat/runtime-errors.wat`, its stdlib load entry, `src/types.rs` registration,
  `RuntimeError::to_record`, the gates, and a derive-crate extension if one is required.
- **OUT:**
  - `LociDiedError` and every other envelope (that is 3b).
  - `runtime_error_to_eval_error_value` (`src/runtime.rs:12281`).
  - All goldens.
  - The `HashError`/`MacroError` kind records.

## Discipline

- Scratch `.wat` goes in `wat-scripts/scratch-pad/`, which is gated. Deliberately-failing
  measurement `.wat` goes in the session scratchpad.
- **One cargo process at a time.** Check with `pgrep -x cargo` before you start one, and never use
  `pgrep -f`.
- Use `cargo nextest run --release`, never `cargo test`. Focused runs too.
- The floor:
  - Run `scripts/floor.sh` in the **foreground**, or via `nohup scripts/floor.sh > <file> 2>&1 &`
    with a foreground `until grep -qE '^ *Summary' <file>; do sleep 30; done`.
  - Read the `Summary` line of `.floor/latest/clean.log`.
  - The floor is **0 failed**. On any red: **do not re-run**. Copy the failing block verbatim, name
    the arm, and report.
- Stage explicit paths; never `git add -A`. `git add` before running any `git ls-files`-based gate.
- No `cargo fmt`.
- Commit prefix `EXCURSUS(003):`, ending with
  `Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>`.
- Push to `origin/reason/little-wat-findings`.

## Report

For each of the following, give the command you ran and the output:
- the variant count;
- the record count;
- the answer to the `AssertionFailed` question;
- the nested-payload decision for each variant that has one;
- each gate and the RED it produced under mutation;
- the floor `Summary` line;
- the commit SHA.
