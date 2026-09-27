# BRIEF — envelope step 3c: every error has a location, and a runtime error has one writer

Excursus 003. Design: `DESIGN-the-error-envelope-and-its-frames.md` (D1, D2). This builds on step 3b
(`65272de6f`).

**Read `wat-rs/CLAUDE.md` in full first.** It is the only place the floor and codemod doctrine reach
you.

**Every count and citation below is a claim.** Two of this campaign's briefs had wrong counts, and a
grep that "found nothing" hid 15 files. Verify before relying on anything, and STOP on a
contradiction. Where the brief's intent is clear and the call is small, make it and name it.

This strike has **two parts. Commit each separately, each on a green floor**, Part A first.

---

## Part A — no error has a `nil` location, and the type system enforces it

### Why

The `:wat::core::Error` surface (`wat/core.wat`, the `Error` declaration near `Fault`) declares
`location <- :wat::core::Span`. It is mandatory. But `WatError::location()` returns an `OwnedValue`,
so an impl can return `nil`, and seven sites do. Each was measured by scanning every
`fn location(&self) -> OwnedValue` body for `Nil`:

| site | what |
|---|---|
| `src/check/error_edn.rs:93` | `CheckErrors`, an aggregate |
| `src/rete/validate/error.rs:665` | `ReteCheckErrors`, an aggregate |
| `src/resolve/error.rs:64` | `ResolveError`, including the aggregate `UnresolvedReferences` |
| `src/macros/error_edn.rs:177,179,180` | `StartupError`'s `Validator`, `SigmaFn`, `MainSignature` arms |
| `src/edn/contract.rs:385` | `FlatMessage` |

**155 tracked `.edn` goldens** carry `:location nil`
(`git ls-files '*.edn' | xargs grep -l ":location nil" | wc -l`). Most are
`#wat.check/CheckErrors` (about 128), `#wat.rete/ReteCheckErrors` (about 20) and
`#wat.resolve/UnresolvedReferences` (about 7). I took those tallies with a `grep -B2` window, so they
include some context noise; re-derive them.

A second defect sits in the same aggregates. Each one holds its errors in a bespoke field
(`:errors`, or `:unresolved`) and reports **`:causes []`**. The floor's own slot for "the errors that
caused this one" is `causes`, and it is empty while the causes sit one key over.

### What to build

1. **Make `nil` unrepresentable.** Change `WatError::location()` (`src/edn/contract.rs`) to return a
   `crate::span::Span`, not an `OwnedValue`. `error_edn()` renders it.
   - There are 14 `impl … WatError for` sites (`grep -rn "impl.*WatError for" src crates`). Each one
     now has to produce a real `Span` or fail to compile.
   - This is the cure. The lint below is only the witness.
2. **Aggregates: the items become `causes`, and `location` is the first cause's.**
   - `CheckErrors`, `ReteCheckErrors` and `UnresolvedReferences` put their items in `causes`, and
     drop the bespoke key. Each item already satisfies the floor (it is a `WatError`), except
     `UnresolvedReference`: that one is a sub-value with `path`/`context`/`span`, and it gains the
     floor.
   - `location` is **the first item's location**. State that in each impl's doc: the aggregate
     happened wherever its first error is. An aggregate with zero items should not exist; measure
     whether one can be constructed. If one can, STOP and report.
   - The message stays the count headline ("N type-check errors").
   - Consumers of the bespoke key:
     - Rust: 6 sites (`grep -rnE 'kw\("errors"\)|"errors"' tests src`). Includes
       `tests/rete/probe_freeze_validator_lift_rete_namespace.rs:69` and
       `src/rete/validate/mod.rs:1527`. Update them.
     - wat: my grep found **0**. A grep that finds nothing proves nothing. The floor and the
       type-checker are the real answer.
3. **Flat messages take the Rust site.**
   - `FlatMessage` gets `#[track_caller]` capture, so its location is where Rust raised it: a `Span`
     with `end` `None`. This is D1/D3: Rust knows where a failure starts, not where it ends.
   - `SigmaFn` and `MainSignature` are the same. For `MainSignature`, if the program's own file is
     in hand at the raise site, a `Span` at that file is more useful than the Rust site. Measure
     whether it is in hand, and use it if so. Name your choice either way.
4. **The validator message (`src/macros/error_edn.rs:157`,
   `SE::Validator(e) => first_line(e.to_string())`).**
   - Widen `FreezeValidatorError` (`src/freeze/validator.rs:29`) to require `WatError`.
   - `StartupError`'s `Validator` arm then delegates `message`/`location`/`causes`/`variant` to the
     inner error, exactly like every other arm, and the `first_line(to_string())` goes.
   - There is exactly one validator in the tree (the rete `defrule` wall, measured in 3b). Confirm
     that.

### Gate A — no `nil` location in any golden (mutation-proven in RELEASE)

- A lint test parses every tracked `.edn` golden. It fails if any map that carries `:message`,
  `:location` and `:causes` (the floor) has a `:location` that is not a `#wat.core/Span`. Parse the
  EDN; do not grep it. Model it on 3b's `tests/lint/no_double_quoted_edn_in_golden_files.rs`.
- **Anchor:** it must be RED on `65272de6f`'s goldens. Report the file count and compare it with
  155.
- **Mutation:** once the signature returns `Span`, there is no way to write `nil` through the trait,
  so the lint's live mutation is a golden. Hand-edit one recaptured golden's location to `nil` and
  confirm RED, then restore it.
- **Also report** that re-adding `OwnedValue::Nil` to a `location()` body fails to COMPILE. That is
  the type-level proof. Paste the compiler error.

### Goldens

Recapture with `UPDATE_EDN=1` (`src/lib.rs:431`). **Read every diff.** Report any golden whose change
is not one of these three:
- `nil` → Span;
- `:errors`/`:unresolved` → `:causes`;
- the validator message.

---

## Part B — a runtime error has one EDN writer

### Why

`RuntimeError` renders through two paths:
1. **`WatError::error_edn()`**: the floor form, the one on the process wire. Step 3a's G2 proves it
   equals `RuntimeError::to_record()` rendered.
2. **`impl ToEdn for RuntimeError`** (`src/edn/error.rs:64`): the kind's `#[derive(ToEdn)]` plus
   `:span`, `:frames` and `:frames-elided`. That makes `:span` a second name for the location, and
   there is no floor. **46 goldens** have a top-level `#wat.runtime/` tag with a `:span` key.

Also, `emit_runtime_error_envelope` (`src/edn/error.rs:45`) has **zero callers**. It is dead.

### What to build

1. **Measure first:** list every PRODUCTION caller (not tests) of `RuntimeError`'s `ToEdn::to_edn()`,
   of `Debug`/`Display` where they render EDN, and of the kind derive. Say which user-visible output
   each one feeds: the REPL, CLI stderr, a golden harness, or `variant()` internally.
2. **The end state is one writer.** A `RuntimeError` renders as its record, meaning `to_record()`
   through the generic value writer, which by 3a G2 is `error_edn()`. So:
   - `:location` replaces `:span`;
   - `:frames`/`:frames-elided` leave the standalone error, because they live on `Failure` (3b).
   - If a production path shows a bare `RuntimeError` to a user and loses frames by this, STOP and
     report that path. Do not decide it yourself.
3. **Retire what nothing needs.**
   - Delete `emit_runtime_error_envelope`.
   - If `error_edn()`'s `variant()` can be sourced from `to_record()` instead of the kind derive,
     retire the `#[derive(ToEdn)]` on `RuntimeErrorKind` (and on `ReteCeiling`, if it has no other
     user).
   - If something still needs the derive, name it and keep it. Do not force the deletion.
4. **Recapture** the 46 goldens, and read every diff.

### Gate B (mutation-proven in RELEASE)

- A test proving that, for every `RuntimeErrorKind` variant (reuse 3a's `all_variants`), the ONE
  writer's output equals `error_edn()`. If you retired a writer, this may reduce to "only one
  exists". In that case, say what now makes a second writer impossible, or unlikely, and whether a
  lint is warranted.
- Mutate one field in the writer and confirm RED.

---

## Scope fence

- **IN:** Parts A and B as written.
- **OUT:**
  - D4, the derived primary location (step 4).
  - Retiring the dead `EntryFormFailure`/`BadReturn` variants (the builder's call).
  - Tail-call frame elision.
  - `HashError`/`MacroError` kind records.

## Discipline

- **One cargo process at a time.** Check with `pgrep -x cargo`, never `pgrep -f`.
- Use `cargo nextest run --release`, never `cargo test`.
- The floor:
  - Run it in the foreground, or `nohup` plus a foreground
    `until grep -qE '^ *Summary' <file>; do sleep 30; done`.
  - Never end your turn while a build or floor runs.
  - The floor is **0 failed**. On a red: **do not re-run**; capture it verbatim and name the arm.
- Stage every path by name before the floor. Never use `git add -A`.
- No `cargo fmt`.
- `.wat` multi-site edits go through the wat-fix codemod. Replay fixtures are required.
- A new test carrying wat-looking strings needs its rune.
- Commit prefix `EXCURSUS(003):`, ending with
  `Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>`.
- Push to `origin/reason/little-wat-findings`.
- **If your budget runs short, stop at a clean boundary.** Part A committed on a green floor is one;
  report exactly where you stopped.

## Report

- Parts A and B:
  - the re-measured counts;
  - each gate's anchor and mutation RED;
  - the compile error proving `nil` is unrepresentable;
  - any golden that changed beyond the named shapes;
  - the floor `Summary` lines, verbatim;
  - the SHAs.
- Part B: the production-caller list.
