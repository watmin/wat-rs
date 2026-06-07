# EXPECTATIONS — Stone 8.2: StdInService reborn

## Scorecard

| # | Row | Verification |
|---|-----|--------------|
| 1 | Gate-probe 82 GREEN (2/2) | orchestrator re-runs |
| 2 | stdin.wat ≈ 20 lines / 3 forms; EOF doctrine comment travels into the handle's None arm | read the file |
| 3 | Trio generalization: ONE `ServiceMsg<R>` / `ServicePeer<R>` / `spawn_service_peer`; three instantiations; ZERO aliases (8.1b R1 lesson) | read src/services/mod.rs + class-grep `WriteServiceMsg\|WriteServicePeer\|spawn_write_service_peer` → zero live |
| 4 | Old stdin machinery DEAD incl. the bridge-layer helpers (`make_event_value`, `unwrap_value_*`, `*_value`, `extract_control_tx`, freeze's `spawn_service`/`join_service`) | class-grep each |
| 5 | eval_kernel_readln: transport swapped, `-> :T` parsing + EDN coerce verbatim; triage incl. `Ok(Err)` → "stdin read failed" | read the diff |
| 6 | Reply-routing proof: two tids, two lines, no crossing | own run (alpha helpers) |
| 7 | EOF cascade: feed-writer dropped → Req → caller disconnects; stdin loop join is Err BY DESIGN | own run + read the test |
| 8 | Rows C/F/J UN-IGNORED (ignore-drawdown −3) | own run; grep `#[ignore` in alpha helpers |
| 9 | lib 0-fail · nursery no-new-reds vs the 4 parked · check --all-targets · clippy touched-surface | own runs |
| 10 | FULL CORPUS (integration-run.sh) green | orchestrator at score |

## Independent prediction

- Runtime band: **20–35 min Mode A** (bigger than 8.1b: the generic
  refactor + three revivals + two new proof tests). 2× cap = 70 min.
- Trap-doors: (a) the generic fn-pointer signature vs closure capture —
  the extractors need no captures, fn pointers suffice; (b) the EOF test's
  teardown (panicked join) — must not reuse the clean finish(); (c) row J's
  original assertion may be deeper-stale than the annotation migration —
  honest delta if its claim needs updating to the 1f-ι contract.
- Orchestrator class-greps at score: `StdInServiceEvent`,
  `spawn_stdin_bridge`, `WriteServiceMsg`, `spawn_write_service_peer`,
  `make_event_value`, `extract_control_tx`, `spawn_service\b`,
  `stdin_thread_value` → all zero-live.

## Calibration record

- 8.1b: 14 min actual vs 12–22 predicted (Mode A; one unreported-delta catch
  R1). The 8.2 band widens for the generic refactor + test revivals.
- The annihilation map after this stone: thread_io.rs holds ONLY the
  surviving forms (ThreadIO + thread-local + eval arms + register/deregister
  + RuntimeServices + ThreadId + next_thread_id) — ALL of which are live,
  perfected machinery. **8.2w (next) lifts the survivors into src/services/,
  git rm's thread_io.rs, and casts the FULL VIGILIA on the completed home**
  (the ward note's trio-completion stamp).
