# BRIEF — the store injector has teeth

**Read `DESIGN.md` beside this first.** It carries why this is **three** tests and not one (a measured 20.7 s
against a 40 s wall this floor has already hit), and the relations, which contain no probability.

## The work, in one paragraph

`wat-scripts/query/faulting-store.wat` loses a store reply **after the write lands** — the condition D1-c's
`Accepted n == real-rows` exists for — and **no floor test drives it** (`grep -rln faulting-store tests` →
nothing). The probe already exposes three zero-arg entry points returning `String`. Add one harness with
**three tests**, asserting `n == real-rows` on both answering paths, `drops-fired > 0` where the fault fires,
and `TimedOut` + `real-rows == 1` on the die path.

## The rooms

| where | why you are going there |
|---|---|
| `wat-scripts/scratch-pad/probe-accepted-is-a-count.wat:93`, `:111`, `:128` | ⭑ **THE THREE ENTRY POINTS**, already zero-arg and already returning `String`: `:sf::run-passthrough`, `:sf::run-die`, `:sf::run-timedout`. `:user::main` (`:170`) only prints them. **Do not change this file.** |
| `tests/services/probe_chaos_gate_has_teeth.rs` | ⭑ **COPY THIS HARNESS.** D2's gate: `startup_from_source` + `FsLoader` (the probe does relative `load-file!`), `apply_function` on a named zero-arg fn, and the `field(summary,key)` / `i64_field` helpers. Its module doc also shows how to record *why* a threshold was refused. |
| `tests/services/probe_ex001_fanout.rs` | the older sibling, and the origin of the `field()` splitter. |
| `wat-scripts/query/faulting-store.wat` | the proxy. `drop-reply-bp` forwards then suppresses its own reply; `die-bp` forwards then exits. **Do not change it.** |
| `docs/excursus/2026/08/001-sns-sqs/accepted-is-a-count/SCORE.md` | the invariant you are gating, and its measured output shape — `passthrough Accepted 1 / real-rows=1`, `timedout Accepted 1 / real-rows=1 / drops-fired=1`, `die TimedOut / real-rows=1`. |

## What each test asserts

```
passthrough   n == real-rows                            (rates 0; both 1)
⭑ drop-reply  n == real-rows   AND  drops-fired > 0     ← D1-c's invariant, the row worth having
die           outcome == TimedOut  AND  real-rows == 1   (write landed, proxy gone)
```

⛔ **No count assertions beyond `drops-fired > 0`.** At `drop-reply-bp=10000` every roll fires, so `> 0` is
deterministic — it only proves the fault happened. Pinning `drops-fired` to a number would gate a mechanism
detail, and this campaign has four errors from gating observations rather than invariants.

## ⛔ Three tests, and say what each costs against the wall

I measured the probe as one program: **20.7 s**, because the `drop-reply` path waits the full **10 s** client
deadline (unshortenable from the callee — `NOTE-a-callee-deadline-ms-is-inert.md`). The floor's per-test wall is
**40 s**, and it has already killed a **24 s** test under `nice -n 19` contention
(`.floor/2026-09-13T04-00-06Z/`).

> Expect roughly **~3 s / ~7 s / ~17 s**. **Report each against the 40 s wall as headroom**, not as bare
> seconds. If any single test exceeds ~20 s, say so explicitly.

## Verify

- `./scripts/floor.sh`, read the **Summary line**. ⚠ The count is **5241** today and **will move** — state the
  new number so nobody reads growth as a shrink.
- ⛔ **Never a piped exit code** (a type-error run this session reported `$?` = 0 through `| head`; true exit 3).
- ⭑ **Run the new tests 3× and show identical relations** — the assertions claim to be probability-free, which
  owes repetition.
- `cargo nextest run --release --no-run`; `cargo clippy --release --workspace --all-targets -- -D warnings` → 0.
- ⭑ **The floor-time delta**, against `528.833 s` (my own guarded run at 5241). The guard just returned ~30 s;
  say how much of it this spends.
- `git diff --stat -- src/ wat/ wat-scripts/` must be **EMPTY** — harness only.
- Happy path `circuit.wat 2000 4 3 8192 true 1000` → `distinct=8000;dup=0`.
- Everything heavy through `./scripts/capped.sh --limit 8g`.

## STOP triggers

1. **STOP-1 — do not modify the probe or the proxy.** Both are already the right shape; if one seems to need a
   change, STOP and report it as a finding.
2. **STOP-2 — do not combine the three into one test.** 20.7 s against a 40 s wall on a floor that has already
   lost a test there.
3. **STOP-3 — if `n == real-rows` does NOT hold**, STOP and report the pair. That is D1-c's invariant; a
   failure is a **finding about `04726854e`**, not something to weaken the assertion for.
4. **STOP-4 — do not assert any count except `drops-fired > 0`.**
5. **STOP-5 — if a test cannot be made to finish well inside the wall**, STOP and say which and why rather than
   shipping something near it.

## Shape to copy

`the-chaos-gate-has-teeth/SCORE.md` — the same stone for the other injector, including how it reported a
captured floor timeout honestly and why it refused a threshold.
