# EXPECTATIONS — one ring per thread, lazily created

Report each row with what you **observed**. A row you could not drive is **ABSENT**, never assumed.

| # | what | how it is judged |
|---|---|---|
| 1 | ⭑⭑ **Ring count tracks THREADS, not waiters — DRIVEN** | a probe creating many process-tier peers/timers **on one thread** shows the ring census (`rings created-so-far=N`) **flat**, not climbing per waiter. Quote N before and after. This is the stone. |
| 2 | ⭑⭑ **Lazy** | a thread doing no process-tier IO creates **zero** rings; show it. |
| 3 | ⭑⭑ **The thread tier still allocates NO ring** | `runtime.rs:27796`'s crossbeam path unchanged; state how you verified. |
| 4 | ⭑⭑ **The wat surface did not move** | `git status` shows **no `wat/` file changed** — or each change is justified in the SCORE. No new wat verb. No outcome-enum variant added/removed/re-meaninged. |
| 5 | ⭑⭑ **Floor** | `./scripts/floor.sh`, Summary verbatim, `.floor/<stamp>/`, `ARM.txt` present or not. |
| 6 | ⭑⭑ **The lifecycle rows still pass** | the manifest runner's tests (`wat::probes`) — they exist so a substrate change cannot quietly break an invariant proven once by hand. Name them. |
| 7 | ⭑ **Circuit byte-identical** | `distinct=8000;dup=0` **and** `timeout=yes;discarded=yes;redial=Connected;retry-on=fresh`. |
| 8 | ⛔ **No `RefCell` double-borrow anywhere** | trap-door 1. Drive select-over-N **with** a Receiver method called inside the same path; a panic here is the expected failure if the borrow discipline is wrong. Say what you drove. |
| 9 | ⛔ **Fork rebuilds, never reuses** | a forked child must not inherit and reuse a pre-fork ring. DRIVEN, with the process-tier tests as the witness. |
| 10 | ⭑ **`ring-ceiling` under concurrency** | run it; report whether the per-UID ceiling is still reachable by a 4-way parallel run. |
| 11 | clippy + tests compile | `--release --workspace --all-targets`; `nextest --no-run`. |
| 12 | ⭑ **Cost as a BAND** | floor wall-clock against the 540.4–561.5 s band; circuit against ~23 s. Never a single-number delta. |
| 13 | ⛔ **What is NOT done is stated** | the `ulimit -l` stopgap removal is the ORCHESTRATOR's acceptance test (it needs a CI run); `after`'s unfaceable raise is untouched; ATTACH_WQ/registered-files/SQPOLL/batching are out of scope. |
