# DESIGN — Stone 251.0 — the symbol-head resolution mechanism

**Parent:** arc 251 (`DESIGN.md`). **Status:** the strike drawn; FM-2-bis probe
committed RED (`tests/probe_arc251_stone0_symbol_head.rs`, `e397e5a9`).

251.0 is the sub-DESIGN stone: it pins the ONE contract decision the rest of the
spine rides on — **how does a symbol head/ref resolve to the entity its keyword FQDN
names?** No code ships in 251.0; the implementation is 251.1 (LIFT + WARD `src/resolve/`).

---

## The grounded gap (from the probe)

`(wat.core/+ 1 2)` → `RuntimeError::UnboundSymbol("wat.core/+")` at eval
(probe C01). The symbol **lexes** and **parses** as one `WatAST::Symbol` (the lexer
de-risk: `is_symbol_break` doesn't break on `.`/`/`) and reaches eval, where the
value-position lookup (`runtime.rs:5349`) finds nothing in env and raises
`UnboundSymbol`. The keyword head still works (probe C02). **The gap is purely
surface-ref → entity resolution.**

## The decisive precedent (grounded this session)

wat ALREADY resolves non-canonical surface spellings to entities:
- `check.rs:1753` — `BARE_PRIMITIVES` / `BARE_CONTAINER_HEADS`: bare names
  (`Some`, `Ok`, `vec`, …) map to their canonical FQDN.
- `resolve_references` (check step 7) — validates call heads against the registry.

A dotted-symbol ref (`wat.core/+`) is just **another surface spelling of the same
entity** `:wat::core::+` already names. The resolution layer that maps bare→FQDN is
the natural seat for symbol→entity.

---

## The mechanism decision (four-questions)

**Candidate A — normalize-layer.** Extend the surface→entity resolution: a
namespaced dotted-symbol ref resolves to the canonical entity the keyword FQDN names
(same shape as bare→FQDN today). The dispatch machinery (`eval_list`,
`dispatch_keyword_head`) is **untouched** — by the time dispatch runs, the head has
already resolved to the entity. Keyword-FQDN stays the internal canonical form
during the transition; both spellings resolve (dual-read).
- Obvious? **YES** — reuses the existing bare→FQDN canonicalization precedent; a
  reader sees "one more surface spelling resolving to the same entity."
- Simple? **YES** — one resolution pass extended; dispatch + eval untouched; atomic.
- Honest? **YES** — *as a stepping stone.* It maps symbol→entity truthfully. It does
  NOT pretend to be the end-state: types becoming genuine forms (move 4, 251.3) needs
  the form to be the REAL representation, not normalized-away — that flip is a named
  later stone, not silently skipped. With that named, A is honest.
- Good UX? **YES** — clean dotted-symbol surface; resolution gives located errors;
  dual-read means the corpus migrates incrementally (substrate-as-teacher) instead of
  in one big-bang.
- → **FOUR YES.**

**Candidate B — native symbol dispatch (flip the internal representation now).**
Make `eval_list` / `dispatch_keyword_head` + every head-reading site in check.rs
accept `WatAST::Symbol` heads natively; symbols become the genuine head
representation immediately.
- Obvious? **NO** for a FIRST stone — flips every head-reading site
  (`if let Some(WatAST::Keyword(head,_)) = items.first()` recurs throughout check.rs)
  at once; the blast radius hides the contract.
- Simple? **NO** — touches many sites simultaneously; not atomic; not surgical/scoped.
- → **DISQUALIFIED at Obvious + Simple for 251.1.**

**Verdict: A (normalize-layer) for 251.1.** Native-representation-as-symbol is the
TRUE destination for TYPES (move 4 needs genuine forms) and lands at **251.3** with
its own probe — there it's required and scoped to the type grammar, not smeared across
every head site. Operators/declarators resolve via the normalize-layer; types become
genuine forms. The HARD-CUT (251.5) then makes the dotted symbol the only accepted
surface.

---

## The contract pinned

> A namespaced surface ref resolves to exactly one entity, by the **resolution layer**
> (the lifted `src/resolve/` home), independent of spelling (keyword FQDN OR dotted
> symbol) during the transition; at arc close, only the dotted symbol resolves.
> Resolution returns a located error (never a bare `UnboundSymbol` for a
> well-formed-but-unknown namespaced ref — it should name the unknown entity + the
> namespace). Dispatch + eval are downstream of resolution and unchanged by 251.1.

## Out of scope = rejected (affirmative cuts)

- **Native symbol-AST head representation** — NOT 251.1; types get it at 251.3. The
  general operator/declarator head flip is **stone 251.6 — COMMITTED, not "if ever
  needed"** (builder, 2026-06-09): it reads `WatAST::Symbol` heads natively and then
  ANNIHILATES `src/resolve/normalize.rs`. The normalize-layer is SCAFFOLDING — symbol→
  keyword-FQDN is the last vestige of the keyword abuse in the bones; proper Clojure has
  symbols canonical end-to-end, no keyword core. 251.1 is normalize-layer only; the
  layer's deletion is scheduled into the final measurement (see `DESIGN.md`).
- **The `:-` arrow / `ann-form`** — 251.4 (folds into the signature cascade).
- **The `wat.type/` namespace + parametrics** — 251.2 / 251.3.
- **HARD-CUT of keyword spellings** — 251.5 (dual-read holds until then).

## The home (251.1 lifts + wards)

`src/resolve/` (provisional name — **intueri-cast at 251.1**, per
`feedback_intueri_names_all_things`). Carves the surface→entity resolution out of the
check.rs/runtime.rs quarries: `BARE_PRIMITIVES`/`BARE_CONTAINER_HEADS`,
`resolve_references`, the value-symbol lookup path. Vigilia 6-8 spell to L1+L2=0;
vigilatum stamp; clippy-clean in-home. Surgical, scoped, bar-raised.

## Next

251.1 — write BRIEF + EXPECTATIONS for the LIFT + WARD of `src/resolve/` + the
symbol-ref resolution extension; intueri-cast the home name; spawn sonnet
(`model:"sonnet"`, background); score against an independent re-run of the probe
(both contracts GREEN) + the lib/corpus baseline.
