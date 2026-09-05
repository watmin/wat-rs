# BRIEF — mode parity gate

Build **one gate** that drives the real `wat` binary as a subprocess in `--check` and run modes over
a small fixture set, and asserts the two arms of the mode-parity invariant. **Ship the instrument,
not the cure.** The gate must be **RED at HEAD on both arms** when you are done — that is the
deliverable, and a green gate here is a failed strike.

## Read in order

1. **`DESIGN.md`, beside this file** — it pins the one contract decision ("agree" is not "same exit
   code") and lists what is affirmatively cut. Read it before you write a line.
2. **`src/distribution/mod.rs:311-410`** — the linear prefix and the four mode returns. You need
   `:316` (Mcp returns), `:351` (`check_only` returns), `:394` (signal wiring), `:397` (the
   RLIMIT_STACK raise). **`:313-315` says why MCP skips the signal wiring — that reasoning stands
   and this strike does not touch it.**
3. **`tests/cli/wat_cli.rs`** — the existing CLI driver. It already spawns the binary as a
   subprocess; **copy its shape for locating the binary and running it.** Do not invent a new way
   to find `target/release/wat`.
4. **`tests/cli/wat_cli__check_good.wat`** — a known-good `--check` fixture and the correct
   zero-arg main shape: `(:wat::core::defn :user::main [] -> :wat::core::nil …)`. Its header also
   records the `UselessMain` wall, which a bare `nil` body trips. **Copy this shape; a main with
   the wrong arity or a nil body will make your fixture fail for the wrong reason.**
5. **`tests/lint/docs_wat_loads_or_declares_why_not.rs:97-104`** — the non-vacuity idiom this repo
   uses (`assert!(!entries.is_empty(), "…the gate is measuring nothing")`). Your gate needs one.

## Implementation sketch

```rust
// tests/cli/mode_parity.rs  (or wherever wat_cli.rs's siblings live)

struct Case { name: &'static str, path: &'static str }

// SOUNDNESS: --check rc=0  =>  the run path does not fail before :user::main
// LIVENESS : run terminates normally  =>  --check does not die by signal
fn verdict(bin: &Path, args: &[&str], fixture: &str) -> Outcome { /* spawn, wait, classify */ }

enum Outcome { Accepted, Rejected, DiedBySignal(i32) }   // 3 arms, not an exit-code table
```

Classify by `ExitStatus::code()` — `None` means killed by a signal, which is the `DiedBySignal`
arm. **Do not compare raw exit codes across modes**: 1-vs-3 on the same rejection is by design.

## Fixtures — in `tests/cli/`, NOT under `docs/`

Two are enough to redden both arms; a third as the positive control.

| fixture | shape | proves |
|---|---|---|
| `mode_parity__empty.wat` | an empty file (or comments only) | **SOUNDNESS RED** — `--check` accepts, run rejects with `MainSignatureError` |
| `mode_parity__deep_freeze_recursion.wat` | ~1000 chained `defn`s, `f_i` calls `f_{i+1}`, `f_1000` returns 0, plus a top-level `(:wat::core::def :user::deep (:user::f0))` to force freeze-time evaluation, plus a valid main | **LIVENESS RED** — `--check` dies by signal, run exits 0 |
| `mode_parity__good.wat` | copy `wat_cli__check_good.wat` | **control** — both modes accept; the gate must be green on this row or it is not discriminating |

⛔ **The deep fixture must NOT go under `docs/arc/`.** `every_docs_wat_loads_or_declares_why_not`
loads every `.wat` there **in-process** — a 1000-deep fixture would abort the lint test itself.
`tests/cli/` is walked by no in-process loader.

Generate the deep fixture with a committed script or commit it outright; either is fine, but if you
generate it, the generator is committed too — a recorded number needs its instrument.

## Blast radius

`tests/cli/` only. **No `src/` change. No cure.** No new dependency. If you find yourself editing
`src/distribution/mod.rs`, you have left the strike.

## STOP triggers

1. **If the gate comes out GREEN on either arm, STOP and report.** Do not adjust the fixture until
   it reddens — that is fitting the instrument to the answer. Report what you drove and what you
   got; the orchestrator re-plans.
2. **If `tests/cli/wat_cli.rs` has no reusable way to locate the binary, STOP** and say so. Do not
   hardcode `target/release/wat` — the path depends on profile and target dir.
3. **If the deep fixture fails to freeze for a reason OTHER than stack exhaustion** (a type error, a
   malformed form), STOP. The orchestrator hit exactly this: a first attempt produced `2 type-check
   errors` and looked like a refutation of the whole finding. **A fixture that fails for the wrong
   reason is worse than no fixture.** Verify the failure mode is a signal death, not a diagnostic.
4. **If a run goes RED anywhere else on the floor, DO NOT RE-RUN.** Capture the failing test's
   entire stdout and stderr verbatim, name the exact assertion, and surface it.

## Prior result to copy for shape

`../vigilia-2026-09-05/probes/probe_vig_value_hash_collision.rs` — note its `calibration` test:
**two drives to fire and two to refuse** before it asserts anything. Your gate wants the same
shape, and the `mode_parity__good.wat` row is where it lives.
