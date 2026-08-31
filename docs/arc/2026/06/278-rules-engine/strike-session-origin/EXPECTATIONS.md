# EXPECTATIONS — the session ceiling's zero point

> Written **before** the strike. Scored against the orchestrator's own re-run.

| # | what | command | expected |
|---|---|---|---|
| 1 | the probe is RED before | `cargo nextest run --release --no-capture -E 'test(a_second_session_on_the_thread)'` | **FAIL**: `control REFUSED` / `probe NO-BREACH`, panicking on `THE CEILING STOPPED ENFORCING` |
| 2 | the probe is GREEN after | same | **1 passed**, both arms `REFUSED` |
| 3 | the control arm is live, not decorative | read the probe's own `verdict("control")` assertion result | `REFUSED` in BOTH runs — if it ever reads `NO-BREACH` the workload stopped crossing 4 MB and row 2 is vacuous |
| 4 | the two existing ceiling gates still hold | `cargo nextest run --release -E 'test(ceiling)'` | all green — **especially** the insert-door gate that pins `limit 4096` and `staged 1` |
| 5 | the fixpoint door still refuses | `cargo nextest run --release -E 'test(round_cap) or test(memory_ceiling)'` | all green |
| 6 | blast radius | `git diff --stat` | the six files in DESIGN + the two probe files. A seventh is a STOP, not a delta |
| 7 | the hot path cost is stated | the report | `alloc_counter`'s doc warns the ceiling check is on the insert hot path and `SESSION_ORIGIN` was `const`-init for that reason. **The report must say what the replacement costs there** — not "it's fine" |
| 8 | rete surface | `cargo nextest run --release -E 'binary_id(wat::rete)'` | all green |
| 9 | the floor | `./scripts/floor.sh`, Summary from the captured log | **5,179 / 5,179**, 21 skipped, exit 0 |
| 10 | clippy | `cargo clippy --release --workspace --all-targets -- -D warnings` | silent, exit 0 |

## The mutation proof

Row 1 → row 2 is the proof. Then make `mark_session_origin` clobber regardless of id, confirm the
probe reddens, restore. **Row 3 is the counter-proof** and it is the one to distrust: verify the
control would go red if the ceiling were disabled, rather than trusting that it reads `REFUSED`.

## Runtime prediction

50–80 minutes. Six files means more rebuild churn than any strike in this chain; budget four or
five release builds at ~2m40s plus the ~370s floor.

## Trap doors named in advance — with the step

- **The probe's 4 MB ceiling is empirical, and machine-dependent.** It was swept on this box; a
  faster allocator or different `Vec` growth could move the crossing point. **Step:** if the
  control reads `NO-BREACH` on your machine, do NOT retune silently — report the value you had to
  use and why, so the next reader knows the number was moved.
- **`session_bytes()` is called on the insert hot path.** A `RefCell<FxHashMap>` there is not free.
  **Step:** if the map lookup measurably costs, say so with a number rather than shipping it
  quietly; a one-entry fast path (the common case is one session per thread) is the obvious
  mitigation and is in scope.
- **Over-claiming is the failure mode that survives a green floor.** The module currently tells the
  truth about what it cannot do. If the new doc says the ceiling now separates sessions, the strike
  has traded a known hole for an unknown one.

## What would make this a failure even if every test passes

A doc claiming the fix separates two sessions sharing a thread. It does not, and the sentence that
said so is the reason this defect was findable at all.
