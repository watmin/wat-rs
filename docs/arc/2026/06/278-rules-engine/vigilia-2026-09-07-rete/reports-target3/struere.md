# Struere Cast Report — `wat-scripts/perf/grid/`

## 1. What I swept

**Inventory re-derivation** (before reading anything): `find wat-scripts/perf/grid -type f | wc -l` → **148**; by extension → **54 .wat, 43 .clj, 29 .txt, 20 .sh, 2 .md**; `wc -l` over sh+wat+clj+txt+md → **16,616**. This matches the brief's numbers exactly — **no delta to report**.

**Read in full (primary surface, as directed):**
- All **20/20 `.sh` files** (2,398 lines, confirmed via `wc -l *.sh | sort -n`, matching the brief's per-file counts exactly): `run-axis.sh`, `run-all.sh`, `check-grid-three-way.sh`, `peragrare-census.sh`, `check-where-shapes.sh`, `check-query-compat.sh`, `check-spec-native.sh`, `check-grid-speed.sh`, `compare-grids.sh`, and all 11 `gen-*.sh`.
- Ran `bash -n` on all 20 — all syntactically clean.
- Ran `grep -m1 -E '^set -' *.sh` across all 20 — **every script uses `set -euo pipefail`**, uniformly (a positive finding — no inconsistency here, contrary to what the brief flagged as worth checking).
- Confirmed zero `rune:` occurrences anywhere in `.sh`/`.wat`/`.clj` in this directory (matches "mora/exigere returned CLEAN").

**Sampled `.wat`/`.clj` (43 clj / 54 wat — too large for full reads):** Sampling rule — one representative from each construction idiom named in `peragrare-census.sh`'s own axis table (H/R/A/L/N), plus the corpus's largest/flagship file for contrast:
- `neg-consumer.wat` (the "axis that convicts", N=consumed)
- `leading-exists.wat` (the 2026-08-24 regression axis, L=leading)
- `retract-multiplicity.wat` (R=present)
- `userfn-head.wat` + its static twin `userfn-head.clj` (H=userfn)
- `where-not-and-not.wat` (nested `:not`, read in full, 102 lines)
- `where-shapes.wat` (the flagship "core" family, read lines 230–269)
- All 11 `gen-*.sh` (already counted above) double as my Clara-side sample for the 11 sized perf axes.

That's 7 `.wat` + 1 `.clj` read directly, plus 11 more Clara programs read via the `gen-*.sh` bodies = effective sample of ~19 of 54 `.wat`-equivalent constructions. I did **not** read the other ~35 `where-*.wat`/`.clj` pairs individually, but I did run a targeted `grep -c` sweep across all 38 `where-*.wat` files for one specific construction pattern (below) to scope a finding beyond my directly-read sample — that count is disclosed as a *measurement*, not treated as a line-cited finding on its own; every file I cite specific lines from was actually read.

## 2. Findings (strongest first)

### Finding 1 — `where-*.wat` corpus: caller does the compile/fire/error-handling infrastructure by hand, 2–18 times per file, when the same corpus already has the extraction (wrong-level, Level 2 mumble)

`where-not-and-not.wat` (read in full, 102 lines) has **8** separate, nearly-identical inline pipelines in `:user::main` — e.g. lines `57`, `60–62`, `65–68`, `71–74`, `77–79`, `82–86`, `89–93`, `96–102` — each one hand-threading `compile-all` → `insert` → `fire-rules`, each with its own full `match` arms for `CompileOutcome::MayNotTerminate`, `FireOutcome::MemoryCeilingExceeded`, `FireOutcome::RoundCapExceeded`. The caller (`:user::main`, a test driver) is doing infrastructure work per row that it has no business seeing repeated eight times.

The same corpus already contains the correct shape: `where-shapes.wat:248–261` defines a single `:wsh::run-row [row] -> String` helper that does this exact compile/fire/query/render pipeline **once**, and `:user::main` (`where-shapes.wat:263–268`) simply `foldl`s it over `(range 1 (row-count)+1)`.

I measured how widespread the un-factored form is: `grep -c "CompileOutcome::Compiled" where-*.wat` across all 38 `where-*.wat` files gives occurrence counts of 1 (×19 files — single-scenario families, fine), 2 (×3), 3 (×1), 4 (×1), 5 (×4), 6 (×4), 8 (×4, incl. `where-not-and-not.wat`), 14 (×1, `where-or-conditions.wat`), 18 (×1, `where-exists.wat`) — **19 of 38 files** repeat the whole pipeline inline more than once, one file (`where-exists.wat`) doing it 18 times. This is a measurement disclosed for scope, not itself a line-cited finding; the line-cited evidence is `where-not-and-not.wat`'s 8 sites versus `where-shapes.wat:248–268`'s single helper.

- **Lens:** wrong-level (abstraction) — composition-doesn't-hold as corollary (a change to the compile/fire contract requires editing up to 18 sites in one file).
- **Level:** 2 (mumble) — every individual row works; the file as a whole doesn't hold under a change.
- **Direction:** extract a `run-row`/`run-scenario` helper (parameterized by rules + facts, as `where-shapes.wat:248` already does) in the worst-offending files, starting with `where-exists.wat` (18) and `where-or-conditions.wat` (14).

### Finding 2 — `run-axis.sh`: an `:accuracy :MISMATCH` never flips the process exit code, so `run-all.sh` can report success while the grid disagrees (composition-doesn't-hold / type-doesn't-enforce, Level 1 lie)

Read in full (383 lines). `ACCURACY`/`ORACLE_ACCURACY`/`PORT_ACCURACY` are initialized to `":match"` (`run-axis.sh:220–222`) and set to `":MISMATCH"` on divergence (`:283`, `:305`, `:311`), each time only *echoing* to stderr — no `exit` call is reachable from any of those branches. The script's last executed statement per size is the successful `echo "#grid/Verdict ..."` (`:382`), so the script's own exit status is **0** regardless of accuracy outcome.

Contrast with two sibling instruments in the same directory, both read in full: `check-grid-three-way.sh` explicitly sets `fail=1`/`FAILED=1` on any of its three pairings mismatching (`:333–347`) and `exit 1`s at the end (`:359–364`); `check-grid-speed.sh` parses the `:accuracy` field straight out of the printed Verdict line and sets `fail=1` on anything other than `match` (`:64–67`), gating on it at `:91–94`. Both treat this exact signal as load-bearing enough to become a process exit code — `run-axis.sh` does not.

`run-all.sh` (read in full, 143 lines) is the project's "sweep everything" entry point, and its *only* failure signal per axis is `run-axis.sh`'s own exit status (`run-all.sh:137–140`). So `run-all.sh`, run bare (e.g. `bash run-all.sh; echo $?`), can print one or more `#grid/Verdict {... :accuracy :MISMATCH ...}` lines to stdout and still exit 0 — the one thing this whole grid instrument exists to check is invisible to its own exit code.

- **Lens:** composition-doesn't-hold / type-doesn't-enforce.
- **Level:** 1 (lie) — the ordinary process contract ("exit 0 ⇒ OK") is violated for the single most important signal the tool produces; the script's own header (`run-axis.sh:42`, "A :MISMATCH also dumps both :derived sets to stderr — never hidden") treats MISMATCH as first-class but never wires it to the interface a caller like `run-all.sh` actually checks.
- **Direction:** `run-axis.sh` should `exit 1` (or a distinct code) whenever `ACCURACY`/`ORACLE_ACCURACY`/`PORT_ACCURACY` ≠ `:match` before the final loop iteration returns, mirroring `check-grid-three-way.sh`'s own pattern — or `run-all.sh` should itself scan the emitted Verdict lines for `:MISMATCH` rather than trusting only the subprocess exit code.

### Finding 3 — `run-all.sh:23` promises "a summary tally on stderr" that the script never computes (type-doesn't-enforce, Level 1 lie)

`run-all.sh:23`: *"Emits every `#grid/Verdict` line from every axis, then a summary tally on stderr."* The actual body (`:122–143`, read in full) contains no such tally: the loop only prints a per-axis `── axis ──` header and, on failure, one `FAILED` line (`:134`, `:138`). Nothing counts `:match` vs `:MISMATCH`, `:us` vs `:clara` vs `:unresolved`, or axes attempted vs completed anywhere in the file.

- **Lens:** type-doesn't-enforce (comment promises a contract the code doesn't keep).
- **Level:** 1 (lie).
- **Direction:** implement the promised tally (counts by `:accuracy` and `:winner`, read straight off the emitted Verdict lines — trivially available in the same loop), or correct the header comment to describe what the script actually emits.

### Finding 4 — `peragrare-census.sh`: three functions leak `while`/`for … read` loop variables as script globals for want of `local`, inconsistent with sibling functions in the same file (values-not-places, Level 2 mumble)

This is the orchestrator's own file, audited on the same terms as the rest. `do_verify` (`:171`), `cell_count` (`:191`), and `do_report` (`:205` and `:212`) each run `while read -r name H R A L N ...` / `for k in ...; do IFS='|' read -r H R A L N <<<"$k"` **without** first declaring `local name H R A L N` (or, in `do_report`, `key`/`names`/`cnt`) — so these become ordinary script-global variables on every call. Compare `verify_fixture` (`:133`, `local name="$1" H="$2" R="$3" A="$4" L="$5" N="$6" s ok=0`) and `test_move` (`:241`, `local label="$1" H="$2" R="$3" A="$4" L="$5" N="$6"`), which use the exact same variable names and *do* declare them local — proving the discipline is known and applied elsewhere in the same file, just not consistently.

- **Lens:** values-not-places — "a function that silently depends on a caller-set global, or leaks a variable because it forgot `local`, is exactly this defect with the safety net removed," per the ward's own translation guidance.
- **Level:** 2 (mumble) — harmless today only because the script's `case "${1:-}" in ... esac` dispatch (`:266–271`) runs exactly one of `do_verify`/`do_pins`/`cell_count`/`do_report` per invocation and then exits; nothing currently chains two of these calls in one process. A future flag doing so (or this file being `source`d rather than executed) would silently observe stale `H`/`R`/`A`/`L`/`N` left over from whichever function ran last.
- **Direction:** add `local name H R A L N` (`do_verify`, `cell_count`) and `local key H R A L N names cnt` (`do_report`) at the top of each function.

### Finding 5 (minor, same file) — `test_move` achieves its self-test by mutating the shared "hand-derived, cited" `TABLE` global and restoring it, rather than parameterizing `cell_count`

`peragrare-census.sh:108` declares `TABLE` as the file's single source of truth, documented in the header as a "hand-derived, cited judgment." `do_pins`' nested `test_move` (`:239–253`) temporarily overwrites this same global — `TABLE_SAVE="$TABLE"; TABLE="$TABLE\nsynthetic-$label ..."` (`:242–244`) — calls `cell_count` against the mutated value, then restores it (`:246`). `cell_count` itself is honest (it only reads `$TABLE` via a here-string, `:196`), but the self-test reaches around that honest interface by mutating the value it's supposed to be a read-only argument to.

- **Lens:** values masquerading as places.
- **Level:** 2 (mumble) — correct today only because nothing between the mutation (`:243–244`) and the restore (`:246`) can fail under `set -e`, and no other function reads `TABLE` concurrently in the same process.
- **Direction:** parameterize `cell_count` (and the underlying aggregation) to take an explicit table value/argument, so `test_move` can pass the synthetic-row-appended string directly with no save/restore dance.

## 3. What I looked for and did NOT find

- **No `set -e`/`-u`/`pipefail` inconsistency** — all 20 scripts set `set -euo pipefail` identically (checked via `grep -m1 -E '^set -' *.sh`), which is a positive, uniform result worth stating rather than assuming.
- **No mutation-hidden-behind-an-innocent-name** in the `check-*.sh` helper functions I read in full (`rewrite_to_spec`, `extract`, `norm`, `report_pair`, `find_java`, `check_stem`, `check_pair`, `query_stems`) — each either flows values in/out honestly or names its environment side effect plainly (`find_java`'s `export PATH`/`JAVA_HOME` is the script's stated job, already flagged as triplicated by solvere's 3S5; I did not pile a naming complaint onto that).
- **No exit-code contract violation** in `check-grid-three-way.sh`, `check-grid-speed.sh`, `check-spec-native.sh`, `check-query-compat.sh`, `check-where-shapes.sh`, or `compare-grids.sh` — all six correctly convert their respective mismatch/accuracy signals into a nonzero exit (verified by reading each in full), which is exactly what makes Finding 2's absence of that pattern in `run-axis.sh` stand out as an inconsistency rather than a house style.
- **No `rune:` exemptions anywhere** in `.sh`/`.wat`/`.clj` under this directory (`grep -rln "rune:"` → 0), consistent with mora/exigere's prior CLEAN.
- All 11 `gen-*.sh` generators (read in full) are honest value-in/heredoc-out templates — no hidden mutation, no unquoted-expansion hazard beyond the already-flagged 3S3 duplication.
- I did **not** read all 38 `where-*.wat`/`.clj` pairs individually — Finding 1's corpus-wide scope rests on a `grep -c` measurement over a single, precisely-named token (`CompileOutcome::Compiled`), disclosed as such, with only two files' line numbers actually read and cited.

**FINDINGS**
