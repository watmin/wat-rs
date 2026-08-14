# BRIEF — STONE 255.1b-i: the type scaffold

Read `DESIGN.md` **in full** first (all 484 lines; the `═══ LOCKED RECORD MODEL ═══` at :389 says so
itself), then `NOTE-2026-08-14-regrounding-the-premise.md` — the note carries **four corrections to
the design's premise**, one of which changes this stone's shape. Do not read the note instead of the
design.

## THE WORK, in one paragraph

Lay the type scaffold for the registry: the enums that make an unanswered question a compile error
(`Arity`, `Purity`, `Determinism`, `ExpandTime`, `DefKind`), the named-optional `MetaField<T>`, the
forced-minimum `Registration` baseline, `FnDef`, `DefDetail`, and `NativeBuiltin`. **Nothing is
registered yet and no behaviour changes** — but the baseline must be wired onto **one** real path so
it is not dead code. Floor held.

## ★ THE ONE DEVIATION FROM THE DESIGN — grounded, and it is not optional

The design's `DefDetail` (line 409) is:

```
DefDetail { Fn(FnDef), Struct(StructDef), Enum(EnumDef), Record(RecordDef),
            Protocol(ProtocolDef), Macro(MacroDef), Native(NativeBuiltin) }
```

**`StructDef`, `RecordDef` and `ProtocolDef` DO NOT EXIST.** Arc 293.2b unified struct+record into
`AggregateDef`; arc 293.3-core replaced protocols with `SurfaceDef`. And the sum already exists —
`TypeDef` (`src/types.rs:404`) carries `Aggregate · Enum · Newtype · Alias · Union · Surface`,
including three kinds the design never mentions.

**Build this instead:**

```rust
enum DefDetail {
    Fn(FnDef),
    Type(TypeDef),        // delegate — TypeDef already owns the type-kind sum
    Macro(MacroDef),
    Native(NativeBuiltin),
}
```

One exhaustive sum over type-kinds, not two. A new type-kind lands in `TypeDef` and `DefDetail`
needs no edit. Flattening `TypeDef`'s variants into `DefDetail` would create a second sum over the
same domain that drifts from the first — the exact asymmetry 255 exists to remove.

## ROOMS — read in this order

1. **`src/value/environment.rs:22` and `:46`** — `FunctionBody::{Wat, Native}` (255.1a, landed) and
   `Function`. `Native` is a unit marker, referenced at 28 sites, **never constructed**. `Function`
   is the loner the design's `FnDef` split addresses — but **the split is 255.1b-ii, NOT this
   stone.** Read it to shape `FnDef`; do not touch `Function`.
2. **`src/types.rs:404`** — `TypeDef`, the sum you delegate to. **`:266`** `AggregateDef`, **`:289`**
   `EnumDef`.
3. **`src/macros/registry.rs:9`** — `MacroDef`.
4. **`src/check.rs:15216`** — `register_builtins`, and specifically the shape of one
   `env.register(name, TypeScheme{…})` call. **332 builtins already carry a `TypeScheme`.** Your
   `FnDef` fields (`params`, `param_types`, `ret_type`, `rest_param`) must be able to receive that
   data without reshaping it — 255.2 populates from here, and if the shapes do not line up now, that
   slice pays for it.
5. **`docs/CONVENTIONS.md:1110`** — *"a module is a DIRECTORY (2026-07-26)"*. New code goes in a
   directory module, not a bare `.rs`.

## THE FORCING — this is the point of the stone, not a detail

From the LOCKED RECORD MODEL (:396–404). The baseline must make "register without answering" a
**compile error**:

- every baseline field **required** — no `Option`
- **enum-typed, never bool** — `Purity::{Pure, Effectful}`, not a fat-fingerable `true`
- **no `Default` impl** — struct-literal completeness is the wall, the same forcing an exhaustive
  match gives
- `MetaField<T> = Unspecified | Specified(T)` — a *named* sum, so digging in is a forced `match`.
  **Not `Option<T>`.** A field with genuinely more than two states gets its own named enum.

Baseline fields: `name` · `arity` · `kind` · `pure` · `deterministic` · `expand_time_legal` ·
`defined_in` · `layer`.

`defined_in`/`layer` are **auto-derived at the registration site** — a wat form cannot claim
`:rust`. In this stone that means the *type* exists and the derivation point is identified; the
derivation itself lands with registration in 1b-iii.

## WIRE IT ONTO ONE PATH

The design requires the baseline not be dead code. Pick **one** existing path, wire it, and say in
your report which you chose and why. The lightest honest option is likely a single construction site
that already knows all eight facts. **If every candidate path forces you to invent a fact you cannot
derive, that is STOP-2** — report it rather than fabricating a value.

## BLAST RADIUS

New directory module + the one wiring site. **No behaviour change. No registration. No `Function`
split. No resolver change. No `runtime.rs` carve.** Those are 1b-ii, 1b-iii, 1b-iv.

## STOP TRIGGERS — each means ship nothing, report the gap

**STOP-1 — the design's cited shape is stale again.** The note caught three missing records; if a
*fourth* cited type is missing or has moved, stop and report it rather than inventing a stand-in.

**STOP-2 — the baseline cannot be honestly populated anywhere.** If no existing path knows all eight
facts, the baseline is mis-shaped or premature. Report which fact has no source. Do **not** add a
`Default`, an `Option`, or a placeholder — that would delete the forcing this stone exists to build.

**STOP-3 — `FnDef` cannot receive `register_builtins`' `TypeScheme` data without reshaping.** Report
the mismatch; it means 255.2's cost was mis-estimated and the shape needs a ruling.

**STOP-4 — the wiring changes behaviour.** This stone is inert. If wiring the baseline alters any
observable, stop.

## THE GATE

1. `cargo build --release` — exit 0.
2. `cargo clippy --release --all-targets` — **zero warnings**, and specifically **no `dead_code`
   allow** anywhere in the new module. If you need one, that is STOP-2 in disguise: the baseline
   is not wired.
3. A **compile-fail proof of the forcing** — demonstrate that omitting a baseline field fails to
   compile. A doc-test, a commented snippet with the exact rustc error captured in your report, or a
   `trybuild` case if one is already set up. **Report the actual error text.** Without this the
   forcing is a claim.
4. `git diff --stat` — new module + one wiring site.
5. Floor: not yours. The orchestrator runs it centrally and weighs by its own re-run.

Run everything **foreground** and block on it; your turn ends when the numbers are in your hands.

## A PRIOR RESULT TO COPY FOR SHAPE

`0a32d5f8` (251.8a) and `851c0d37` (251.8a-ii) — small diffs, mutation-tested gates, honest deltas
reported rather than smoothed, and in 8a-ii's case a STOP-adjacent judgment flagged at its real
confidence instead of shipped quietly. That register is what a good report reads like.
