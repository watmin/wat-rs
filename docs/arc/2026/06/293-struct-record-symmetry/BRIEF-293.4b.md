# BRIEF — 293.4b: the generated surface dispatcher (`:Surface/method` routes by runtime type)

**The work, in one paragraph.** After 293.4a a `defsurface` carries method members and a type satisfies them via a
`defn :T/<name>`. But you cannot CALL the method polymorphically: `(:t::Shape/area s)` is an UnresolvedReference. Make a
**`:Surface/method` call head** dispatch on the receiver's runtime type to that type's `:T/<method>` defn — exactly the
arc-232 protocol dispatch, LIFTED onto surfaces, with ONE change: surfaces have no `extend-type`, so the dispatcher
routes to the plain **`defn :<T>/<method>`** (a `sym.functions` entry), NOT an `extend:<P>:<T>` impl. This is a
**3-layer mirror** of the protocol path (resolve → check → runtime), since the RED surfaces in all three.

## The one contract decision (pinned)
A head `:S/m` where `S` is a registered `TypeDef::Surface` and `m` is a **method member** of `S` is a
**surface-method call**: it requires ≥1 arg (the receiver), dispatches on the receiver's concrete type `T` to the
function `:<T>/<m>`, and has the type of `m`'s sig (args from `m`'s `ArgSpec`, return `m`'s ret). Disambiguation from
record/field accessors and protocol methods: a field accessor `:<Rec>/<field>` is a `sym.functions` entry under a
RECORD fqdn; a protocol method `:<P>/<m>` has `P` registered as `protocol_def` in `runtime_def_values`; a surface
method `:<S>/<m>` has `S` registered as `TypeDef::Surface` in the type registry with `m` among its method members.
Check the surface registry in the SAME spot the protocol check happens, after the protocol check (or before — they are
disjoint: a name is a protocol OR a surface, never both).

## Read in order (the rooms — grounded 2026-06-28)
1. **The RESOLVE layer (the FIRST wall — the RED is a `Resolve(UnresolvedReferences)` error).** Grep how `:P/method`
   protocol heads are accepted by the resolver so they don't become UnresolvedReferences:
   `grep -rn 'protocol\|/method\|UnresolvedReference\|call head' src/resolve*.rs src/resolve/` — find where a `:P/method`
   head is recognized as resolvable (the stem-is-a-protocol check). Add the sibling: a `:S/m` head where `S` is a
   surface with method member `m` resolves. (This is why the probe RED'd at resolve, before check.)
2. **The CHECK layer — `src/check.rs:5789`** ("Arc 232 Stone 232.3 — protocol-method call-site check"). It splits the
   head at `/`, checks `env.get_protocol_methods(protocol_fqdn)`, finds the matching sig, types the call. MIRROR it for
   surfaces: if the stem is a `TypeDef::Surface` (via the type env) and `m` is a method member, type the call as `m`'s
   sig — receiver (arg 0) must satisfy `S` (the 293.4a `assignable`/`struct_satisfies_surface` path), remaining args
   assignable to `m`'s `ArgSpec.fixed_params[1..]`, result = `m`'s ret. The surface's method members live on
   `SurfaceDef.members` (the `SurfaceMember::Method { name, args, ret, .. }` from 293.4a).
3. **The RUNTIME layer — `src/runtime.rs:5101`** ("Arc 232 Stone 232.3 — protocol-method dispatch"). It splits at `/`,
   checks `runtime_def_values[stem]` is a `protocol_def`, evals the receiver, reads its `concrete_type_fqdn`
   (Record→`:class_fqdn`, Struct→`type_name`, …), looks up `extend:<P>:<T>`. MIRROR it for surfaces: if the stem is a
   `TypeDef::Surface` with method member `m` → eval receiver → `concrete_type_fqdn` (reuse the EXACT same receiver-type
   extraction — Record/holon-Record/Struct/RustOpaque) → call **`:<T>/<m>`** as a normal function (`sym.functions`
   lookup + apply with the full arg list, receiver included). Surfaces are reachable at runtime — `TypeDef::Surface`
   already appears in `sym` (runtime.rs:1475, 9667, 9709, 13055).
4. **`src/types.rs` / `src/types/surface.rs`** — `SurfaceDef.members` + `SurfaceMember::Method` (built in 293.4a) is
   your source of "is `m` a method member of `S`, and what is its sig." A small accessor like
   `SurfaceDef::method_member(name) -> Option<&SurfaceMember>` may help all three layers; add it if it cleans the call sites.

## Implementation sketch (the strike path)
- Resolve: a `:S/m` head where `S ∈ surfaces` and `m ∈ S.method_members` resolves (sibling to the protocol head check).
- Check (`check.rs:5789` neighborhood): a surface-method call arm → type as `m`'s sig; receiver satisfies `S`.
- Runtime (`runtime.rs:5101` neighborhood): a surface-method dispatch arm → eval receiver → concrete `T` → call `:<T>/<m>`.
- Reuse the protocol path's receiver-type extraction + the `/`-split + the arity check verbatim; the ONLY semantic
  change is the lookup target (`:<T>/<m>` defn vs `extend:<P>:<T>` impl).

## Blast radius (bounded)
`src/resolve*` (1 head-recognition site), `src/check.rs` (1 call-site arm near 5789), `src/runtime.rs` (1 dispatch arm
near 5101), maybe a small helper on `SurfaceDef`. NO change to 293.4a's parse/satisfy; NO `extend-type` (that's 293.4c);
NO `defprotocol` touch (that's 293.4d). The protocol path stays exactly as is — surfaces get a PARALLEL arm.

## STOP triggers (halt + surface; do NOT improvise)
- **STOP-1 (surface not reachable in a dispatch layer):** if, in the resolve OR runtime layer, you cannot reach the
  surface registry (the `TypeDef::Surface` defs) to ask "is `S` a surface with method `m`?" — STOP and report the seam
  (do NOT hardcode or guess; the protocol path proves the registry is reachable — find the surface equivalent).
- **STOP-2 (receiver type extraction gap):** if a satisfier's concrete type can't be read by the protocol path's
  existing extraction (Record/holon-Record/Struct/RustOpaque) — STOP and name the value shape; do not add a guess arm.
- **STOP-3 (ambiguity with field accessors):** if `:<T>/<field>` (a record field accessor) and `:<S>/<m>` (a surface
  method) can collide on the same head string and you can't disambiguate by registry kind — STOP and surface it.

## The gate (the disconfirming probe, committed RED)
`tests/types/probe_arc293_4b_surface_dispatch.rs` + `.wat` — a `:t::Shape` surface with method member `area`; two
records `:t::Circle`/`:t::Square` each with their own `:T/area` defn; `:t::describe [s <- :t::Shape]` calling
`(:t::Shape/area s)`. The `.rs` evals `(:t::circle-area)` → π·2²≈12.566 and `(:t::square-area)` → 9.0, asserting the
dispatcher routes each by runtime type. **Verified RED at HEAD:** `UnresolvedReference ":t::Shape/area"`. UN-IGNORE it;
it goes GREEN at 293.4b. Do NOT touch `probe_arc293_acceptance_demo` (293.4d's gate — needs `extend-type` too, stays `#[ignore]`'d).

## You are a LEAF
Anchor cwd `/home/watmin/work/holon/wat-rs`; `pwd` first; reject any `.claude/worktrees/` path. Do NOT spawn subagents.
Do NOT commit. Build incrementally (`cargo build --release -p wat`). Read every diff end-to-end. Self-verify the
EXPECTATIONS scorecard. If a STOP fires or the work exceeds this brief, halt and report.
