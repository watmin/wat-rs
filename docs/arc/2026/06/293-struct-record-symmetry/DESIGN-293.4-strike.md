# 293.4 — methods-are-accessors + `defsurface` subsumes & annihilates `defprotocol`

> **Status: STRIKE — lair studied 2026-06-27. DESIGN + RED gate already exist** (`293/DESIGN.md` § 293.4 +
> `probe_arc293_acceptance_demo.rs`, `#[ignore]`'d RED). This doc draws the build + decomposition. **Resumed because
> 293.4 unblocks 118** — `Seqable` is the first method-surface (`first`/`rest`/`empty?` are method accessors).

## The thesis (DESIGN R1, `FORMA SOLA SUFFICIT`)
"Methods are accessors." A surface requires *accessors* (`:T/name`); a satisfier backs each with a **field** (free
accessor) OR a **method** (a `defn :T/name`) — its private choice, invisible to the interface. This dissolves the
field/method seam and subsumes `defprotocol` (a protocol = a method-only surface). The **acceptance demo** proves it:
`Circle` backs `color` with a field, the holon `Vector` backs it with a method (the monkeypatch), one `describe`
consumes all three by runtime-type dispatch.

## The lair (grounded 2026-06-27)
- **`defsurface` is FIELD-ONLY today.** `SurfaceDef.members: Vec<(String, TypeExpr)>` (`types.rs:233`);
  `parse_defsurface` (`src/types/surface.rs:48`) parses `[name <- :T …]` only; `struct_satisfies_surface`
  (`surface.rs:26`) = width-subtyping over fields. **Method members `(area [self] -> :f64)` do not parse.**
- **The dispatcher machinery to LIFT exists (arc 232):** `parse_defprotocol_form` + the protocol method dispatch via
  `extract-classifier` + `apply` (`runtime.rs:670`, `preregister_protocol_names` `runtime.rs:1546`, the 232.1 top-level
  arm `runtime.rs:1895`). 293.4 RE-HOMES this under `defsurface`/`definterface`, not re-invents it.
- **`defprotocol` annihilation target is SMALL:** exactly ONE live wat use — `:wat::spawn::Locus` (`wat/spawn.wat:224`,
  consumed in `wat/service.wat:926` + `spawn.wat:242/262`). Plus the Rust machinery: `runtime.rs` (parse + dispatch +
  preregister), `check.rs`, `value/value.rs`, `check/env.rs`, `freeze/env.rs`, `stdlib.rs` (load-order).
- **`extend-type` survives, demoted** — today an arc-232 subtype-edge form (`types.rs:1605`); becomes the typed
  foreign-accessor adapter (adds `:T/accessor` impls to a type you don't own — the monkeypatch; collisions =
  `DuplicateDefine` compile errors).

## The one contract decision (pinned — DESIGN R1/R2)
A surface member is **an accessor signature `:T/name (self, …) -> ret`**, backed by EITHER a field (auto-accessor from
`defrecord`/`defstruct`) OR a method (`defn :T/name`). Satisfaction is **structural + width-open**: a type satisfies a
surface iff for every member it exposes a matching accessor (field-or-method), assignable signature. NO `:satisfies`,
NO declaration at the satisfier.

## Decomposition (sub-strikes — the acceptance demo is the final GREEN gate)
- **293.4a — method members in `defsurface`.** `SurfaceDef.members` → `Vec<SurfaceMember>` where
  `SurfaceMember = Field{name, ty} | Method{name, sig}` (or equivalent); `parse_defsurface` parses `(name [self …] ->
  ret)` (reuse the argspec/method parser `parse_defprotocol_form` uses); satisfaction extended: a Method member is
  satisfied by a `defn :T/name` whose type matches. Own-probe: a `defsurface` with a method member parses + a record
  with a matching `defn :T/m` satisfies it (RED today — method members don't parse).
- **293.4b — the generated dispatcher.** `:Surface/method s` routes by `s`'s runtime type to `:T/method` (LIFT the
  arc-232 `extract-classifier`+`apply` protocol dispatch under the surface). Own-probe: `(:geo::Shape/area circle)`
  dispatches to `:geo::Circle/area`.
- **293.4c — `extend-type` as the foreign-accessor adapter.** Add `:T/accessor` impls (field-style `(name [self] ->
  ret body)`) to a foreign type (the holon `Vector` monkeypatch); collisions are `DuplicateDefine`. Own-probe: the
  monkeypatch teaches `:wat::holon::Vector` to satisfy `:geo::Shape`.
- **293.4d — ANNIHILATE `defprotocol`.** Migrate `:wat::spawn::Locus` → `defsurface`/`definterface` (fix-wat the `.wat`;
  the `extend-type ThreadOpts/ProcessOpts` impls become method-accessor adapters); rip the Rust `defprotocol` machinery
  (parse/dispatch/preregister across the 6 files); retirement-table the head. **The acceptance demo
  (`probe_arc293_acceptance_demo`, un-ignore) goes GREEN — the arc's gate.**
- **293.1-owed — the `src/aggregate/` home.** Lift construction + surface machinery out of `runtime.rs`/`types.rs`/
  `check.rs` (the *"reduce src/*.rs"* directive). Can interleave or follow.
- **293.5 — close + amend** (workspace SET-diff ∅; ward `src/aggregate/`; amend 291's `/from-map`).

## Out of scope (rejected, named)
- `Seqable` itself — that's 118 (293.4 is the *mechanism*; Seqable is its first consumer).
- Polymorphic surfaces `<T>` — v1 is monomorphic (`surface.rs:9` already notes "no `<T>` shipped here"); deferred unless
  the demo needs it (it doesn't).
- `Value` repr unification — the variant-level wire law stays (293 DESIGN out-of-scope).

## Expectations (the gate)
| # | what | command | expected |
|---|---|---|---|
| 1 | acceptance demo GREEN | `cargo nextest run --release -E 'test(shape_demo_fields_and_methods_and_the_monkeypatch)'` (un-ignore) | PASS at 293.4d |
| 2 | `defprotocol` is GONE | `grep -rn 'defprotocol' wat/ src/ --include=*.wat` (non-comment) | only the retirement-table teaching error |
| 3 | whole workspace green | `cargo nextest run --release` | floor 0 |

**Pairs:** `293/DESIGN.md` (§ 293.4 + the HOLDER×SURFACE model 183+) · `probe_arc293_acceptance_demo.rs` (the gate) ·
`src/types/surface.rs` (field-only today) · `runtime.rs:670` (the `defprotocol` machinery to lift+kill) ·
`118/DESIGN-118.2` (the consumer: `Seqable` → the HOF family).
