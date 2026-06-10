# Arc 251 — The Clojure-faithful symbolic surface (types-as-forms)

**Status:** OPEN 2026-06-06; **PROMOTED to THE active arc 2026-06-09** — the
arc-213 program-over-the-wire serializer surfaced the `/`-in-keyword non-EDN abuse
(the third symptom this session after struct-destructure and `::`), and the builder
made the call: *"the wat invented forms are nearing their end — they were a bridge
to get us here… we go for parity."* 251 is now 213's prerequisite. Destination
settled: full core.typed/Typed Clojure acceptance.

> ### ⛕ PARITY REFINEMENT (2026-06-09) — `->` retires fully
> The pre-today moves (below) kept `->` for the **return** arrow and for fn-types.
> Full core.typed parity (builder, 2026-06-09, confirmed against the User-Guide /
> getting-started reference) retires `->` ENTIRELY:
> - **`:-`** = ascription, in BOTH param AND return position:
>   `(wat.core/defn add5 {:doc "..."} [x :- wat.type/i64] :- wat.type/i64 (wat.core/+ 5 x))`
> - **`:->`** = the function-type arrow, only INSIDE a type expression:
>   `[f :- [wat.type/i64 :-> wat.type/i64]]`
> - **`ann-form`** `(ann-form expr type)` = expression ascription (verified parity);
>   **`defalias`** names a type; **`ann`** annotates a var. There is NO `ann-type`
>   wrapper — a parametric arg type is just the form inline in the `:-` slot:
>   `[m :- (wat.type/HashMap wat.type/String wat.type/i64)]`.
> Move 1 below expands accordingly (`<-`→`:-` AND return `->`→`:-`); move 4's fn-type
> arrow becomes `:->`. Everything else in the design holds.

**Name is PROVISIONAL** — `251-types-as-forms` names move 4 (one facet). The true
scope is broader: the keyword→symbol role inversion of the head/type/operator
surface. intueri-cast the real name when the DESIGN settles (per arc-250 precedent
+ `feedback_intueri_names_all_things`).

**Origin:** the arc-109 NOTEs, built up over the clojure-ination (arcs 247 fn-first,
249 threading + total-pure macros). The builder has internalized core.typed and the
direction is no longer a pointer but a destination:
- `docs/arc/2026/04/109-kill-std/NOTE-typed-form-and-type-namespace.md` (★ ADDENDUM 2026-06-06 — moves 1-4 + `ann-form`)
- `docs/arc/2026/04/109-kill-std/NOTE-generic-bracket-syntax-edn.md` (★ ADDENDUM 2026-06-06 — parametrics-as-forms dissolves the bracket lexer)

Builder, 2026-06-06: *"we basically landed on fully accepting clojure typed forms."*

---

## The grounded reframe — wat currently INVERTS Clojure

This is the insight that reorganizes the whole arc. Grounded against the lexer
this session:

| Role | Clojure | wat TODAY | grounded at |
|---|---|---|---|
| heads / operators / types / declarators | **symbols** (`+`, `defn`, `t/Int`) | **keywords** (`:wat::core::+`, `:wat::core::i64`, `:wat::core::defn`) | `lex_keyword` lexer.rs:634; resolved by FQDN |
| local binders / value-position names | symbols (`x`) | symbols (`x`) | `lex_symbol` lexer.rs:794; `WatAST::Symbol` = binders, struct-destructure, `(name rhs)` def-shorthand |
| data (map keys, enum-ish markers) | **keywords** (`:foo`) | — (keywords are occupied by the operator/type role) | — |

wat put **keywords** where Clojure puts **symbols**, and reserved **symbols** for
the one place Clojure also uses them (binders). "Fully accepting clojure typed
forms" is, at its root, **inverting that role assignment** for the
head/type/operator positions. Everything else (the type namespace, parametrics-as-
forms, the dotted surface, `:-`) rides on that inversion.

### The de-risk discovered this session

`is_symbol_break` (lexer.rs:457-469) breaks on whitespace, `()[]{}"`, `;`, `,` —
**but NOT on `.` or `/`.** So `wat.core/+` and `wat.type/i64` **already lex as
single symbol tokens today.** The dotted Clojure surface is NOT a lexer rewrite.
The work is in the **parser + resolver**: teach a symbol *head* to resolve to the
same entity a keyword head resolves to now. The lexer is already at the destination.

### What the inversion DISSOLVES

The `WatAST::Keyword` type/value context-polymorphism — a keyword is type-or-value
by *position only* (named the `src/ast/` ward-enabler at arc 244's close) — is a
problem ONLY because `:wat::core::i64` is the same lexeme in type and value
position. Move types into their own **symbol** namespace (`wat.type/i64`) and the
lexeme itself carries the distinction: a `wat.type/` symbol is a type, a `:foo`
keyword is data. The disambiguation isn't *solved* — it's **dissolved**. This is
the same shape as every convergence in the record: dig reveals the representation
change eliminates the problem rather than requiring it be patched.

---

## The destination (core.typed faithful)

```clojure
(fn
  [xs :- (wat.type/Vector wat.type/i64)]
  -> wat.type/i64
  (wat.core/foldl wat.core/+ 0 xs))
```

vs today:

```
(:wat::core::fn [xs <- :wat::core::Vector<:wat::core::i64>] -> :wat::core::i64
  (:wat::core::foldl :wat::core::+ 0 xs))
```

Near-verbatim core.typed: `wat.type/` for `t/`, `wat.core/` for `clojure.core/`,
`:-` the annotation marker, `->` the return arrow, parametric types as ordinary
forms the reader reads.

---

## The moves (from the NOTEs, consolidated)

1. **`<-` → `:-`** — core.typed's annotation arrow. `:-` stays a *keyword*
   (a syntactic marker inside the binder vector — clojure-honest; core.typed does
   exactly this). Independent of the symbol swap mechanically; co-located with type
   tokens on every signature line.
2. **`:wat::type::` / `wat.type/` namespace** — types move out of `:wat::core::`
   (which keeps the operators). Mirrors core.typed's `t/`.
3. **Dotted clojure-form surface** — `wat.type/i64`, `wat.core/+` (segments by `.`,
   name by single `/`). The post-keyword rendering. **Lexes today** (the de-risk above).
4. **Parametrics become FORMS** — `(wat.type/Vector wat.type/i64)`,
   `(wat.type/HashMap wat.type/String wat.type/i64)`, fn types `[wat.type/i64 -> wat.type/bool]`.
   **This is the payoff:** deletes the `lex_keyword` bracket machinery (angle/paren
   depth, whitespace hard-error, operator-`<` disambiguation, comma rules —
   lexer.rs:605-730), ends `keyword/of`'s string-concat type construction, ends the
   run-threads Bundle-surgery type destructuring, and puts types inside the
   total-pure macro engine's form domain (arc 249).
5. **`ann-form`** — `(ann-form expr type)` — the local-ascription site. Three
   annotation sites — binder (`x :- T`), return (`-> T`), expression
   (`(ann-form e T)`) — all taking the SAME type-form grammar.
6. **Rust interop joins the inversion** — `:rust::whatever::SomeThing` →
   `rust.whatever/SomeThing` (builder, 2026-06-09). The `:rust::` keyword interop
   surface is the SAME keyword-as-head abuse as `:wat::core::`; under the role
   inversion it becomes a namespaced symbol in the `rust.` namespace, exactly
   parallel to `wat.core/` and `wat.type/`. **This falls out of the existing
   machinery for free:** the 251.1b normalize transform's `ns_to_wat_path` is
   generic over the namespace, so `rust.lru/LruCache` already normalizes to
   `:rust::lru::LruCache` with no special-casing — `:rust::` is already a reserved
   prefix (`src/resolve/reserved.rs`), and the `use!` coverage gate operates on the
   post-normalize `:rust::` keyword, unaffected. The work is purely the **251.5
   corpus cut** (rewrite `:rust::a::b::C` source spellings to `rust.a.b/C`) plus the
   `use!` declaration surface (`(wat.core/use! rust.lru/LruCache)`). One inversion,
   three namespaces (`wat.core`, `wat.type`, `rust`); keywords return to data.

---

## Sequencing — the verdict (four-questions, grounded)

The builder asked: symbol-swap-first, or `:-`-first? Grounded answer below.

**Candidate A — symbol-first (the role inversion is the spine); `:-` folds into the signature-rewrite cascade.**
- Obvious? **YES** — the disk shows wat inverts Clojure; the clojure-ination IS that
  inversion; the type namespace (2), dotted surface (3), and parametrics-as-forms (4)
  ALL require symbols-as-heads/types to exist first. The lexer already reads dotted
  symbols, so the foundation is reachable immediately.
- Simple? **YES** — one representational change (keyword→symbol for head/type
  positions) is the atomic spine; `:-` rides its cascade rather than being a separate
  concern.
- Honest? **YES** — folding `:-` avoids double-churning every signature line; builds
  no throwaway scaffolding. (NOTE's explicit anti-pattern: "three churns of every
  signature in the corpus.")
- Good UX? **YES** — the corpus surface changes once, not twice.
- → **FOUR YES.**

**Candidate B — `:-`-first, alone.**
- Obvious? **NO** — `:-` is an arrow swap touching every signature line; the symbol
  swap touches those SAME lines again. Doing the small co-located thing first is not
  obvious sequencing.
- Honest? **NO** — NOTE: *"Don't ship `:-` in isolation if the `wat.type/` + dotted
  form are coming right behind it — that's three churns of every signature."* Doing it
  first knowingly double-churns.
- → **DISQUALIFIED at Obvious + Honest.**

**Verdict: symbol-first.** `:-` is NOT a prerequisite for anything and must NOT go
first alone; it folds into whichever stone rewrites the signatures. The dotted
surface (3) and the keyword-as-type/op HARD-CUT land LAST (the biggest reader-
surface change). Parametrics-as-forms (4) is the payoff that kills the bracket lexer.

---

## Warded-homes discipline — lift-and-ward, relentless (builder direction 2026-06-06)

> *"we lift and ward segments into warded homes — surgical, scoped and bar raised — relentless."*

Arc 251 is not only the surface inversion — it is a **mining run** on the
runtime.rs / check.rs quarries (per `project_warded_homes_pattern`). The segments
this arc touches are exactly the resolution/dispatch machinery that today lives
scattered and flat. As each stone touches a segment, it **lifts that segment into a
warded home** and earns it: surgical (diff-scoped), bar-raised (vigilia to L1+L2=0),
stamped (vigilatum in-code), re-verifiable (clippy-clean in-home + git drift-check).
No stone leaves its segment flat. Relentless.

The segments + their (provisional, intueri-at-settle) homes:

| Segment (today, flat) | grounded at | lifts into |
|---|---|---|
| surface-ref → entity resolution (the bare→FQDN canonicalization + call-head validation) | `check.rs:1753` `BARE_PRIMITIVES`/`BARE_CONTAINER_HEADS`; `resolve_references` (check step 7); `runtime.rs:5349` value-symbol lookup; `:5428` head dispatch | **`src/resolve/`** (provisional) — the canonical surface→entity home |
| the type-form grammar + `wat.type/` namespace + parametrics-as-forms | `lex_keyword` bracket machinery `lexer.rs:605-730` (DELETED); type representation | the existing **`src/types/`** warded home (extended) |

Stamps follow the current discipline (live vigilia cast + clippy-clean) until arc
250 (`vigilatum-integrity`, STUBBED) lands self-enforcement; nothing here waits on it.

## Provisional stone sketch (FM-2-bis probes drive the real decomposition; each stone lifts+wards its segment)

The "symbol-first" spine stages. Provisional — pin at each sub-DESIGN with a
disconfirming probe:

- **251.0** ✓ sub-DESIGN (`DESIGN-STONE-251.0.md`) + FM-2-bis probe
  (`tests/probe_arc251_stone0_symbol_head.rs` — C01 symbol head `wat.core/+` RED at
  HEAD via `UnboundSymbol`; C02 keyword head GREEN/preserved). Locks the mechanism:
  **normalize-layer** (extend the existing surface→entity resolution; dispatch
  untouched), NOT native-symbol-dispatch — see the sub-DESIGN's four-questions.
- **251.1** LIFT + WARD `src/resolve/` — split into stepping stones (four-questions
  verdict B; `DESIGN-STONE-251.1.md`), each single-axis to verify:
  - **251.1a** LIFT `src/resolve.rs` (flat, 709 lines, unstamped) → `src/resolve/`
    warded home. PURE MOVE — green→green, NO behavior change. intueri names the
    internal modules; vigilia to L1+L2=0; stamp. (The transform then lands in
    already-warded ground — recovery-doc Proactive-slicing.)
  - **251.1b** ADD the normalize transform: a dotted-symbol ref (`wat.core/+`)
    resolves to the entity its keyword FQDN names (same shape as the bare→FQDN
    precedent); resolve gains validate **+ normalize** (transform, lifting its current
    "does NOT transform" limitation). Probe `probe_arc251_stone0_symbol_head` RED→GREEN.
    Dual-read (keyword + symbol). Re-earn the stamp.
  - **251.1c — RETIRED** (misread, grounded against the disk 2026-06-09): the
    `BARE_PRIMITIVES`/`BARE_CONTAINER_HEADS` tables are NOT bare→FQDN resolution — they
    are a bare-scalar-TYPE deprecation lint (`walk_type_for_bare` emits
    `BareLegacyPrimitive`: "you wrote bare `:i64`, use the FQDN"). Resolve is ALREADY the
    single CALL-HEAD authority after 251.1b. The bare-TYPE discipline is a TYPES concern
    that 251.2 subsumes — once types are `wat.type/i64` symbols, the bare-`:i64`-keyword
    lint has nothing to lint. **251.1 closes at a+b**; the home is warded after 251.1b.
  - (Lexer untouched throughout — it already reads the token.)
- **251.2** the `wat.type/` namespace: scalar type ATOMS move out of `:wat::core::` into the
  type namespace; the lexeme-carries-the-role property replaces position-polymorphism
  (DISSOLVES the `WatAST::Keyword` type/value split for atoms). Parametrics stay keyword
  (→ 251.3 forms); dual-read bridges. Work lands in `src/types.rs` + `src/types/` (re-earn
  the stamp). **STRIKE-READY** (2026-06-10): sub-design `DESIGN-STONE-251.2.md`; probe
  `tests/probe_arc251_stone2_type_namespace.rs` (C01 RED at HEAD — `wat.type/i64` not yet
  recognized as i64; C02 dual-read GREEN).
- **251.3** parametrics-as-FORMS `(wat.type/Vector wat.type/i64)` — types become genuine
  forms (the macro engine computes over them); rewrite `keyword/of` to a quasiquote template
  (no string-concat); rewrite the run-threads type-destructure to form ops. The `<>`
  `lex_keyword` `angle_depth` machinery (lexer.rs:637-730) DELETION + corpus migration DEFER
  to the unified 251.5 sweep (churn-once — can't delete `<>` lexing while the corpus uses it).
  **STRIKE-READY** (2026-06-10): sub-design `DESIGN-STONE-251.3.md`; probe
  `tests/probe_arc251_stone3_parametric_form.rs` (C01 RED — a parametric FORM in a binder slot
  is rejected by the keyword-only type-slot readers; C02 `<>` dual-read GREEN). Build 251.3a =
  `parse_type_form` (List→Parametric) + wire the 5 type-slot readers.
- **251.4** `:-` annotation arrow + `ann-form` expression-ascription — folded into the
  signature-rewrite cascade (the corpus-wide `<-`→`:-` + keyword-head→symbol-head sweep).
- **251.5** HARD-CUT the keyword-as-type/operator surface (one-canonical-path) — the
  dotted symbol form becomes the ONLY surface form; resolution rejects keyword spellings.
  Includes the **`:rust::` interop corpus cut** (move 6): `:rust::a::b::C` source
  spellings → `rust.a.b/C` symbols, and `use!` declarations → `(wat.core/use! rust.a.b/C)`.
- **251.6 — NATIVE SYMBOL DISPATCH (ANNIHILATE the normalize-layer).** Flip `eval_list` /
  `dispatch_keyword_head` + every `if let Some(WatAST::Keyword(head, _)) = items.first()`
  head-reading site across check.rs/runtime.rs to read `WatAST::Symbol` heads NATIVELY —
  the big-bang 251.0 deliberately deferred, safe now that the corpus is symbols-only
  (post-251.5) so the blast radius no longer hides the contract. Then **DELETE
  `src/resolve/normalize.rs`** — with native dispatch it has nothing left to translate.
  Symbol is canonical end-to-end; the keyword-FQDN internal core is GONE. This is the act
  that zeroes the normalize half of the final measurement. (Disconfirming probe: a symbol
  head dispatches with NO normalize pass in the pipeline.)
- **251.N** INSCRIPTION (every touched segment a stamped home; FM-11 grep clean; THE FINAL
  MEASUREMENT verified — wat_edn codec + normalize.rs both deleted, vanilla EDN reader
  round-trips the corpus).

### ⊹ THE FINAL MEASUREMENT (the acceptance gate — builder, 2026-06-09)

The arc is DONE when the **wat_edn translation layer is DELETED** — not bypassed,
*deleted* — and a wat program still round-trips through a **plain, spec-conforming EDN
reader**. That deletion IS the proof that wat is EDN; it is the session-wide lesson at
its largest scale (*the right fix deletes code*). Post-251 this is dead code to rip out
(sized 2026-06-09):
- the `::`↔`.` magic in wat-edn — `translate_and_validate_ns`, the `Keyword::try_ns`/`ns`
  rewrite (**~21 sites**).
- the wat↔EDN keyword codec in `edn_shim` — `keyword_from_wat_path`, `ns_to_wat_path`,
  `make_qualified_keyword`, `strip_keyword_colon`, `type_path_to_namespace` (**~27 sites**).
- the **wire-escape mode** (`Lexer::new_wire` + the `_`↔`,` swap, woven through
  parser/lexer/vocab/writer/value) — its sole job is smuggling `:Foo<A,B>` parametric
  commas, which **251.3 dissolves** (parametrics-as-forms).
- the bracket `lex_keyword` machinery in BOTH lexers — **251.3 deletes** it.
- the 213.S bridge (`src/wat_edn_bridge.rs`) keyword translation collapses to a pure
  structural pass-through (a wat keyword IS an EDN keyword; the `/`-in-name STOP vanishes).
- **the arc-251 normalize-layer ITSELF** (`src/resolve/normalize.rs`) — it is the SAME
  species of translation magic (symbol→keyword-FQDN), the **last vestige of the keyword
  abuse living in the bones.** It is SCAFFOLDING, scheduled for demolition (builder,
  2026-06-09): once dispatch reads `WatAST::Symbol` heads NATIVELY (stone 251.6 below),
  normalize has nothing left to translate and is **DELETED**. Symbol is canonical
  end-to-end; no keyword-FQDN core remains. Keeping normalize would leave wat "symbols on
  the skin, keywords in the bones" — NOT proper Clojure. The dual-read it provides is the
  bridge; the translation it performs is also condemned.

Corpus side: **~10,400 `::` occurrences** across `.wat` files → `.`/`/` (mechanical:
`:a::b::c` → `a.b/c`; `:a::b::T/m` → `a.b.T/m`). **251.N does not close until the
translation-magic grep is ZERO — wat_edn codec AND `src/resolve/normalize.rs` deleted —
and the round-trip holds with a vanilla EDN reader.** This also retires 213's whole
keyword-codec problem (don't patch it — 251 deletes it).

Roughly 6-10 stones. Campaign-scale — bigger than arc 234. Substrate-as-teacher
cascade will be wide (every `.wat` file + every test fixture + src synthesis sites).

---

## What this unblocks

- **Arc 235** (records with rich VSA encodings) — its DESIGN's recommended Option B
  (`:wat::holon::Thermometer<:f64,0.0,100.0>` bracket-parametric) is named-for-deletion
  by this arc. Post-251, Thermometer/Blend/Permute encodings express as ordinary
  forms — `(wat.type/Thermometer wat.type/f64 0.0 100.0)` — vastly simpler than
  phantom-type machinery in check.rs. Arc 235 re-grounds its DESIGN on types-as-forms
  after 251 closes.
- **Arc 232** (defprotocol-extend-type — THE MAIN QUEST rejoin) — held behind the
  4-arc gate (246 ✓ / 245 ✓✓ / 249 ✓ / 235). Arc 251 inserts ahead of 235 (235's
  syntax depends on it). The gate becomes 5; the 5th is the honest one.

---

## Cross-references

- `docs/arc/2026/04/109-kill-std/NOTE-typed-form-and-type-namespace.md` — moves 1-4 + `ann-form` + core.typed reference
- `docs/arc/2026/04/109-kill-std/NOTE-generic-bracket-syntax-edn.md` — parametrics-as-forms dissolves the bracket lexer; live arc-249/245 evidence
- `docs/arc/2026/05/235-records-with-rich-vsa-encodings/DESIGN.md` — the dependent (Option B superseded by this arc)
- `src/lexer.rs:457` (`is_symbol_break` — reads dotted symbols today), `:634` (`lex_keyword` — the bracket machinery to delete), `:794` (`lex_symbol`)
- `feedback_intueri_names_all_things.md` — the arc/stone/primitive names come from spawned intueri casts
- arc 244 close — the `WatAST::Keyword` type/value split named the `src/ast/` ward-enabler; this arc DISSOLVES it instead
