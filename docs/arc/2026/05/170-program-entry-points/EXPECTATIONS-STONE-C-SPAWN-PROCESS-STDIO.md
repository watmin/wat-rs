# Arc 170 Stone C EXPECTATIONS — spawn-process stdio reshape

**One slice.** Substrate reshape + wat-level wrappers + consumer sweep. Closes the original "fatal flaw."

## Runtime band

**120-180 min sonnet.** Hard cap 300 min (1.7×; bigger than Stone A). Wakeup at T+3600s (runtime cap).

## Scorecard (10 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `spawn_process_child_branch` opens 3 OS pipes + dup2s fd 0/1/2 | grep + read |
| B | spawn-process child delegates to `bootstrap_wat_vm_process` (Stone A's helper) | grep + read |
| C | `Process` struct has 4 fields (stdin IOWriter, stdout IOReader, stderr IOReader, ProgramHandle); NO `tx`, NO `rx` | grep + read |
| D | `process-send` / `process-recv` dispatch retired; Pattern 2 teacher (`arc_170_stone_c_...`) in `collect_hints` emits migration hint pointing at `Sender/from-pipe`/`Receiver/from-pipe` | grep + read |
| E | Wat-level `Sender/from-pipe` + `Receiver/from-pipe` wrappers minted; EDN round-trip works | cargo test |
| F | `probe_spawn_process_stdio` PASSES — child println, parent captures via Process/stdout | cargo test |
| G | `probe_spawn_process_stdin` PASSES — parent writes, child reads via readln | cargo test |
| H | `probe_sender_receiver_from_pipe` PASSES — wrapper round-trip | cargo test |
| I | Consumer sweep complete; pattern hint fires 0 times in workspace tests | grep cargo output |
| J | Workspace ends green (count may shift; honestly accounted) | full workspace |

**10 rows. All must PASS.**

## Discipline mirror (orchestrator-side)

- FM 9: independent re-run of each load-bearing row pre-commit
- FM 12: `model: "sonnet"` explicit
- FM 16: no Bash/tool-availability preamble in BRIEF (none added)
- FM 17: pre-action sweep on commit — verify each probe body exercises path its name claims (Row F/G/H path-honesty)
- Constraint pattern corrected per memory `feedback_brief_constraint_contradictions` — narrow surface; SCORE + TIERS.md amendment EXPLICITLY in scope
- Atomic commit on success; ScheduleWakeup at T+3600s with re-schedule if predicted upper-bound (180min) approaches
- Reap orphans (`pkill -9 -f "target/release/deps/test-"`) before each cargo invocation

## Mode B trigger

- Substrate refactor reveals embedded typed-channel test infrastructure that can't easily migrate → split or stage
- Sender<T>/Receiver<T> existing types can't accept `from-pipe` cleanly → distinct types or arc 146 dispatch deferral
- Workspace falls below pre-Stone-C 167 by more than expected typed-channel-consumer count → unexpected regression
- Consumer sweep can't migrate run-hermetic-with-io fully → fix wrapper first
