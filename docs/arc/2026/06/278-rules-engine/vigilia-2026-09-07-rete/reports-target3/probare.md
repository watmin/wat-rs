# PROBARE — Cast Report (TARGET 3 — the grid) — **FINDINGS**

> Written verbatim as returned, before any synthesis. Evidence only — this file carries NO status.

### 1. What I swept

- Re-derived file/line inventory: `find . -type f -name '*.EXT' | wc -l` per extension, and `cat *.EXT | wc -l` — confirms the brief's **148 files / 16,616 lines** (54 `.wat`, 43 `.clj`, 20 `.sh`, 29 `.txt`, 2 `.md`) exactly.
- Re-derived comment fractions with `grep -h '^\s*#'`/`'^\s*;'` and blank-line counts per extension — confirms the brief's **795/2398 (.sh), 973/3568 (.clj), 2546/8689 (.wat)** exactly, plus blanks (161/551/816) which the brief didn't give.
- Ran the widened self-claim sweep across **all five extensions** (not just `.sh`/`.md` as the handed-down grep did): `[0-9]+ of [0-9]+`, `[0-9]+(\.[0-9]+)?%`, spelled-out numbers near axis/file/form nouns, `always` claims. Read every hit's surrounding context; discarded ones that were per-record data values (e.g. `where-record.wat`'s `60 of 200` match-count comments describing generated test data, not the instrument itself).
- For every retained candidate, checked it against the disk (`ls`, `grep -c`, `wc -l`) and, where the claim was date-stamped, against `git log`/`git show`/`git blame` (read-only, permitted) to find the count *at authoring time* vs. *today*.
- Sampled `.wat`/`.clj` form-density on the full corpus (not just a subset) since `wc`/`grep -c` over all 54+43 files is cheap and exact — no sampling was actually needed.
- Did not execute any `.sh`/`.wat`/`.clj` in this directory.

### 2. Density assay (canonical measure) — contradicts the brief's suggested verdict

| ext | total | comment | blank | form (total−comment−blank) | ratio (form:comment) | verdict |
|---|---|---|---|---|---|---|
| .sh | 2398 | 795 | 161 | 1442 | 1.81:1 | **mixed** |
| .clj | 3568 | 973 | 551 | 2044 | 2.10:1 | **mixed** |
| .wat | 8689 | 2546 | 816 | 5327 | 2.09:1 | **mixed** |

Using the spell's *stricter* Lisp definition — a form is a line beginning with `(`, not just "any non-comment line" — the ratio is even lower: `.wat` 3631 form-starts / 2546 comments = **1.43:1**; `.clj` 1173/973 = **1.21:1**. Both methods land the two Lisp-family extensions inside the **mixed (1:1–3:1)** band, not the >3:1 **substance-rich** band the brief said "the numbers suggest."

**Verdict, stated plainly: this corpus is MIXED, not substance-rich**, under either form-counting convention. It is not prose-heavy or a wish (nowhere near <1:1) — it's real spec+commentary in roughly 1.2–2.1 forms per comment line, which the spell explicitly says to treat as "spec + commentary, check the purpose declaration" rather than "substance-rich." The `.md` files (513 lines total) are description-by-purpose (translation/tracking docs) and correctly exempt from the density metric.

### 3. Self-claim sweep — table

| claim | location | derived truth | verdict |
|---|---|---|---|
| "Thirteen `GRID-native-vs-clara-*.txt` files sit in this directory" | `compare-grids.sh:6` | At the commit that added this line (`d9fb1b88f`, 2026-09-02), **19** such files already existed in the directory before this commit's own 3 new ones (→22 after). Today: **25**. Never 13 at any point in this file's history. | **FALSE** — wrong even when written (commit message itself hedges "Thirteen-plus"; the file text doesn't) |
| "The other nine axes sweep a SIZE ladder" / "The nine perf axes are where speed is measured" | `check-where-shapes.sh:14`, `:29` | Blamed to commit `30d15b4410` (2026-08-01), where exactly **9** `gen-*.sh` perf axes existed — accurate then. `gen-neg-consumer.sh` (08-13) and `gen-leading-exists.sh` (08-24) were added after, and were never back-filled into this comment. Current count: **11**. | **STALE / now FALSE** (off by 2) |
| "every one of the 47 recorded `GRID-*.txt`" / "0 of 47 carry `:oracle-accuracy`" | `check-grid-three-way.sh:16-17` | The commit that introduced this (`545771b2f`, 2026-09-03) states in its own message "47 recorded grids" — accurate then, spanning **both** `wat-scripts/perf/grid/` (then 26, now 29) **and** `docs/arc/2026/06/278-rules-engine/` (21, unchanged) = 47. Three more grid files landed since (2026-09-04, two on 2026-09-06) → true total today is **50**. The *ratio* (0 carry `:oracle-accuracy`) still holds at n=50 — verified `grep -l ':oracle-accuracy' *.txt` returns nothing in either directory. | Numerator **TRUE**, denominator **STALE** (47→50); also the line never discloses it spans a directory outside `wat-scripts/perf/grid/` |
| "that test FALSE-POSITIVED 3 of 33 cells on two same-build sweeps (2026-09-02)" | `compare-grids.sh:112` | Three `GRID-native-vs-clara-2026-09-02T*.txt` files exist, each with exactly **33** `#grid/Verdict` lines and `:runs 5` — structurally consistent with the claim. But the "3 false positives" number depends on an algorithm (`dw`/`dr` delta-vs-spread test) that **compare-grids.sh no longer runs** — it was replaced by the disjoint-interval test the very next lines describe — and the comment names no specific pair among the three same-day files. | **unverifiable-as-stated** (population checks out; the specific count can't be re-derived from current code+data without re-executing a retired algorithm) |
| "Every one of the other 36 `where-*` axes writes its predicate in a FENCE" | `where-inline-computed.wat:5` | 38 `where-*.wat` files exist. Excluding `where-inline-computed.wat` itself and `where-inline-keyword.wat` (also documented, at its own header, as an inline-predicate-position axis) leaves exactly **36**. | **TRUE** |
| "21 of 21 :winner :us, 21 of 21 :accuracy :match" | `GRID-2026-07-31.txt:41` | The file logs 29 verdict lines, but several supersede earlier entries for the same (axis,size) after re-runs. Deduping to the canonical latest verdict per (axis,size) as of that line yields exactly **21** distinct pairs, all `:winner :us`, all `:accuracy :match`. | **TRUE** |
| Header: "nine axes, 27 rungs, all :us" + "27 / 27 :winner :us" + "27 / 27 :accuracy :match" | `GRID-2026-08-01.txt:1,6-7` | `grep -c '^#grid/Verdict'` = 27; distinct `:axis` values = 9; `:winner`/`:accuracy` tallies both 27/27. | **TRUE**, exact |
| "the seam claimed '21 of 21 :us' — seven axes of nine" | `GRID-2026-08-01.txt:45` | `GRID-2026-07-31.txt` covers exactly **7** distinct axes (missing `deep-cascade` and `fanout`, i.e. A0/A1, which the same file explains were "not in the runner" yet); current runner covers 9. | **TRUE** |
| "TIGHTEST CELL: fanout [40000] min 1.0794 — one run of three within 8% of parity" | `GRID-2026-08-01.txt:58` | `|1.0794−1.0|=7.94%≈8%`; scanning all 27 cells' `:min` values, 1.0794 is the value closest to 1.0 in the whole file. | **TRUE** |
| "108 cells" / "8 = (2-1)+(2-1)+(3-1)+(3-1)+(3-1)" | `peragrare-census.sh:64,233,236` | Dimension cardinalities implied (H,R,A,L,N = 2,2,3,3,3): 2×2×3×3×3 = 108. Internally self-consistent. | **TRUE** |

### 4. What I looked for and did not find

- No new instance of the four already-rowed false claims (`CLARA-TRANSLATIONS.md:418`, `check-where-shapes.sh:22`, `run-all.sh:23`, `CLARA-TRANSLATIONS.md:363`) beyond their existing citations.
- Widened `%`-claims, `always`-claims, and spelled-out-number-near-noun greps across `.wat`/`.clj` (not just `.sh`/`.md`) turned up only qualitative engine-behavior claims (e.g. "hash-join always reads the other side's complete persistent state") that require executing code to falsify — out of scope for a read-only cast, and not the corpus-shape claims `probare`'s clause targets.
- No `rune:probare(...)` exemptions anywhere (consistent with the four prior wards' zero count).
- `run-all.sh:6`'s "20 of 22" (referenced only, not asserted as current) traces to a table that lived in a `docs/arc/` document outside this directory's scope and was never reproduced inside `wat-scripts/perf/grid/` at any commit — I did not chase it further since verifying it would require reading files outside the target directory beyond what a `git log -S` search incidentally surfaced.
- `check-grid-three-way.sh:296` "expected exactly 1" is a runtime field-cardinality guard, not a static self-claim about the corpus — not applicable to this clause.

**FINDINGS**
