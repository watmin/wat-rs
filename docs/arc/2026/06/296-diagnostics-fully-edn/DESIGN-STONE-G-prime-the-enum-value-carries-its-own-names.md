# 296 · DESIGN STONE G′ — the ENUM value carries its own field names

> **STATUS: DRAWN 2026-08-15. STRIKE-READY.** The mirror of Stone G, on the one carrier G never
> touched. **Read `DESIGN-STONE-G-the-value-carries-its-own-names.md` first — it is BUILT, and it is
> the worked template this stone copies.** G′ invents nothing G did not already establish.

## HOW THIS SURFACED — and the question that decided the shape

Wave B3's brief warned a rider about `field-N` in value snapshots. Builder: *"i thought we
annihilated /every/ field-NNN producer......"* Measured: **one** survives in all of `src/` —
`edn_shim.rs:2727`, inside `value_to_json_natural`'s **enum** arm.

The first design drafted for it was a graceful degradation: when names are unavailable, emit
`{"_type": …, "_fields": [1,2]}` instead of fabricating `{:field-0 1}`. Builder: *"when are names not
known?... whose source code is lost when?...."*

**Answer, from the disk: never.** Traced all three ways `enum_variant_field_names` can return empty:

| # | failure path | can it hold a live `EnumValue`? |
|---|---|---|
| 1 | `types` is `None` | **No.** `SymbolTable.types` is set at world construction (`freeze/env.rs:292`). An `EnumValue` requires its enum to have been **declared**, so it cannot exist in a world with no `TypeEnv`. `None` here is a caller rendering before the world exists — a boundary bug. |
| 2 | `type_path` absent from the registry | **No.** Built locally ⇒ the type is registered (contradiction). Arrived from outside ⇒ it belongs in `ForeignRecordValue` / `ForeignVariant`, whose fields are *"self-carried from the wire"* (arc 278 Stone A) — which is exactly why Stone G records that ForeignRecord *"has never had this bug."* |
| 3 | variant absent, or not `Tagged` | Same as 2. |

**Nobody's source code is lost.** Either the type is ours — registered, names knowable — or it is
foreign, and foreign values already self-carry. There is no third population.

⛔ **So `_fields` was a form for a state that should not exist**, and building it would have been
worse than the bug: once `_fields` is a supported output, a wire decoder that mints an `EnumValue` for
an unknown type stops being a defect and becomes a feature. That is constraint engineering run
backwards — deriving the *cannot* and then shipping an escape hatch for it. Builder's ruling:
*"should be unrepresentable — this is the greatest fix."*

## THE DEFECT

`EnumValue` carries only positional `fields`:

```rust
pub struct EnumValue {
    pub type_path: String,
    pub variant_name: String,
    pub fields: Vec<Value>,          // ← no names
}
```

So naming them at render time needs an external registry lookup, and the three ways that lookup can
fail all collapse into one arm answering `field-0`, `field-1`. **That is not a degraded rendering. It
is a lie with a plausible shape** — a consumer cannot distinguish a genuine field named `field-0`
from a failed lookup. Builder, on the identical defect in G: *"the field-??? values are dishonest."*

## THE SHAPE — G's, one carrier over

```rust
pub struct EnumValue {
    pub type_path: String,
    pub variant_name: String,
    /// Field names in declaration order. **Same length as `fields`, always.**
    /// Arc 296 G′: carried, never looked up — the enum mirror of `AggregateValue.names`.
    pub names: Arc<Vec<String>>,
    pub fields: Vec<Value>,
}
```

**This does not fix the three causes — it deletes the question.** No registry is consulted, so an
absent type and a variant mismatch cannot arise; and because names are supplied ALONGSIDE the values
by whoever builds them, a names/values arity mismatch is unrepresentable rather than rendered.

Unit variants carry an empty `names` (same length as their empty `fields`) — no special case.

## ⛔ WHERE THE NAMES COME FROM — never a human's fingers

**G's table governs verbatim.** Its first draft hand-transcribed names into Rust literals and the
builder stopped it: *"we did that exact move recently?"* A literal is a SECOND place the names are
stated, and a **right-count/wrong-name** literal renders confidently and wrongly — worse than the
`field-N` this arc annihilates, because it looks like an answer.

| site kind | source of names |
|---|---|
| holds a registry | **`EnumDef::variant_names_arc(variant)` — MUST BE MINTED (see below)** |
| type known statically, no registry | a `wat_field_names_from!` const — **LANDED** (`wat-source-derive`) |
| rebuilding from an existing value | `ev.names.clone()` — carry the source value's own |
| generic constructor | registry lookup; an unregistered type is an **ERROR**, not a fallback |

### The one new piece

`AggregateDef::names_arc()` exists (`src/types.rs:295`). **There is no enum equivalent.** Mint its
mirror, and mint it as the ONE DOOR:

```rust
impl EnumDef {
    /// Field names of a tagged variant, declaration order. `None` when the variant
    /// is absent or is a Unit — the caller RAISES; it does not fabricate.
    pub fn variant_names_arc(&self, variant: &str) -> Option<Arc<Vec<String>>> { … }
}
```

The names are already in the registry — `EnumVariant::Tagged { name, fields: Vec<(String, TypeExpr)> }`
(`src/types.rs:319`). This function only stops every caller from re-walking `def.variants` by hand.

## THE WORKLIST — impose the change and read rustc

**Do NOT survey for it.** Every count taken by grep on this arc has been wrong — four times before
today, and twice more today (a `git log -1 <ref> -- <path>` "touched" list that answered a different
question, and G's own stale 7-vs-1). The compiler's count is the honest one.

A pre-count for sizing only, **which the rider must replace with rustc's**: `grep -rn "EnumValue {"`
reports **106** sites — 87 `src/runtime.rs`, 7 `src/edn_shim.rs`, 3 `src/services/verbs.rs`,
3 `src/io.rs`, 2 `src/intrinsic/mod.rs`, 1 each in `value/value.rs`, `test_runner.rs`,
`rust_deps/sqlite.rs`, `rete/purity.rs`.

## THEN, AND ONLY THEN — delete the fallback

With names on the value there is nothing left to fall back *from*:

- `src/edn_shim.rs:2727` — the sole surviving `format!("field-{}", i)` in the whole tree. **Delete.**
- `src/edn_shim.rs:2783` `enum_variant_field_names` — its three silent `vec![]` arms and the function
  itself. Its only consumer is the site above. **Delete the whole function.**
- `value_to_json_natural`'s `types: Option<&TypeEnv>` parameter, if nothing else needs it after the
  above — a door nobody walks through is a door that rots. Check before removing; report either way.

**The disconfirming evidence already exists.** G records that an earlier session deleted these
fallbacks *without* G and read the screams: **4 reds out of 4413**, every one real — including a
heretic test that pinned `{:field-0 3 :field-1 4}` as its EXPECTED value. The fallback is not
load-bearing; it was hiding defects.

## THE GATE

| # | assertion |
|---|---|
| 1 | `grep -rn 'format!("field-' src/` → **0**. The census goes to zero and the wall is that it stays there. |
| 2 | `enum_variant_field_names` no longer exists |
| 3 | A tagged variant renders with its **declared** field names on the JSON surface — a fixture whose names are `x`/`y` renders `x`/`y` |
| 4 | A unit variant still renders as a plain string (`value_to_json_natural`'s documented contract) |
| 5 | EDN rendering of variants is **byte-identical** — Stone A.0's `#tag [positional]` never consulted names and must not change |
| 6 | No name literal was typed by hand anywhere in the diff — every name traces to the registry, a `wat_field_names_from!` const, or a carried `.clone()` |
| 7 | floor green, clippy 0 |

**Row 6 is the one the builder stopped G's first draft over. It is not optional.**

## STOP TRIGGERS

- **STOP-1 — a construction site has no honest source of names.** Do NOT invent a literal. Report it;
  a site that cannot name its own fields **is the finding**.
- **STOP-2 — a generic constructor reaches an unregistered type.** Raise. Do not fall back.
- **STOP-3 — you are tempted to keep a positional fallback "just in case."** That case is the one this
  stone proved cannot occur. Re-read the three-row table above.
- **STOP-4 — the EDN variant rendering changes.** Stone A.0 is untouched by this stone.
- **STOP-5 — more than ~6 reds you cannot immediately attribute.** Report and stop; the orchestrator
  re-plans. Expect reds — the migration is wide by nature and the cascade IS the worklist
  (`docs/SUBSTRATE-AS-TEACHER.md`); a red that names a site is the substrate telling you where to go.

## Kin

- `DESIGN-STONE-G-the-value-carries-its-own-names.md` — **BUILT.** The template, the generator
  discipline, and the where-the-names-come-from table this stone obeys.
- `NOTE-value-to-edn-renders-fields-positionally.md` — the original defect record.
- Arc 278 Stone A — `ForeignRecordValue`, the sibling that self-carries and never had this bug.
- Arc 278 Stone A.0 — the EDN `#tag [positional]` variant encoding, deliberately name-free.
