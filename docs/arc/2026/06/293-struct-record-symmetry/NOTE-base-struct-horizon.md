# NOTE — the base-struct horizon: one foundation, kind-as-a-capability-label

**Status: RESOLVED into TWO layers (builder, 2026-06-26) — the user-facing layer is IN 293; the substrate
repr-collapse stays the optional horizon.** The builder untangled a conflation in the first draft of this NOTE:

- **LAYER 1 — user-facing label-unification (IN SCOPE for 293, = the (C) construction unification, reframed).**
  `(wat.core/defstruct)` / `(wat.core/defrecord)` / `(wat.holon/defrecord)` become **three thin LABEL-MACROS
  over ONE shared base-struct emission** (ctor · accessors · `/from-map` · surfaces — identical for all three),
  differing ONLY in `{which type-reg primitive they emit, the capability}`. The behavior is enforced by *which
  primitive the label emits*: `defstruct → structtype → Value::Struct → never crosses`; `defrecord → recordtype
  → Value::wat__Record → always crosses`; holon adds VSA. **The label IS the capability gate; R8's wall is free**
  (each label produces the right variant → the existing hard `is_portable_type` wall holds). The caller can't
  know or care which variant — only the behavior. *"defstruct never satisfies edn-repr; defrecord must always."*
  293.2-parity (`defstruct → macro over structtype`) is the FIRST step; it generalizes to all three labels over
  one base. **One alignment to fold in:** the ctor convention (`defstruct` builds via `:T/new`, `defrecord` via
  `:T`) becomes ONE convention for all three.

  > **RESOLVED (builder, 2026-06-26): CONSTRUCTION PARITY — unify on `:T`; DROP `/new` for structs.** The builder's
  > call: *"i want parity in construction… using structs vs core-records vs holon-records should be identical…
  > dropping /new feels like the better thing — i haven't had any complaints with our use of records."* So all three
  > aggregate holders construct by the **bare type name** — `(:geo::SPt 1 2)` exactly like `(:geo::Circle "red" 2.0)` —
  > and `:T/new` is annihilated for structs. The keyword does double duty (type in `[x <- :T]`, ctor fn in `(:T …)`);
  > records already do this and it is accepted. Four-questions clean (Obvious: one rule = the type name is the ctor;
  > Simple: one convention, and it IS the (C) annihilation — `defstruct` becomes a full macro emitting `:T`+accessors
  > +`/from-map`, `register_struct_methods` dies; Honest: extends the existing record overload, no new lie; UX: the
  > "operate on them uniformly" the arc exists for). **Blast radius:** ~8 `.wat` + a handful of `.rs` wat-string
  > fixtures carry `:T/new` struct construction → fix-wat the `.wat`, hand-sub the `.rs` (audit prose per the
  > 293.2-rename lesson). **NEWTYPES FOLD IN (builder, 2026-06-26):** *"the name is the ctor just like records… we'll
  > fold the newtype ctor into 293."* So newtypes ALSO drop `:T/new` → `(:Price 100.0)`. The rule is now **TOTAL: every
  > type-name is its own constructor** — struct, core-record, holon-record, AND newtype, all via bare `:T`, no `/new`
  > anywhere. Newtypes share the `register_struct_methods`/`register_newtype_methods` `/new` codegen
  > (`runtime.rs:~1172`, arity-1 tuple-struct), so the annihilation covers them in the same strike. This RESOLVES the
  > "one convention for all three" alignment above → it is `:T` for ALL aggregate+newtype citizens, and feeds the
  > 293.2 `/from-map` strike (the companion macro emits the bare-`:T` ctor for every holder). *(Other `/new` or
  > construction-convention sites — a broader audit — deferred; builder: "we'll audit more stuff later.")*
- **LAYER 2 — substrate repr-collapse (STILL the optional horizon, NOT required for Layer 1).** Collapsing the
  three `Value` variants → one repr + a label. Layer 1 ships WITHOUT it (three variants stay, R8 intact). This
  is the high-blast-radius part (serialization, the gate, closure-extract) and remains a later, deliberate,
  optional strike — pursued only if the label can carry the categorical wall (see constraint below). The whole
  reason it's separable: **the caller can't know or care what the substrate is doing** — so the substrate's
  variant-count is invisible to the user-facing label-unification.

> **SHARPENED (builder, 2026-06-25 — SUBSTRATE, not macro/label):** I wrote "the label IS the capability gate"
> above; the builder corrected the locus — **the EDN-repr capability is a SUBSTRATE property of the PRIMITIVE,
> not a macro/label thing.** Grounded: `is_portable_type` (`check.rs:13001`) is the substrate's edn-repr gate
> and it keys **categorically on the `TypeDef` variant the primitive mints** — `TypeDef::Record(_) => true`
> (`:13056`), `TypeDef::Struct(_) => false` (`:13061`, the 4b-i/R8 wall; the `:12990` doc-comment "portable iff
> every field portable" is STALE pre-4b-i, fix it). **There is NO wat-level edn-repr surface/predicate** — it
> is this Rust function on the variant. So the builder's rule, verbatim on the substrate:
> **`(structtype …) ⟹ TypeDef::Struct ⟹ can NEVER satisfy edn-repr`; `(recordtype …) ⟹ TypeDef::Record ⟹ MUST
> satisfy edn-repr`** — categorical, enforced by the substrate, keyed on the minting primitive. The macro
> (`defstruct`/`defrecord`/`holon::defrecord`) is **PURE SUGAR that inherits it**; it neither sees nor carries
> the gate. This RELOCATES "the label IS the capability gate" → **the SUBSTRATE PRIMITIVE is the gate; the label
> is only which primitive you spelled.** edn-repr is the 293 **nominal HOLDER** wall (DESIGN § HOLDER), not a
> structural surface — "satisfy" = holder-membership, checked categorically, NEVER a field-check. The Layer-2
> repr-collapse below is correct ONLY if this categorical key survives as a substrate property (the constraint
> already pinned in "THE CONSTRAINT THAT MUST SURVIVE"). Pairs `feedback_ground_codebase_claims_in_codesign`.

## VERIFIED — holon is a record-refinement, not a third wire-kind (disk-grounded 2026-06-25)

A read of the holon encoding (Explore agent + cross-checked greps) CONFIRMED the builder's model with
three mechanism corrections (months-cold memory had the wire direction inverted). The grounded truth:

- **Holon records are EDN-portable + wire-shippable** — `TypeDef::Record`, pass `is_portable_type`; encode
  `edn_shim.rs:2973-2976`, decode `:2472-2556`.
- **EDN ↔ hologram are two interconvertible encodings of the SAME data.** `struct_form` (positional fields)
  and `holon_form` (the symbolic `Bind/Bundle/Atom` hologram) are each derivable from the other.
- **`holon_form` is DERIVED from the fields** — `to-holon` per field (`Record.wat:225-265`).
- **The hologram is precomputed in memory** (`value/value.rs:336`, `holon_form: Arc<HolonAST>`); base records
  carry only `struct_form`, NO `holon_form` (`value/value.rs` `wat__Record`).
- **All USER EDN round-trips through holon losslessly** (`holon-rs/.../holon_ast.rs:695-745`; only the internal
  `SlotMarker` sentinel is non-encodable — not data).

THREE corrections to the spoken model:
1. **The wire ships the `holon_form` (as an EDN tagged literal), NOT the raw field data** — `struct_form` is
   PROJECTED from the hologram's Bundle leaves on receipt, NO recompute (`edn_shim.rs:2480-2506`; arc 234.7b =
   "no recompute"). The hologram is canonical on the wire; the field-view derives from it.
2. **"the vectors" = the symbolic `HolonAST`, not a dense hypervector.** The dense float vector (similarity math)
   is never stored on the value and never transmitted — computed from the AST on demand. "Don't transmit the
   vectors" is true for the DENSE vectors; the symbolic form that crosses IS EDN.
3. **"self-update for parity" → immutable coherent-rebuild.** There IS a named PARITY invariant
   (`runtime.rs:8754`) but wat is value-semantic: `assoc` (`runtime.rs:13706-13778`) returns a NEW record with
   BOTH `struct_form` + `holon_form` rebuilt coherently. No in-place mutation exists.

**The resolved label model (unchanged by the corrections):** ONE categorical wire wall — **struct vs record**
(edn-repr / `is_portable_type`). holon is **not a third categorical wire-kind**; it is a **structural + repr
refinement of record** — it carries `struct_form` (so it satisfies core-record surfaces BY CONSTRUCTION), plus
`holon_form` as canonical identity, plus the VSA surface. The nominal `holon::Record <: Record` edge
(`types.rs:1422`) is REDUNDANT for surface satisfaction (structural covers it, once records carry field types —
293.3-records). The only irreducibly-nominal thing is the struct/record edn-repr wall.

## The builder's model (verbatim intent)

> *"it is best to have struct and record just be built on the same foundation and the 'struct-ness' vs
> 'record-ness' vs 'holographic-record-ness' are just labels on an underlying 'base-struct' or something… in
> my mind they really are just structs… and they control what you can do with their fields."*

**One underlying aggregate — a "base-struct" (named, typed, fixed fields). The three kinds are CAPABILITY
LABELS on that base, controlling what you may do with the fields:**

| label | = today's | capability the label grants/withholds |
|---|---|---|
| (bare) | **struct** | non-EDN, in-locus, holds resources; **may NOT cross the wire** |
| EDN-portable | **core record** | EDN-representable; **may cross the wire** |
| holographic | **holon record** | EDN + **VSA ops** (similarity/holographic encoding) |

Everything else — construction, named accessors, `/from-map`, structural surfaces, width subtyping — is the
**base-struct**, shared identically. Only the label differs. This is the **holder × surface** model taken to
the *value-representation* level: one repr, the label IS the holder (the nominal capability tag).

## Why it's the truest decomplection

Today there are THREE `Value` variants (`Value::Struct`, `Value::wat__Record`, `Value::wat__holon__Record`)
with parallel machineries. The base-struct collapses them to **ONE repr + a label** — the maximal "operate on
them uniformly," and it matches the builder's actual mental model ("they really are just structs"). It is the
endpoint the whole arc points at.

## THE CONSTRAINT THAT MUST SURVIVE (the one thing to get right)

R8 / 4b-i made EDN-portability a **categorical KIND wall**: *"a struct shall never cross the wire — by kind,
not by field-check."* That wall is load-bearing (the security spine — "shared memory becomes only values"; a
non-portable resource can NEVER leak across a boundary). Collapsing to base-struct + label is only correct
**if the label is as un-leakable as the variant was** — i.e. the label is a NOMINAL, immutable, type-level
property the checker enforces categorically (a bare-labeled base-struct can NEVER be assigned to a portable
slot, the same hard wall, just keyed on the label instead of the variant). **If the label degrades the wall
to a soft runtime/field check, the unification is WRONG** (it would reopen exactly the leak R8 annihilated).
So: base-struct + label is the ideal endpoint **iff the label preserves the categorical wire wall.** That is
the single design question to answer before this strike.

## Blast radius (why it's a later, bigger strike, not now)

Collapsing the 3 `Value` variants → 1 + label touches: serialization (EDN encode/decode keys off the variant
today), the `is_portable_type` wire gate, `closure_extract`, and every exhaustive `match` on the three
variants. High blast radius — hence "horizon," converged-toward, not bolted on mid-arc.

## Pairs
`STRIKE-4b-struct-state.md` (R8 / the wire-boundary law this must preserve) · `DESIGN.md` § "Out of scope —
unifying the Value reprs" (this NOTE is the builder's answer to that future-arc pointer: yes, eventually, with
the wall-preserving constraint) · `REALIZATIONS.md` R1 (row polymorphism + the nominal-holder fusion) ·
`feedback_uniform_operation_or_decomplect_is_catastrophic`.
