# Stone 245.0 — the instrument design: how a wat file gets warded

**Status:** ✅ **DESIGN SETTLED 2026-06-04.** Arc 245 opens (enabled-by gate cleared: 237.9 INSCRIPTION shipped, the numeric+equality stdlib surface is stable). This stone answers the STUB's instrument question — *what does "warded" mean for a wat file, and with what tool* — before any ward is cast. Grounded by a spawned **intueri** naming cast + live source verification.

---

## What 245.0 settles (the five answers)

1. **The instrument is `vigilia` itself** — cast with wat-kind spell selection. **No new aggregator is minted.**
2. **The stamp word stays `vigilatum`** — only the bar clause and comment syntax change.
3. **The wat bar (L2 floor) is `checker-clean + suite-green`** — the wat analog of "clippy-clean in-home."
4. **The primary wat warding hazard is stale arc-archaeology comments** — wat's analog of Rust's borrow/clippy surface; the L1 floor must include a comment-fidelity pass.
5. **`wat/core.wat` is FORBIDDEN from warding until its archaeology is reconciled** (a verified live lie sits in its header).

---

## §1 — The instrument: REUSE vigilia, do not mint a "wat-ward"

The STUB asked: is the instrument an intueri-named adapted procedure (a *"wat-ward"*), or vigilia cast on wat source? A spawned **intueri** naming cast (2026-06-04) returned a flat **REUSE** verdict, and I do not disagree:

- **vigilia is already kind-adaptive by its own spec.** It casts only the spells whose discipline matches the target's kind, and it *already names the wat spells*: `cernere` (phantom forms / language-spec conformance — "where applicable: wat / DSL / language files") and `probare` (substance). There was never a wat-shaped hole in the grimoire; the dig revealed it already fit. (The recurring convergence: *the substrate already had it.*)
- **"Warded" is language-independent.** Per `feedback_warded_means_annihilated` + `project_warded_homes_pattern`: *warded* is the CLAIM (failure-classes annihilated), `L1+L2=0` is the MEASUREMENT. Both survive the language change intact — `src/collection/mod.rs` and a warded `wat/list.wat` make the same promise in different syntax.
- **Minting "wat-ward" is the new-primitive-that-already-exists anti-pattern.** It would overclaim novelty (the inward set transfers verbatim), fork the grimoire's verb surface, and create two aggregators that must be kept consonant.

Four-questions (both options), from the cast:

| | Obvious | Simple | Honest | Good UX |
|---|---|---|---|---|
| **REUSE vigilia+selection** | Y | Y | Y | Y |
| MINT new "wat-ward" | N | N | N | N |

**Decision: cast `vigilia` on wat source with the wat-kind spell selection below. No new instrument.**

## §2 — The spell-transfer matrix

What the warding cast runs against a wat file, by whether the discipline transfers:

**Transfers verbatim (the universally-applicable inward set):**
- `intueri` — names + structure + comments (does the wat code speak)
- `solvere` — braided concerns; misplaced logic
- `purgare` — dead code / unused forms
- `struere` — per-form craft (values not places; the type-discipline wat *does* have — defclause annotations, `<-` param types)
- `sequi` — state threading (wat's monadic `(s,d)->(s,D)` service handlers)
- `temperare` — wasteful computation

**wat-kind additions (the language-specific concerns — this is what makes it a *wat* ward):**
- `cernere` — **phantom forms / language-spec conformance.** Does every form conform to the wat language spec; are there phantom/retired constructs (`define-dispatch`, `define-alias`, the `'2` suffix) still present or referenced?
- `conferre` — **doc-vs-territory fidelity.** The load-bearing one for wat (see §5): do the heavy stdlib comments match live behavior?
- `probare` — substance vs description (where a file is more spec than code).
- `circumspicere` — **cast LAST** (the surround: what the inward lenses left uncovered).
- `secare` — only where the file uses parallel primitives (`wat/kernel/*` — channels, threads). Skip for pure stdlib leaves.

**Does NOT transfer (Rust-specific):**
- **clippy** — there is no wat linter (see §6). Its slot in the bar is taken by `checker-clean` (§3).
- **the `src/<noun>/` lift** — wat files are stdlib *source*, warded **in place**. There is no module to lift into a home; "home" framing does not apply. A wat file is warded where it lives.

## §3 — The wat bar: `checker-clean + suite-green`

The Rust stamp's L2 phrase is "clippy-clean in-home." wat has no clippy, so the floor is rebuilt from what exists **now**:

- **L2 — structural: `checker-clean`.** The wat type-checker accepts every form in the file with **zero warnings/diagnostics**. This is wat's clippy-analog: the language's own static gate passes clean on the file's forms.
- **L2 — behavioral: `suite-green`.** Every `wat-tests/` test exercising the file's forms passes; the documented behavior is the tested behavior. wat needs this second L2 because the checker proves *less* about a Lisp stdlib than clippy+borrowck prove about Rust — the behavioral suite carries the weight the static gate can't.
- **L1 — the spell convergence**, including a mandatory **comment-fidelity pass** (`cernere` + `conferre`) — see §5. L1 lies before L2 mumbles; a file with a stale-but-compiling comment is L1-divergent even at suite-green.

`L1+L2=0` remains the measurement; the *bar clause* is what changes: **`checker-clean + suite-green`** (reads as a floor, not a goal — parallel in shape to "clippy-clean in-home").

## §4 — The wat stamp

The warding token stays `vigilatum` (the Latin past-participle "watched/guarded" — it names the language-independent **result** of vigilia, and keeps **one grep-able warding token** across the whole substrate, Rust and wat alike). Only the syntax (`;;` not `//!`) and the bar clause change:

```
;; vigilatum: <UTC-ts> — vigilia <N>-spell L1+L2=0, checker-clean + suite-green
```

`<N>` records the actual count of spells cast on that file (the wat-kind selection varies: a pure leaf is fewer; a `kernel/` file adds `secare`). Placed at the top of the file, as the Rust stamp sits at the top of `mod.rs`.

## §5 — The primary wat warding hazard: stale arc-archaeology

**This is the arc's central deposit.** Rust's dominant warding hazard is borrow-shape / clippy pedantry. **wat's is stale comments** — specifically arc-archaeology: the stdlib files carry dense historical narration referencing retired forms, and that narration rots while the code moves on.

**Verified live lie (the worked example, confirmed this session against the disk):** `wat/core.wat:14` and `:33` both read *"…the per-type leaves (…) **and the DispatchRegistry** remain for other ops."* The `DispatchRegistry` was **HARD CUT** — `grep -rn DispatchRegistry src/` returns **0**. The comment is **half-stale**: the per-Type leaves (`:Vector/length` etc.) genuinely *do* remain (`src/collection/eval.rs:26/49/75/92`), but the `DispatchRegistry` clause is a **Level-1 active lie**. The precise fix is to cut the `and the DispatchRegistry` clause, keep the leaves clause.

**Why this is load-bearing for the whole arc:** a `vigilatum` stamp asserts "this file's failure-classes are annihilated and it speaks truth." Stamping a file whose own header comments lie makes the stamp **vouch for the lie** — the worst failure mode. So:

- The wat L1 floor **must** include a `cernere` (phantom-form) + `conferre` (doc-vs-territory) pass: every historical comment is verified-live or cut/dated.
- **`wat/core.wat` is FORBIDDEN from warding until its archaeology is reconciled** (`:14`, `:33`, and the broader ~80 lines of historical narration referencing `define-dispatch`, `define-alias`, the `'2` suffix). The STUB's "do not ward a file you are about to rewrite" reasoning thus extends *past* 237's churn — even on a now-frozen file, the comments are a live lie surface.

## §6 — Deferred enabler: the wat-native linter

There is **no wat linter today**, and warding does not wait for one. The linter is a deliberate downstream artifact (banked in `~/work/holon/scratch/`): it gets **built in wat**, which means **wat must be stable first** — the language a linter lints must settle before the linter (itself a wat program) can exist. Warding the corpus is part of what *gets* wat stable enough to build it.

**The lineage (record it so it is not lost):** corpus-warding now (`checker-clean + suite-green`) → wat stabilizes → the wat-native linter is built in wat → it adds an **automated L2 gate** (the true clippy-analog) → at that point the wat stamp's bar clause **graduates** (a forward stone re-derives the stamps with the linter in the floor). Until then, `checker-clean + suite-green` is not a stopgap — it is *the* honest bar a language that cannot yet lint itself can promise.

## §7 — Scope, bar-by-kind, slicing

**Scope (crawled): 61 files, ~11.4k LOC** — `wat/` stdlib (26 files, ~4.3k LOC) + `wat-tests/` (35 files, ~7.2k LOC).

**Bar by kind** (per `feedback_selective_lift_and_ward`):
- **`wat/` stdlib — BLANKET bar.** The foundation every program composes on; "all of it at the bar" is the defensible call. Uniform trustworthiness is the point.
- **`wat-tests/` — SELECTIVE.** Tests must be correct + honest, but the warding instrument and bar may differ (e.g. `vocare`-style test-vantage matters more than `temperare` waste). Decide the test bar when the stdlib is warded; do not blanket-ward tests on reflex.

**Slicing (confirmed from the STUB, reordered by the §5 finding):**
1. **245.0** — this design. ✅
2. **245.1** — ward a small **clean leaf** end-to-end first (prove the instrument + materialize the stamp + bar concretely on an easy file): candidate `wat/holon/Log.wat` (20 LOC) or `wat/list.wat` (18 LOC).
3. **245.2** — `core.wat` **archaeology-reconciliation FIRST** (cut/date the stale comments — §5), *then* ward. The hardest file; do not ward before reconciling.
4. **245.3** — remaining stdlib core (`list`, `Record`, `runtime`, `stream`, `edn`).
5. **245.4** — stdlib `holon/*` family.
6. **245.5** — stdlib `kernel/*` family (adds `secare` — parallel primitives).
7. **245.6** — `wat-tests/` (per the selective bar decided after the stdlib closes).
8. **245.N** — INSCRIPTION + the wat-corpus-warding doctrine (the stale-archaeology hazard is the headline deposit).

## §8 — The first target

**245.1 = ward one small clean leaf** to prove the wat-warding loop end-to-end and pin down the concrete stamp + bar on an easy file before the archaeology-laden ones. This mirrors the homes-walk's first convergence (prove the ritual on a bounded target, then scale). Target selection (a leaf with the least archaeology) is the first act of 245.1.

---

## Provenance

- **intueri naming cast** (spawned 2026-06-04, embedded discipline, read `core.wat` / `holon/Log.wat` / `list.wat` / `src/collection/mod.rs` / this STUB): returned REUSE + `vigilatum`-stays + `checker-clean + suite-green` + the stale-archaeology catch. Verdict adopted in full.
- **Live verification** (orchestrator, same session, per `feedback_ground_against_right_target`): independently weighed the cast's absence-claim against the disk — `grep -rn DispatchRegistry src/` = 0 (lie confirmed), and sharpened it to the half-stale precise clause. The cast's flagship catch survived grounding against the LIVE target, not the cast's own re-search.
- **User constraint** (2026-06-04): no wat linter today; it is built in wat once the language is stable → §6 deferred-enabler lineage.
