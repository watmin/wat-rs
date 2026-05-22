# Intueri Findings — `holon-rs/src/kernel/holon_ast.rs`

**Spell:** intueri (datamancy grimoire)
**Target:** `/home/watmin/work/holon/holon-rs/src/kernel/holon_ast.rs`
**Agent:** claude-sonnet-4-6
**Date:** 2026-05-22 (very-late)
**Cast by:** orchestrator (claude-opus-4-7) per `feedback_spells_cast_via_subagent`
**Duration:** ~1 min wall-clock (read-and-report)

## Spell verdict

**Spark verdict: lives, with four Level 2 mumbles. ZERO Level 1 lies.**

The substrate algebra is honest. The variant names (Atom, Bundle, Bind, Permute, Thermometer, Blend, SlotMarker, plus the 9 leaves) speak truth per their doc-comments + actual behavior. The cast confirms the substrate's foundation is NOT where the lying lives — the lies are at the wat-rs verb-dispatcher layer above (the surface that wraps these primitives).

The four mumbles found are at **supporting infrastructure** (constants + helper functions) — they create friction on arrival but do not lie about semantics.

## Level 1 Findings (lies)

**None.**

Intueri's specific finding on the `Atom` variant question (orchestrator deliberately asked the spell to weigh this fresh):

> *"The substrate-level `HolonAST::Atom` name is a legitimate algebraic name — it means 'opaque-identity wrap,' and the variant doc at lines 122–130 says exactly that. The lying is at the verb-layer dispatch in `wat-rs`, not here. The variant name on the substrate side is honest: it wraps one AST and renders it opaque, which is what a Lisp `atom` does to a non-pair."*

This is the canonical finding. The substrate's `HolonAST::Atom` is honest. The wat-rs `:wat::holon::Atom` VERB is the Level 1 lie. That's a different scan target (Stone 224.2).

## Level 2 Findings (mumbles)

### Finding 1 — `PRIM_TAG_STRING` casing anomaly (line 601)

```rust
const PRIM_TAG_STRING: &str = "String";
```

Every other PRIM_TAG is lowercase (`"symbol"`, `"i64"`, `"f64"`, `"bool"`, `"char"`, `"keyword"`, `"nil"`, `"tag"`). `PRIM_TAG_STRING` is `"String"` — capital S. The comment block at lines 593–599 explains this is legacy preservation: the old `Atom(Arc<dyn Any>)` registry used `"String"` (the Rust type name), so the value is frozen to preserve vector identity backward compatibility.

The comment explains the WHY. But the constant's NAME (`PRIM_TAG_STRING`) promises nothing about the anomaly — a reader scanning the constants block sees nine lowercase values and one capitalized one, wonders if it's a typo, and must read the comment to learn it is not.

**Level 2 — mumbles.** The constant name doesn't surface the lie-vs-legacy distinction.

**Proposed direction:** rename to `PRIM_TAG_STRING_LEGACY` and add inline `// "String" not "string" — frozen for vector backward-compat; see block comment above`. The name itself should telegraph that this one is different.

### Finding 2 — `slots()` method name inverts the noun (lines 505–518)

```rust
pub fn slots(&self) -> Vec<f64> { ... }
pub fn ranges(&self) -> Vec<(f64, f64)> { ... }
```

`slots` returns "the Thermometer values in pre-order." A "slot" in the broader holon/wat vocabulary means a receptive-field placeholder — the thing that gets filled by a Thermometer value. So `slots()` returns the *fillers*, not the slots. The actual slot (the shape-without-value) is what `SlotMarker` represents; calling the *values* `slots` inverts the noun.

The partner method `ranges()` returning `(min, max)` pairs is fine — that name speaks.

But `slots()` returning the *values* creates a terminological collision with `SlotMarker` (which IS the slot). A reader who has just read the `SlotMarker` variant doc arrives at `slots()` expecting "returns the SlotMarkers" and finds floating-point values.

**Level 2 — mumbles.**

**Proposed direction:** rename to `thermometer_values()`, or at minimum `slot_values()`. The parallel partner `ranges()` is fine as-is.

### Finding 3 — `leaf_seed` undersells SHA-256 cryptographic anchor (lines 696–703)

```rust
fn leaf_seed(type_tag: &str, payload: &[u8], global_seed: u64) -> u64 {
```

The function hashes a type-tag + payload + global-seed through SHA-256 and returns the first 8 bytes as a `u64`. It is only called in `encode` for leaf nodes (lines 710–749). The name `leaf_seed` correctly scopes it to leaves, but `seed` is vague — this is a SHA-256-derived deterministic seed for the ChaCha RNG used in `deterministic_vector_from_seed`. The relationship between "what the function produces" and "how that output is consumed" is invisible in the name.

This is a minor mumble because the function is private and small (8 lines). But it is the critical cryptographic primitive that makes the entire deterministic vector identity guarantee work, and its name does not signal that.

**Level 2 — mumbles.**

**Proposed direction:** `sha256_leaf_seed` (emphasizes cryptographic anchor) or `leaf_vector_seed` (emphasizes the output is consumed by a vector generator).

### Finding 4 — `write_atom_payload` carries legacy "atom" prefix (lines 611–617)

```rust
fn write_atom_payload(out: &mut Vec<u8>, type_tag: &str, payload: &[u8]) {
```

This helper writes the `[TAG_ATOM, len(type_tag), type_tag, len(payload), payload]` canonical bytes. It is called for **all 9 vocabulary leaves** (Symbol, String, I64, F64, Bool, Char, Keyword, Nil, Tag) AND for `Atom(inner)`. The name `write_atom_payload` has "atom" in it because the original protocol came from the `Atom(Arc<dyn Any>)` era — but now it is the primitive-leaf serializer for the entire vocabulary tier.

The legacy comment at lines 627–630 acknowledges this: *"Primitive leaves use the same `[TAG_ATOM, type_tag, payload]` shape the legacy `Atom(Arc<dyn Any>)` did."* So the "atom" in `write_atom_payload` is a historical artifact. A reader arriving fresh sees `write_atom_payload` called for a `Symbol` or `Bool` and must pause to learn why a symbol write is called "atom."

**Level 2 — mumbles.**

**Proposed direction:** `write_leaf_canonical` (emphasizes: this is canonical-byte serialization for a vocabulary leaf). The internal byte format keeping `TAG_ATOM` as the tag byte is an implementation detail that the function's name need not perpetuate.

## Rune evaluations

**None.** No existing intueri runes were found in the file.

## Spark assessment

> *"The spark lives in the variant documentation. The module-level doc tells a story — three tiers, the closure property, the legacy canonical-bytes contract, the SlotMarker sentinel's reason for existing. Most variant docs answer WHY (the `Atom` variant doc explains the algebraic significance of opaque-identity; the `SlotMarker` doc explains the template-equality contract; the `Keyword` doc explains the arc 221 migration). The test suite is organized by concern, each section labeled, with WHY comments on the tests that verify non-obvious invariants (`atom_holon_differs_from_direct_encoding`, `atom_wrap_is_repeatable`). These are marks of someone who cared."*
>
> *"The dim spots are all naming — specifically: `PRIM_TAG_STRING`'s silent exception to the lowercase convention, `slots()` delivering values where the domain says 'slot' means placeholder, `write_atom_payload` carrying 'atom' while serving the whole vocabulary tier, and `leaf_seed` underselling its SHA-256 cryptographic anchor role. None of these hide bugs. All of them create a slight friction on arrival — the reader pauses, reads a comment, resolves the confusion, moves on. Intueri's job is to name the pauses."*

## Disposition

The findings are real but **lower-stakes than the verb-layer Level 1 lies surfaced in the doctrine dialogue.** The four mumbles can be:

1. **Fixed now** — small Rust renames + comment refreshes; ~30 min cumulative
2. **Tracked as a future-cleanup sub-stone** — included in arc 224 Phase 2 fix-arc planning
3. **Deferred to a substrate maintenance arc** — bundled with similar mumbles found in subsequent intueri casts

**Recommendation:** defer the holon_ast.rs renames into arc 224 Phase 2 planning. The substrate algebra is honest; the renames improve readability but don't fix lies. Bundle with whatever runtime.rs (Stone 224.2) + check.rs (Stone 224.3) surface.

## Cross-references

- arc 224 DESIGN.md — substrate naming honesty audit scope
- intueri SKILL.md — `~/work/holon/datamancy/intueri/SKILL.md`
- arc 221 DESIGN.md — substrate-doctrine work that surfaced this audit
- `feedback_inscription_immutable` — these findings stay as historical record even after fixes ship
