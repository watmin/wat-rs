# EXPECTATIONS — can a user def change a stdlib verdict?

| # | what | how it is judged |
|---|---|---|
| 1 | ⭑⭑ **A verdict, in one word: SOUND / UNSOUND / UNKNOWN** | with the evidence that supports it, and an explicit statement of which kind of evidence it is. |
| 2 | ⭑⭑ **If UNSOUND: a reproduction** | a user program + the stdlib function whose verdict changes, runnable, with both outcomes shown. |
| 3 | ⭑⭑ **If SOUND or UNKNOWN: the path, read** | which state body-infer consults that user code can reach, and why every route from `defined_values` to a stdlib verdict is closed. ⛔ "I tried N things and none worked" is UNKNOWN, not SOUND — label it honestly. |
| 4 | ⭑ **The five attacks, each tried and reported** | redefinition · type shadowing · `extend-type` · ambiguity · `defclause`/acronym. Say what each did. |
| 5 | ⭑ **`redef_allowed`: default state, and whether the answer depends on it** | a soundness claim that holds only in the default configuration must say so. |
| 6 | ⭑⭑ **Reach vs consequence kept apart** | a user def being *visible* during stdlib inference is already known. A witness must change a **verdict**. Do not conflate them. |
| 7 | ⛔ **Tier B not built; nothing "fixed"** | `git diff` shows a probe and a finding, nothing else. |
| 8 | ⭑ **Floor** | it should be untouched by a read-and-probe stone; confirm rather than assume. |
| 9 | ⭑ **What would settle an UNKNOWN** | the specific experiment or reading, named precisely enough that someone else could run it. |
