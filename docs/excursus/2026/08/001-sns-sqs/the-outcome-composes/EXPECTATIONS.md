# EXPECTATIONS — the outcome composes

Written BEFORE the strike. Scored against my own re-run, never the report.

| # | what | the command | expected |
|---|---|---|---|
| 1 | ★ the send hack is gone | `grep -n 'Millisecond 1' wat-scripts/queue/sqs.wat` | **zero** occurrences. Today: 2 (`:241`, `:443`) |
| 2 | ★ `-flush-outbox` ceases to exist | `grep -c 'flush-outbox' wat-scripts/queue/sqs.wat` | **zero**. It is an arm that exists only because the combination was missing (STOP-5) |
| 3 | ★ an internal arm cannot reply — by SHAPE | edit `probes/internal-arm-replies.wat`'s `-tick` to the union-spelled reply and `--check` it | **REJECTED**, and the message names the arm and the reason — not `:wat::core::foldl` |
| 4 | ★ the runtime guards are DELETED, not kept | `grep -n 'has no client to reply to' wat/service.wat` | **zero**. Today: 3 (`:1666-1674`). Keeping them means the type did not close it (rung 2, not rung 3) |
| 5 | ★ nothing is lost | `./target/release/wat wat-scripts/fanout/circuit.wat` | `total=8000; distinct=8000; dup=0` |
| 6 | `Stop` carries sends | read the new enum | `Stop` has `sends`, has **no** `arms` |
| 7 | reply is Option, not a vector | read the new enum | `reply <- (Option :- [:R])` in `Outcome`; **absent** from `SelfOutcome` |
| 8 | no Rust change | `git diff --stat src/` | **empty** (STOP-3) |
| 9 | the migration is recorded | `ls wat-scripts/fixes/` | a new committed codemod; **no hand-edited `.wat`** (STOP-2) |
| 10 | the whole corpus migrated | `grep -rc '(:wat::service::Outcome::Reply\b' wat/ wat-scripts/ tests/ docs/` | old variants at **zero**; today 351 construction sites across 149 files |
| 11 | the phase split | re-run the circuit | **reported**, against `setup=8600 publish=2407 drain=72516 stop=2452` |
| 12 | wall time | re-run the circuit | **reported, not promised**, against 91.5 s |
| 13 | floor | `./scripts/floor.sh` — Summary line, never a piped exit code | 0 failed, `FLOOR=0`, ≥5184 tests |

**Runtime prediction:** 3–5 hours. The enum edit is small; the 351-site codemod and the serve
loop's consumption of the new fields carry all the risk.

## Trap doors, named in advance

- **The bootstrap.** `wat/service.wat` is frozen into the binary at build time. There is **no Rust
  change** here, so `fix.wat`'s second horn does not apply — but if the codemod needs a `fix` verb
  that does not exist, the first horn does, and that is STOP-1 rather than something to improvise
  around.
- **Hand-editing 351 sites.** The single most likely failure, and it would look like success. STOP-2
  exists because a hand-migrated corpus passes rows 5–13 while destroying the reason the codemod is
  the mechanism.
- **Keeping the runtime guards "just in case."** Row 4 fails on that. A guard retained after the type
  makes its case unwritable is dead code that quietly asserts the type did not work.
- **Firing on nothing:** rows 5–13 all pass if the enum gains fields and *nothing adopts them* — the
  queue keeping both `Millisecond 1` alarms and simply threading `sends: []` everywhere. **Rows 1, 2
  and 4 are what catch that**, and they are why they are starred.
- **Row 12 is not a target.** Deleting two 1 ms timers should help and I have not measured by how
  much; arithmetic bounds it at seconds, not tens of seconds. A slower number is a finding to report,
  not a reason to withhold the stone — the invariant in row 5 is the thing that is not negotiable.
