# SEQUI — Cast Report (TARGET 3 — the grid) — **FINDINGS**

> Written verbatim as returned, before any synthesis. Evidence only — this file carries NO status.

## What I swept

**Inventory** — re-derived exactly as briefed: 148 files, 16,616 lines total; 54 `.wat`, 43 `.clj`, 29 `.txt`, 20 `.sh`, 2 `.md` (`find`/`wc` commands run against `wat-scripts/perf/grid/`). No delta.

**Trigger metric** — re-ran `grep -hoE '\$\{?(GRID|WAT|CLARA|JAVA|ORACLE)[A-Z_]*' *.sh | sort | uniq -c`: this yields **41** raw token lines (because `$WAT_BIN` and `${WAT_BIN` count separately), or **36** truly-distinct variable names once braced/unbraced forms are collapsed. Neither equals the handed-down "37" — reporting the delta rather than adopting it, per instructions. Per-name counts I spot-checked did hold: `GRID_DIR` 31, `WAT_BIN` 27 (21 unbraced + 6 braced), `JAVA_HOME` 12 (9+3).

**Read in full** (all 20 `.sh`, 2,398 lines, matching inventory exactly): `run-axis.sh` (383), `check-grid-three-way.sh` (364), `peragrare-census.sh` (271), `check-where-shapes.sh` (162), `check-query-compat.sh` (145), `run-all.sh` (143), `compare-grids.sh` (143), `check-spec-native.sh` (97), `check-grid-speed.sh` (95), and all eleven `gen-*.sh` (39–90 lines each).

**Sampled**: `accum.wat` (header/records lines 1-60, and the `main`/session-threading chain lines 150-226 of 226) — one `.wat` file, chosen because it's the file `run-axis.sh` and `gen-accum.sh` both target, to check whether the "weaker surface" claim held for a live chain.

**Not read**: the other 53 `.wat`, all 43 `.clj`, all 29 `.txt`, both `.md` — per the brief's guidance and because prior wards already cover them.

## ★ The flagged surface — refuted as stated

I drove sequi's exact question against `run-axis.sh:207-382`. The nine accumulators (`RATIOS`, `WAT_NSS`, `CLARA_NSS`, `WAT_WALLS`, `CLARA_WALLS`, `ACCURACY`, `ORACLE_ACCURACY`, `PORT_ACCURACY`, `HAS_ORACLE`) are all reset at `:215-223`, which is the **first thing** executed inside `for SIZE in "$@"; do` (after only `SIZE_JSON`/`CLJ_TMP` setup). Every other variable interpolated into the `:382` `#grid/Verdict` line (`MEAN`, `MIN`, `MAX`, `WAT_MEAN`, `CLARA_MEAN`, `WAT_NS_MIN/MAX`, `WAT_WALL`, `CLARA_WALL`, `WALL_RATIO`, `WALL_WINNER`, `FIRE_SHARE`, `WINNER`, `ORACLE_FIELDS`) is unconditionally recomputed at `:332-381`, every SIZE iteration, with no path that skips the recomputation (the only early exits, at `:254-256` and `:271-272`, are hard `exit 1`s that never reach the Verdict line). So **no value from a previous SIZE or previous RUN can survive into a Verdict** — the reset discipline is correct and deliberate. This is a genuine refutation, not a shrug: I read every line the theory named and the composition holds.

One secondary, unconfirmed-as-live wrinkle I noticed but am not filing as a finding: `HAS_ORACLE` (`:223`, set `:303`) is never reset mid-RUN-loop, so if the SAME wat binary emitted `:oracle-derived` on run 1 but not run 2 of the same SIZE, `ORACLE_ACCURACY`/`PORT_ACCURACY` would report `:match` based on only 1 of `GRID_RUNS` samples. I found no evidence this actually happens (the binary is deterministic per invocation) — noting it as a theoretical gap, not a finding.

## Findings

**F1 — a recurring sequi violation: three functions whose usage looks like a boolean check but whose real effect includes an invisible global-counter mutation.**

Translating the spell's "Types hide what the function does" example to bash: a function's call site is its only visible "signature" (there's no type system), and in all three cases the call site is `check_X "$stem" || FAILED=1` — which reads as a pure predicate. Each function's body, however, also increments a caller-scoped global tally that the call site never mentions:

1. `check-where-shapes.sh:90-133` (`check_pair`) — increments `ROWS_TOTAL` at `:128`. Global initialized at `:139`, called at `:149`, consumed at `:158`.
2. `check-query-compat.sh:67-125` (`check_stem`) — increments `ROWS_TOTAL` at `:123`. Global initialized at `:129`, called at `:132`, consumed at `:141`.
3. `check-spec-native.sh:37-74` (`check_stem`) — increments `ROWS_TOTAL` at `:69`. Global initialized at `:78`, called at `:84`, consumed at `:93`.

The chain in each file is: `for/while over discovered stems → check_X(stem) [return 0/1, ALSO ROWS_TOTAL += n] → final echo of $ROWS_TOTAL`. The state (`ROWS_TOTAL`) threads through a bash-default global rather than through the function's visible interface (its exit code and stdout diagnostics), exactly the "hidden state via global counter" shape, just with `fetch_add` replaced by `VAR=$((VAR+n))` and no `local`.

**Currently produces a correct number** (I verified: the summary line that prints `$ROWS_TOTAL` is only reached when `$FAILED -eq 0`, i.e., every call succeeded and therefore contributed its `wn`/`nn`/`wc -l`, so no undercounting exists on disk today). The defect is the sequi kind, not a solvere kind: nothing in the chain enforces the coupling. Any future edit that reorders the loop, adds an early `continue` on a soft-fail path, retries a stem, or refactors `check_pair`/`check_stem` to be called twice would silently produce a wrong `$ROWS_TOTAL rows` claim in the passing summary line, with nothing — no type, no lint, no assertion — to catch it. This is precisely sequi's "good UX" failure mode: a caller cannot safely rewire the chain.

- **Recommendation**: thread explicitly. Have `check_pair`/`check_stem` `echo` the row count on stdout (separated from the human-readable diagnostic lines, which already go to files under `$OUT_DIR`, or route diagnostics to stderr) and accumulate at the call site: `n=$(check_pair "$stem")` / `ROWS_TOTAL=$((ROWS_TOTAL + n))`. This is domain state (the reported row count), so per the spell's own rule it gets **no rune** — thread it, don't suppress it.

**F2 — confirmed CLEAN on the ★-flagged surface** (see above): `run-axis.sh:207-382`'s accumulator-reset chain is honest; no cross-size/cross-run leakage exists into the `#grid/Verdict` line.

## What I looked for and did not find

- **Zero `rune:` tokens** in the target directory (`grep -rn "rune:" .` → none), reconfirming `exigere`'s measurement independently this session.
- **No hidden coordination via fixed/shared temp paths across processes** — every script uses `mktemp`/`mktemp -d` scoped to its own run, with `trap ... EXIT` cleanup (`run-axis.sh:205`, `check-grid-three-way.sh:117`, `check-where-shapes.sh:88`, `check-query-compat.sh:48`, `check-spec-native.sh:26`, `check-grid-speed.sh:49`); none of these paths are shared between unrelated invocations.
- **No other function of the `check_X`/`test_X` shape mutating an undisclosed global** beyond the three in F1 and `peragrare-census.sh`'s `test_move` (already prior art 3T5 — same *family* of defect, not re-filed).
- **`find_java()` in `check-grid-three-way.sh:98-112` / `check-query-compat.sh:25-42`** exports `PATH`/`JAVA_HOME` as its real effect while being called as `find_java || exit 1`. Considered this under sequi's lens and judged it **host-idiom, not a violation**: the function's name states its purpose (make java findable), bash has no channel other than `export` to hand an environment change back to the caller's shell, and the mutation is exactly what "find java" must do — analogous to the spell's `&mut self` exemption.
- **No leakage across GRID_RUNS or across SIZE values** in `run-axis.sh` (the ★ surface) — traced explicitly, see above.
- **11 `gen-*.sh` generators**: all single-pass heredoc emitters; the embedded Clojure `-main` bodies use an honest `let`-threaded chain (`s`, `t0`, `t1`, `f`, `codes`) — no bash-side accumulators at all.
- **`accum.wat:150-226`** (sampled): the wat session chain (`session → staged → fired → derived`) is threaded entirely through named `let` bindings, the textbook honest shape the spell defends — confirms the brief's note that `.wat` is the weaker surface for this ward.

FINDINGS
