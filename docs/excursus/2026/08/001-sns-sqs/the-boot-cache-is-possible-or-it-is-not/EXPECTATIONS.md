# EXPECTATIONS — the boot cache is possible, or it is not

| # | what | how it is judged |
|---|---|---|
| 1 | ⭑⭑ **Can the CHECK be cached, or only the expansion?** | answered first and explicitly. 51 % vs 83 % turns on it. If `check:body-infer`'s output has no serialisable form, say so — that is the finding. |
| 2 | ⭑⭑ **The state, split into data vs handles** | `TypeEnv` / `MacroRegistry` / `SymbolTable` / expanded `Vec<WatAST>`: what is pure data, what is a Rust handle, and is the handle side **re-registerable by name** at load? |
| 3 | ⭑⭑ **A round-trip, timed, at representative size** | both directions, **warm and cold**. Say what payload you used and why it is representative. |
| 4 | ⭑⭑ **Which conclusion your number supports** | a fast crude prototype proves GO; a slow one proves **nothing** (zero-copy could be an order faster). State this explicitly rather than letting a number imply more than it can. |
| 5 | ⭑⭑ **GO or NO-GO, with the fraction of 430 ms it would remove** | the deliverable. A recommendation, not a decision. |
| 6 | ⭑ **Boot unchanged** | three runs on the default path, and stderr silent; the prototype behind a test/feature/env gate. |
| 7 | ⛔ **Nothing else optimised; `.config/nextest.toml` untouched** | `git diff` and a sha256. |
| 8 | ⭑⭑ **Floor** | Summary verbatim, `.floor/<stamp>/`, `ARM.txt` or not. |
| 9 | clippy + `--no-run` | `--release --workspace --all-targets`. |
| 10 | ⭑ **What you could not determine** | an honest ABSENT beats a plausible GO. If the answer needs a spike nobody has done, name the spike. |
