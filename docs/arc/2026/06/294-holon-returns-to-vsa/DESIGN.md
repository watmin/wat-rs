# Arc 294 — Holon returns to VSA: EDN is canonical; the data over its encodings; the hologram a derived index; the strange loop closing

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
*any* EDN of any depth encodes; capacity bites **only at the `build-hologram` derive site**, where it is a true
statement about the *encoding*, never about whether the data/record can exist. (This is *why* the EDN-canonical
flip is correct: data is unbounded; the index is where the bound honestly lives.)

### A holon record = EDN data (canonical) + a derived hologram (side-by-side, local)
- Stores the **fields (EDN)** — canonical, identical to a core record. Identity / Eq / Hash key on the **data**.
- The hologram is derived via `build-hologram` **on demand** for VSA ops (similarity), and may sit side-by-side
  as a *local* cache — but it is never the wire form and never the identity.
- Constructs **identically** to a core record. Holon-ness becomes *"this record's data can be holographically
  encoded for VSA,"* a capability over EDN — not a third storage repr, not a wire tier.

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

## Open questions (to resolve before/within the strike — four-questions each)

1. **Does the hologram stay a NAMED type or become `build-hologram`'s anonymous output?** (Is "HolonAST the VSA
   type" kept under a cleaner name e.g. `Hologram`, or is it just composed Bind/Bundle/Atom values?)
2. **Reflection-IR migration:** signatures-as-`HolonAST::Bundle` (arc 143/201, `metadata-of`/docs) → WatAST, or a
   dedicated reflection form? Scope of that sub-strike.
3. **`build-hologram` home + name** (intueri): a wat verb? `:wat::holon::build-hologram` over `EdnRepresentable`?
4. **Holon record storage:** derive-hologram-lazily (store only fields) vs eager side-by-side cache? (The Kanerva
   bound argues lazy — don't pay/limit until VSA is asked.)
5. **Identity semantics:** confirm Eq/Hash by data is correct for VSA use (does any consumer *need* holographic
   identity? — grep the `holon_form`-keyed Eq consumers before flipping).
6. **How much of the 1161 HolonAST mentions is genuinely vestigial** vs VSA vs reflection — a full census before
   the purge (this DESIGN sampled, did not exhaust).

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
