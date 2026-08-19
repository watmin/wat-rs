# EXPECTATIONS — clause heads get TCO

**Written before the strike, 2026-08-18, against `663e5dae`** (floor 4714/4714, clippy 0,
ignores 13). Fixed here so the result cannot move the goalposts.

## The scorecard

| # | what | command | expected |
|---|---|---|---|
| 0 | baseline floor before any edit | `scripts/floor.sh` Summary | **4714 passed, 0 failed** |
| 1 | ★ RED→GREEN | run `probe-clause-tco-deep-defclause.wat` | was `rc=139` SIGSEGV → **prints 200000** |
| 2 | the control still passes | `probe-clause-tco-deep-defn.wat` | **200000**, unchanged |
| 3 | ★★★ **`:ensure` STILL FIRES** | `probe-clause-tco-ensure-still-fires.wat` | **`PostconditionFailed`, rc≠0** — identical to baseline |
| 4 | `:guard` still selects | `probe-clause-tco-guard-selects.wat` | **120**, unchanged |
| 5 | `NoMatchingClause` diagnostics unchanged | floor's existing clause tests | 0 FAIL |
| 6 | non-tail clause calls unmoved | floor | 0 FAIL |
| 7 | ⚠ **`Clause` clone stays cheap** | the new field is `Option<Arc<Function>>`, never a by-value `Function` | inspect the diff |
| 8 | floor | `scripts/floor.sh` Summary | **≥4714 passed, 0 failed, 0 timed out** |
| 9 | clippy | `cargo clippy --release --all-targets` | **0** |
| 10 | ignores | the `#[ignore]` grep | **13** |

**Row 3 is the stone.** A green row 1 with a broken row 3 is a *worse* substrate than we started
with: it would mean the change silently deletes post-conditions authors wrote and the checker
promised, in exchange for stack depth.

Rows 5 and 6 are what make it safe to ship language-wide — **every** `defclause` in wat routes
through this dispatcher (arithmetic, `into`, `conj`, every user-written multi-arity verb).

## Independent prediction

**Runtime: 90–150 minutes.** The field addition and the `eval_tail` arm are small. The real work is
extracting selection out of `eval_call_to_defclause_with_vals` without moving behaviour — a ~200-line
function that interleaves arity → type → guard → binding → body → `:ensure`.

**Time-box: 300 minutes.**

## Trap doors — named before the strike

1. **★ `:guard` evaluates in clause-arg scope**, so selection is *not* separable from binding. The
   extracted helper must return the bound scope too, and the non-tail path must keep using it.
   Selection that skips guard evaluation would silently pick the wrong arm.
2. **★ `:ensure` runs AFTER the body.** A tail call abandons the frame, so an ensure-bearing clause
   MUST take the ordinary path. This exclusion must be an explicit branch, never an accident of
   ordering.
3. **`emit_tail_call` re-evaluates raw args.** The tail path already has *evaluated* vals — build the
   `EvalSignal::TailCall` directly rather than routing back through `emit_tail_call`, or the args
   evaluate twice (and any effectful arg fires twice).
4. **Duplicating the selection loop instead of extracting it.** Two copies of clause dispatch is the
   exact "N ways to do a thing" defect this project keeps deleting. Extract; do not copy.
5. **`Clause` derives `Clone`.** A by-value `Function` field turns every clone deep. `Arc` it.

## What would make me call this Mode B

- Row 3 red, or `:ensure` reachable via the tail path at all.
- Selection logic duplicated rather than extracted.
- Args evaluated twice on the tail path.
- Any `#[ignore]` added, or a test deleted to make the floor green.

## What I will re-run myself before committing

Rows 1, 2, 3, 4, 8, 9, 10 — independently, on my own invocation, per FM 9. **Row 3 especially.**

## What this stone is NOT allowed to claim

That deep recursion is now safe everywhere. It gives **clause heads** the tail path `defn` heads
already have. A clause recursing inside a `cons`, an argument, or an `:ensure` still consumes stack
and still dies silently — that is task #58, untouched.
