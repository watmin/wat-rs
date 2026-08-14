# NOTE — re-grounding 255 against the disk (2026-08-14)

> ## ⚠ CORRECTED 2026-08-14, SAME DAY, AFTER READING THE WHOLE DESIGN
>
> The first version of this note was written from `DESIGN.md`'s **first ~60 lines** and was wrong
> in three ways. The corrections are kept visible rather than silently rewritten, because two of
> them are a recorded failure recurring:
>
> 1. **It asked "does `BuiltinRegistry` deserve to exist at all?" as the arc's real first
>    question.** The design already answers it, twice — **"The registry IS `sym`"** (line 117,
>    restated at 447). There is no bespoke registry; builtins register into `sym.functions` +
>    `sym.binding_metadata`, the same structures user forms use. A `═══ LOCKED RECORD MODEL ═══`
>    section at line 389 is marked *"read THIS; the sections above are the derivation."*
> 2. **It four-questioned the `type_sig` deferral and declared it overruled.** The design had
>    already overruled it 70 lines below the recommendation: *"This **upgrades** the day-one
>    four-questions trim: type-sig is not a separate deferred system — it is
>    `Function.param_types`/`ret_type`"* (line 131). The ruling stands; the derivation was
>    redundant.
> 3. **It said 255 was "entirely unbuilt, design-only."** That came from grepping for
>    `BuiltinRegistry` — a name the design *killed*. **255.1a IS LANDED:**
>    `FunctionBody::{Wat, Native}` exists at `src/value/environment.rs:22`, referenced at 28
>    sites; `Native` is a unit marker never yet constructed, which is exactly what 255.1b begins.
>
> **The failure class, named:** re-deriving an arc's conclusions instead of reading its design —
> `[[feedback_ground_the_substrate_not_just_the_chronicle]]` applied to our own design docs, the
> second instance (24o: *"The B/typed-causes principle I 'derived' was already in
> DESIGN-296-typed-causes.md"*). **Read the whole design before measuring against it.**

---

## WHAT SURVIVES — measurements the design does NOT have

These are the reason this note still exists. Each is one grep with its range stated.

### 1. 332 builtins ALREADY carry a `TypeScheme` — the design does not know this

`register_builtins` (`src/check.rs:15216–20033`, 4,817 lines) contains **332** `env.register(name,
TypeScheme{…})` calls. The design says type-sig knowledge lives *"in `infer_list`'s per-builtin
knowledge"* (line 149) and treats populating `param_types`/`ret_type` as "the heavy part."

**So 255.2 is smaller than the design thinks** — a third of the work is already data, in the right
shape, in the right place. What remains is the **141** hand-written keyword arms inside
`infer_list`'s Keyword block (`src/check.rs:2518–5568`, 3,050 lines).

**The honest table of where builtin knowledge lives today:**

| what | count | form | site |
|---|---|---|---|
| check-time **type signature** | **332** | **DATA** — `TypeScheme` | `register_builtins` |
| check-time **hand inference** | **141** | code — match arms | inside `infer_list` |
| **runtime dispatch** | **678** | code — match arms | `runtime.rs` (scattered) |
| **resolve** | **0** | — | the blanket-accept, `src/resolve/walk.rs:257` |

### 2. The counts have moved since the design was written

| | design says | measured 2026-08-14 |
|---|---|---|
| dispatch arms | 454 / "~483" | **678** |
| `runtime.rs` | (MODULARIZATION-NOTES: 23,801) | **35,066** (+47%) |
| `check.rs` | (MODULARIZATION-NOTES: 15,108) | **20,863** (+38%) |

Together the two megafiles are **64%** of `src/`'s 87,962 lines. 37 bare `src/*.rs` remain against
23 directories.

**⚠ BOUNDS:** the 332 is brace-bounded and clean. The 678 counts keyword literals at line-start in
`runtime.rs` and may include non-dispatch occurrences. The 141 counts keyword arms in `infer_list`'s
span. **None is a census** — the design already names the right instrument: *"substrate-as-teacher
drives completeness: any builtin missing from the registry → resolve rejects real corpus code → add
it."* That cascade is the worklist, not any number here. (On 2026-08-13 two greps of the same corpus
returned 1025 and 998; this arc does not trust grep counts.)

### 3. The megafile carve — the builder RULED it into 255, and my first note advised against it

`DESIGN.md:275`, **"255 IS ALSO THE MEGAFILE CARVE (builder, 2026-06-21)"**:

> *"when we build 255 we rip out as much shit from the megafiles as we can to
> `src/<namespace>/<scope>.rs` — we've been attacking those huge files strategically to make the
> migration more tractable later."*

The first version of this note argued the opposite — *"'registry AND breakup' is two arcs braided
and fails Simple"* — which contradicts a standing ruling recorded in the very document it was
re-grounding. **Retracted.**

And the design shows why the Simple objection was wrong: **the carve is not relocation, it is where
the new declarations live.** Each namespaced home exposes its own `register_builtins(&mut …)`;
`runtime.rs` becomes an assembler that calls each home; the central match shrinks toward nothing.
*"Soundness fix + carve, one motion."* Nothing is braided, because the declarations must be written
somewhere regardless — and their home is the namespace they belong to.

`docs/CONVENTIONS.md:1110` already rules the target shape: *"a module is a DIRECTORY (2026-07-26)"*.
`docs/MODULARIZATION-NOTES.md` (queued 2026-05-08, gated on "after arc 109 wraps") is the separate,
still-unnumbered general breakup; 255 carves only what it touches.

### 4. ⛔ THE DESIGN'S LAYER-2 `DefDetail` SUM IS STALE — 255.1b-i cannot be briefed as written

The LOCKED RECORD MODEL specifies (line 409):

```
DefDetail { Fn(FnDef), Struct(StructDef), Enum(EnumDef), Record(RecordDef),
            Protocol(ProtocolDef), Macro(MacroDef), Native(NativeBuiltin) }
```

Measured 2026-08-14 — **three of those records do not exist**, and the sum they belong to already
does:

| design cites | disk |
|---|---|
| `StructDef` | **NOT FOUND** — arc **293.2b** unified struct+record into `AggregateDef` (`types.rs:266`) |
| `RecordDef` | **NOT FOUND** — same unification (266/STUB.md was re-opened by exactly this) |
| `ProtocolDef` | **NOT FOUND** — arc **293.3-core** replaced it with `SurfaceDef` |
| `EnumDef` | ✓ `types.rs:289` |
| `MacroDef` | ✓ `macros/registry.rs:9` (exact) |
| `Function` | ✓ `value/environment.rs:46` (design said `env.rs:35` — moved) |
| `NativeBuiltin` | not found — **expected**, it is new in 255.1b-i |

**And the sum already exists.** `TypeDef` (`src/types.rs:404`):

```rust
pub enum TypeDef {
    Aggregate(AggregateDef),  // 293.2b — struct AND record
    Enum(EnumDef),
    Newtype(NewtypeDef),      // ← design does not mention
    Alias(AliasDef),          // ← design does not mention
    Union(UnionDef),          // ← design does not mention (stone 237.1)
    Surface(SurfaceDef),      // ← 293.3-core, replaces Protocol
}
```

So `DefDetail` as specified would (1) name three types that do not exist, (2) miss three kinds that
do, and (3) **duplicate `TypeDef`'s job** — a second exhaustive sum over the same domain, which is
the asymmetry 255 exists to remove.

**THE FORK, and it is 255.1b-i's real first decision:**

- **(a) `DefDetail { Fn(FnDef), Type(TypeDef), Macro(MacroDef), Native(NativeBuiltin) }`** — delegate
  the type-kinds to the sum that already owns them. **Simple: YES** (one sum over type-kinds).
  **Honest: YES** (a new type-kind lands in `TypeDef` and `DefDetail` needs no edit).
- **(b) flatten `TypeDef`'s variants into `DefDetail`** — two exhaustive sums over one domain, drifting
  apart from the moment they are written. **Simple: NO. Honest: NO.**

**(a) is the shape.** Recorded here rather than silently chosen; it is a deviation from a LOCKED
section and the deviation is a measurement, not a preference.

⚠ **This is the mirror of failure #3 above.** There I ignored the design and re-derived it; here the
design is stale and the disk wins. Neither is "trust the doc" or "trust the code" — it is *read
both, and when they disagree, say so in writing before anyone builds.*

---

## THE ARC'S STATE, corrected

- **255.1a — LANDED.** `FunctionBody::{Wat, Native}` (`src/value/environment.rs:22`).
- **255.1b-i — NEXT**, and already specified by the design (line 470): the type scaffold —
  `Arity`, `Purity`/`Determinism`/`ExpandTime`/`DefKind`, `MetaField<T>`, the baseline
  `Registration`, `FnDef`, `DefDetail`, `NativeBuiltin`; **wire the baseline onto ONE path so it is
  not dead code**; floor held.
- Then 255.1b-ii (the `FnDef` split, ~31 sites) → 1b-iii (register from HOMES, first home a small
  pure template, carve those arms out of `runtime.rs`) → 1b-iv (resolver rewrite, delete the
  blanket-accept) → 255.1c… per-home repeats → 255.2 reflection verbs → 255.3 consumers collapse.

**Nothing in this note re-plans the arc.** The design's decomposition stands; these are corrections
to its premise's *numbers* and a retraction of my own bad advice.
