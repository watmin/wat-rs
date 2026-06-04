# Stone 245.1 — SCORE — the first wat ward (and the bar it forced into honesty)

**Closed 2026-06-04. `wat/list.wat` is the first warded wat file** — and warding it twice-refined the bar the 245.0 design had only guessed at. The deposit is not the 18-line file; it is the discipline the first ward forged, exactly as the homes-walk's first convergence forged the Rust ward.

## The target

`wat/list.wat` — two `defalias` forms (`:wat::list::reduce`, `:wat::list::fold`) over the atomic fold primitive `:wat::core::foldl`, plus a header. Chosen as 245.1 for being the smallest, cleanest, most-representative leaf (the canonical alias-over-a-primitive pattern) — a loop-prover before the archaeology-laden core files.

## The guard (the wat-kind vigilia selection, per 245.0 §2)

Proportionate selection (vigilia's own rule — cast only the kind-applicable spells): **intueri, cernere, conferre** (inward) + **circumspicere** (last). Honestly N/A on a 2-form alias file with no functions/state/parallelism: solvere (nothing braided), purgare (no dead code — the aliases are live registered surface), struere (2 trivial forms), sequi (no chains), temperare (no computation), secare (no parallelism). **4-spell guard.**

### Round 1 — the measurement
- **cernere: CONVERGED** — every form (`defalias`, `foldl`) is a live registered form (`src/freeze.rs:1412`, `src/check.rs:16482`); no phantom, no retired form in use (the 241.12 `define-alias`→`defalias` migration is the live form, `src/remedy/retirement.rs:81`).
- **conferre: CONVERGED** — all four comment claims verified live (foldl/foldr atomic primitives; the two registrations; the unshipped rename plan; the 241.12 migration).
- **intueri: 2 L1 + 1 L2** — header archaeology (stale "forward-looking" framing of a live namespace; a speculative scheduled-arc rename promise; verbose arc-109/6-language narration).
- **circumspicere: 1 escalated L1** — the surround catch (below).

### The discipline that fired (the real value of casting over hand-checking)

**1 — a confident cast finding, WITHDRAWN by right-target grounding.** intueri's headline L1 was "`:wat::core::foldr` exists nowhere in the corpus" — it had grepped `wat/` (the *usage* corpus). But `foldr` is a *registered primitive* defined in `src/` (`src/check.rs:16499`, `src/collection/transform.rs:291 eval_vec_foldr`, dispatched `src/runtime.rs:5809`). An **absence-claim grounded against the wrong target** (usage vs definition) — `feedback_ground_against_right_target` in the live. Weighed against the *defining* target, the L1 was a phantom and was withdrawn; the comment ("foldl and foldr remain the atomic forms") is TRUE. The cast that flattered nothing (conferre, which read `src/` live) was right; the loud one was wrong. *Casting is not a rubber stamp — two casts disagreed, and the orchestrator adjudicated against the disk, not against its prior.*

**2 — circumspicere found the gap all three inward lenses turned their backs on.** The header ships the claim "both delegate to the atomic `:wat::core::foldl` primitive" — and that claim was enforced by **zero executing tests**. Verified: `register_defalias` (`src/runtime.rs:3793-3808`) registers a **silent `NilLit` stub and returns `Ok(())`** on a missing target, deferring the error to call-time; and the only `:wat::list::reduce` test (`tests/wat_arc143_manipulation.rs:124`) uses it as a rename *string literal*, never executing it. A `vigilatum` stamp here would have attested a truth the build could not catch going false — the exact failure-class this arc annihilates, found in the first file.

### Round 2 — convergence
- Header rewritten to present-tense truth (the speculative rename + stale framing + archaeology cut; the true 241.12 provenance + the alias-insulation WHY kept). **intueri re-verify (clean-eyed): CONVERGED** (L1=0, L2=0; only L3 taste).
- circumspicere's finding **closed by construction**: `wat-tests/core/list-fold-aliases.wat` added — two deftests that *execute* `(:wat::list::reduce …)` and `(:wat::list::fold …)` and assert the fold (`= 10`), 2/2 green. The closure is verified empirically (the test runs green), not by a re-cast opinion.

**Guard verdict: L1+L2=0 across the 4-spell wat-kind guard.**

## The deposit that matters: the bar, twice-refined

245.0 §3 wrote the wat L2 floor as **"checker-clean + suite-green."** The first ward proved that phrase wrong in two stages — the experience-acquisition loop firing on the bar itself.

**Refinement 1 — TEETH (from circumspicere).** "suite-green" is toothless: a file can pass the suite while its own forms are never executed (list.wat did). The bar sharpens to **the file's own forms must be exercised by a passing test** — a green suite where the forms are never called does not clear it. (Four-questions: keeping it loose FAILS Honest; deferring the test via a "coverage-pending" stamp FAILS Honest — the lying-certificate shape; full teeth now passes all four. Disposed: `feedback_runes_illegal_when_solvable` + `feedback_dont_document_non_fixes`.)

**Refinement 2 — HONESTY about the gate (from grounding `scripts/green-gate.sh`).** "suite-green" is not merely toothless — it is a **lie**: there *is* no green integration suite. The project's real gate (`green-gate.sh`) is `cargo build --tests --workspace` + `cargo test --lib -p wat` (the 895/0/1 lib baseline); it **deliberately excludes the full `cargo test --test test` RUN** because the arc-170 stdio/fork/lifeline tests leak processes (documented `green-gate.sh:22-29`; "ONCE 170 lands, add the full run"). The raw integration suite is ~53-red at baseline (verified pre-existing: HEAD-with-changes-stashed = 53 failed; a mix of arc-170 leaks + a real broken edn fixture `:test::Wrapper/new` unresolved at `wat-tests/edn/roundtrip.wat:71`). So the honest test-side clause is **per-file, run-level, named**: an intueri naming cast replaced "forms-exercised" (which mumbles — drops the green-state and the artifact) with **`deftest-green(<name>)`** — it mirrors "checker-clean" in shape, claims only a named green deftest at stamp time (never whole-suite, never routine-gating), and is **self-verifiable at source / fails loud on drift** (`feedback_mark_the_source_not_memory`).

**The corrected wat bar (supersedes 245.0 §3/§4):**
- **L2 structural — `checker-clean`** (the checker accepts every form; verified via `stdlib::tests::every_stdlib_file_parses` inside the green lib gate).
- **L2 behavioral — `deftest-green(<name>)`** (the file's own forms are exercised by a NAMED, deterministic, currently-green deftest, runnable via `cargo test --release --test test <name>`).
- **L1** — the spell convergence including the comment-fidelity pass (cernere + conferre).

**The routine-gating dependency (recorded, not deferred-as-done):** these deftests are runnable + green NOW but live in the gate-*excluded* integration suite, so nothing re-runs them routinely. That is **future-rot protection**, not the present guarantee — and it belongs to task **#151** (non-leaky integration tier in the gate) + **arc 250** (self-enforcing vigilatum stamps). The stamp honestly attests the *stamp-time* truth (forms exercised by a named green deftest); it makes no routine-gating claim. When #151/250 land, the named deftests gain routine teeth and a forward stone re-derives the wat stamps' confidence.

## The stamp

```
;; vigilatum: 2026-06-04T02:28:55Z — vigilia 4-spell L1+L2=0, checker-clean + deftest-green(list-fold-aliases)
```
Top of `wat/list.wat`, mirroring the Rust `//! vigilatum:` convention (`;;` syntax, the corrected wat bar clause).

## Verification (against the project's actual gate, not the excluded suite)

- **Real gate green:** `green-gate.sh` PASS — test-build clean + **lib 895/0/1** with 245.1 in tree.
- **checker-clean:** `stdlib::tests::every_stdlib_file_parses ... ok`.
- **deftest-green:** `cargo test --release --test test list_fold_aliases` → `reduce-sum-i64 ok`, `fold-sum-i64 ok` (2/2).
- **No regression:** the ~53 integration-suite failures are pre-existing + gate-excluded by design (verified baseline = 53 with changes stashed out); none touch `list.wat`, `foldl`, or the new test.
- **git-state verified** (sonnet's strike): exactly `M wat/list.wat` + the new test file; no stray, no unauthorized git op.

## FM-11 — DONE, no deferral

The file is warded in fact: header reconciled to present-tense truth, forms exercised by a named green deftest, checker-clean verified in the green lib gate, stamp re-derived from the corrected bar. No "coverage-pending," no rune, no deferred fix — the one solvable gap (the unexercised claim) was closed inline with a real test. The routine-gating-against-rot is an *affirmative forward enabler* (#151/250), not a hole in this stamp.

## Deliverables

- `wat/list.wat` — header rewritten to present-tense truth + `vigilatum` stamp (first warded wat file).
- `wat-tests/core/list-fold-aliases.wat` — the execution test that gives the alias-delegation claim teeth.
- `DESIGN-STONE-245.0.md` §3/§4 — forward-corrected with the twice-refined bar (this SCORE is the source of the refinement).

## The close

245.1 closes. The first wat ward stands — and it did exactly what a first ward must: it taught the bar. "suite-green" was a guess that grounding proved both toothless and false; the ward forced it into `checker-clean + deftest-green(<name>)`, a bar that means precisely what it says and is verifiable at source. Next: **245.2 — `core.wat` archaeology-reconciliation** (the `DispatchRegistry` half-stale comment at `:14`/`:33` and the broader ~80 lines of historical narration must be verified-live-or-cut *before* the file can be warded — 245.0 §5).
