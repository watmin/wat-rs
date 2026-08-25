# BRIEF — HOME #4, PHASE 1: the doctest runner faces its failures

DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-HOME-4-the-string-carve.md` — read whole.

**This is PHASE 1 ONLY: the runner.** The 19-verb string carve is phase 2 and is NOT yours. The
split exists because phase 1's SIZE IS UNKNOWN — nobody can say how many examples are broken until
the runner can report more than one, and bundling an unknown-size diagnostic with a known-size
carve is how both get estimated wrong.

## Your role

You are a rider, not the orchestrator. **Ending your turn ENDS you** — nothing wakes you. Run every
command in the FOREGROUND and block on it.

**You may not spawn sub-agents.** Anchor: `/home/john/work/holon/wat-rs`. `pwd` first. You do not
commit, push, stash, revert, or checkout.

`cargo build --release` is yours (a `wat/*.wat` edit needs it — `include_str!` at Rust-compile
time). `cargo nextest` on a NARROW `-E` filter is yours for the one test you are unlocking.
`scripts/floor.sh` and clippy are NOT — the orchestrator takes those centrally.

## ⛔ THE RUNNER RAISES ON THE FIRST FAILURE IT IS SUPPOSED TO COLLECT

`wat/doctest.wat:38` — `:wat::doctest::verify-examples` returns a `Vector<Failure>`. It is a
COLLECTOR. And at `:60-68` it uses raising unwraps on the per-example path:

```wat
expected-ast (:wat::core::Option/expect (Example/expected ex) "…missing expected")
got          (:wat::core::Result/expect (:wat::eval-ast! (Example/expr ex))     "…expr eval failed")
want         (:wat::core::Result/expect (:wat::eval-ast! expected-ast)          "…expected eval failed")
```

**Three defects, and they compound:**

1. **It raises instead of recording.** The first example whose expr fails to eval aborts the whole
   run, so the `Vector<Failure>` never returns. You learn about exactly one, ever. That is the
   `no-hidden-failures` law inverted inside the very verb built to uphold it.
2. **The message is a constant.** `"verify-examples: expr eval failed"` names no intrinsic, no
   example, no expression. Measured: the diagnostic points at `doctest.wat:64:35` — inside the
   runner — and nothing else.
3. **Together they made a red look like a deferral.** `probe_arc255_ivb2b_verify_examples.rs:32` is
   `#[ignore]`d as *"metadata-of reflection … not yet built"*. Both halves ARE built. The runner runs
   and reports a real failure. That ignore has been holding a red, not a gap.

`:wat::doctest::Failure` already carries `:fqdn` and `:reason` — the naming this needs is available
and the raise path simply does not use it.

## The work

**Make each per-example unwrap a FACED OUTCOME that conj's a `Failure` and keeps going.** Match the
`Result`/`Option`, and on the failing arm push a `Failure` carrying the example's `fqdn` and a
`reason` that says which stage failed (expr eval / expected eval / missing expected) and what the
error was. Then continue the fold — one bad example must not hide the next.

Copy the shape already in the file: `:55-58` builds exactly such a `Failure` for the
non-pure∧deterministic case, conj'ing rather than raising. That arm is right; the three below it are
not.

Then **un-ignore** `verify_examples_reports_no_failures` and read the real count.

## The rooms

1. **`wat/doctest.wat:27-75`** — `verify-examples`. The three raising unwraps and, at `:55-58`, the
   correct shape to copy.
2. **`wat/doctest.wat`'s `Failure` record** — `:fqdn` / `:reason`, the fields that make a red
   locatable.
3. **`tests/reflection/probe_arc255_ivb2b_verify_examples.rs`** — the ignored test and its
   `verify_examples_failure_count` helper.
4. **`src/intrinsic/reflect.rs:610-612`** — three `@example` lines asserting a call returns `true`
   when the call RAISES. Almost certainly among what you will find; do not assume it is the only one.
5. **`src/intrinsic/bytes.rs:34-44`** — a well-formed preamble, for what a CORRECT `@example` looks
   like.

## The acceptance rows YOU run

- **Row 1 — the runner NAMES its failures.** After the fix, its output identifies each failing
  example by `fqdn` and stage. Report the full list verbatim — that list is this phase's real
  deliverable.
- **★ Row 2 — the count is REAL.** `verify_examples_failure_count` returns a number instead of
  raising. Report it. **This number is unknown to everyone right now** and it decides phase 2's
  shape.
- **Row 3 — one bad example does not hide the next.** Deliberately break a SECOND example, confirm
  BOTH are reported, then restore it. A collector that stops at the first is the defect being fixed;
  prove it is fixed rather than asserting it.
- **Row 4 — every reported failure is resolved**, and for each you state which side was wrong: the
  example (fix the doc) or the intrinsic (a real behaviour bug — see STOP-2).
- **Row 5 — `verify_examples_reports_no_failures` is UN-IGNORED and GREEN.** Not un-ignored; green.
- **Row 6 — the sibling ignore, measured.** `tests/reflection/probe_arc255_reflection_parity.rs:70`
  carries the same stale reason. Run it with `--run-ignored` and report what happens. Do NOT fix it
  — just say whether it is also hiding a red rather than a gap.

Report each row's command and output **verbatim** — never a summary, never a `| head`/`| tail`
window. A row you could not run is reported as not-run, never as passed.

## Blast radius

- `wat/doctest.wat` — the three unwraps become faced outcomes
- `tests/reflection/probe_arc255_ivb2b_verify_examples.rs` — the `#[ignore]` comes off
- whichever `///` `@example` lines are wrong — doc-comment edits only

Nothing else. No intrinsic BEHAVIOUR changes (STOP-2). No new verbs. No string carve — that is
phase 2.

## STOP triggers — each ships NOTHING and surfaces the gap

1. **The real failure count is large (say, more than ~10).** STOP and report the full list. That is
   a scoping decision about a population nobody has seen, and it belongs to the orchestrator.
2. **Fixing an example requires changing what an INTRINSIC DOES.** STOP. An example that documents
   behaviour the code does not have is either a wrong doc or a real bug, and telling them apart is
   not a doc edit. Report the intrinsic, the example, and what actually happens.
3. **The runner cannot name a failure without restructuring `Example`/`Failure`.** STOP and report —
   changing a registry record's shape is a design decision, not a fix.
4. **Row 3 shows failures still mask each other** after your change. STOP — the collector is still
   raising somewhere you have not found, and shipping a half-collector is worse than the honest
   raise it replaced.

A STOP means: leave the tree as it is, write the report, end your turn.

## What you own that nobody can reconstruct

**The list.** Row 1's full inventory of which examples were lying and how, and row 2's number. That
population has been invisible behind a one-example abort for months; you are the first to see it.
