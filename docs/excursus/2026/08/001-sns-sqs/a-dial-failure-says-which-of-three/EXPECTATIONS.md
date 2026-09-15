# EXPECTATIONS — a dial failure says which of three things happened

Written **before** the strike. Graded on the orchestrator's **own** re-runs.

Baseline (`e8bd008c2`): floor **5251**/5251, 0 FAIL, clippy 0, happy path `distinct=8000;dup=0`.

⚠ **Report the floor delta as a BAND, never a number.** This box's noise floor is at least **±16 s**
(a run came in 16 s *faster* than baseline), and the executor's box runs +18…30 s. Precision claimed
across earlier stones was inside the noise.

| # | what | command | expected |
|---|---|---|---|
| 1 | ⭑⭑ **the three variants are named** | the diff | every converted site distinguishes `Refused` / `Rejected` / `Failed` |
| 2 | ⭑⭑ **the cause survives** | the diff | each carries `(:wat::kernel::Failure/message c)`. A hardcoded string with no cause is the defect restated |
| 3 | ⛔ **`Rejected` says what it MEANS** | the message text | names a **stale address / different process**, not a death. That is the whole finding |
| 4 | ⛔ **the site label survives** | the message text | "queue", "topic-worker: inbox", … — today's string has this and it must not be lost for the cause |
| 5 | ⛔ **it still RAISES** (STOP-3) | the diff | no behaviour change. Supervision is deferred |
| 6 | ⭑ **it is a CODEMOD, recorded** | `wat-scripts/fixes/` | a committed migration, **idempotent** — applying twice changes nothing |
| 7 | ⛔ **the census is the authority** (STOP-1) | the SCORE | the codemod's own count, **with matches**, not reconciled to the brief's 35 |
| 8 | ⛔ **the 18 correct sites untouched** (STOP-4) | the diff | not swept in; a note on whether they should be later |
| 9 | ⭑ **`Refused` is driven** | `probe-crash-surface-connect-refused.wat` | still reports, now naming the site |
| 10 | ⚠ **`Rejected` reachability stated** | the SCORE | the crash-surface matrix lists it UNREACHABLE. "Could not drive it" is an honest pass; a fabricated fixture is not |
| 11 | happy path | the record invocation | `distinct=8000;dup=0`, every counter identical |
| 12 | floor | `./scripts/floor.sh` → **Summary** | `5251 passed` or higher, 0 FAIL. A SHRINK is a finding |
| 13 | clippy | `--all-targets -D warnings` | 0 |
| 14 | ⚠ the comment pass, stated | the SCORE | prose still saying "peer is dead" is a manual pass; say which you did |

## Runtime prediction

**90–150 minutes.** The helper is small; the cost is writing the codemod, its census, and the
`wat/service.wat` rebuild.

## Trap-door risks, ranked

1. ⭑⭑ **Row 4 lost.** The easy mistake is adopting the exemplar verbatim — which drops the site label
   and keeps only the cause. Today's message has the label and not the cause; the fix needs **both**.
2. **Row 7 reconciled to 35.** If the codemod's census disagrees, the census is right. Bending it to
   match a brief written from a single-line grep is how a wrong number becomes a decision.
3. **Row 5 drifting into a fix.** Making these tolerant is the supervision ruling, not this stone.
4. **A legitimate `_`** rewritten to satisfy a count (STOP-2).
