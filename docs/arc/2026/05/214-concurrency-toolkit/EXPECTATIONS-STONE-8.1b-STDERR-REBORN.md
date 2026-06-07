# EXPECTATIONS — Stone 8.1b: StdErrService reborn

## Scorecard

| # | Row | Verification |
|---|-----|--------------|
| 1 | Gate-probe 81b GREEN (2/2) | orchestrator re-runs nursery probe |
| 2 | stderr.wat ≈ 15 non-comment lines (two records + one pure fn) | read the file |
| 3 | Write-pair generalization: ONE `spawn_write_service_peer` in the home; stdout + stderr both instantiate; zero loop duplication | read src/services/mod.rs |
| 4 | `StdErrServiceEvent` + `spawn_stderr_bridge` + wat-side stderr spawn/boot DEAD (grep finds no live refs) | class-grep `StdErrServiceEvent\|spawn_stderr_bridge\|StdErrService/spawn\|StdErrService::Event` |
| 5 | eval_kernel_eprintln mirrors println (Req → peer → Result-carrying reply; error arm SURFACES) | read the diff |
| 6 | ThreadIO: stderr_tx/stderr_ack_rx dead; stderr_reply_rx live; thread_id renamed | read the diff |
| 7 | freeze.rs boots stderr peer; drop-order sound (RS dropped before BOTH peer joins) | read the diff |
| 8 | MiniUniverse boots both write peers; eprintln_and_read; row E reborn | run alpha helpers test |
| 9 | lib + nursery + alpha helpers + check --all-targets + clippy green | orchestrator re-runs |
| 10 | FULL CORPUS green (integration-run.sh) — once per slice-stone per the 5.1 lesson | orchestrator runs at score |

## Independent prediction

- Runtime band: **12–22 min Mode A** (the 8.1 template is complete; this is
  the mechanical second application + a rename cascade the type system
  drives). 2× cap = 45 min.
- Expected friction: ThreadIO re-shape sites across the test corpus (the
  rename names them via cargo check --all-targets); the MiniUniverse
  two-pipe extension.
- The orchestrator's class-greps at score: old-name grep
  (`StdOutServiceMsg\|StdOutServicePeer\|spawn_stdout_service_peer\|stdout_thread_id`)
  must return only historical comments/docs; `StdErrServiceEvent` must be
  zero.

## Calibration record

- 8.1 (the template's first application): sonnet flailed 30 min on the rig
  (puppet-class + missing doctrine) → Mode-B kill + orchestrator diagnosis.
  THIS brief embeds the doctrine + the true-universe rig already exists —
  the two upstream defects are gone.
