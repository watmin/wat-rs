# Arc 251 — The Clojure-faithful symbolic surface (types-as-forms)

**Status:** OPEN 2026-06-06. Builder-directed; the destination is settled (full
core.typed/Typed Clojure acceptance), the stone sequencing is being drawn.

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
  - **251.1c** CONSOLIDATE: migrate `check.rs:1637` `BARE_PRIMITIVES`/
    `BARE_CONTAINER_HEADS` into the resolve home — resolution becomes the single
    surface→entity canonicalization authority. Behavior-preserving. Re-earn the stamp.
  - (Lexer untouched throughout — it already reads the token.)
- **251.2** the `wat.type/` namespace: type atoms move out of `:wat::core::` into the
  type namespace; the lexeme-carries-the-role property replaces position-polymorphism
  (DISSOLVES the `WatAST::Keyword` type/value split). Work lands in `src/types/`
  (re-earn the stamp).
- **251.3** parametrics-as-FORMS `(wat.type/Vector wat.type/i64)`; **DELETE** the
  `lex_keyword` bracket machinery (lexer.rs:605-730); rewrite `keyword/of` to a
  quasiquote template (no string-concat); rewrite the run-threads type-destructure to
  form ops. This is where types become genuine forms (the macro engine computes over
  them); `src/types/` extended + re-stamped.
- **251.4** `:-` annotation arrow + `ann-form` expression-ascription — folded into the
  signature-rewrite cascade (the corpus-wide `<-`→`:-` + keyword-head→symbol-head sweep).
- **251.5** HARD-CUT the keyword-as-type/operator surface (one-canonical-path) — the
  dotted symbol form becomes the ONLY form; resolution rejects keyword spellings.
- **251.N** INSCRIPTION (every touched segment a stamped home; FM-11 grep clean).

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
