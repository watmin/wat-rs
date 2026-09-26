# BRIEF — envelope step 3b: every failure variant of `LociDiedError` carries a `Failure`

Excursus 003. Design: `DESIGN-the-error-envelope-and-its-frames.md`, § D2 (RULED). This builds on
step 3a (`694200942`), where every runtime error kind became a declared record reachable through
`RuntimeError::to_record`.

**Read `wat-rs/CLAUDE.md` in full before anything else.** Nothing else carries its floor and codemod
doctrine to you.

**Every count and citation below is a claim.** The step 3a brief was wrong three times: a variant
count, a citation, and a rule the tree contradicted. Verify each item you rely on before you build
on it. On a contradiction, STOP and report what you measured.

## Why: the defect, as it stands today

`:wat::kernel::LociDiedError` (`wat/kernel/diagnostics.wat`, the `defenum` after `Failure`) is the
report of how a peer died, and it crosses process boundaries as EDN. Its failure variants carry a
bare `message <- String`: `Panic` (plus `failure <- Option<Failure>`), `RuntimeError`,
`EntryFormFailure`, `MainSignature` and `BadReturn`. The Rust constructors put
`to_wire_edn(e)`, **the error's serialized EDN**, into that string (`src/process/died.rs:107,129,150`;
`src/kernel/error.rs:261`). Written out again, the string arrives double-quoted.

Measured on the goldens (`git ls-files '*.edn' | xargs grep -lE '"#wat\.'`): **21 files** hold a
string whose content is tagged EDN.
- **1 file** comes from that mechanism: `tests/diagnostics/probe_ex003_silent_failure_pair__f031_stderr.edn`
  has `:message "#wat.runtime/TypeMismatch {…}"`.
- **20 files** come from a SECOND mechanism, which the design did not name. In
  `tests/rete/probe_arc278_D10_then_field_types__bound_var.edn`, `StartupError.error` IS a
  structured `#wat.rete/ReteCheckErrors` record, but that record's own `:message` holds its own
  serialized EDN.
  - `ReteCheckErrors::message()` (`src/rete/validate/error.rs`, the `WatError` impl) returns prose.
  - Its `Display` and `Debug` impls return `to_wire_edn(self)`.
  - So some startup path builds `:message` from `Display`/`to_string()` instead of `message()`.
    **Trace which path, and name it.**

## What to build

1. **The shape.** In the `defenum`:
   - `Panic`, `RuntimeError`, `StartupError`, `EntryFormFailure`, `MainSignature` and `BadReturn`
     each become `[failure <- :wat::kernel::Failure]`. `StartupError` is included, for one failure
     shape across all six.
   - `Disconnected` and `Stopped` stay bare.
   - `Panic`'s separate `message` field goes. Its text is `failure.error.message`.
   - `Failure` gains `frames-elided <- :wat::core::i64`. Step 2's cap records it on the
     `RuntimeError` today, and the `Failure` is where frames now live.
   - Update the doc lines of both forms. Fix `AssertionFailure`'s stale `(Option :- [Location])`
     prose while you are there; `Location` retired in step 1.

2. **The constructors build a `Failure`. They never serialize.**
   - **From a `RuntimeError`:** `error` is `re.to_record()` (step 3a); `frames` and `frames-elided`
     come from the error's captured frames (step 2).
   - **From a `StartupError`:** `error` is the structured `:wat::core::Error` it already carries;
     `frames` is empty; `frames-elided` is 0.
   - **From a `FlatMessage`, or a plain Rust message** (`SendError::Failed`'s reason, main-signature,
     bad-return, and OS-level failures):
     - `error` is a `:wat::core::Fault`: the message, `causes []`, and `location` = the Rust site
       that raised it, taken via `#[track_caller]` as a `Span` with `end` `None`. This is D1/D3:
       Rust knows only where a failure starts.
     - `frames` is the capped wat `CALL_STACK` snapshot plus the Rust frame, using step 2's helper.
       Do not write a second capper.
   - **From a panic:** with an `AssertionPayload`, use the existing Failure builder. Without one,
     use a `Fault` built from the panic message and the panic's own location (from the hook, if it
     is captured; measure whether it is).
   - **Delete** the `*(message: String)` constructors once nothing calls them. A string-taking door
     left open is how this defect comes back.

3. **`LociDiedError/message` becomes derived.**
   - For a failure variant it returns `failure.error.message`.
   - The two unit variants keep their constants.
   - Its 444 tracked uses in `.wat` (measured with
     `git ls-files '*.wat' | xargs grep -oh "LociDiedError/message" | wc -l`, comments included)
     keep compiling unchanged.
   - Its return value changes: for a `RuntimeError` it was the whole EDN blob, and becomes the
     one-line headline. **Measure** whether any wat or Rust test matched on EDN inside that string.
     My grep for `"#wat.` in `.wat` code found 0, and a grep that finds nothing proves nothing about
     the behaviour. The floor is the real answer; report every test it moved.
   - `died_error_payload_message` and `eval_died_error_to_failure` (`src/kernel/error.rs`) simplify
     to projections.

4. **The parent's decode must stay structural.** `loci_died_error_from_reason`
   (`src/kernel/error.rs`) parses a death line with `edn_to_value`. When that fails, it falls
   through to an opaque `Panic` wrapping the raw reason, **silently**. With structured failures, a
   missing registration would take that exit and look like a panic.
   - A reason that IS a `LociDiedError` chain but fails to decode must not quietly become a panic.
     Its `Fault` message must say that the death report could not be decoded, and why (the
     `EdnReadError`).
   - Keep the opaque arm for reasons that truly are opaque. Name which reasons still reach it.
   - Measure which callers pass `types: None`, and what decode does with records in that case.

5. **The second mechanism.** Fix the path that builds a structured error's `:message` from
   `Display`: use `message()`. Also decide whether `ReteCheckErrors`'s `Display` should keep
   returning wire EDN. A `Display` that emits EDN is exactly how this path got its data. If other
   `WatError` types implement `Display` the same way, list them. Fix them only if they reach a
   `:message`.

## Gates (each mutation-proven in RELEASE: break it, see RED, restore)

- **G1, no double-quoting anywhere.** A lint test that parses every tracked `.edn` golden and fails
  if any string value, at any depth, parses as tagged EDN. Parse it; do not use a text grep.
  - **Anchor:** before your changes it must be RED and list the 21 files. Run it on the unchanged
    tree first and report that count. A gate that has never been red has proven nothing.
  - **Mutation:** after the cure, put `to_wire_edn` back into one constructor, and the gate must go
    RED again.
- **G2, typed across the boundary.** An end-to-end test: a child process fails with integer division
  by zero.
  - The parent must receive `LociDiedError::RuntimeError`, whose `failure.error` is a
    `:wat::runtime::DivisionByZero` record (its class, not its text), with non-empty `frames` and
    `LociDiedError/message` equal to the headline.
  - Use the right idiom from `docs/CONVENTIONS.md` § "Test idioms" (EDN-over-stdio for a program
    claim).
  - **Mutation:** remove the `DivisionByZero` registration. G2 must go RED, and must NOT pass
    because the report was silently decoded as a `Panic`.
- **G3, one failure shape per variant.** For each of the six failure variants, drive one real
  producer and assert that the payload is a `Failure` whose `error` is a record, not a string.
  - Say which producer reaches each variant.
  - If a variant has no drivable producer, say so. Do not write a test that constructs the value by
    hand and call that coverage.

## Goldens

- Recapture with `UPDATE_EDN=1`. See the `assert_edn_matches_file!` bless mode (`src/lib.rs:431`).
  - **44 `.edn` files** name a failure variant: `git ls-files '*.edn' | xargs grep -lE
    "LociDiedError\.(Panic|RuntimeError|StartupError|EntryFormFailure|MainSignature|BadReturn)"`.
  - **24 `.rs`/`.wat` files** mention one. Some are prose, and some assert.
- **A recaptured golden is a claim.** It is whatever the code printed. Read the diff of every one.
  G1 proves that nothing is double-quoted; the diff proves nothing else changed. Report any
  recaptured golden whose change is NOT the envelope reshape.
- `.wat` edits across many files go through the wat-fix codemod (`wat-rs/CLAUDE.md`), never
  hand-edits or sed. My measurement found **no** wat `match` pattern on these variants, so there
  should be little `.wat` to change. If you find more, measure it before you pick a tool.

## Scope fence

- **IN:** `LociDiedError`/`Failure` shape, every constructor, the derived `/message`, the decode
  bridge, the Display→`:message` path, G1–G3, and the golden recapture.
- **OUT (step 3c):**
  - `src/macros/error_edn.rs:157` `Validator(e) => first_line(e.to_string())`.
  - Retiring the second `RuntimeError` EDN writer (the kind's derive, the one that writes `:span`).
  - `ReteCheckErrors`'s `:location nil`, a floor violation for a multi-error aggregate; name it,
    don't fix it.
- **OUT (step 4):** D4, the derived primary location.

## Discipline

- **One cargo process at a time.** Check with `pgrep -x cargo`, never `pgrep -f`.
- Use `cargo nextest run --release`, never `cargo test`.
- The floor:
  - Run `scripts/floor.sh` in the foreground, or `nohup` plus a foreground
    `until grep -qE '^ *Summary' <file>; do sleep 30; done`.
  - Do not end your turn while a build or floor runs.
  - Read the `Summary` line of `.floor/latest/clean.log`. The floor is **0 failed**.
  - On any red: **do not re-run**. Copy the failing block verbatim, name the arm, and report.
- **Stage every new and changed path by name BEFORE the floor**; two lints read `git ls-files`. Never
  use `git add -A`.
- No `cargo fmt`.
- A new test file that carries wat-looking string data needs its rune, like
  `probe_excursus003_step3a_wat_records.rs`.
- Commit prefix `EXCURSUS(003):`, ending with
  `Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>`.
- Push to `origin/reason/little-wat-findings`.

## Report

For each of the following, give the command you ran and the output:
- the traced Display→`:message` path;
- G1's anchor count on the unchanged tree;
- each gate's mutation RED;
- the producer for each variant;
- the tests whose expectations moved, and why;
- any recaptured golden that changed beyond the reshape;
- the reasons that still reach the opaque arm;
- the floor `Summary` line, verbatim;
- the commit SHA(s).
