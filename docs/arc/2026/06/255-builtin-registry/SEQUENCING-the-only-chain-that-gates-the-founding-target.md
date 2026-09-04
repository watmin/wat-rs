# SEQUENCING — one chain gates the founding target; everything else is parallel

> **Builder, 2026-09-04:** *"we have many paths before .... the steps to where we need to go .... the
> curious thing.. they lead to the destination... but... getting them all complete is actually
> required to reach the destination... so.... which steps make the next more tractable?"*
>
> Every number below was measured on 2026-09-04 and every one carries how to re-derive it. This
> document is a MAP, not a plan: it records what gates what, and it is falsifiable.

## THE ONE CHAIN

```
Phase 3a — resolve asks the registry; `is_reserved_prefix` dies   ⭐ THE ARC'S FOUNDING TARGET
  └── gated on the corpus flip being DECIDABLE
       (patch `is_resolvable_call_head` → build → sweep → revert; today 130 of 615 files fail)
       └── corpus = 37 names = 31 VERBS + 6 NON-VERB ARTIFACTS
            │
            ├── (a) THE SIX ARTIFACTS NEED A STORY, NOT A REGISTRATION      ⛔ HARD PREREQUISITE
            │       :wat::type::Tuple · ::i64 · ::String · ::Vector   arc 251 dual-read spelling
            │       :wat::core::None                                  an Option unit VARIANT
            │       :wat::rete::f64::>X                               a deliberately un-minted
            │                                                         negative-control probe head
            │       ★ NONE of these can EVER hold a registry row. The corpus therefore cannot
            │       reach 0 by registering, and Phase 3a's real gate is a RULING about how the
            │       flip treats them — work nothing else in this campaign produces, on no list
            │       before this document.
            │
            ├── (b) ~29 RETE_OPS rows called in the corpus     (of 37 unregistered, 74 total)
            │        └── (b0) THREE ORPHAN core_name TARGETS                ⛔ HARD GATE
            │              :wat::core::Vector · :wat::core::cond · :wat::core::reduce
            │              `no_dangling_or_chained_aliases` PANICS on an alias whose target is
            │              not a registered row (`src/intrinsic/mod.rs:2119`). Three rows gate
            │              twenty-nine.
            │              ⚠ Each needs a DIFFERENT answer and one may have none:
            │                 cond    — "zero runtime registry entry at all" (DESIGN-CAMPAIGN)
            │                 reduce  — a wat-side `defalias` since Stone 1c-f; `defalias` writes
            │                           to `sym.functions`, never `registry()`. The FOURTH-registry
            │                           fork owns it.
            │                 Vector  — a bare type constructor, deliberately left unhomed by the
            │                           per-type HOME campaign's own STOP-3.
            │
            └── (c) :wat::eval-ast! + :wat::eval-with-defs!    334 corpus sites, independent of (b)
```

## EVERYTHING ELSE IS PARALLEL — it gates nothing on the chain, and nothing on the chain gates it

```
Phase 3b — check asks the registry          UNBLOCKED. 432/432 round-trip (STONE-the-round-trip-closes).
                                            Kills register_builtins' 302 duplicates of 325.
the DEBT ledger split                       121 → 41 + 60 + 20. Honesty; makes the FINISH LINE
                                            reachable. Gates nothing.
the 270 both-axes grading batch             holon 91 · kernel 49 · time 41 · io 29 · core 20 …
                                            IS the roadmap's last phase, not a prerequisite to it.
the 34 special forms with no @syntax        reflection completeness.
the 19 rows that lie about arity            of 87 Variadic rows.
the FOURTH registry                         stdlib macros + every wat-defined verb, invisible.
                                            ⚠ (b0)'s `reduce` is a SYMPTOM of this one.
```

## ⛔ A SUSPICION I HELD AND MEASURED FALSE

I expected the **DEBT split to be a prerequisite for Phase 3b** — the theory being that if `CheckEnv`
starts synthesising schemes from registry rows, the 60 rows served by a custom `infer_*` arm could be
shadowed by a weaker rank-1 scheme.

**Measured false.** `src/check.rs`'s own comment at the arity fallback states the order:

> *"moot in practice (both are dispatched by their own literal match arms long before this
> fallback), but stated rather than relied upon."*

The literal `infer_*` arms are matched FIRST. Phase 3b does not need the split.

⚠ **But note what that comment does and does not promise.** It says the current dispatch order makes
it moot. It does not say a future consult placed EARLIER would be safe — and Phase 3a's own corpus
experiment is exactly such an earlier consult, one layer down. **If Phase 3b's implementation hoists
the registry consult above the literal arms, this measurement expires.** Re-measure at that stone;
do not carry this line forward as a licence.

## The measured ground, with re-derivation

```
corpus                 37 names · 638 sites · 130 of 615 files fail the flip
                       WORKLIST-the-121…md carries the four-step procedure; RUN IT, do not cite it.
registry               553 rows · 70 SpecialForm · 37 alias · 87 Variadic · 517 no @syntax
                       ./target/release/wat wat-scripts/scratch-pad/255-registry-census.wat
RETE_OPS               74 rows · 37 registered · 37 not · 3 orphan core_name targets
                       grep -oP '^\s+core_name: "\K[^"]+' src/rete/vocabulary.rs | sort -u
                       then comm -23 against the census's name list
round trip             432 of 432 · generics 87/87 · freeze list EMPTY
                       -E 'test(probe_can_doc_types_reconstruct_the_checker_scheme)' --no-capture
DEBT                   121 = 41 SpecialForm(wrong shape) + 60 infer-arm(stronger authority) + 20 owed
                       ⚠ the 60 needed a MULTI-LINE-AWARE arm detector; a single-line regex returned
                       "0 of 80" and would have said the whole ledger is owed. CALIBRATE on
                       `:wat::core::=` / `<` / `foldl` / `map` before trusting any count.
grading backlog        totality Unreviewed 378 · expand Unreviewed 295 · BOTH 270 · Partial 50
                       ★ only Totality and ExpandTime HAVE an Unreviewed pole. Purity, Determinism
                       and Category do not — they are complete by construction. The endgame is two
                       axes, not five.
```

## What this document does NOT settle

- **Whether (b0) is solvable.** One of the three orphans may have no answer under today's mechanisms,
  and that would make (b) — and therefore the founding target — depend on the FOURTH-registry fork.
  **Finding that out is one stone and it changes the whole chain's viability**, which is why it is
  the recommended first strike.
- **What the six artifacts' story IS.** Only that they need one and that registration cannot be it.
- **Any ordering among the parallel work.** They are parallel; pick by value, not by dependency.
