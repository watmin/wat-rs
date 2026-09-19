# CONFORMARE — Cast Report (TARGET 3 — the grid) — **FINDINGS**

> Written verbatim as returned, before any synthesis. Evidence only — this file carries NO status.

**What I swept.** Commands run (all read-only: `find`, `wc`, `grep`, `cat -n`, `sed -n`):
- `find . -type f | wc -l` and per-extension count → confirmed **148 files**, **54 .wat / 43 .clj / 29 .txt / 20 .sh / 2 .md**.
- `wc -l` over those extensions → confirmed **16,616 lines** total. No deltas from the handed-down inventory this time.
- `grep -hoE 'exit [0-9]+' *.sh | sort | uniq -c` → **35 × exit 1, 9 × exit 2, 1 × exit 3** — matches exactly.
- `grep -rn '#grid/Result|#grid/Verdict|#grid/Error'` → **65 / 5 / 2** — matches exactly.
- Read in full: `run-axis.sh` (383), `check-grid-three-way.sh` (364), `peragrare-census.sh` (271, the orchestrator's own file), `check-where-shapes.sh` (162), `check-query-compat.sh` (145), `run-all.sh` (143), `compare-grids.sh` (143), `check-spec-native.sh` (97), `check-grid-speed.sh` (95) — 1,803 of the 2,398 `.sh` lines.
- Sampled, not read whole: two `gen-*.sh` at the size extremes (`gen-negation.sh` 39, `gen-accum.sh` 90) to check arity-error handling and wire-shape consistency; four `.wat` axes at size extremes for the `CompileOutcome`/`FireOutcome` match arms (`retract-multiplicity.wat`, `accum.wat`, `where-collection.wat`, plus a corpus-wide `grep` census across all 54 to confirm the pattern rather than guess from a sample). **Sampling rule stated**: read every `.sh` file that terminates a process boundary or is explicitly called out as central; for the `.wat`/`gen-*.sh` families (54 and 11 near-identical files respectively), read enough to characterize the shared idiom, then `grep -c`/`grep -l` the *entire* population to turn "looks like a pattern" into a measured count before reporting it.
- `grep -rn 'rune:'` over `*.sh *.wat *.clj *.txt *.md` → **0** hits. Confirms the zero-rune claim independently.

**Literal-surface statement.** No Rust exists in this target and no `enum *Error`/`Result<T,E>`/`span` field/`From` impl is definable here — confirmed by the extension census above (0 `.rs` files). I audited the translated surface instead: exit-code vocabulary, `#grid/*` wire-line variants, and the `.wat` axes' `CompileOutcome`/`FireOutcome` match arms, per the brief's translation.

**Failure-vocabulary manifest.**

| Channel | Values seen | Meaning defined | Enforced by |
|---|---|---|---|
| shell exit code | `1` (35), `2` (9), `3` (1) | Nowhere centrally — each site's neighboring `echo` is the only definition; `2` alone covers ≥4 unrelated conditions (bad argv in `compare-grids.sh`, missing-ladder-rung / missing-ORDER-entry / unknown-axis in `run-all.sh`, a no-op oracle-rewrite in `run-axis.sh`, a too-short sweep in `check-grid-speed.sh`) | Nothing — no script branches on the numeric value of another script's exit code; every consumer treats "nonzero" as one bit. `exit 3` (`compare-grids.sh:141`) is the sole exception at that value and nothing tests for it specifically either. |
| `#grid/Result` | 65 occurrences, fields `:axis :size :derived [:oracle-derived] :native-ns/:clara-ns […]` | `run-axis.sh:38-42` (header) | Regex/`grep -oP` at each consumption site (`run-axis.sh`, `check-grid-three-way.sh`) — no schema, already flagged by prior art (3S1-6) as duplicated/divergent. |
| `#grid/Verdict` | 5 | `run-axis.sh:38-42` | same, consumed by `compare-grids.sh` and `check-grid-speed.sh`. |
| `#grid/Error` | 2 (one is a doc-comment) | `check-grid-three-way.sh:200-201` | Not decoded structurally — recovered only incidentally, because the emitted text happens to contain `:axis "$stem"` which the *generic* "no `#grid/Result` for this axis" fallback grep (`:279-285`) also matches. It is a real, exercised variant (not a vestige) but rides on a coincidence of string content, not a dedicated check. |

**Every exit-code call site I could find (44 of 45, one `${1:?...}` implicit site not textually "exit N") is preceded by an `echo …>&2` naming the script and the specific condition** — that part of the corpus is disciplined. The gap is not "silent exits"; it is diagnostic data that is *available* at the point of failure and gets thrown away anyway, or captured-then-discarded at a process boundary. Two concrete instances:

---

### Finding 1 (L1) — `run-axis.sh:264` discards Clara's stderr on the corpus's primary, CI-gated perf path, while the rest of the corpus (and this very file's sibling call) captures it

`run-axis.sh:264`:
```bash
CLARA_OUT="$(cd "$CLJ_TMP" && clojure -Sdeps "$CLARA_DEP" -M -m "$AXIS" 2>/dev/null || true)"
```
When Clara produces no `#grid/Result` (`:267-271`), the failure report is:
```bash
echo "run-axis: Clara side produced no #grid/Result for axis=$AXIS size=[$SIZE] run=$RUN:" >&2
echo "$CLARA_OUT" >&2
```
`$CLARA_OUT` is stdout only — the JVM/Clojure/clara-rules exception, stack trace, or deps-resolution error that would explain *why* went to `/dev/null` and is gone. This is the exact defect the file's own comment two blocks above (`run-axis.sh:227-228`) names and fixes for the **wat** side: *"stderr is CAPTURED, not discarded: `2>/dev/null` made a wat-side failure loud but REASONLESS — you learned the axis produced nothing and never why."* The wat side now captures to `WAT_ERR` and `cat`s it (`:253`); the Clara side three lines later still has the bug the comment describes.

This is not a one-off oversight against an unstated norm — the corpus has an established, near-unanimous convention for this exact call. `grep -c 'clojure -Sdeps' *.sh` finds exactly 4 sites that launch Clara: `check-grid-three-way.sh`, `check-query-compat.sh:90-91`, `check-where-shapes.sh:103-104`, and `run-axis.sh:264`. The other three all capture Clara's stderr to a file and surface it on failure:
- `check-query-compat.sh:90-93`: `> .clara.txt 2> .clara.err` … `echo "[$stem] Clara FAILED"; tail -30 "$OUT_DIR/$stem.clara.err"`
- `check-where-shapes.sh:103-107`: `> .clj.txt 2> .clj.err` … `echo "[$stem] the Clara side FAILED"; tail -30 "$OUT_DIR/$stem.clj.err"`
- `check-grid-three-way.sh:218-223`: captures to `clara.err`, `cat`s it whole on JVM failure.

**3 of 4 Clara-invocation sites in this directory follow capture-and-surface; the 4th — the one `check-grid-speed.sh:51` runs as a CI gate, and the one every `run-all.sh` sweep goes through — is the outlier.** This is distinct from `temperare`'s 3M1 (the `guard` wrapper asymmetry, a resource-enforcement concern) — mine is about diagnostic capture at a process boundary, the same call, a different axis.

Direction: thread `2>"$CLARA_ERR"` through exactly like `WAT_ERR`, and `cat`/`tail` it in the `[ -z "$CLARA_LINE" ]` branch — the same shape already proven three times over in this directory.

---

### Finding 2 (L1, corpus-wide) — every `.wat` axis destructures rich failure diagnostics from `CompileOutcome`/`FireOutcome` and then discards them for a static string, while the same file proves it knows how to interpolate

Sampled `retract-multiplicity.wat:68,90,104`, `accum.wat`, `where-collection.wat:260-262,308-309,348-349`, then measured the whole population. Representative arm (`retract-multiplicity.wat:68`):
```
((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds)
  (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded"
                                     :wat::core::None :wat::core::None))
((:wat::rete::FireOutcome::RoundCapExceeded __cap __still)
  (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded"
                                     :wat::core::None :wat::core::None))
```
`__limit`, `__used`, `__rounds`, `__cap`, `__still` (and `__rule`, `__fact-type` for `CompileOutcome::MayNotTerminate`) are bound by the match arm and then **never referenced anywhere** — `grep -rn '__limit\|__used\|__rounds\|__cap\|__still\|__rule\b\|__fact-type' *.wat` outside the match/assertion line itself returns **zero** hits. The message is a hardcoded string; if this arm ever fires, the entire diagnostic surface a user gets is that static sentence — no actual limit, no actual usage, no round count, no offending rule name.

Measured across the full population:
- `grep -l 'session memory ceiling exceeded" :wat::core::None :wat::core::None' *.wat` → **54/54** files.
- `grep -l 'fixpoint round cap exceeded" :wat::core::None :wat::core::None' *.wat` → **54/54** files.
- `grep -l 'MayNotTerminate __rule __fact-type' *.wat` → **54/54** files.

This is not a capability gap: `where-collection.wat:260-262`, in the **same file**, for a *different* failure path, builds a dynamic message that interpolates a live value —
```
(:wat::core::String/concat "where-collection: unknown row " (:wat::core::i64::to-string row))
```
— proving `assertion-failed!` takes an arbitrary `String` and the corpus already knows how to build one from bound locals. Every one of the 54 files chooses the static string for the three outcome-enum arms specifically, despite having the identical interpolation tool in hand two hundred lines away.

This also contrasts directly with `mora`'s positive finding (cited in the prior-art block) that `run-axis.sh:246-251` distinguishes `124` (timeout) from `137` (SIGKILL) with named mechanisms — that discipline exists at the shell layer for the *outer* memory guard, but is absent at the `.wat` layer for the *inner*, structurally-richer signal the engine itself raises.

Structural direction: this is exactly the shape the repo's own `wat-fix` codemod doctrine exists for (noted for completeness, not attempted — no edits were made): 54 textually-identical match arms, one rewrite, verified by dry-run diff. The trailing two `:wat::core::None` arguments to `assertion-failed!` may be a legitimate `spanless-by-domain` case (no source span exists for a runtime resource-cap trip) — but that is a different position than the *message string*, which has no such excuse.

---

### Finding 3 (L2) — `peragrare-census.sh --cell` (the in-scope orchestrator file) collapses "empty but valid" and "invalid axis value" into the same silent `0`

`peragrare-census.sh:189-198`, doc-commented by the author as `# H R A L N -> population of that cell (0 if none / invalid)`:
```bash
cell_count() {
  local qH="$1" qR="$2" qA="$3" qL="$4" qN="$5" n=0
  while read -r name H R A L N; do
    [ -z "$name" ] && continue
    if [ "$H" = "$qH" ] && … ; then n=$((n+1)); fi
  done <<<"$TABLE"
  echo "$n"
}
```
called from the CLI entry point `--cell` at `:269` with no validation against `H_VALUES`/`R_VALUES`/etc. (`:183-187`). `peragrare-census.sh --cell record absent none none na` (a real, populated combination) and `peragrare-census.sh --cel rekord absent none none na` (a typo'd axis value) both print `0` with exit code `0` — indistinguishable. The comment self-documents the ambiguity ("0 if none / invalid") rather than resolving it; this is the catalogue's own `Honest?` question failing on its own introspection tool: the constructor (`--cell` query) can produce the "empty" signal without any indication of whether the query was even well-formed. Low severity (a debug/introspection surface, not a gate), but it sits in the file the ward specifically called mine-and-in-scope.

Related, same file: `do_report()` (`:200-201`) calls `check_membership` bare (relying on `set -e` to abort on drift) while `do_verify()` (`:170`) calls the identical function guarded (`check_membership || fail=1`, accumulated into a summary). Two authorship patterns for the same failure inside one file — not a hidden bug (both paths do print `check_membership`'s own drift diagnosis before terminating), but an `Obvious?`/`Simple?` inconsistency worth naming.

---

**What I looked for and did not find.** No Rust/enum surface (confirmed absent, as instructed). No additional `rune:` exemptions anywhere (0, independently re-derived). No place where an exit-code call site prints *nothing* before exiting (44/44 textual sites carry an `echo`). No divergence in the `35/9/1` or `65/5/2` counts from the handed-down figures — this vigilia's numbers held. I did not re-litigate `run-axis.sh`'s `ACCURACY`/`ORACLE_ACCURACY`/`PORT_ACCURACY` never-exiting-on-MISMATCH shape (`:283,305,311`) — verified it also applies to the two three-way fields beyond the one 3T1 already cites, but that is a gating/exit-code defect (struere's lens: the diagnosis is printed, only the exit code fails to trip), not a diagnostic-completeness gap, so I left it to struere's row rather than duplicating it here.

**FINDINGS**
