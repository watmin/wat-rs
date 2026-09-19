# AMEND — 251.8c: the new test trips a MAIN-ONLY gate (`no_loose_string_assert`)

**Fold into `4e9fb9e17`. Do not land as a follow-up commit.** The orchestrator's floor found this; the
stone is otherwise **green and verified** (see §Verified below).

## The red, verbatim, first capture

```
FAIL wat::lint no_loose_string_assert::tests_carry_no_loose_string_assert
     ( 152/5919)

🔥🔥🔥 LOOSE STRING ASSERTIONS — 5 site(s) assert a value with contains/starts_with/
ends_with where an exact `assert_eq!` belongs. A loose check passes on reordered fields,
malformed maps, and appended garbage.

THE FIX (RUBRIC: docs/CONVENTIONS.md § 'Test idioms' -> 'The .edn golden'): a deterministic
STRUCTURED value goes in a co-located `<probe>__<label>.edn` golden, compared via
`wat::assert_edn_eq!(actual, include_str!("...edn"))` (parses both sides, structure-exact) —
capture the whole value, never guess. A scalar -> byte-identical `assert_eq!`. EXEMPT a
legitimately-loose one (a value that varies per run: path/pid/hash/timestamp, or a targeted
absence over a large output) with a per-site `// rune:lint(loose-assert) — <reason>`.

Drive it to ZERO. Offenders:

src/check.rs:25213
src/check.rs:25214
src/check.rs:25215
src/check.rs:25216
src/check.rs:25255
```

**This gate is MAIN-ONLY** — it does not exist on grok-rete, so nothing warned you. Same class as the
replay's recurring main-only-gate collisions; the fold rule applies and the repair belongs at the step.

## The sites

`must_reject_string_arg`'s four `contains` (`:25213`–`:25216`) and the `nope` row's two (`:25255`).

## ⭐ A MEASURED OPTION — the exact form that works here, and why

⛔ **A whole-error `assert_eq!` will NOT work naively**: the colon and slash sources differ in length,
so the spans differ, and byte-equality on the full rendered error fails on column numbers.

**But the stone's own claim gives you an exact assertion.** The orchestrator measured, verifying this
commit: for every shape the colon and slash rows produce a **byte-identical `:message` field**:

```
colon: TypeMismatch {:message ":user::f: parameter #1 expects :wat::core::i64; got :wat::core::String…
slash: TypeMismatch {:message ":user::f: parameter #1 expects :wat::core::i64; got :wat::core::String…
```

So the strongest exact form is **pair each shape and compare the two errors' `:message` for equality** —
which is precisely what 251.8c asserts ("slash and colon heads share the call path"), and it is
structure-exact rather than a substring guess. The `.edn` golden route is also open if you prefer the
rubric's first option.

⚠ **This is an option, not an instruction.** You built the test; if a better shape exists, take it and
say why. ⛔ **Do not reach for `rune:lint(loose-assert)` unless the value genuinely varies per run** —
these messages do not.

## What the fold must keep

The test's **non-vacuity** property: the five colon rows are the control. If a future change breaks the
shared call path, the test must fail — so whatever assertion shape you choose must still distinguish
"colon and slash agree" from "both are broken in the same way". State in the score how it does.

## Verified, so you need not re-derive

Independently re-run by the orchestrator at `4e9fb9e17`, **checking the inner error variant and message
rather than the exit code**:

| shape | colon | slash | |
|---|---|---|---|
| statement `do` | `TypeMismatch` `:user::f: parameter #1 expects i64; got String` | **byte-identical** | ✓ |
| let-bound | same | **byte-identical** | ✓ |
| plain nested arg | same | **byte-identical** | ✓ |
| `format "{v}" :v` | same | **byte-identical** | ✓ |

`check::tests::stone_251_8c_slash_and_colon_heads_share_the_call_path` → 1 passed.
Floor **5919 run / 22 skipped**, exactly the predicted count; **clippy 0**; census `no STOP-8`.
**The only red in the tree is the one above.**

## ⭐ AND YOU WERE RIGHT, THE BRIEF WAS WRONG

Recorded because it matters more than the fix: the brief claimed four shapes were "already closed" and
the residue was `format`'s `:v`. **Those four rows were probe artifacts** — the orchestrator read
`rc=1` as "caught" when a *different* error was firing, in the same document that warned against
exactly that. Your diagnosis — `check_program` walking pre-normalization `FunctionBody` snapshots, the
class being every namespaced-symbol call in a function body — is the true one, and killing the
hypothesis was the brief's first ask.

## After the fold

Re-run your walls, then `pulsare_yield kind=scored`. The orchestrator re-runs floor + clippy + census
uncontended. ⛔ **Still do not push, and 8d does not start until the floor is green.**
