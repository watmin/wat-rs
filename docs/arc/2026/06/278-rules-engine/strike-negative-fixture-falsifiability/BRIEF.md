# BRIEF — make every negative fixture fail for its own reason

A `.wat.bad` that dies at startup for a missing entry point proves nothing about the wall it was
written for. Build the gate that forbids that, resolve the fixtures already in that state, and 200
`assert!(is_err())` tests become falsifiable without one of them being edited.

## Read in order

1. `tests/types/probe_arc294_9a_kwargs_ctor.rs:35-43` — the idiom, at its purest: `startup_from_file`
   then a bare `assert!(r.is_err(), …)`. **This is the shape 200 tests share.** Its fixture,
   `probe_arc294_9a_kwargs_ctor_bad.wat.bad`, has no `:user::main`.
2. `probe-c18.py.txt` beside this brief — the orchestrator's classifier and its own truncation
   lesson. Run it first; it is a worked reference, not a claim to take on faith.
3. `tests/types/probe_arc237_8a_no_implicit_coercion_arith_f64_i64.wat.bad:1` — an expired premise
   stating itself: *"237.8a's reject retired. Type-checks."* Five siblings share it
   (`8a` ×3 counting `arith_i64_f64` and `cmp_i64_f64`, `8b` ×2, `8c`, `8d`).
4. `tests/function/probe_diagnostic_non_keyword.wat.bad:1-2` — claims an error *at EVAL*. With no
   main, nothing evals. A different way to be untestable.
5. `tests/program/wat_arc170_slice_1e_user_main_nil_*.wat.bad` — the two **legitimate** cases: the
   main signature IS their subject. Your gate must let these through by rune, not by exception list.
6. `tests/lint/every_walking_gate_declares_non_vacuity.rs` — the house style for a gate that requires
   a declared guard or a rune naming the mechanism. Copy its shape.

## Driven by the orchestrator at HEAD `545771b2f`

- **281** `.wat.bad` fixtures. **230 have no `:user::main`; 48 declare one returning nil.**
- Both doors are startup failures: no-main → `#wat.kernel/MainSignatureError "not defined"`;
  `(:user::main [] -> nil nil)` → `#wat.macro/MainSignatureError` (UselessMain). Exit 4 either way.
- **200 of 283** tests naming a `.wat.bad` assert only `is_err()`/`!ok`.
- **17** fixtures fail with `MainSignatureError` today — stable over three runs.
- Decisive: `probe_arc294_9a_kwargs_ctor_bad.wat.bad` with its offending construct made VALID
  **still fails**, with `main not defined`.

## The two pieces

1. **The gate.** Discovered, never listed — walk every `.wat.bad` under `tests/`, `wat-scripts/` and
   `docs/`, run it, and FAIL any whose error is `MainSignatureError`, naming the file and printing
   its actual error. A fixture whose subject genuinely IS the main signature carries a per-file rune
   (`;; rune:lint(...) — <reason>`) in the fixture itself; the two `wat_arc170_slice_1e_user_main_nil_*`
   are the first two and their reason is real.
2. **Resolve the 17.** For each: give it a real, runnable `main` so it fails for its own reason —
   **or**, where the premise has expired, resolve it with evidence. The six `arc237_8*` files claim
   the behaviour is now legal; **verify that against the current binary** before deciding whether
   each becomes a positive fixture or is deleted. Do not guess, and do not batch-decide them.

## Blast radius

`tests/**/*.wat.bad` (the 17), one new gate under `tests/lint/`, and whatever `.rs` change resolving
an expired fixture requires. **No `src/` change is expected** — if you believe one is needed, that is
STOP-2.

## STOP triggers

1. **If a fixture's premise is expired and you cannot determine what the behaviour should be now**,
   stop and report it. A negative fixture converted to the wrong polarity is worse than a dead one.
2. **If resolving any fixture requires a `src/` change**, stop and report — that is a live engine
   question, not a test-corpus question, and it outranks this strike.
3. **If the gate's population differs from 281**, stop and report the delta before proceeding; the
   discovery glob is load-bearing and a silent difference means we are measuring different things.
4. **If any fixture DID NOT FAIL at all**, stop and report it immediately — a `.wat.bad` that passes
   is a hole the whole corpus is presumed not to have.

## Mutation proofs — run all three, report all three

1. **Delete the `main` you added to one resolved fixture** → the gate REDs, naming that file.
2. **Remove the rune from `wat_arc170_slice_1e_user_main_nil_wrong_return.wat.bad`** → the gate REDs.
   Proves the exemption is rune-driven, not hard-coded.
3. **Take one resolved fixture and make its offending construct VALID** → its own test now goes RED.
   ★ **This is the strike's whole point** — the same edit against an unresolved fixture leaves the
   test green. Show both halves.

Verify every restore by **hash**, not `git diff`: `git checkout <sha> -- <path>` STAGES.

## What to report

- The gate's output, its population count, and the runtime.
- All three mutation results.
- A line per resolved fixture: what it now fails with, and why that is its own reason.
- For each expired-premise fixture: the evidence you drove, and the disposition you chose.
- Scoped nextest `Summary` lines including `binary_id(wat::lint)`.
- **Anywhere this brief was thin or wrong. Be blunt.** Two consecutive strikes here had their ★
  finding be a false claim inside a file the brief told the rider to trust. Assume there is a third
  and go looking before you build.

Do not commit.
