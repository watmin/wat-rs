# Arc 294 — Holon returns to VSA: EDN is canonical; the data over its encodings; the hologram a derived index; the strange loop closing

> **▶ 293 and 294 close TOGETHER. The live close ORDER + STATUS is `CLOSE-SEQUENCE-293-294.md` (the single
> maintained tracker). This DESIGN is the gut's model/flaws — context, not the sequence.**

> **Name (intueri, crowned by the builder 2026-06-27):** `holon-returns-to-vsa` — *"that one is just pleasant to
> read."* HolonAST was minted for VSA, accreted the AST + wire roles to force `EdnRepresentable` into being, and
> now sheds them to return to its origin: pure VSA. The strange loop closes. (Thesis: **EDN is canonical.**)

**Status: SCOPED — the foundation gut (2026-06-27).** This arc was discovered *inside* arc 293 (struct/record
symmetry): chasing construction parity surfaced that the **holon record was built backwards**, and pulling that
thread unravelled a single inversion expressed as **six grounded flaws**. 293's HOLDER × SURFACE thesis (R2/R3/R4)
**stands and is proven** — this arc is the **value-layer foundation beneath it**, and 293's remaining construction
work (ctor-parity, `/from-map`) folds into 294. Builder: *"we built it well enough to find catastrophic flaws —
now we gut and rebuild it better … i'll never say no to going to disk."*

> **PATH NOTE (amend-with-recognition):** this DESIGN is the live working contract. We refine it ON DISK, not in
> volatile context. Superseded passages get a dated `⊘ SUPERSEDED` note; nothing is deleted.

## The one inversion (the disease)

**A derived encoding was made canonical, and the data it derives from was demoted to a cache.** Everything below
is one symptom or another of that single fault. The cure, stated once: **EDN is the canonical data; everything
else (the holographic vector, the wire form) is *derived from* EDN, never the other way.**

The builder's words that frame it: *"my expectation is that edn goes in and vectors get built … holon can host
all of edn … edn is how we enforce 'data can ship outside shared memory'."*

## The six flaws (each grounded against the disk THIS session)

1. **Construction split-brain.** Structs construct via `:wat::core::struct-new :T a b c` (**varargs** →
   `Value::Struct`, `runtime.rs:11680`); core records via `:wat::Record::of :T [a b c]` (**vector** →
   `Value::wat__Record`, `runtime.rs:13244`); holon via `:wat::holon::Record::of :T [fields] <holon-form>`
   (**vector + precomputed hologram** → `Value::wat__holon__Record`, `runtime.rs:13298`). **Two-plus paths, no
   canonical** — exactly where catastrophic flaws breed (one-canonical-path, arc 237). The *output reprs* must
   differ (the wire wall); the *arg shapes* differing is incidental rot.

2. **Holon record built backwards (hologram-canonical).** `Value::wat__holon__Record` stores BOTH `struct_form`
   (fields, "fast Rust-side access") AND `holon_form` (the vector) — and the **vector is canonical**:
   *"Identity lives here; Eq and Hash delegate to this field"* (`value/value.rs:329, 673, 924`). The wire ships
   the hologram (234.7b, `edn_shim.rs:2456`) and *projects* the fields from it. The **derived VSA index became
   the identity**; the data became its projection. Backwards.

3. **The `#wat-edn.holon/*` tags are scar tissue.** They exist only to serialize a HolonAST losslessly to EDN for
   the wire (`edn_holon_tag_to_ast` / `holon_ast_to_edn`). With EDN canonical, the wire ships *plain native EDN*
   and the tags vanish.

4. **`HolonRepresentable` is redundant with `EdnRepresentable`.** It is a strict supertrait adding only
   `to_holon_ast` / `from_holon_ast` (`comms/mod.rs:134`), and **all ~54 uses are in `comms/mod.rs` — wire-only.**
   Every impl (`String`/`Vec`/`HashSet`/`HashMap`/tuples) is EDN-reducible data that is *already*
   `EdnRepresentable`. **`holon-repr == edn-repr`.** One wire/portability contract: **EDN** (= the Holder wall:
   `is_portable = holder != Struct`).

5. **HolonAST-as-the-code-AST is vestigial.** WatAST is the primary AST (**3412 mentions vs HolonAST's 1161**).
   wat *bootstrapped* on HolonAST-as-AST, then built WatAST and outgrew it. The remaining HolonAST roles:
   (a) **VSA hologram** (`hologram.rs` — real, keeps); (b) **reflection IR** (signatures as `HolonAST::Bundle`,
   `special_forms.rs`, arc 143/201 — migratable to WatAST); (c) **conversion glue** (`watast_to_holon` /
   `holon_to_watast`, `runtime.rs:14597/15375` — the vestigial bridge). The universal-AST role is the rot.

6. **The strange loop is ready to close.** HolonAST was minted for VSA (arc 057), became the universal AST
   (143/201), and was overloaded as the wire form *to force `EdnRepresentable` into existence*. It was a
   **bridge**. Having birthed EDN-repr, the bridge is redundant — **the thing HolonAST built to escape itself now
   retires it from the wire and the AST.** HolonAST returns to its origin: **pure VSA.**

## The target architecture (the cure, one model)

### EDN is the one canonical data + wire + portability form
- **Wire = plain native EDN.** Everything crosses as `EdnRepresentable`. `HolonRepresentable`, the
  `#wat-edn.holon/*` tags, and the HolonAST↔tagged-EDN round-trip are **annihilated**.
- **Portability = EDN-representability**, the one gate (= the Holder wall — a `Struct` can't be EDN-repr, can't
  cross; a `Record` / holon-record can). "Data can leave shared memory iff it is EDN." One contract, not two.

### The hologram is a DERIVED INDEX over EDN — `(build-hologram form)`
A single clean codec — a recursive form-walker over any `EdnRepresentable` value — produces the holographic
vector **on demand**. (This already substantially exists as `to_holon_inner` / `to-holon`, arc 228
"typed-entities doctrine"; 294 makes it THE canonical, sole encoder and removes the stored-canonical hologram.)

The encoding rules (builder-derived; matches the extant classifier-wrap):
```
set       → (Bind (Atom "Set")    (Bundle items…))            ; no positional binds
vec, list → (Bind (Atom "Vector") (Bundle (Bind (Atom i64) v)…)) ; index → value
map       → (Bind (Atom "Map")    (Bundle (Bind k-enc v-enc)…))  ; key → value
scalar    → (Bind (Atom <TypeName>) (Atom <value>))           ; i64/f64/bool/char/string/keyword/nil
```
The **classifier-wrap `(Bind (Atom TypeName) …)` carries the type** so a decoded hologram round-trips AND
type-checks: `(Bind (Atom "i64") (Atom 42))` ⇒ "an i64 holding 42". Worked example, builder's:
`{#{1 2 3} true}` ⇒ `(Bind (Atom "Map") (Bundle (Bind (Bind (Atom "Set") (Bundle (Bind (Atom "i64") (Atom 1))…))
(Bind (Atom "Bool") (Atom true)))))`.

### HolonAST reduces to `Hologram` — the keystone (RESOLVES open Q#1)

Strip HolonAST's borrowed roles (code-AST → WatAST; wire → plain EDN) and what remains is **not a syntax tree** —
it is `Atom`/`Bind`/`Bundle`/`Permute`/`Thermometer`/`Blend`, the **MAP-VSA algebra** (`holon_ast.rs:59`,
comments: *"MAP's M / A / P"*) that `encode(ast, vm, scalar) -> Vector` (`holon_ast.rs:695`) evaluates into a point
in hyperspace. It was never an AST; it was the **hologram** wearing an AST's coat — the truth hiding in the first
half of its own name. **So `HolonAST` is RENAMED `Hologram`** (intueri), homed to `src/holon/`. The strange loop
closes *in the rename itself*: returning HolonAST to VSA is not a migration, it is calling it what it always was.
This is the arc's keystone — *holon returns to VSA*, made literal in one word.

### The canonical three-layer pipeline
```
EDN (the data; canonical; the wire) ──(build-hologram)──▶ Hologram (symbolic MAP algebra; Atom/Bind/Bundle;
                                                                    the type-classifier-wrap imposes types)
                                       ──(encode)────────▶ Vector (dense hyperspace; where similarity lives)
```
Each layer derives the next, one direction. The classifier-wrap `(Bind (Atom TypeName) …)` rides in the
**Hologram** layer and is what `from-holon` reads to recover a *typed* value on the way back. `{k v}` →
`(Bind (Atom "Map") (Bundle (Bind k-enc v-enc) …))` — a tagged-map holding a bundle that impls the map.

### The Kanerva law — width-bounded per frame, depth-UNBOUNDED
Capacity (`:dims` / `:capacity-mode`, user-tunable `CapacityExceeded` vs panic — `config.rs`) caps **fan-out per
`Bundle` frame** (≈ N items at d dims — e.g. 100 @ 10k). **Depth is free** (nesting is hologram composition). So
*any* EDN of any depth encodes; the width-per-frame bound is the only limit. **Capacity bites wherever the hologram
is (re)built — and per Q-C that is EAGERLY, on every holon-record mutation (the parity guarantee).** So a holon
record's data IS capacity-bound (it must always carry a valid, in-parity hologram); plain records / raw EDN are
unbounded. `CapacityExceeded` (user-tunable) fires at the mutation, loud, never silent — *"this data won't fit a
hologram of these dims."* (⊘ SUPERSEDES an earlier draft that said capacity bites "only at a lazy derive site" —
Q-C is eager parity, not lazy.)

### A holon record = EDN data (canonical) + a hologram held in PARITY at all times (Q-C, builder-decided)
- Stores the **fields (EDN)** — canonical. **Identity / Eq / Hash key on the EDN data** (Q-D: *"the edn is the
  identity"*), never the hologram. The wire ships plain EDN.
- The hologram is **derived AND held in strict parity with the data at all times** (Q-C: *"the hologram must be in
  parity with data at all time, whatever the cost to compute it … callers can not dodge the data and hologram being
  out of sync — this is a strong guarantee"*). Every mutation (`assoc`/`dissoc`/construct) rebuilds BOTH coherently;
  you can never observe a record whose hologram ≠ its data. **NOT lazy.** The hologram is derived (data is canonical)
  but never stale and never absent — the parity invariant (existing `runtime.rs:8754`) is KEPT and made law.
- Constructs **identically** to a core record (same EDN fields); holon-ness adds the in-parity hologram. Holon-ness
  is a capability over EDN — not a third storage repr, not a wire tier.

### Construction = ONE holder-dispatched primitive
`(aggregate-new :T field…)` — **varargs** (the `struct-new` shape won the four-questions: mirrors the user
surface `(:T a b c)`, simplest macro emission `(aggregate-new :T ~@fields)`, no extra bracket), **holder-
dispatched** (reads the type's holder → the right repr). `struct-new`, `Record::of`, `holon::Record::of` all die
into it. The holon case derives its hologram internally via `build-hologram` (no precomputed-form arg). Folds the
293 **ctor-parity** decision (unify on `:T`, drop `/new` — `293/NOTE-base-struct-horizon.md`) and `/from-map`.

## What is ANNIHILATED vs KEPT

- **Annihilated:** `HolonRepresentable` (trait + 7 impls' holon methods) · `#wat-edn.holon/*` tags + the
  HolonAST↔tagged-EDN wire round-trip · the stored-canonical `holon_form` on records (→ derived) · `struct-new` +
  `Record::of` + `holon::Record::of` (→ `aggregate-new`) · `:T/new` (→ `:T`) · HolonAST-as-the-code-AST + the
  `watast_to_holon`/`holon_to_watast` glue (where vestigial).
- **Kept:** **EDN** (the universal data/wire) · **`EdnRepresentable`** (the one wire contract) · **WatAST** (the
  AST) · the **VSA hologram** (`hologram.rs`, Atom/Bind/Bundle) rebuilt as `build-hologram`'s output · the
  **Holder** trit (the categorical wall) · 293's **Surface** system (structural row-poly).

## Homes — the gut is ALSO a megafile evacuation (builder's guiding light, 2026-06-27)

The wat-rs directive: **kill the top-level `src/*.rs` megafiles; every concern lives in `src/<ns>/<scoped>.rs`.**
20 homes already exist (`argspec/`, `check/`, `value/`, `comms/`, `types/`, `intrinsic/`, …) — but **no `src/holon/`**,
and the holon/VSA concern is sprawled across exactly the megafiles + two orphan top-level files:
- `src/hologram.rs` (409, the VSA store) · `src/wat_edn_bridge.rs` (234) — top-level orphans.
- HolonAST in `runtime.rs` (684 — `to-holon`/`from-holon` verbs, the ctor primitives, the `watast↔holon` glue) ·
  `check.rs` (148 — the holon type + the `BundleResult`/`Holons` aliases) · `edn_shim.rs` (93 — the wire round-trip,
  **annihilated**) · `types.rs` (24 — the aliases).

**294 mints `src/holon/`** and lands the *survivors* there: `build-hologram` (the codec), the hologram store (from
`hologram.rs`), the HolonAST type's VSA role + its aliases (`BundleResult`/`Holons`), the `to-holon`/`from-holon`
verbs. The wire round-trip + the `#wat-edn.holon/*` tags + `HolonRepresentable` (comms, 81) are **annihilated, not
moved**. Net: the megafiles SHED their HolonAST footprint; the holon concern gets ONE home. **The annihilation is a
homing** — every 294 strike lands its survivors in `src/holon/` (or `src/aggregate/` for construction), never back
in `runtime.rs`/`types.rs`/`check.rs`. (Construction homes to `src/aggregate/` per 293; the two homes are siblings.)

## Open questions — RESOLVED (builder, 2026-06-27) + the one deferred

- **Q1 — hologram a named type? RESOLVED:** keep it, **rename `HolonAST → Hologram`**, home `src/holon/` (the
  keystone § above).
- **Q-A — reflection-IR migration? RESOLVED:** signatures-as-`HolonAST::Bundle` is *"an abuse of holon-ast — it must
  migrate to **wat-ast**"* (builder). Reflection signatures move to **WatAST**, not `Hologram`. Consumers to carry:
  `metadata-of` / `signature-of-defn`/`-fn` / the docs system. (The measurement sizes the sub-strike.)
- **Q-C — storage: lazy vs eager? RESOLVED → EAGER PARITY.** The hologram is in parity with the data at ALL times,
  whatever the compute cost; every mutation (`assoc`/construct) rebuilds both coherently; callers cannot observe a
  desync — a **strong guarantee.** (⊘ Supersedes the apparatus's earlier 'lazy, Kanerva argues lazy' lean.) Capacity
  bites at the mutation, user-tunable.
- **Q-D — identity safety? RESOLVED → the EDN is the identity** (builder). Eq/Hash key on the data; no
  holographic-identity veto. (The measurement still confirms no live consumer breaks on the flip — execution check,
  not a decision.)
- **Q-B — codec name + home? DEFERRED to its strike (intueri).** `build-hologram` was a placeholder *to communicate
  the concept* (builder: *"i made this up … intueri names it /if we must have/ — idk if we have such a thing"*).
  `to-holon` may already BE the codec (no new name). Decide at the strike — IF a distinct named thing is needed.
- **Q-E — NOT a question; the SURVEY** (in flight): the role-census + blast-radius sizing. Grounds execution,
  decides nothing. (Builder: *"more like a survey."*)

## Census (294.0) — measured + WEIGHED against the disk (2026-06-27)

The Explore survey + the orchestrator's own spot-check (`PROBA NE DUBITES`). 1161 HolonAST mentions / 23 files
(±5-10% per-role estimates):

| role | ~count | fate |
|---|---|---|
| **VSA-ALGEBRA** (Atom/Bind/Bundle/Permute/encode/hologram store/codec) | ~375 | **KEEP** → `Hologram` in `src/holon/` |
| **LEAVES/TYPES** (leaf variants, `BundleResult`/`Holons` aliases, type paths) | ~200 | **KEEP** (the `Hologram` type) |
| **REFLECTION-IR** (signatures-as-Bundle) | ~175 | **MIGRATE → WatAST** (Q-A) |
| **CONVERSION-GLUE** (`watast↔holon`, the verb handlers) | ~175 | mostly **DIES**; `to-holon` survives as the codec |
| **WIRE** (`HolonRepresentable` + tags + round-trip) | ~136 | **ANNIHILATE** |
| **TESTS** | ~100 | follow their subject |
| **VESTIGIAL CODE-AST** | **0** | confirmed empty — HolonAST is NEVER the engine code-AST |

**Verdicts (weighed):**
- **Q-E:** VESTIGIAL=0 confirmed (own grep + survey). Real volume, sound direction, no surprise coupling.
- **Q-A (reflection → WatAST):** **BOUNDED.** Option A — `function_to_signature_ast` returns WatAST,
  `DefInfo.signature: WatAST`, rewrite the **3 positional walkers** (`extract-arg-names`/`-types`/`rename-callable-name`
  — they key on the `Symbol("->")` sentinel + `children[0]`), ~**15 call sites**; the public verbs
  (`signature-of-defn`/`-fn`) can emit at the boundary for back-compat. Not trivial, not deep.
- **Q-C (eager parity):** **IMPLEMENTABLE.** `holon_form` has exactly 4 read contexts (identity / wire / VSA ops /
  `record-assoc` rebuild); the rebuild already derives from `(class_fqdn, RecordDef.field_names, struct_form)`
  (`runtime.rs:13753+`), so maintaining parity on every mutation is the existing path made canonical. (The survey's
  "lazy viable" verdict is moot per the builder's eager call — recorded.)
- **Q-D (EDN identity):** **NO VETO, CONFIRMED against the disk** — `hologram.rs:68` is `Vec<HashMap<HolonAST,
  HolonAST>>` (records never keys); similarity is cosine on `Vector`. Pre-condition (defrecord the sole constructor,
  isomorphism held across all 3 ctor paths) holds.

**FLAW #7 (census bonus) — the equality split-brain.** Rust `PartialEq` keys on `holon_form` (`value.rs:676`,
comment *"struct_form is access optimization; not identity"* — the backwards framing); wat-surface `=` keys on
`struct_form` (`runtime.rs:8129`). Two equality contracts on one type, equivalent only by the construction
invariant. **Q-D collapses them into ONE contract on the data** — the flip is a decomplection, not just an enabler.

## 294.a — direct-EDN measurement (✅ LANDED 2026-06-27 — collections+scalars; base records → 294.c) — see `BRIEF-294.a-direct-edn-measurement.md`
**SCORE (weighed against the disk by the orchestrator's own re-run):** `(:wat::holon::cosine {:a 1 :b 2} {:a 1 :b 3})`,
`[1 2 3]`, strings, i64 — **all measure directly now**, no manual `to-holon`. Check widened (`is_holon_or_vector ||
is_portable_type`) across the 4 measurement handlers (cosine/dot · coincident? · coincident-explain · simhash);
runtime `pair_values_to_vectors` lifts any EDN value via `to_holon_inner`. **Struct still rejects** (Holder wall
holds). Suite **3464/0** (my re-run, 32s). Two tests flipped to new-correct behavior (i64/string now measure — the
old rejections were the inversion). ⊘ **Base records DEFERRED to 294.c** (`STOP-1`, grounded): `to_holon_inner`
(`runtime.rs:14565`) cannot yet lift `Value::wat__Record` — it needs the RecordDef field-names threaded in, which
IS the EDN-canonical-record machinery of 294.c. The base-record check-pass/runtime-reject gap **pre-existed** 294.a
(old `is_holon_or_vector` already accepted `:wat::Record`); 294.c closes it by *lifting* base records (thesis-aligned),
not by rejecting them. (`presence?` uses a TypeScheme registration, not a handler — left as-is; widen in a follow-up
if its runtime needs it.) R2's letting-go is substantially met for the common EDN case.

## 294.b — the `#holon` relaxed literal (✅ LANDED 2026-06-27 — `664193f5`) — the clj↔wat seam
**SCORE (weighed against the disk by the orchestrator's own re-runs):** `#holon` ships as the data-typed sibling of
`quote` (Option A) across reader/checker/runtime + 6 quote-mirror registration sites — **no new `WatAST` variant**,
+62/−6 over 10 files. Probe `holon_tag_makes_heterogeneous_edn_measure` **GREEN**; the showpiece fixture
`cosine.wat` measures **0.9999…≈1.0**; the bare heterogeneous map (no `#holon`) **still type-errors** (monomorphic
wall intact); full workspace **4088/0**. **The byte-identical bridge is PROVEN LIVE:** one file
`wat-scripts/demos/holon-literal/literal.edn` (the exact bytes `#holon {:kw ["a" "b"] true #{1 :foo "bar"} 3.0 nil}`)
reads as **plain data in Clojure** (`{holon identity}` data-reader → `{:kw [...], true #{...}, 3.0 nil}`) AND as a
**measured hologram in wat** (the same literal in `cosine.wat` → cosine 1.0). Showpiece + README + `data_readers.clj`
homed at `wat-scripts/demos/holon-literal/`. ⊘ The full wire-service round-trip (a clj app → a *running* wat service
→ vectors back, R3's fulfillment bar) remains for the IPC layer; 294.b proves the **literal-level** byte-identity.

### (history — the strike as drawn)
**RED gate:** `tests/types/probe_arc294b_holon_literal.rs` — re-verified RED this session on **exactly** the gap:
`ArityMismatch { expected: 2, got: 4 }` (`#holon {…}` parses as TWO forms — the source reader has NO `#tag <form>`
dispatch, only `#{` at `wat-reader/lexer.rs:318`) + the heterogeneous-map `TypeMismatch`es (monomorphic
`infer_map_literal`). ⊘ **CORRECTION (amend-with-recognition, 2026-06-26):** the originally-committed probe had a
**malformed (odd-cardinality) map** (`… 3.0}` dangling) → it died at PARSE (`MalformedBraceLiteral`), NOT on the
`#holon` gap; the cache's "ArityMismatch" was never reproducible from the committed bytes. Fixed: paired the
dangling value (`3.0 nil`) AND moved the wat source OUT of the inline Rust string into a **real slurped fixture**
[`wat-scripts/demos/holon-literal/cosine.wat`] (`fs::read_to_string`, precedent `tests/nursery/probe_arc214_stone81b_*`).
The fixture IS the showpiece source — the SAME bytes the Rust probe measures are what `cargo wat` runs and what the
Clojure data-reader will read (one file, two readers).
**One contract decision:** `#holon <form>` is a **reader-level tag** that consumes the next form into a Hologram-
literal AST node; the enclosed form is read as **EDN data (heterogeneous)** and types as **`Hologram`** — NOT
type-checked as a monomorphic collection. You declare what it IS (holon/EDN), not what it holds.
**Four rooms (grounded this session):**
1. **Reader** — `crates/wat-reader` (lexer `:318` does `#{`; the parser/AST). Add `#holon <form>` dispatch →
   a marked Hologram-literal `WatAST` node (mirror the `#{` set path; ground the exact node when building).
2. **Checker** — `src/check.rs`: the marked node types as `Hologram` via the `to-holon` codec (reuse 294.a's
   `is_portable_type` widening / `to_holon_inner` path), bypassing `infer_map_literal` (`:13615`).
3. **Runtime** — `src/runtime.rs`: the marked node evaluates to a Hologram value via `to_holon_inner` (`:14396`).
4. **Clojure side** — a one-line `holon → identity` data-reader (`data_readers.clj` / `*data-readers*`).
**Acceptance test (the showpiece — Clojure IS installed):** the SAME bytes `#holon {:kw [...] true #{...} 17.0 {...}}`
run in BOTH — a measurable `Hologram` in wat, plain identity-data in Clojure — proving the byte-identical bridge.
**intueri (deferred to the strike):** `#holon` vs qualified `#wat/holon` (clj discourages unqualified data-reader
tags) — weigh against the byte-identity goal. **STOP if** the reader-tag needs a new core `WatAST` variant or
touches the lexer's hot path beyond a clean `#holon` arm — surface it.

### ✅ DECIDED (four-questions, 2026-06-26): **Option A — `#holon` is the data-typed sibling of `quote`.**
Four-questioned A (desugar to a special-headed `List`, no new AST variant) vs B (new `WatAST::Holon` variant):
**A = YES/YES/YES/YES; B fails Simple** (a new core variant braids holon-lifting across every `WatAST` exhaustive
match — the opposite of decomplect). Full table + the grounding (the `quote` precedent read live at `check.rs:4389`
— *"the argument is DATA … the type checker does not recurse into it"*; the runtime `Value::wat__WatAST =>
watast_to_holon` arm at `runtime.rs:14431`; `parse_reader_macro` at `parser.rs:293`) in **`BRIEF-294.b-holon-literal.md`**.
The strike: `#holon <form>` → (reader macro) `(:wat::holon::literal <form>)` → checker types `:wat::holon::HolonAST`
without recursing → runtime `to_holon_inner(eval_quote(args, span)?, span)` (capture-as-data, then lower). Mirror
`quote` at all 6 registration sites (`special_forms.rs:220/346`, `resolve/boundary.rs:57` = `Boundary::AllData`,
`rete/purity.rs:214`, `macros/eval.rs:155` + `expand.rs:195`, `runtime.rs:7852`). NO new `WatAST` variant. The clj
`{holon identity}` data-reader + cross-read land AFTER the Rust is green (orchestrator).

## Decomposition (provisional — sequence after the open questions settle) [original below, amended above]
The build sequence was reordered this session (clj-unlock-forward, smallest-grounded-first): **294.a** (this) →
294.b `#holon` literal → 294.c EDN-canonical record + flaw #7 → 294.d wire → 294.e `aggregate-new` → 294.f
`Hologram` rename + `src/holon/` → 294.g reflection→WatAST + close. **294.a contract (pinned):** the holon
measurement surface (`cosine`/`dot`/`coincident?`/`presence?`/`simhash` + explain/floor) accepts **any
`EdnRepresentable` value** — collections, scalars, AND base records — lifting internally via `to_holon_inner`; only
non-EDN (`Struct`) rejects (the Holder wall). ONE rule, no special cases — the *"construct a holonic record"*
base-record rejection (`runtime.rs:16314`) IS the inversion 294 annihilates. RED gate
`tests/types/probe_arc294a_edn_measures_directly.rs` (verified RED at HEAD). Fulfills R2's letting-go (*"plain EDN
measured directly, the manual to-holon gone"*).

## Decomposition (provisional — sequence after the open questions settle)
- **294.0** — the census + the disconfirming probes (EDN-wire round-trip without tags; `build-hologram` over a
  nested EDN value; a holon record equal-by-data). Commit RED.
- **294.1** — `build-hologram` as the sole, clean EDN→hologram codec (over `EdnRepresentable`).
- **294.2** — flip holon record to **EDN-canonical** (fields stored/identity; hologram derived; lazy).
- **294.3** — **wire = plain EDN**: annihilate `HolonRepresentable` + the `#wat-edn.holon/*` tags + the round-trip.
- **294.4** — **construction unification**: `aggregate-new` (holder-dispatched, varargs) + ctor-parity (`:T`, drop
  `/new`) + `/from-map`; `struct-new`/`Record::of`/`register_struct_methods` die. (Subsumes 293 ctor-parity.)
- **294.5** — HolonAST-as-AST purge + reflection-IR migration (the vestigial bridge).
- **294.6** — close + amend 293; resume 291.

## Blast radius (high — this is a gut)
Touches: `value/value.rs` (the `wat__holon__Record` variant + Eq/Hash) · `edn_shim.rs` (the wire round-trip +
tags) · `comms/mod.rs` (`HolonRepresentable` + 7 impls) · `runtime.rs` (the three ctor primitives + the
conversion glue + `to-holon`) · `special_forms.rs` (reflection signatures) · the macro emission
(`wat/Record.wat`, `wat/core.wat`) · ~8 `.wat` + `.rs` ctor call sites. **Weigh relentlessly; the suite floor is
0 so a binary read; read diffs end-to-end (the `sed`-corrupts-prose lesson).**

## Pairs
`293/DESIGN.md` (HOLDER × SURFACE — the thesis this sits under) · `293/NOTE-base-struct-horizon.md` (ctor-parity
decided) · `291/STRIKE-4b-struct-state.md` (R8 — the EDN wire wall) · `comms/mod.rs` (the `EdnRepresentable` /
`HolonRepresentable` split) · `hologram.rs` (the VSA store that keeps) · `config.rs` (the Kanerva capacity) ·
`feedback_uniform_operation_or_decomplect_is_catastrophic` · `project_holon_universal_ast` (the strange loop).
