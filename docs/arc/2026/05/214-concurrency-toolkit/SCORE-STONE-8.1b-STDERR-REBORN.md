# SCORE — Stone 8.1b: StdErrService reborn (the write-pair generalization)

**Mode A.** Sonnet flight: ~14 min (predicted band 12–22 — calibration HOLDS;
the 8.1 lesson worked: doctrine embedded + true-universe rig pre-existing =
zero flail). One orchestrator scoring catch fixed before commit (R1 below).

## Scorecard (every row = orchestrator's own re-run/read)

| # | Row | Result |
|---|-----|--------|
| 1 | Gate-probe 81b 2/2 GREEN | ✓ own run (arc214 nursery filter: 58/0) |
| 2 | stderr.wat = 44 lines / 3 forms (Req + Rep + ONE pure handle) | ✓ read whole — exact stdout.wat mirror; 303→44 |
| 3 | Write-pair generalization: ONE `spawn_write_service_peer(label,…)`; both services instantiate; zero loop duplication | ✓ read diff — label feeds thread-name + diagnostics; loop body byte-identical to the proven 8.1 loop |
| 4 | Old stderr machinery DEAD | ✓ class-grep: `StdErrServiceEvent\|spawn_stderr_bridge\|StdErrService/spawn\|StdErrService::Event` → 3 retirement-record comments only (Bucket C) |
| 5 | eval_kernel_eprintln mirrors println | ✓ read diff — Req struct → `services.stderr_ctrl` → `Ok(Ok)/Ok(Err→MalformedForm "stderr write failed")/Err→ChannelDisconnected` |
| 6 | ThreadIO re-shape (`stderr_reply_rx` in; `stderr_tx`/`stderr_ack_rx` dead; `stdout_thread_id`→`thread_id`) | ✓ read diff + class-grep `stdout_thread_id` → ZERO |
| 7 | freeze.rs: both peers boot labeled; drop-order sound (deregister → uninstall → drop sym → drop RS → join stdin/stdout/stderr); dead Option fields (`stdout_thread_value`/`stderr_thread_value`) purged | ✓ read diff whole |
| 8 | MiniUniverse boots BOTH write peers; `eprintln_and_read`; finish() deregisters both → drops every sender → joins both | ✓ read diff; alpha helpers 7/0/3 in 0.02s (own run) |
| 9 | lib 943/0/1 · check --all-targets 0 errors · clippy: touched surface clean | ✓ own runs |
| 10 | FULL CORPUS (integration-run.sh): **649/0/54, error-class histogram all-zero** — identical to the 8.1 baseline | ✓ own run |

## Orchestrator scoring catches

**R1 — the back-compat aliases (FIXED pre-commit).** Sonnet reported "Deltas
from BRIEF: None" while shipping `pub type StdOutServiceMsg = WriteServiceMsg`
+ `StdOutServicePeer` alias + a `spawn_stdout_service_peer` wrapper the BRIEF
never asked for (the BRIEF said *sweep every reference*). Two names for one
thing in a warded home is the names-must-not-lie violation 8.1w-R2 existed to
kill. Class-grep showed the aliases served exactly ONE caller — thread_io.rs's
own re-export + one `::Req` send; every real consumer already spoke canonical.
Orchestrator killed the alias block, re-pointed the 2 sites, re-ran all gates.
Lesson re-confirmed: **the cast reads a sample; the class-grep reads the set**
— and an unreported delta is still a delta.

**R2 — the BRIEF's own Gate-2 defect (FM-9, orchestrator-owned).** The BRIEF
demanded "nursery → fully green" without a pre-flight baseline re-run. The
committed HEAD has 4 deliberately-RED parked probes (arc-255
reflection-parity ×2 + undefined-builtin ×2 — the banked 255 disconfirming
gates). Stash round-trip PROVED the baseline: HEAD = 849/6/3 (my 2 strike-RED
81b gates + those 4); post-work = 851/4/3 (+2 green, −0 regressions). Sonnet
handled the impossible gate honestly (its own checkout verification) instead
of "fixing" parked probes — exactly the right behavior under a defective gate.

**Verified-true claims:** sonnet's "4 pre-existing failures" — stash-proven.
"ProcessPanics path untouched" — git diff shows zero hunks in spawn_process /
fork / panic_hook / process_stdio. "clippy clean" was imprecise (239
pre-existing flat-file warnings exist) but the per-home gate (the standard)
holds: src/services/ zero findings; no touched hunk introduces one.

## The annihilation map advances

thread_io.rs: 979 → 903 lines (the commit message's "821" was wrong — this is
the verified `wc -l`), holding ONLY stdin's old path (StdInServiceEvent,
the stdin bridge, ThreadIO's stdin half) + the eval arms + RuntimeServices +
registration. **8.2 (stdin reborn — the reply-routing proof, un-ignores the 3
readln tests) takes the quarry to ZERO + git rm.**
