# EXPECTATIONS — every sweep names what it reads (report-only)

| # | what | how it is judged |
|---|---|---|
| 1 | ⭑⭑ **One table per sweep**: input × user-writable? × how closed | four sweeps, reported separately. A merged table fails this row — it hides the asymmetry that bit Tier B. |
| 2 | ⭑⭑ **The completeness method, named, with its stopping rule** | transitive callee closure from each entry point, or something better — and say where you stopped and why that is the frontier. |
| 3 | ⭑⭑ **Lines read, cited** | `file:line` for each door. The eighth door was found by reading; this row is the reading. |
| 4 | ⭑⭑ **Every input classified user-writable or not, with the mechanism** | ⛔ "keyed by `:wat::`" is closed by construction; "keyed by any name" is NOT, whatever the prefix. |
| 5 | ⭑ **Where the closure could not be completed** | dynamic dispatch, trait objects, runtime-keyed registries — name the exact site. This is what B must wall. |
| 6 | ⭑ **Cross-check against the instrument, without relying on it** | `src/spike_probe.rs` covers body-infer only. Say where reading and instrument agree and where they diverge. |
| 7 | ⛔ **Nothing changed** | `git diff` shows the finding (and at most a throwaway probe). No behaviour, no perf, no fixes. |
| 8 | ⭑ **Floor untouched** | confirm rather than assume. |
| 9 | ⭑ **The verdict for B and C** | given the table: is a structural gate (B) sufficient to close every user-writable door, and if not, which ones need something else? A recommendation, not a decision. |
