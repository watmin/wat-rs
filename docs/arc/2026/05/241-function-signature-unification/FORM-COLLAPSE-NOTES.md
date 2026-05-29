# Form-collapse dialogue notes — `def` / `defn` / `defstruct` metadata maps (2026-05-28)

**Status:** PROPOSED, not shipped. Notes capture settled dialogue decisions; no substrate change yet. An implementation arc needs to be opened after dialogue completes.

**Scope distinction:** Arc 241 unifies the **argspec parser** (the canonical `[name <- :T]` triple shape across binding sites). These notes capture the **form-level collapse** that emerged in dialogue alongside the parser work: `def-restricted`, `defn-restricted`, and `struct-restricted` get absorbed into plain `def`, `defn`, and (renamed) `defstruct` via optional metadata-map clauses. The defstruct rename + form-collapse lands inside arc 241 per user direction 2026-05-28. The remaining type-declaration renames queue under arc 109 — see `../../../04/109-kill-std/NOTE-type-decl-def-prefix-renames.md`.

---

## Guiding light (user direction, 2026-05-28)

> *"a guiding light for wat has been 'there's exactly one way to do a thing' so if that one way includes configuration params i'm good with it"*

One form per noun. Configuration via tagged optional clauses lives inside the one way; doesn't split it.

> *"this is four questions territory -- do not concern yourself with cost of impl - that's irrelevant - we care about correctness"*

The verdicts below ran the four questions atomically with cost-of-impl excluded.

---

## What was settled

1. **`def-restricted` / `defn-restricted` / `struct-restricted` retire as separate form names.** Plain `def`, `defn`, and `defstruct` absorb restriction (and any future binding-level metadata) via an optional metadata map. The macro layering is preserved: `defn` is a macro over `def + fn`, so locking the metadata map at `def` extends it to `defn`.

2. **Metadata is expressed via an optional `{...}` HashMap clause** between the name and the value-expr (`def`) / argspec Vector (`defn`) / fields-Vector (`defstruct`).
   - Adopting Clojure's metadata-map idiom serves the "open to extension" criterion: new metadata keys land without parser changes.
   - Convergence with Clojure validated by `user_no_literature`.
   - Substrate primitive already shipping (arc 215 `{...}` HashMap literal).
   - Taxonomy distinction: **binding metadata** (annotations about the binding — `:restricted-to`, `:doc`, `:deprecated`) lives in the map; **form logic** (per-clause behavior — defclause's `:guard` / `:ensure`) stays positional. They encode different things, so they're allowed to differ.

3. **Empty is illegal everywhere it would be a divide-by-zero.**
   - Empty `:restricted-to []` → error
   - Empty `{}` map → error
   - Empty value for any key inside the map → error

4. **Unknown keys in the map are errors at parse.** No silent typo-tolerance. New metadata keys land via explicit substrate updates that teach the parser to accept them.

5. **Position-fixed.** The map (if present) sits between `:name` and the next required slot. Single optional slot in that band.

6. **Discrimination at parse is trivial.** `{...}` = metadata; `[...]` = argspec / fields-Vector; anything else = value-expr (for `def`). No backtracking.

7. **`def*` prefix convention ratified.** A four-questions cast picked `def*` prefix uniformly for all top-level definers over the current bare-noun convention and the arc-227 tail-`def` convention. The verdict requires explicit lock: `def` in the prefix means *"top-level definition"* (concept), not *"expansion through the def primitive"* (mechanism). Per `feedback_inscription_immutable`, arc 198's inscription stays; this is forward-correction. **`struct` renames to `defstruct` in arc 241 scope; other type-decl renames queue in arc 109** — see the sibling NOTE for the queue + per-name intueri questions.

8b. **Docstrings (and any other binding-level metadata) follow the same uniform mechanism** — `:doc` at form level documents the form; `:doc` inside a per-binding metadata map documents that binding. Same key, two loci, same `{...}` mechanism. No separate docstring slot needed; the uniform metadata-map subsumes the question:
   ```scheme
   (:wat::core::defstruct :counter::Client
     {:doc            "client capability handle"            ; form-level :doc → docs the type
      :restricted-to  [:counter::]
      :field-metadata {server-id {:doc "issuance witness"   ; per-field :doc → docs the field
                                   :restricted-to [:counter::]}}}
     [server-id <- :wat::core::Uuid
      ...])
   ```
   The δ-verdict produced this scoping as a byproduct of uniform composition; no additional design work needed for `:doc`.

8. **Struct has TWO loci of metadata; argspec stays RIGID.** Form-level metadata (ctor restriction) lives directly in the `{...}` map. Per-field metadata lives in the form-level map under the `:field-metadata` key whose value is `{field-symbol → metadata-map}`. The argspec Vector stays uniform 3-slot triples at every position. Per-field-distinct whitelist capability from arc 203 is preserved (each field's inner metadata-map is independent). Same `{...}` mechanism reused at both loci — `:restricted-to` carries the SAME meaning whether applied to the form-binding or to a field-binding.

   Four-questions cast ran atomically on four candidate shapes 2026-05-28 (cost-of-impl excluded per user direction):

   | Candidate | Verdict |
   |---|---|
   | (α) `:restricted-fields [list of symbols]` with ctor whitelist shared as accessor whitelist | NO Obvious + NO Simple + NO Honest |
   | (β) Vector of symbols + separate `:accessor-restricted-to` key | NO Obvious (cross-key coordination implicit) |
   | (γ) `:restricted-fields {symbol → whitelist}` map (with rename to `:restricted-ctor` for ctor) | NO Simple — heterogeneous mechanism (Vector for ctor; Map-of-Vector for fields) at two loci |
   | **(δ) `:field-metadata {symbol → metadata-map}` — uniform mechanism at both loci** | **YES YES YES YES** |

   Per `feedback_simple_is_uniform_composition` ("N identical compositions IS simple"), (δ) wins on Simple — one mechanism (`{...}` metadata map containing `:restricted-to`) applied at form-level AND per-field. (α), (β), (γ) each fail at least one axis. The user's prior bias was γ-with-rename; user honored the discipline's verdict. **Locked: (δ); key name `:field-metadata`.** Contaminating the argspec with a 4th metadata slot (the earlier proposal) is REJECTED — it broke argspec rigidity. The 4-slot extension is OFF the table.

9. **Ctor restriction retained — primary justification shifts.** The user surfaced that ctor restriction is NOT load-bearing for the capability-secret-witness pattern (where field-read restriction on the witness + server-side registry validation is the actual security boundary). It's retained because it IS the only language-level expression mechanism for **module-boundary enforcement on type construction** (Builder pattern; versioning/migration safety; interface-contract enforcement). The form-level metadata-map slot houses it.

---

## Proposed forms (PROPOSED — not yet on disk in code)

### `:wat::core::def`

```scheme
(:wat::core::def :my::ns::my-const
  {:restricted-to [:my::ns::]}                  ; OPTIONAL metadata map
  <value-expr>)
```

### `:wat::core::defn` (macro over `def + fn`)

```scheme
(:wat::core::defn :my::ns::my-fn
  {:restricted-to [:my::ns::]}                  ; OPTIONAL metadata map
  [arg1 <- :T1
   arg2 <- :T2]
  -> :Ret
  body)
```

### `:wat::core::defstruct` (renamed from `:wat::core::struct`; absorbs `struct-restricted`)

```scheme
(:wat::core::defstruct :my::ns::MyType
  {:restricted-to  [:my::ns::]                          ; ctor restriction (form-level binding)
   :field-metadata {field1 {:restricted-to [:my::ns::]} ; per-field metadata map; same {...} mechanism as form-level
                    field2 {:restricted-to [:my::ns::]}}}
  [field1 <- :T1                                        ; argspec stays RIGID uniform 3-slot triples
   field2 <- :T2                                        ; per-field restriction lives in form-level :field-metadata
   field3 <- :T3])                                      ; (no entry in :field-metadata = public access)
```

**Counter/Client capability re-expressed:**

```scheme
(:wat::core::defstruct :counter::Client
  {:restricted-to  [:counter::]                                                ; ctor restriction
   :field-metadata {server-id {:restricted-to [:counter::]}                    ; witness — issuer-only read
                    client-id {:restricted-to [:counter::]}}}                  ; witness — issuer-only read
  [server-id <- :wat::core::Uuid                                               ; argspec rigid; per-field restriction
   client-id <- :wat::core::Uuid                                               ;   lives in :field-metadata above
   peer!     <- :wat::kernel::ThreadPeer<counter::Response,counter::Request>]) ; no field-metadata entry = public access
```

### `:wat::core::defenum` (renamed from `:wat::core::enum`; queued for arc 109 implementation)

```scheme
(:wat::core::defenum :app::Status
  {:variant-metadata {:Error {:doc "raised when the operation fails"}}}     ; OPTIONAL form-level metadata
  :Ok                                                                       ; positional — unit variant (next is keyword)
  :Pending                                                                  ; positional — unit variant
  :Error [code    <- :wat::core::i64                                        ; positional — tagged variant (followed by argspec Vector)
          message <- :wat::core::String])
```

**Variant grammar (verdict from four-questions cast 2026-05-28 on candidates A/B/C/D — `feedback_four_questions_inline`):**

A variant is a keyword (variant name) optionally followed by an argspec Vector (its tagged-variant fields). Discrimination is local + unambiguous via one-token look-ahead:
- See keyword → variant name
- Peek next: another keyword (or end of form) → current is UNIT variant
- Peek next: Vector `[...]` → current is TAGGED variant; consume Vector as canonical argspec

The earlier-considered candidates were each rejected by the four questions:
- (A) Vector-wrap every variant (`[[:Ok] [:Pending] [:Error [...]]]`): fails Honest (empty brackets ceremony for unit variants)
- (B) Outer variants Vector + mixed inner shapes (`[:Ok :Pending [:Error [...]]]`): fails Simple (outer Vector wraps positional items the look-ahead grammar can handle without wrapping)
- (C) Outer Vector + Vector-wrapped tagged (`[:Ok :Pending [:Error [...]]]`): same Simple failure
- **(D) Positional variants with look-ahead** — YES YES YES YES — variant kind announced by what follows; canonical argspec inside tagged variant; cleanest visual; matches familiar Lisp positional-pair patterns (e.g., `let` bindings)

**`:variant-metadata` key locked by intueri cast 2026-05-28** (paired evaluation with `:field-metadata` for sibling consistency):

- Verdict: `:field-metadata` + `:variant-metadata` (full word "metadata" with singular-locus noun)
- Rejected: `:field-meta`/`:variant-meta` (terse — mumbles); `:fields-metadata`/`:variants-metadata` (plural — Level 1 LIES at structural layer; the inner map is per-locus not aggregate); `:field-info`/`:variant-info` (semantic void)
- Friction: "metadata" is 9 characters; visual weight in dense maps. Accepted as cost of precision.
- Forward-extension: the `<singular-locus-noun>-metadata` pattern accommodates `:arg-metadata` on `defn` as a future third sibling cleanly

**Variant name shape** — variants are keywords (consistent with wat's type-naming convention: `:wat::core::String`, `:counter::Client`). PascalCase capitalization for variant names. Per-variant-metadata inner-map keys are these same keywords (`{:Error {:doc "..."}}`).

**Per-variant metadata follows defstruct's δ pattern.** The variants positional list stays RIGID (no inline metadata slots between variants); per-variant metadata lives at form level in `:variant-metadata`. Same uniform-composition doctrine that produced δ for defstruct's per-field metadata.

### Reflection — `:wat::runtime::metadata-of`

```scheme
(:wat::runtime::metadata-of :my::ns::my-fn)
;; → {:restricted-to [:my::ns::] :doc "..."}      ; the metadata HashMap (HolonAST-encoded)
;; → :nil                                          ; if no metadata attached
```

Sits in the `<aspect>-of-<thing>` family next to `body-of`. Single sibling, not split.

---

## Argspec stays RIGID — per-binding metadata lives at form level

The canonical argspec stays uniform 3-slot triples at every position:

```
name <- :T               ; rigid uniform; NO 4th metadata slot inside argspec
```

The earlier proposal to extend the canonical triple to an optional 4th `{metadata}` slot is REJECTED — it broke argspec rigidity. Per-binding metadata for any binding site (defstruct fields; future per-binding metadata at any locus) lives in form-level metadata under a key whose value is `{binding-symbol → metadata-map}`. The pattern at defstruct is `:field-metadata`; future binding sites that need per-binding metadata follow the same shape (key name TBD per their context; structure identical).

Arc 241's `parse_argspec_triples` parses the canonical 3-slot triple uniformly across all binding sites. Form-level parsers (defstruct's, future defn's if it ever needs per-arg metadata) decode the per-binding metadata map separately and associate by symbol.

---

## Naming verdicts

### `:wat::runtime::metadata-of` — intueri cast 2026-05-28 (LOCKED)

Candidates evaluated:

| Candidate | Verdict | Level |
|---|---|---|
| **`:wat::runtime::metadata-of`** | **YES YES YES YES** | **0 (speaks)** |
| `:wat::runtime::meta-of` | NO Obvious | 2 (mumbles — Clojure-terseness loses the noun) |
| `:wat::runtime::extract-metadata` | NO Honest + NO UX | 1 (LIES — wrong family; `extract-*` takes HolonAST input, this takes binding-name) |
| `:wat::runtime::metadata-of-def` | NO Simple + NO UX | 2 (mumbles — leaks `def` substrate mechanic the caller shouldn't see) |

Friction acknowledged: implied object invisible (asymmetric with `signature-of-defn` / `signature-of-fn`); consistent with `body-of`; accepted as cost of not over-specifying.

### `def*` family direction — four-questions cast 2026-05-28 (RATIFIED)

Verdict: `def*` prefix uniformly applied to all top-level definers. YES YES YES YES. Bare-noun + tail-`def` candidates disqualified. Lock: def-as-concept, not def-as-mechanism.

### `:wat::core::defstruct` — intueri cast 2026-05-28 (LOCKED)

Candidates evaluated:

| Candidate | Verdict | Level |
|---|---|---|
| **`:wat::core::defstruct`** | **YES YES YES YES** | **0 (speaks)** |
| `:wat::core::defstructure` | NO Simple + NO UX | 2 (mumbles — verbose against family contraction pattern; `defn` not `deffunction`; `defmacro` not `defmacroexpander`) |

**Locked: `:wat::core::defstruct`.** Multi-language convergence (C, Rust, Go, Swift, Zig all ship "struct"); arrives clean for LLM-first audience without project lore.

### `:wat::core::defenum` — intueri cast 2026-05-28 (LOCKED)

Cast as a sibling-pair with `defstruct` for family consistency. Candidates evaluated:

| Candidate | Verdict | Level |
|---|---|---|
| **`:wat::core::defenum`** | **YES YES PARTIAL YES** | **0 (speaks)** |
| `:wat::core::defenumeration` | NO Simple + NO UX | 2 (verbose against family pattern) |
| `:wat::core::defvariant` | NO Obvious + NO UX | 2 (points at the PART not the whole) |
| `:wat::core::defsum` | NO Obvious + NO UX | 2 (requires type-theory lore — Haskell/ML; fails LLM-first) |
| `:wat::core::defunion` | NO Obvious + NO Honest + NO UX | 2 (C-union ambiguity; Rust abandoned the term deliberately) |

**Locked: `:wat::core::defenum`.** The PARTIAL on Honest is the historical "C-enum = flat constants" ghost; Rust normalization (enum = tagged union) has resolved this for the LLM-first audience. Form body completes the picture when reader sees tagged variants. Acceptable friction; not a lie.

### Family-consistency check (post-locks)

| Name | Status |
|---|---|
| `def` | shipping |
| `defn` | shipping |
| `defclause` | shipping |
| `defmacro` | shipping |
| **`defstruct`** | locked here (lands in arc 241) |
| **`defenum`** | locked here (implementation queues in arc 109) |
| `defrecord` | queued — arc 227 reconciliation (records flavor-split: base + holonic) |

No scope collision. All contracted-noun-stem siblings; reads as one family at any grep of `:wat::core::def`.

---

## What this supersedes

- **Arc 210** (`:restricted-to` keyword-tag pivot for `def-restricted` / `defn-restricted`) — superseded; the metadata map absorbs it.
- **Arc 198** (def-restricted / defn-restricted minting) — access-control SEMANTICS preserved (caller-prefix whitelist; arc 198's storage HashMap; arc 198's walker). The form-NAMES retire; the mechanism stays. Arc 198 INSCRIPTION stays as historical record per `feedback_inscription_immutable`.
- **Arc 203** (struct-restricted) — same shape: SEMANTICS preserved (ctor whitelist + per-field accessor whitelist via the same arc 198 mechanism), form-NAME retires, INSCRIPTION stays as historical record.
- **The bare-noun convention for type-declarations** — superseded by the `def*` prefix verdict. struct retires here (in 241 scope); enum/newtype/typealias/typeunion/recordtype + arc 227's Record::def queue in arc 109.

---

## Open follow-ups (for the implementation arc, whenever it opens)

- Metadata-map propagation through serialization / IPC — user direction: YES; the map moves with the binding through reflection.
- HashMap-key shape: `{:wat::core::Keyword → :wat::holon::HolonAST}` per the typed-entities + arc 216 encoding doctrines. Verify alignment with existing reflection-layer carrier types when implementation begins.
- Error-message wording for the rejected cases (empty `{}`, empty value, unknown key) — align to a single canonical voice; not site-by-site drift.
- Whether `:default-field-metadata` form-level shorthand for "fields inherit this map unless overridden" earns its place. User dialogue 2026-05-28 leaned toward verbose-is-honest (no inheritance shorthand); the question stays open as a future surface evolution rather than a 241 concern.

---

## What's parked

- **`enum` tagged-variant fields** — share `parse_field` with the legacy plain-struct pair-Lists; migration follows arc 241's canonical argspec triple. The form-name rename to `defenum` queues in arc 109.
- **Legacy `define`** retirement (`define ⇒ defn`) — orthogonal; user has authorized but sequencing TBD.
- **defclause** — `:guard` / `:ensure` stay positional per the taxonomy distinction; not affected by this work.
- **`recordtype` + arc 227's `Record::def` reconciliation** — queues in arc 109 with its own intueri reconciliation work.

---

## Cross-references

- `docs/arc/2026/04/109-kill-std/NOTE-type-decl-def-prefix-renames.md` — the sibling NOTE carrying the remainder of the rename family + per-name intueri questions
- `docs/arc/2026/04/109-kill-std/NOTE-reconsider-atomize-materialize.md` — sibling NOTE convention this file mirrors at the dialogue layer
- `docs/arc/2026/05/198-defn-restricted/INSCRIPTION.md` — origin of the access-control mechanism this collapse re-homes
- `docs/arc/2026/05/203-struct-restricted/DESIGN.md` — origin of the capability-secret-witness motivation; the dialogue surfaced its ctor-restriction component as defense-in-depth rather than load-bearing
- `docs/arc/2026/05/210-restricted-to-keyword-tag/DESIGN.md` — the partial pivot this collapse supersedes
- `docs/arc/2026/05/215-clojure-data-literals/` — minted `{...}` HashMap literals the metadata map reuses
- `docs/arc/2026/05/227-defrecord-defservice/` — the records flavor-split arc; `Record::def` reconciliation queues in arc 109
- `docs/arc/2026/05/241-function-signature-unification/DESIGN.md` — the parser-unification arc this dialogue runs alongside
- `feedback_wat_llm_first_design` — one canonical path; doctrine the collapse honors
- `feedback_inscription_immutable` — past arcs stay; forward-correct via new arcs
- `feedback_spells_cast_via_subagent` — intueri cast via subagent; protocol still owes per-name casts on `defstruct` (and the rest of the family in arc 109)
- `feedback_four_questions_inline` — the discipline that produced the def*-prefix verdict
- INTERSTITIAL § Convergence #11 / #12 — the "walked into a Clojure room" pattern; intueri's evaluation of `meta-of` rejected borrowing the terse Clojure spelling

---

## Next move

Implementation arc for the def/defn/defstruct collapse needs opening once arc 241's parser unification lands sufficient foundation (the canonical `parse_argspec_triples` with the 4th-slot extension). The arc-109 NOTE captures the remainder of the type-decl rename queue; per-name intueri casts owed before any of them lock.
