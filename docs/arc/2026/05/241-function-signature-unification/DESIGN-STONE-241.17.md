# DESIGN — Stone 241.17 — `:wat::core::defmacro` SIGNATURE MIGRATION (closes arc 177 + def-family parser unification complete)

**Status:** STRIKE-READY (2026-05-29 very late). User direction: *"target acquired - annihilation enqueued - this arc lives as long as it must. 177 is closed by our work here - the next 241 stone is the closure for 177."*

**Battle plan revision:** arc 177 (defmacro-syntax-clojure) ABSORBED into arc 241. Stone 241.17 = defmacro signature migration; Stone 241.18 = INSCRIPTION closes BOTH arcs. The def-family parser unification (Stones 241.1-241.5 + this) reaches genuine completion.

## What arc 177 was

Arc 177 (`docs/arc/2026/05/177-defmacro-syntax-clojure/DESIGN.md`) was a STUB opened 2026-05-12:

> *"revise defmacro syntax — specifically make the args like defn, fn and overall more clojure-y."*

The design sketch said "TBD; user fills the design." Cross-references named arc 166 (defn args shape to mirror) + arc 167 (fn flat signature) + arc 172 (Scheme → Clojure macro flavor swap, shipped) + arc 173 (Clojure macro feature parity) + arc 174 (defclause sibling).

**Stone 241.17 fills the TBD + closes arc 177 by absorption** — defmacro signature mirrors defn shape (Vector-of-triples per canonical `parse_argspec_triples`). Arc 177's DESIGN.md becomes historical record per `feedback_inscription_immutable`.

## The shape migration

### Current (paren-pair-with-type form; arc 010/150 lineage)

```scheme
(:wat::core::defmacro
  (:wat::core::defn
    (name :AST<wat::core::nil>)
    & (rest :AST<wat::core::Vector<wat::WatAST>>)
    -> :AST<wat::core::nil>)
  `(:wat::core::def ~name (:wat::core::fn ~@rest)))
```

**3 items:** defmacro head + signature-list + body. The signature-list is itself an internal list shape: `(macro-name (param :Type) (param :Type) & (rest :Type) -> :ReturnType)`.

### Target (mirror defn — canonical Vector-triple form)

```scheme
(:wat::core::defmacro :wat::core::defn
  [name <- :AST<wat::core::nil>
   & rest <- :AST<wat::core::Vector<wat::WatAST>>]
  -> :AST<wat::core::nil>
  `(:wat::core::def ~name (:wat::core::fn ~@rest)))
```

**6 items:** defmacro head + macro-name keyword + argspec Vector + `->` symbol + return-type keyword + body. Mirrors defn shape exactly.

### Optional metadata-map (mirror defn — Stone 241.6 storage)

Post-stone, defmacro inherits the metadata-map mechanism from Stone 241.6 just as defn does:

```scheme
(:wat::core::defmacro :my::macro
  {:doc "this is a docstring"}
  [x <- :AST<wat::core::nil>]
  -> :AST<wat::core::nil>
  `(:my::wrapper ~x))
```

7 items: defmacro head + macro-name keyword + metadata-map + argspec Vector + `->` symbol + return-type keyword + body.

This isn't load-bearing for Stone 241.17 — metadata-map is optional; defmacro's metadata-map handling can leverage the same `binding_metadata` storage Stone 241.6 minted. If sonnet ships the basic 6-item shape first and metadata-map landing requires a follow-up (Stone 241.17.fix?), that's acceptable.

## Substrate scope

### S1 — Migrate `parse_defmacro_form` to canonical parser

`src/macros.rs:320` — `parse_defmacro_form` currently calls `parse_defmacro_signature` (line 355) to parse the old paren-pair-list signature shape.

Target: `parse_defmacro_form` reads 6+ items (head + name + argspec + `->` + return-type + body); calls `parse_argspec_triples` on the argspec Vector (item 2); MacroDef construction unchanged.

### S2 — DELETE `parse_defmacro_signature`

`src/macros.rs:355` — the legacy signature parser. ~80+ lines. DELETED.

### S3 — HARD-CUT-rejection for old paren-pair shape

If a defmacro form arrives with the OLD shape (3 items where item 1 is a List with paren-pair params), the substrate emits `MalformedForm` with structured reason pointing at the new shape:

```rust
"old defmacro signature shape (paren-pair with type) is retired (Stone 241.17); use canonical Vector-of-triples form: (:wat::core::defmacro :name [param <- :Type ...] -> :Ret body)"
```

NOTE: No new RETIREMENT_TABLE entry — `:wat::core::defmacro` keyword stays alive; only the inner SIGNATURE SHAPE changes. The rejection is shape-internal, not keyword-replacement.

### S4 — Cascade migration of 29 wat/ defmacro callers

Per-file judgment for each:
- `wat/core.wat:180` — defn macro (canonical example; user-facing-critical)
- `wat/runtime.wat:18` — (verify presence post-Stone-241.12 deletion of define-alias macro)
- `wat/test.wat` × 13 sites — test infrastructure macros (deftest etc.)
- `wat/Record.wat` × 2 sites
- `wat/holon/*.wat` × 6 sites — algebra macros (Log/Sequential/Amplify/Bigram/Trigram)

Bulk pattern transformation (mechanical):
- OLD: `(defmacro (NAME (P :T) ... & (R :T) -> :RET) BODY)`
- NEW: `(defmacro NAME [P <- :T ... & R <- :T] -> :RET BODY)`

### S5 — Cascade migration of 36 tests/ references

Per-file judgment. Many tests reference defmacro in AST-construction or string-form fixtures. Sonnet audits + migrates each. Tests with `:wat::core::defmacro` in:
- Quoted wat-source strings → migrate fixtures
- AST-construction code → migrate construction calls
- Comment references → preserve historical (per `feedback_inscription_immutable`)
- Assertions about defmacro structure → update post-migration shape

### S6 — Doc cascade migration

`docs/USER-GUIDE.md` + `docs/CLOJURE-ROSETTA.md` + `docs/INTENTIONS.md` reference defmacro. Update active examples to new shape; preserve historical comments.

### S7 — Reflection emitter audit

Per Stone 241.12/13/14/15/16 trap-door precedent: grep for `Keyword.*defmacro` AST emitters; migrate emitter output shape to new form.

### S8 — Probe verification

`tests/probe_arc241_stone17_defmacro_canonical.rs` (NEW). Contracts:
- C01: defmacro with new canonical Vector-triple shape WORKS
- C02: old paren-pair shape REJECTED with structured error
- C03: defmacro with `& rest` rest-binder in canonical shape WORKS
- C04: defmacro metadata-map (if shipped) WORKS

### S9 — SCORE doc

Per `feedback_score_present_check_before_closure`. `SCORE-STONE-241.17.md`.

## Locked decisions

### D1 — Defmacro signature mirrors defn shape EXACTLY

6-item form: head + name + argspec Vector + `->` + return-type + body. Same shape as defn per Stone 241.6/241.7. Triple-arrow argspec. `&` rest-marker in Vector.

### D2 — Old paren-pair shape HARD CUT

Per `feedback_hard_cut_admits_no_bypasses`. No shims. Old shape REJECTED at startup-check with structured reason pointing at canonical.

### D3 — `parse_argspec_triples` consumed (Stone 241.1 mint's third major consumer beyond fn/defclause)

defmacro becomes the THIRD entity-kind routing through canonical argspec parser. Phase 1's parser unification reaches genuine COMPLETION at this stone.

### D4 — Cascade per substrate-as-teacher

29 wat/ + 36 tests/ + docs. Mechanical bulk transformation. Auto-fixer pattern available if surfaces; ephemeral discipline (per Stone 241.11 precedent).

### D5 — Metadata-map on defmacro DEFERRED if scope creeps

If basic 6-item form lands clean, ship. If metadata-map adds significant scope, defer to Stone 241.17.fix. Don't bundle if it risks scope creep.

### D6 — Vigilia NOT required (no new namespaced home)

### D7 — INTERSTITIAL orchestrator-exclusive

### D8 — SCORE-write at end

### D9 — Stone 241.18 (INSCRIPTION) OFF-LIMITS

## Trap-door audit

### T1 — `parse_defmacro_signature` deletion may cascade through callers

The function may be called from sites beyond `parse_defmacro_form`. Grep first. Likely zero external callers (private fn) but verify.

### T2 — Old paren-pair shape vs new Vector shape disambiguation

The substrate must distinguish 3-item old form from 6-item new form. Item count is the discriminator:
- 3 items → old form → HARD-CUT-reject (or item-1 is a List with paren-pair params → reject)
- 6 items → new form → process via canonical
- 7 items → new form with metadata-map → process via canonical

Edge case: malformed forms in between → existing MalformedForm path. Sonnet handles.

### T3 — Bulk sed for 29 wat/ + 36 tests/ migrations may match unintended sites

The pattern `(:wat::core::defmacro\n  (NAME` → `(:wat::core::defmacro NAME` is shape-level. Bulk sed risky; per-file edit safer for this many sites. Auto-fixer-with-syntax-awareness preferable (parse old AST + emit new AST) but adds complexity.

Sonnet judges: bulk-sed with per-file diff review, OR auto-fixer-with-parser. Either acceptable if EPHEMERAL.

### T4 — `wat/core.wat:180` defn macro is THE canonical defn definition

This is the macro that ALL defn callers depend on. The migration must preserve defn's runtime behavior exactly. After migration, the defn macro at wat/core.wat:180 still expands correctly; all defn callers continue to work. Sonnet verifies via lib + workspace test suite.

### T5 — Tests use defmacro in wat-source strings vs AST-construction

Sonnet must distinguish. wat-source strings inside `r#"..."#` literals migrate textually. AST-construction code migrates via call-site rewrite.

### T6 — Reflection emitter trap-door (Stone 241.12/13/14/15/16 class)

Grep for `Keyword.*defmacro` AST emitters. If any construct old-shape AST, migrate.

### T7 — Sonnet "preserve paren-pair for compatibility" temptation

Per D2 + `feedback_hard_cut_admits_no_bypasses`. STOP if surfaces.

## STOP triggers — REJECTION

1. Compile errors not traced to defmacro migration cascade
2. Lib < 890 (post-Stone-241.14.fix baseline)
3. **180 min elapsed** (this stone is BIG — comparable to Stone 241.16)
4. holon-rs touched (STOP-5)
5. Old paren-pair shape preserved as "compatibility" path → D2 + `feedback_hard_cut_admits_no_bypasses` violation
6. `parse_defmacro_signature` PRESERVED (D3 violation — DELETED is the action)
7. Files outside permitted scope (`src/macros.rs` / `src/closure_extract.rs` if reflection emitters touched / `wat/*.wat` (29 callers) / test files referencing defmacro (36 files; per-file judgment) / doc files / `tests/probe_arc241_stone17_*` / SCORE doc)
8. Stone 241.17 probe < N/N
9. Stone 241.x or arc 237/238/242 probes regress
10. Clippy > 940 (looser gate; substrate refactor + 29+36 cascade)
11. Auto-fixer crate survives commit
12. Sonnet writes to INTERSTITIAL → D7 + `feedback_sonnet_never_drafts_interstitial` violation
13. SCORE-STONE-241.17.md NOT authored at end → D8 + `feedback_score_present_check_before_closure` violation
14. Stone 241.18 scope touched (INSCRIPTION) → D9 violation

## FM 2-bis evidence

`tests/probe_arc241_stone17_defmacro_canonical.rs` (NEW; written + verified disconfirms at HEAD before BRIEF spawns).

## Calibration

**Target band: 90-180 min Mode A.** BIG cascade (29 wat/ + 36 tests + docs) but mechanical shape transformation. Stone 241.16 ship at ~33.8 min suggests under-band possible given mature apparatus, but 65+ migrations is more than Stone 241.16's ~30 sites.

Per `feedback_stone_briefs_cite_prior_score`: BRIEF cites SCORE-STONE-241.16.md (parse_define_form deletion + 30-site cascade + 2 trap-doors); SCORE-STONE-241.13.md (substrate scaffolding deletion + 6 test files); SCORE-STONE-241.14.md (storage refactor + walker rewrite + 5-site test cascade).

## What this unblocks

**Stone 241.18 — INSCRIPTION closes BOTH arc 241 + arc 177.** Explicit acknowledgment:
- Arc 177 absorbed into arc 241; the TBD design filled by Stone 241.17
- def-family parser unification COMPLETE (fn/defn/defclause/defmacro all route through canonical parse_argspec_triples)
- The Clojure conversion at the def-family layer DONE (define dead; defn lives; defmacro signature symmetrical)
- 6-stone campaign expanded to 7 stones (12 + 13 + 14 + 14.fix + 15 + 16 + 17 + 18 INSCRIPTION)

**The Clojure conversion continues** at the broader form-level via remaining arcs:
- Arc 172 — comma-to-apostrophe-dispatch
- Arc 173/174 — clojure macros + features
- Arc 175/176/177 ABSORBED here (177 closed via 241.17)
- Arc 178/179/180/181 — other syntax-clojure work
- Arcs 175/176 — enum/struct syntax clojure (still queued)

**Arc 237.8b** reopens after Stone 241.18

**The def-family parser unification is GENUINELY COMPLETE post-stone:** Stone 241.1 mint canonical parser; Stones 241.2/3 migrated A1/A2/A3 fn + A4 defclause; Stone 241.17 migrates defmacro. All four entity kinds with argspecs now route through `parse_argspec_triples`. The "ONE canonical argspec parser" goal from arc 241's start ACHIEVED.
