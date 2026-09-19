# Intueri Cast Report — `wat-scripts/perf/grid/`

## 1. What I swept

**Inventory re-derivation** (`find . -type f | wc -l`, `find . -type f -exec cat {} + | wc -l`, per-extension counts): **148 files, 16,616 lines** — 54 `.wat`, 43 `.clj`, 29 `.txt`, 20 `.sh`, 2 `.md`. **No delta** from the brief's numbers.

**Read in full (2,398 lines, all 20 `.sh`):** `run-axis.sh`, `check-grid-three-way.sh`, `peragrare-census.sh` (mine, audited without special standing), `check-where-shapes.sh`, `check-query-compat.sh`, `run-all.sh`, `compare-grids.sh`, `check-spec-native.sh`, `check-grid-speed.sh`, and all eleven `gen-*.sh`.

**Read in full (513 lines, both `.md`):** `CLARA-TRANSLATIONS.md` (422 lines), `REMAINING-CLARA-MOUTHS.md` (91 lines).

**`.wat`/`.clj` sample (stated rule):** I read **all 16 sized/correctness axes** (the shared subject of `run-axis.sh` / `check-grid-three-way.sh`'s `SIZES` table — every non-`where-*` `.wat`) — 4 read completely end-to-end (`negation.wat`, `retract-multiplicity.wat`, `neg-consumer.wat`, `asym-join.wat`), the other 12 read via their full documentation header plus code-shape skim (`accum.wat`, `strat-neg.wat`, `deep-cascade.wat`, `fanout.wat`, `min-finding.wat`, `node-share.wat`, `user-reduce.wat`, `leading-exists.wat`, `accum-over-derived.wat`, `accum-lead-rule-cascade.wat`, `userfn-head.wat`, `accum-lead-rule-cascade.wat`). I then sampled **8 `where-*.wat` files** for naming-convention spread (`where-not-and-bound`, `where-not-and-not`, `where-query-params`, `where-accum-group`, plus headers of `where-boolean`, `where-join-left`, `where-shapes`, `where-fact-bind`) and **2 `.clj` twins** (`where-not-and-bound.clj`, plus the 11 `gen-*.sh`-embedded Clara programs, which are themselves `.clj` source). **Not read**: the remaining ~38 `where-*.wat`/`where-*.clj` pairs (beyond header skims of a few), and none of the 29 `.txt` grid-result files (out of scope per the brief — data, not instrument).

## 2. Findings

### Finding A (Level 1 — lies) — `REMAINING-CLARA-MOUTHS.md:1` vs `:10,22,34,47,57,69,78,88`
The file's own name promises an open worklist ("Remaining Clara mouths"). Every one of its 7 numbered items is marked **`— DONE`** (lines 10, 22, 34, 47, 57, 69, 78), and the file's own closing section header at line 88 is literally **`## This list is empty.`** A reader arriving at this filename — which two other files in this corpus (`check-where-shapes.sh:52`, `check-spec-native.sh`) point at as a locked reference — gets a name that says "here is what's left" and a body that says "nothing is left." The name was true once; it has not been true since 2026-08-17 (line 90) and nothing renamed it.
**Direction:** rename to something the closed content actually states, e.g. `CLARA-MOUTHS-CLOSED.md` or fold the still-useful cross-reference (the `check-query-compat.sh` pointer at line 63-67) into `CLARA-TRANSLATIONS.md` and retire this file.

### Finding B (Level 1 — lies) — `run-all.sh:46`
```
[neg-consumer]="250|500|1000"    # dial: items. THE THREE-WAY AXIS — ... RED until task #94 is closed.
```
`git blame` dates this line to commit `839d02a3` (2026-08-13 11:16:51 -0700). Task #94 was closed **the same day**, an hour later, by commit `ff581b6f` (2026-08-13 12:20:31 -0700, message: *"278: #94 CLOSED — the stratifier now propagates POSITIVE dependencies, oracle then native"*). `neg-consumer.wat:38-43` (the axis's own header) confirms this explicitly: *"THIS AXIS FOUND AND THEN CLOSED task #94 ... Fixed in ff581b6f ... It must now read :accuracy :match on ALL THREE columns; any MISMATCH is that regression returning."* `run-all.sh`'s comment has told every reader for **26 days** that this axis is still red, directly contradicting the axis's own file.
**Direction:** delete "RED until task #94 is closed" or replace with `#94 closed ff581b6f 2026-08-13 — a MISMATCH here is a regression of that fix, not an open item`.

### Finding C (Level 2 — mumbles, stale claim) — `CLARA-TRANSLATIONS.md:418`
```
No axis required marking UNVERIFIED ... fully grounded all six forms above.
```
`git log -p` shows this sentence introduced at `5fcf4bda` (2026-07-03), when the document held exactly 6 axis sections (A2, A3, A5, A6, A7, A8). It was **not updated** when A9 was added at `372e65f6` (2026-08-24) or when A10 was added at `545771b2` (2026-09-03) — both diffs leave this line untouched. Today the document has **8** axis sections (A2, A3, A5, A9, A6, A7, A8, A10, confirmed via `grep -n "^## A"`), and the summary table immediately above the sentence (lines 407-415) has **7** rows — it never gained an A9 row at all, despite A9 having a full, detailed section at lines 160-201 (its own worked bug story, the strongest section in the doc). "Six" is wrong by one against the table it's summarizing and wrong by two against the document's actual content.
**Direction:** add the missing A9 row to the summary table; update "all six forms" to the true count (or drop the number and say "every form above").

### Finding D (Level 2 — mumbles, vague date) — `peragrare-census.sh:32` (mine, no special standing)
```
;      vs stratum-by-fn-name (the userfn-head.wat cure, closed 2026-09-0x).
```
`userfn-head.wat` was added in a single commit dated **2026-09-07** (`git log --format="%H %ad" -- userfn-head.wat`), one day before `peragrare-census.sh` itself was committed (2026-09-08, `2961853f`). The exact date was sitting one `git log` call away when this line was written, and the file elsewhere (line 41: "Measured 2026-09-07") shows the author *had* the precise date on hand that same session. "2026-09-0x" reads as a placeholder that was never resolved — a WHY/WHEN comment that could have been precise and wasn't.
**Direction:** `closed 2026-09-07`.

## 3. What I looked for and did not find

- **Runes**: grepped `rune:` across the full sample — zero hits, consistent with the brief's claim of zero exemptions in this directory.
- **Stale `run-axis.sh`/`check-*.sh` self-claims**: every dated incident narrative in `run-axis.sh` (the freshness wall, the blast door, the repeats section) and in `check-grid-speed.sh` (the CI-gate rationale) cross-checks against the code immediately below it — no drift found there.
- **Filename-vs-content mismatches among `gen-*.sh`**: checked all 11 against their target `<axis>.wat`'s workload description (Item/Bad/Ok shapes, dial names, `:size` arity) — all 11 names matched their generated Clara program's actual axis.
- **Single-letter/short names in the sized `.wat` corpus**: `H`/`R`/`A`/`L`/`N` in `peragrare-census.sh` were the one candidate (compact axis-letter naming spanning many functions); I judged this a defensible design choice (a stated 5-dimensional grid coordinate system, documented at the file header and at each function signature comment) rather than a mumble, and did not raise it as a finding.
- **`:clara-ns == :fire-ns` duplication in `gen-fanout.sh`/`gen-deep-cascade.sh`/`gen-accum.sh`**: noticed the two fields carry the identical value; judged this a (possibly dead-instrumentation / purgare) observation, not a naming lie — both names are individually accurate, so I did not raise it under intueri.
- **The 12 sized axes I did not read end-to-end**: I read their full documentation headers and code shape but did not read every line of their `main` bodies; I cannot rule out a header/body mismatch specific to those 12 the way I could for the 4 I read completely. Flagging this as an honest boundary of this sweep, not a finding.

## Closing word

**FINDINGS**
