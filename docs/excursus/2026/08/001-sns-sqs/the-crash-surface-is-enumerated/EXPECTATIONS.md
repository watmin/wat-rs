# EXPECTATIONS — the crash surface is enumerated

**Written BEFORE the strike**, from `83ec505fd`.

## What this stone is graded on

Not a green floor — **nothing in the substrate changes**, so a green floor is close to free here and
proves only that the scratch probes type-check. It is graded on whether **each of 27 cells carries
evidence**, and on whether the `UNREACHABLE` cells are honest.

| # | what | check | expected |
|---|---|---|---|
| 1 | the surface is re-derived, not copied | the FINDING's own count | **27** failure variants / **10** enums, matching DESIGN |
| 2 | ⭑ instrument controls pass | stated in the FINDING | `RecvOutcome` = **6** · `Closed` present · `Item` → `NextOutcome` · the two `ReadFrameOutcome` are distinct |
| 3 | every cell classified | read the matrix | no blank cells; each is `FIRES` / `UNREACHABLE` / `NOT SWEPT` |
| 4 | ⭑⭑ every `FIRES` carries a runnable command | pick 3 at random and run them myself | they fire, and I see what the FINDING says I will see |
| 5 | ⭑⭑ every `UNREACHABLE` names the missing mechanism | read them | a mechanism, not *"no test covers it"* — the latter is a restatement, not a reason |
| 6 | the §2d store-fault cell is explicit | grep the FINDING | named, with the six `sqs.wat` arm lines and the measured zeros |
| 7 | `disrupt`'s reach is established | read | which variants `disrupt-bp` actually provokes — it is the one existing lost/closed mechanism |
| 8 | the ranked injector list exists | read | ranked by cells-bought-per-work, with the store-fault entry called out |
| 9 | no substrate change | `git diff --stat -- src/ wat/` | **EMPTY** |
| 10 | probes type-check | floor | **5237 passed, 0 FAIL**, no `ARM.txt` |
| 11 | counts are occurrences | spot-check one arm count myself | `grep -o … \| wc -l`, not `grep -c` |
| 12 | partial labelled as partial | read | if incomplete, `NOT SWEPT` present and the four priority enums done |

## Runtime prediction

**50–80 minutes.** The census is mechanical; the *firing* is the long pole — some cells will need a
scratch probe each, and a few will resist. Floor ~490–520 s.

## ⚠ Two null results I will accept, and one I will not

**Accept:** a cell that is genuinely `UNREACHABLE` with a named missing mechanism. **Accept:** a partial
sweep, labelled, with the four priority enums finished. ⛔ **Reject:** an `UNREACHABLE` whose reason is
*"nothing drives it"* — that is the observation restated as its own explanation. The reason must say what
mechanism is absent, e.g. *"no fault can be injected into a store call: the queue's only knobs suppress
its own reply to its caller, and the store process is never signalled."*

## What would make me reject a green result

- Row 4 failing on any of the three `FIRES` cells I pick — a command that does not fire is a phantom cell.
- An instrument whose controls were not stated (row 2). Four corrections were needed to produce the
  denominator; a sweep that reports no corrections has probably not run the controls.
- `git diff` touching `src/` or `wat/` (STOP-2 violated: the census became the cure).
- A count taken with `grep -c`.
