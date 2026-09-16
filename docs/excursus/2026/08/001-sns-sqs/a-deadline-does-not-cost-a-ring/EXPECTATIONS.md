# EXPECTATIONS — a deadline does not cost a ring

⛔ **Rows 1 and 2 gate everything. If row 1 is ABSENT, stop and report** — do not proceed to a fix.

| # | what | how it is judged |
|---|---|---|
| 1 | ⭑⭑ **The CI red reproduces LOCALLY** | `( ulimit -l 64 ; cargo nextest run --release -E 'test(chaos_gate)' )` fails carrying the **same arm**: `IoUring::new(4) … Cannot allocate memory (os error 12)`. Quote it verbatim. A different arm = a different bug = STOP. |
| 2 | ⭑⭑ **The limit at which it flips** | binary-search `ulimit -l` and report the threshold (fails at N, passes at M). This is the headroom number the fix has to move. |
| 3 | ⭑ **Rings per deadline, counted** | show that one `(:wat::kernel::after …)` on the process tier costs one `IoUring::new(4)` — by instrumentation or by strace/counter, not by reading the source. |
| 4 | ⭑⭑ **The fix passes AT THE SAME LOWERED LIMIT** | row 1's exact command, now green. A pass at the default 8192 KB is **not** this row. |
| 5 | ⭑ **The thread tier still has no ring** | `src/runtime.rs:27796`'s crossbeam path unchanged; state how you verified. |
| 6 | **The `.expect` at `src/comms/process.rs:1105` is gone** | ring-creation failure is an `io::Error`, never a panic. |
| 7 | ⭑⭑ **Floor** | `./scripts/floor.sh`, Summary line verbatim, `.floor/` path, `ARM.txt` present or not. |
| 8 | clippy + tests compile | `cargo clippy --release --workspace --all-targets`; `cargo nextest run --release --no-run`. |
| 9 | happy path | the circuit one-liner → `distinct=8000;dup=0`. |
| 10 | ⭑ **Cost, if any, is reported as a BAND** | if the change touches the hot path, give the circuit wall-clock against the 540.4–561.5 s floor band / the ~23 s circuit run — never a single-number delta. |
| 11 | ⛔ **What is NOT fixed is stated** | `after` still raises rather than returning an outcome unless the builder ruled otherwise. Say so plainly. |
| 12 | blast radius | `git status --short`; no `wat/` changes. |
