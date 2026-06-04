# Stone 245.2 — SCORE — warding core.wat (and the debt it dragged into the light)

**Closed 2026-06-04. `wat/core.wat` — the most important wat stdlib file — is warded.** What "roll into 245.2" turned out to mean: not a comment fix, but a three-front reconciliation that uncovered real arc-237 cleanup debt and forced the deftest-green bar onto the file the whole language composes on.

## The target

`wat/core.wat` — the `:wat::core::*` surface: the short-name aliases (dissoc/keys/values/concat), the polymorphic arithmetic defclauses (`+ - * /`), the ordering defclauses (`< > <= >=`), and `defn`. It carried ~120 lines of arc-archaeology comment over ~50 lines of forms — and was FORBIDDEN to ward (245.0 §5) until that archaeology was verified-live-or-cut.

## The guard (wat-kind vigilia, 4-spell)

**cernere CONVERGED** (every form live; retired forms appear only as retirement prose, never called — the cernere trap correctly avoided). **conferre 3 L1** (stale citations). **intueri 6 L1 + 5 L2** (archaeology bloat). **circumspicere 3 findings** (coverage gap + a stale graveyard).

## Front 1 — comment reconciliation (245.2a, `ba1ec6df`)

conferre grounded **3 stale-archaeology L1 lies**, all of one species — comments that survived arc-237's intrinsic-evacuation and arc-246's collection move while the code beneath them changed:
1. `"and the DispatchRegistry remain"` (lines 14/33) — `DispatchRegistry` is `grep`-0 in src/. Cut the clause.
2. The four `infer_contains/get/conj/assoc` citations pointed at `src/check.rs` — arc 246 moved them to **`src/collection/infer.rs`** (the `eval_*` halves stayed true in `src/runtime.rs`). Retargeted.
3. `infer_dispatch_call` narration — exists only in stale comments; `get` routes via the `infer_get` intrinsic. Cut.

intueri found the file failed Simple — ~120 comment lines of genealogy/tombstones burying ~50 of forms. The reconciliation **cut 135 lines and added 30** of live WHY (load-order, alias-vs-dispatch, the two-layer recipe + arity table, clause-absence rejection, defn=def+fn, metadata-peel, NaN-ordering). **Forms held byte-identical** — verified by a form-integrity diff check (every changed line is a comment or blank). green-gate 895/0, `every_stdlib_file_parses` ok.

## Front 2 — the deftest-green coverage, and a graveyard (245.2b, `a0f16151` + `61c72047`)

circumspicere's central catch: **no dedicated deftest exercised core.wat's forms** (only incidental i64-2-ary plumbing in time/stream/service tests), and the one Rust file that *looked* like coverage — `tests/wat_polymorphic_arithmetic.rs` — was a **stale RED graveyard (13/32 failing)** asserting pre-237.8b mixed-type *promotion* the substrate correctly rejects now. It was compiled by the gate but never *run* by it (a `--test`, not `--lib`), so it hid. Real arc-237 cleanup debt, surfaced by the ward.

**The fork (resolved by the four-questions, not handed to the builder):** spin-out FAILS Honest (a known RED graveyard left while warding the file it tests); repair-in-place FAILS Simple (a Rust `#[test]` can't satisfy the deftest-green bar, so it'd leave split redundant homes). **Retire + replace** passes all four. Executed replace-then-remove, each committed:
- **245.2b-i** — two named wat deftests: `wat-tests/core/core-arithmetic.wat` (39 deftests: `+ - * /` at 0/1/2/3+-ary × i64+f64, ordering, div-by-zero, and the load-bearing **cross-type rejections** `(+ 1 2.0)→NoMatchingClause` via run-hermetic Failure-match) and `wat-tests/core/core-equality.wat` (10 deftests: `:wat::core::=` for i64/f64/string + cross-type rejection). **49 deftests, deterministically green across 2 runs** (the run-hermetic rejection tests don't fork-flake — they're quick check-error snippets). A coverage-preservation map proved all **19 valid** Rust tests migrate before deletion.
- **245.2b-ii** — retired `tests/wat_polymorphic_arithmetic.rs`. The agent's own catch: `poly_div_f64_zero_errors` was *itself* stale (f64/0.0 = ∞ per IEEE-754, not an error). Removing the graveyard also cleared 14 of the integration suite's pre-existing RED tests.

## Front 3 — convergence + stamp (245.2c)

Re-cast on the committed tree (the 246 lesson: never claim converged without re-measuring): **conferre CONVERGED** (0 L1+0 L2 — all 3 prior lies fixed, no new drift, every claim grounded to a live `src/` file:line). **intueri 2 L2** — residual `via 237.4 rich error` arc-coordinates in the inline `-`/`/` 0-ary comments (genealogy the reconciliation hadn't touched, since inline comments were held byte-identical); stripped → **L1+L2=0**. circumspicere's findings closed by fact (the deftests exist + are green; the graveyard is gone).

**Stamp:** `;; vigilatum: 2026-06-04T04:01:56Z — vigilia 4-spell L1+L2=0, checker-clean + deftest-green(core-arithmetic)`.

## Verification (against the real gate)

green-gate PASS — test-build clean + **lib 895/0/1** with the ward in tree; `every_stdlib_file_parses` ok (checker-clean); `core_arithmetic` 39/39 + `core_equality` 10/10 deterministic; forms byte-identical (integrity check empty); the integration suite shed 14 RED graveyard tests.

## Deposits / lessons forged

- **Arc-archaeology is wat's borrow-checker.** core.wat proved the 245.0 §5 thesis at scale: 3 stale-citation L1s + ~135 lines of genealogy. The reconciliation pattern (conferre grounds the lies, intueri cuts the bloat, history goes to the arc record) is the template for the remaining stdlib.
- **A gate-excluded test is a place debt hides.** The graveyard was RED for the life of arc 237→245 because the gate compiles but doesn't run the integration suite (the arc-170 leak exclusion). Warding the *file* dragged the *test* debt into the light. This is more grist for #151 / arc-250.
- **The four-questions are the decision procedure even when a fork feels weighty** — minted `feedback_weighty_fork_still_four_questions` after reaching for the builder three times on four-questions-resolvable forks (the bar teeth; the graveyard ×2).
- **Brief substrate agents with the shell discipline, and don't make them run the gate.** A `./scripts/green-gate.sh` in a brief tripped the firewall and sonnet over-generalized to "bash is blocked" and gave up. Fix: substrate briefs use plain single commands only and leave the gate to the orchestrator (who verifies independently regardless).

## FM-11 — DONE, no deferral

The file is warded in fact: comments reconciled to live truth, forms exercised by named green deftests, the stale graveyard retired (coverage migrated first), stamp re-derived from the converged guard against the committed tree. No rune, no "coverage-pending," no deferred fix. The routine-gating-against-rot of the new deftests is the standing #151/arc-250 enabler, not a hole in this stamp.

## The close

245.2 closes. core.wat — the foundation every wat program composes on — now tells its current story and proves its own arithmetic, ordering, and equality behavior with green deftests; its stamp means exactly what it says. Two stdlib files warded (list ✓, core ✓). Next: **245.3** — the remaining core stdlib (`Record`, `runtime`, `stream`, `edn`) — where the edn `:test::Wrapper/new` fixture break (conferre/circumspicere flagged it live) will be the next reconciliation's opening catch.
