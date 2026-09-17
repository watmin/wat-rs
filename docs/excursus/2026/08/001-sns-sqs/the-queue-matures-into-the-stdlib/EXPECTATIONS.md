# EXPECTATIONS — the queue matures into the stdlib

| # | what | how it is judged |
|---|---|---|
| 1 | ⭑⭑ **Zero `:user::` names in `wat/queue.wat`** | grep it. Any hit is the defect this stone exists to prevent. |
| 2 | ⭑⭑ **The five rust probes still pass** | `probe_queue_long_poll`, `probe_queue_depth`, `probe_ex001_queue`, `probe_async_publish`, `probe_queue_visibility` — name them and their results. They rendezvous with the half that STAYS. |
| 3 | ⭑⭑ **The migration is a recorded codemod** | `wat-scripts/fixes/<name>.wat`, dry-run diffed before applying, **idempotent** (re-run = 0 changes — show it). No python/sed touched a `.wat`. |
| 4 | ⭑⭑ **Load order holds** | `wat/queue.wat` at manifest 50 names nothing defined after it. State how you verified, beyond "it built". |
| 5 | ⭑ **`sns-fanout.wat:41`'s `load-file!` is resolved** | say what it became and why the definitions do not arrive twice. |
| 6 | ⭑⭑ **Floor** | Summary verbatim, `.floor/<stamp>/`, `ARM.txt` or not. |
| 7 | ⭑⭑ **The circuit is byte-identical** | `distinct=8000;dup=0` **and** `timeout=yes;discarded=yes;redial=Connected;retry-on=fresh`. A relocation that changes behaviour has failed. |
| 8 | ⭑ **The load gate passes** | `every_wat_scripts_file_loads` — the whole `wat-scripts/` corpus still parses and type-checks. |
| 9 | ⭑ **What moved vs what stayed** | the split, name by name, with the reason. The client-API/driver line is the stone's judgement and must be reviewable. |
| 10 | clippy + `--no-run` | `--release --workspace --all-targets`. |
| 11 | ⭑ **Cost as a BAND** | floor against 540.4–561.5 s; circuit against ~23 s. |
| 12 | ⛔ **No behaviour change claimed or made** | and no brackets work. Say so explicitly. |
