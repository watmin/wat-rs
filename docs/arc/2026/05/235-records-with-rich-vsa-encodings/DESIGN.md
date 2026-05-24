# Arc 235 — Records with rich VSA encodings

**Status:** PROPOSED (2026-05-24). NOT OPEN. Notes captured for later contemplation; arc actually opens post-arc-234-closure per spawn-block winding discipline.

**Origin:** User raised during arc 234 Stone 234.3b in-flight:
> *"how do we get thermometer values natively supported?... they are concrete in wat and fuzzy in holon ?... that's a core piece of the comparison tooling in vsa land — our records should embrace the full expressivity of what holon allows."*

This DESIGN is the captured contemplation — not a locked plan. Treat as ground state for future work; orchestrator drafts proper sub-DESIGNs when arc 235 opens.

---

## The gap

Arc 234 v1 records encode every field via `(:wat::holon::Atom (:wat::holon::to-holon <value>))` — exact-leaf wrapping. The `holon_form` faithfully represents the typed values BUT loses VSA's "similar things have similar vectors" property for continuous quantities.

Cosine of `(:myapp::Voltage 5.0)` vs `(:myapp::Voltage 5.1)`:
- Today: ~0.5 (different leaves → different vectors → no proximity)
- VSA-meaningful: ~0.95 (graded similarity reflects scalar proximity)

### What records use vs miss (honest count — 2026-05-24 correction)

HolonAST has **16 variants** total (per arc 221+230 substrate state):

| Category | Variants | Records use? |
|---|---|---|
| Leaves (9) | Nil, Bool, I64, F64, String, Symbol, Keyword, Char, Tag | YES — typed per field |
| Composites (3) | Bind, Bundle, Permute | Bind + Bundle YES (structure); **Permute NO** |
| Special (4) | Atom, Thermometer, Blend, SlotMarker | Atom YES (wraps every value); **Thermometer NO; Blend NO**; SlotMarker is encoder-internal |

**Records access 12 of 16 variants (~75%).** The missing 3 user-relevant variants are the **special-encoding** primitives where VSA's proximity-semantics live:

- **Thermometer** — gradient scalar encoding (proximity in scalar → proximity in vector)
- **Blend** — fuzzy semantic mix (categorical blending)
- **Permute** — position-aware / sequence-rotation encoding

The structural majority (Bind/Bundle/Atom + all 9 leaves) IS accessible. The narrow but load-bearing gap: the special-encoding subset specifically — where the substrate's "fuzzy comparison tooling in VSA land" (per user 2026-05-24) lives.

Records should be able to leverage these three — the choice is per-field, encoding-specific to the domain.

---

## The Thermometer min/max discovery

`HolonAST::Thermometer { value: f64, min: f64, max: f64 }` — the gradient encoding REQUIRES bounds. The scale's min + max define the gradient's resolution.

This is structurally important: Thermometer ISN'T "wrap an f64." It's "represent this value WITHIN this domain." Without bounds, the substrate can't auto-encode — the user must specify the domain.

Implication: **"default numerics to Thermometer" doesn't work.** A field declaration `[magnitude <- :f64]` has no bounds info. The user must declare bounds somewhere.

This pushes the design firmly toward **explicit encoding annotation** at field-declaration time. No silent default to Thermometer is possible.

---

## Four declaration shapes considered

### A — Per-field encoding annotation

```
(:wat::Record::def :myapp::Voltage
  [magnitude <- :f64 :encoding :Thermometer :min 0.0 :max 100.0])
```

Pros: explicit; user picks; bounds declared with field.
Cons: verbose; encoding+bounds mixed with field decl; multi-property field syntax.

### B — Phantom-typed wrapper (RECOMMENDED)

```
(:wat::Record::def :myapp::Voltage
  [magnitude <- :wat::holon::Thermometer<:f64,0.0,100.0>])
```

Or with named bounds:
```
[magnitude <- :wat::holon::Thermometer<:f64, :min 0.0, :max 100.0>]
```

The phantom-typed wrapper makes encoding INHERENT to the field's type. struct_form holds the underlying `:f64` for fast access; holon_form encodes via Thermometer at the declared (min, max). The type system tracks both layers structurally.

Pros: 
- Type-system-native; encoding is part of the field's type
- Composable with parametric types (`:Bundle<:Thermometer<:f64,...>>` etc.)
- struct_form/holon_form contract honest at the type level

Cons:
- Requires phantom-type machinery in check.rs (the `:Thermometer<T,...>` type that erases to T at the value level)
- Bounds-as-type-parameters (literal numbers in type positions) — possible per wat's parametric syntax but warrants verification
- Each encoding primitive (Blend, Permute) needs its own phantom-typed wrapper

### C — Record-level encoding profile

```
(:wat::Record::def :myapp::Voltage
  :encoding {:magnitude {:type :Thermometer :min 0.0 :max 100.0}}
  [magnitude <- :f64])
```

Separates encoding concerns from field declarations.

Pros: clean separation; easy to swap profiles for experimentation.
Cons: two surfaces to maintain; users hunt across both to understand a field.

### D — Default-by-type-class — **REJECTED** (min/max requirement structurally blocks)

Considered: "numerics → Thermometer automatically." Blocked by Thermometer's bounds requirement; substrate can't supply min/max without user input.

---

## Mandate vs opt-in — the design question (2026-05-24 resolution)

User raised: should rich VSA encoding be MANDATED for records, or opt-in? Honest answer: **opt-in is the right shape, mandate is overreach.**

Three structural reasons + one user-agency reason:

1. **The substrate already refuses to assume.** `HolonAST::Thermometer { value, min, max }` requires bounds; Blend needs weights; Permute needs rotation amounts. The substrate refuses to fabricate domain knowledge — that refusal is wisdom; the surface should honor it.
2. **Same type ≠ same encoding intent.** `[port <- :i64]` (opaque identifier; ports are NOT proximity-meaningful) vs `[temperature <- :i64]` (measurement; 22°C IS similar to 23°C). Same type; different encoding need; only the user knows which is which.
3. **Mandate forces fabricated knowledge.** Mandating Thermometer for numerics would force users to invent (min, max) for every numeric field — including UUIDs (opaque), hashes (opaque), counts (no natural max). Fabricating domain knowledge is dishonest substrate work.
4. **User agency.** The wat-record hologram thesis is "structural dual-form" — THAT is mandated (arc 234 ships). The hologram property is preserved structurally regardless of encoding. **Encoding-richness is a SEPARATE concern: domain-semantic.** Conflating them overreaches.

**Resolution:**
- **STRUCTURE is mandated** (arc 234, ✓ shipped): every record has struct_form + holon_form; the dual-form is non-negotiable.
- **ENCODING is opt-in** (arc 235): default = exact (Atom-wrap); per-field opt-in via phantom-typed wrappers.

**Optional future strict-mode variant** (NOT arc 235; far-future): if a domain demands VSA-strict encoding for every field (e.g., an embedding record used in similarity search), a future arc could add `(:wat::Record::def-strict ...)` that REJECTS unannotated fields at expand-time. Strict-by-record, not strict-by-substrate. The substrate stays flexible; users who need strictness get a tool.

**Discipline that lives alongside opt-in:**
- USER-GUIDE chapter on "choosing field encodings" — when to reach for Thermometer/Blend/Permute
- Possibly: lint warning when a numeric field has no annotation, asking "is exact comparison intended here?" (tunable; annoying for IDs; useful for measurements)
- Tutorial examples emphasize encoding-annotated forms where they matter

---

## Recommended path (B, with caveats)

**Recommendation: Option B (phantom-typed wrapper).** Encoding becomes structural in the type system; composes uniformly; honest about what's exact vs graded. Aligns with the opt-in resolution above.

**Open questions:**

1. **Bounds as type-parameter literals** — can wat's type system accept `0.0` and `100.0` as type-position arguments? Or do they have to be runtime args + the type just says "this field uses Thermometer encoding (bounds elsewhere)"?
2. **Auto-calibrating Thermometer** — sometimes the user doesn't know bounds in advance. Is there a `:wat::holon::Thermometer<:f64>` form (no bounds) that defers calibration to a later step (e.g., a corpus-derived calibration call)? If so, that's a separate Thermometer variant.
3. **Blend's parameter shape** — Blend doesn't have numeric bounds; it has categorical mix weights. Each encoding primitive's user-facing surface needs design.
4. **Permute's parameter shape** — Permute encodes position. Per-field declaration might say "use position-aware encoding for this field's sequence values."
5. **Backwards compatibility** — arc 234 records use exact encoding. Arc 235 introduces opt-in richer encodings. The DEFAULT (no annotation) stays exact. Existing records keep working unchanged.
6. **Cross-record cosine** — when comparing records, the encoding must be consistent for cosine to be meaningful. Two records with the SAME class share encoding profile (per-class is the natural granularity). Cross-class comparison gets murky.

---

## Implication for arc 234 in-flight + closure

**No mid-arc-234 thermometer integration.** Arc 234 ships v1 records with exact encoding. The `cosine` proximity works at exact-match level only.

**Arc 234 INSCRIPTION should note:**
- Records embrace ~1/4 of HolonAST's encoding expressivity (Atom only)
- Thermometer + Blend + Permute encoding access via records is arc 235 future work
- The hologram property is structurally in place (struct_form + holon_form dual-form); the encoding-quality upgrade is layered on the stable foundation

**No arc 234 stones need encoding-awareness retrofitted.** Arc 235 will:
- Mint phantom-typed wrappers in check.rs
- Upgrade `:wat::Record::def` macro to recognize wrappers + emit appropriate construction
- Make `:wat::Record/assoc` encoding-preserving (pattern-match the original right child's encoding constructor; rebuild with the same)
- Make `:wat::core::record->map` encoding-decoding (extract typed value from struct_form regardless of holon_form encoding)
- Verify `:wat::holon::*` verbs (234.5) handle encoded leaves correctly (Thermometer/Blend/Permute should already work since they ARE HolonAST primitives; the dispatch path doesn't care about variant)

---

## Pre-requisite: arc 234 closure

Per spawn-block winding (`feedback_spawn_block_winding`): arc 235 cannot open until arc 234 closes (235 would be 234's child if spawned during 234's active context).

**Order:** arc 234.7 INSCRIPTION → arc 235 opens.

---

## Initial stone sketch (when arc 235 opens)

- **235.0** DESIGN: lock encoding-declaration mechanism (recommend B); resolve bounds-as-type-args question; minimum-viable Thermometer surface
- **235.1** substrate: `:wat::holon::Thermometer<T, min, max>` typed wrapper minted in check.rs
- **235.2** macro: `:wat::Record::def` recognizes Thermometer wrapper at expand-time; emits `(:wat::holon::Thermometer value min max)` construction
- **235.3** assoc encoding-preservation: pattern-match original Bind right child; rebuild with same encoding
- **235.4** Blend variant + macro + assoc support
- **235.5** Permute variant + macro + assoc support
- **235.6** auto-calibration form (`Thermometer<T>` no bounds; calibrate from corpus)
- **235.7** demos: VSA-meaningful proximity (cosine 5.0 vs 5.1 ≈ 0.95 with bounds 0..100)
- **235.8** INSCRIPTION

Roughly 5-8 stones. Substantial arc.

---

## Cross-references

- `docs/arc/2026/05/234-wat-record-hologram/DESIGN.md` — predecessor (v1 records)
- `holon-rs/src/kernel/holon_ast.rs` line ~101 — `HolonAST::Thermometer { value, min, max }` definition
- `src/runtime.rs` line ~5376 — `:wat::holon::Thermometer` substrate primitive registration
- `src/runtime.rs::eval_algebra_thermometer` — Thermometer construction
- `feedback_spawn_block_winding.md` — why arc 235 waits for arc 234 closure
- `feedback_wat_llm_first_design.md` — minimum-form discipline informs the wrapper-syntax choice
- `project_typed_entities_doctrine.md` — classifier-wrap doctrine; phantom-typed wrappers extend it to encoding

---

## Why this matters

The whole point of the wat-record hologram (arc 234's thesis) is dual-form: struct_form for fast access + holon_form for VSA. Today only the STRUCTURE of holon_form is honest; the ENCODING within holon_form is degraded to exact-leaf for every value.

The full hologram property activates when:
- Structure honors VSA composition (✓ shipped in arc 234)
- Encoding honors VSA proximity (⏳ arc 235)
- Verbs operate uniformly across encodings (✓ for the structure; needs validation for encoded leaves)

Arc 235 completes the hologram's value-proposition. Until then arc 234 records are "structurally correct VSA carriers" — useful for type-safe data + basic operations — but the proximity-as-similarity property awaits encoding upgrades.
