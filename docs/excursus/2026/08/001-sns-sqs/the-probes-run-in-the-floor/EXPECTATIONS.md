# EXPECTATIONS — the probes run in the floor

| # | what | how it is judged |
|---|---|---|
| 1 | ⭑⭑ **Each seed row is its own nextest test and appears by name in the Summary** | quote the new test names from the run. A loop over rows fails this row. |
| 2 | ⭑⭑ **Every row's markers are unique to the PASSING world** | for each row, say what the failing world printed instead. A row that would pass on a broken probe is a fence around nothing. |
| 3 | ⭑⭑ **A deliberately broken row FAILS, driven** | break one probe's expectation (or the probe) and show the runner going red with the **whole** captured output, not a window. Restore it. |
| 4 | ⭑ **A hanging probe is a FAIL, not a hang** | drive a timeout and show the runner SIGKILLing and reporting. A blocked `wat` ignores SIGTERM — 125 s measured. |
| 5 | ⭑ **A missing/renamed path is a FAIL** | point a row at a path that does not exist; the runner must go red, not skip. |
| 6 | ⭑⭑ **Every seed row's expectation came from a SCORE** | cite the SCORE and the driven line for each row. A row derived from reading the probe is not admissible. |
| 7 | ⭑⭑ **Floor, with the cost as a BAND** | Summary verbatim + the added wall-clock against the 540.4–561.5 s band. If the seed rows cost more than ~15 s, STOP and report rather than growing the floor. |
| 8 | ⭑ **No flaky assertion was added** | confirm no row asserts a count or rate; `probe_chaos_gate_has_teeth`'s header is the precedent (*"do not assert `fires > 0` at 200 bp"*). |
| 9 | clippy + `--no-run` | `--workspace --all-targets`. |
| 10 | ⛔ **What is still unguarded is stated** | how many of the 382 remain parsed-only after this stone, and which of this session's invariants still have no row. |
