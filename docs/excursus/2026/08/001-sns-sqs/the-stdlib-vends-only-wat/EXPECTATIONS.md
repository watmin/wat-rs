# EXPECTATIONS — the stdlib vends only `:wat::`

| # | what | how it is judged |
|---|---|---|
| 1 | ⭑⭑ **Zero non-`:wat::` names vended, measured from the SYMBOL TABLE** | not grep — grep got this wrong once (6 false files unanchored, 3 real anchored). Report the count and the instrument. |
| 2 | ⭑⭑ **A gate enforces it** | a test that fails if a future stdlib file vends outside `:wat::`. Drive it: add such a name temporarily, show the gate red, remove it. |
| 3 | ⭑⭑ **The witness is DEAD** | `(:wat::core::defclause :repl::turn …)` + a trivial `main` must no longer change any stdlib verdict. Show before/after. |
| 4 | ⭑⭑ **A distributed binary still gets a REPL** | `src/distribution/mod.rs` embeds a program calling `(:repl::turn …)`. Driven, not reasoned. |
| 5 | ⭑ **The codemod is recorded and idempotent** | `wat-scripts/fixes/`, dry-run diffed, re-run = 0 changes. No python/sed on `.wat`. |
| 6 | ⭑ **`check.rs:848`'s comment is updated** | it argues from a fact this stone removes; a prefix test is now valid. |
| 7 | ⭑⭑ **Floor** | Summary verbatim, `.floor/<stamp>/`, `ARM.txt` or not. Tier A's ~252–259 s should hold. |
| 8 | ⭑ **`:wat::` reservation did not refuse the stdlib's own rename** | trap-door 3. If it cannot tell stdlib from user, say so — that is a bigger finding than the rename. |
| 9 | clippy + `--no-run` | `--release --workspace --all-targets`. |
| 10 | ⛔ **Tier B not built; `defclause` not "fixed"** | `git diff` scope. |
| 11 | ⭑ **The new attack-surface count** | re-run the spike's measurement against the renamed tree: how many user-declarable names remain reachable during the stdlib sweep? The answer should be 0. |
