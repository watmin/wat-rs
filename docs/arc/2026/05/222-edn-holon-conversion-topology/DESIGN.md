# DESIGN — Arc 222 — EDN ↔ holon direct path + 3×2 conversion topology

**Opened:** 2026-05-22 (placeholder; full DESIGN ratified after arc 221 Phase A ships)
**Branch:** `arc-170-gap-j-v5-deadlock-state`
**Depends on:** arc 221 Phase A (Stone 221.1 `HolonAST::Char` leaf + Stone 221.2 wat-rs `value_to_atom` Char + Uuid arms) — without these leaves, EDN→holon cannot be honest.
**Recommended:** arc 221 Phase B also ships first (Keyword + Nil + Tag leaves + canonical-bytes seed fix) for full EDN-syntax coverage on HolonAST; arc 222 can start with just Phase A in place but Phase B closes the doctrine completeness.

## Mission

Mint the missing cells in the EDN ↔ wat ↔ holon conversion topology. Inscribe the doctrine that HolonAST primitives (Atom/Bundle/Bind/Permute/Thermometer/Blend/SlotMarker) are SUBSTRATE INTERNALS while EDN + wat literals are the SURFACE for data construction. Make holon-as-data-host first-class without forcing users through algebraic ceremony.

## Triggering observation

User-articulated 2026-05-22 during the arc 221 atomization dialogue:

> *"i want to be able to do comparisons of some data and some edn ... i can holon-ify the data and the edn->data->holon and measure them equally without having to see the guts of (Bundle (Bind ...) ...) ... and i can take holon-data => wat-data right?.. the encoding of the literals is interpretable in same way edn is?.. its just another encoding technique?..."*

> *"ok - so - we're arguring for a 3x3 grid?... is that right?.. wat->edn wat->holon edn->wat edn->holon holon->wat holon->edn ... is that... 3x2 ?... Atom, Bundle, Bind, Permute are all internals - you can use them, but you can also just use the data - holon can host the data in its natural form?.."*

## The 3×2 conversion topology

```
            edn         wat         holon
edn          •       edn→wat    edn→holon
wat       wat→edn       •       wat→holon
holon   holon→edn   holon→wat       •
```

Three first-class representations × two directions = 6 conversion cells. Identity diagonals trivial.

**Cell status as of 2026-05-22 (pre-arc-221):**

| Cell | Status | Path / Verb |
|---|---|---|
| edn → wat | ✓ mature | `edn_to_value` at `src/edn_shim.rs:342` (typed + untyped); arc 219 strict-EDN |
| wat → edn | ✓ mature | `wat_edn::write` + `value_to_edn_with` at `src/edn_shim.rs:1537+`; arc 218 IMPECCABLE |
| wat → holon | ⚠ partial | `value_to_atom` at `src/runtime.rs:13800-13837`; arc 221 Phase A adds Char + Uuid arms |
| **edn → holon** | **✗ MISSING (direct)** | currently chained `edn → wat → holon`; arc 222 mints direct verb |
| holon → wat | ⚠ partial | path exists via `atom-value` accessors but incomplete; arc 222 audits |
| **holon → edn** | **✗ MISSING (direct)** | can't render HolonAST as EDN literal text; arc 222 mints `to-edn-string` |

## Doctrine to inscribe

**HolonAST primitives (Atom/Bundle/Bind/Permute/Thermometer/Blend/SlotMarker) are SUBSTRATE INTERNALS.** The algebraic assembly language. Available as a power-user dropdown for:

- Custom Bind compositions (specific tag-payload shapes outside the EDN canon)
- Gradient encoding via Thermometer (continuous value → vector)
- Weighted-sum composition via Blend (VSA experimentation)
- Opaque-identity wrap via Atom (when explicit identity boundary is desired)
- Permutation encoding for sequence-position-aware structures

**EDN + wat literals are the SURFACE.** Users live here. They write data in its natural form:

```wat
{:a true :b "hello"}     ; map literal
#{1 2 3}                 ; set literal
'(1 2 3)                 ; list literal
[1 2 3]                  ; vector literal
\a                       ; char literal
#uuid "..."              ; tagged literal
```

**Substrate auto-compiles literal-form into algebraic-form.** No `(Bundle (Bind ...))` ceremony required for the common case of "I have data, host it in holon-space for measurement."

**Holon hosts data natively.** The 16 HolonAST variants after arc 221 ships (9 leaves + 3 composites + 4 special) cover full EDN syntax via leaves (untagged primitives) + Bundle+Bind composition (collections + tagged literals).

## Stones (provisional — ratified post-arc-221)

### Stone 222.1 — `edn → holon` direct verbs

`src/runtime.rs` (or new module `src/edn_holon.rs` if cleaner):

- `:wat::holon::from-edn-string` — takes `String`, parses via `wat_edn::parse`, walks `OwnedValue` tree producing `HolonAST` directly (skip Value layer). Returns `Value::holon__HolonAST(...)`.
- `:wat::holon::from-edn-literal` — takes wat literal (parser captures as WatAST), walks WatAST producing `HolonAST` directly. Returns `Value::holon__HolonAST(...)`. Equivalent to existing `:wat::holon::Atom (:wat::core::quote <form>)` but ergonomic.

Implementation: a `edn_to_holon` walker function that mirrors `edn_to_value` but produces `HolonAST` nodes directly per arc 216.7 doctrine + arc 221 leaf completeness.

### Stone 222.2 — `holon → edn` direct verb

- `:wat::holon::to-edn-string` — takes `Value::holon__HolonAST(h)`, walks `h` producing canonical EDN literal text. Reverse of Stone 222.1's `from-edn-string`.

Implementation: a `holon_to_edn` writer function that produces canonical EDN literal text per the encoding doctrine.

### Stone 222.3 — `holon → wat` completeness audit + any missing verbs

Audit current `atom-value` accessors + related verbs for completeness; mint any missing primitives. Goal: full HolonAST → Value reconstruction (the inverse of `value_to_atom`).

### Stone 222.4 — Doctrine inscription

`DESIGN-222` § "Doctrine" section (this DESIGN's "Doctrine to inscribe" expanded):
- 3×2 conversion topology
- Substrate-primitives-as-internals; literals-as-surface
- Holon hosts data natively
- When to drop down to algebraic dropdown
- Cross-references to arc 216.7 + arc 221

### Stone 222.5 — INSCRIPTION + USER-GUIDE + cross-references

- arc 222 INSCRIPTION
- USER-GUIDE updates for the 6 verbs + the doctrine
- CLIFFNOTES Currently update
- 058 changelog row
- Cross-references to arc 220, 221, 216

## What this arc does NOT do

- Add HolonAST variants (arc 221 is the substrate doctrine arc; arc 222 builds on it)
- Add new EDN-spec capabilities (BigInt/BigDecimal stay out-of-spec)
- Touch wat-edn substrate (the wire format)
- Touch Value runtime layer (the runtime convenience)
- Mint surface-syntax sugar for collection-construction macros (separate concern; defer if needed)

## Calibration (provisional)

| Stone | Predicted | Notes |
|---|---|---|
| 222.1 | 60-90 min | `edn_to_holon` walker + 2 verbs + ~5 probe tests |
| 222.2 | 45-60 min | `holon_to_edn` writer + 1 verb + ~5 round-trip tests |
| 222.3 | 60-90 min | audit + any missing verbs; cascade-dependent on completeness |
| 222.4 | 30-45 min | doctrine inscription in DESIGN + INSCRIPTION |
| 222.5 | 30 min | paperwork |

**Total estimate:** 3.5-5.5 hours of wat-rs work; smaller than arc 221's holon-rs + wat-rs split.

## Unblocks

- The cosine-similarity use case for data ↔ EDN comparison (immediate)
- LLM workflows that read EDN as data and compare via holon-space (wat-MCP horizon)
- Round-trip preservation through any topology cell (essential for arc 217 Clojure-IPC bridge)

## Open questions (DESIGN review)

1. **Should `:wat::holon::Atom` verb be extended to recognize wat literals (auto-quote + lower)?** Currently requires explicit `(:wat::core::quote ...)`. Could be unified with `:wat::holon::from-edn-literal`.
2. **Should `from-edn-literal` use the wat lexer's existing literal parsing or duplicate it?** Single source of truth vs separation of concerns.
3. **Should `to-edn-string` produce strict-EDN (per arc 219) by default, or wat-extension EDN for round-trip identity?** Strict-EDN is the right Clojure-interop default; wat-ext-EDN preserves more wat-specific info.
4. **Collection-construction macros (`:wat::holon::set`, `:wat::holon::map`, etc.) — fold into arc 222 or separate?**

These are deferred to post-arc-221-Phase-A DESIGN ratification.

## Cross-references

- arc 220 INSCRIPTION (when Slice 5 ships) — `:wat::core::Char` + `:wat::core::List` minting + IPC contract
- arc 221 DESIGN (`docs/arc/2026/05/221-holon-ast-primitive-layer-honesty/DESIGN.md`) — HolonAST primitive-layer leaves; Char/Keyword/Nil/Tag
- arc 216 Stone 216.7 (encoding doctrine) — 3 categories: Primitives/Collections/Tagged; arc 222 builds the surface verbs for the doctrine
- arc 217 (Clojure-IPC bridge — the named consumer for EDN↔holon work)
- INTERSTITIAL 2026-05-22 — the doctrine-emergence narrative
- `project_3x2_conversion_topology` memory — the doctrine in summary form
- `project_wat_reveals_holon` memory — the dynamic that surfaced this work
