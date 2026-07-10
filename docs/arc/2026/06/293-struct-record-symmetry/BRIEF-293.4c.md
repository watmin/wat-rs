# BRIEF — 293.4c: `extend-type` as the foreign-accessor adapter (the monkeypatch)

**The work, in one paragraph.** `extend-type` today (arc-232) is a protocol adapter: `(:extend-type :T :P (m …))`
registers an `ExtendDef` under `extend:<P>:<T>` and the protocol dispatcher looks it up. 293.4c makes `extend-type`
ALSO the **surface** foreign-accessor adapter: when its 2nd arg is a `defsurface`, each impl method registers as a
**`:<T>/<method>` callable** — the SAME key the 293.4b dispatcher and 293.4a satisfaction already use — so a foreign
type you don't own (a built-in like `:wat::core::String`, or the holon `Vector`) is taught to SATISFY the surface and
DISPATCH through it, with NO `defn` you could write. Collisions (extending a `:<T>/<method>` that already exists) are
`DuplicateDefine`. This solves the Expression Problem structurally (DESIGN R1 — the monkeypatch *"insane in Ruby, sane
by types"*).

## Grounded current state (the RED, decoded)
The probe `(:t::tag-of "hello")` **type-checks** (startup passes) but the **runtime dispatcher rejects**:
`MalformedForm ":t::Tagged/tag" — "surface-method receiver must be a type that satisfies :t::Tagged; got a
wat::core::String value"`. So the check side half-accepts (likely via the arc-232 `register_extend` path firing for any
extend-type, check.rs:8886), but (a) `extend-type` registers no `:<T>/<method>` callable, (b) satisfaction is
Aggregate-only, (c) the dispatcher's receiver guard reads Record/Struct/RustOpaque only. Make all three consistent:
a foreign type satisfies a surface **iff every method member resolves to a `:<T>/<method>`** (defn OR extend-provided),
and the dispatcher reads ANY receiver via `type_name()`.

## The one contract decision (pinned)
`extend-type :T :S` where `S` is a `TypeDef::Surface`: for each impl `(m [self …] -> :ret body)`, register a callable
under `:<T>/<m>` (a `sym.functions` entry + the check-side scheme), exactly as if `defn :<T>/<m>` had been written.
Collision with an existing `:<T>/<m>` (real defn or prior extend) = `DuplicateDefine` (compile error). The protocol
path (`extend-type :T :P`) is UNCHANGED — branch on whether the 2nd arg names a surface or a protocol.

## Read in order (the rooms — grounded 2026-06-28)
1. **`src/runtime.rs:6090` (`parse_extend_type_form`)** — parses `extend-type` into `ExtendDef { type_name,
   protocol_name, impl_clauses }`. It does NOT know surface-vs-protocol. Either here or at the registration sites, branch.
2. **`src/runtime.rs:675` + `:1907`** (the two `extend-type` registration arms — top-level + stdlib) — today store
   `extend:<P>:<T>` in `runtime_def_values`. ADD: if the 2nd arg names a `TypeDef::Surface`, instead register each
   impl clause as a `:<T>/<m>` function in `sym.functions` (mirror how `defn :<T>/<m>` accessor fns are registered —
   grep how a record accessor or a `defn` lands in `sym.functions`). Collision check → `DuplicateDefine`.
3. **The check side** — `src/check.rs:8886` (the `register_extend` pass) + the surface satisfaction at
   `src/check.rs:14380` (the 293.4a path, gated on `TypeDef::Aggregate`). Make satisfaction work for a NON-aggregate
   candidate: a foreign type `T` satisfies surface `S` iff for every member, `resolve_method(":<T>/<m>")` succeeds (the
   293.4a resolver) — drop the hard Aggregate gate for the method-member case (a field member still needs an aggregate's
   fields; a pure-method surface can be satisfied by any type with the `:<T>/<m>` callables). Also register the
   extend-provided `:<T>/<m>` into the check env schemes so `resolve_method` finds them.
4. **`src/runtime.rs` ~5140 (the 293.4b dispatcher receiver guard — the RED source)** — today extracts
   `concrete_type_fqdn` from Record/holon-Record/Struct/RustOpaque and re-checks satisfaction. GENERALIZE: derive the
   FQDN from `receiver.type_name()` (colon-prefixed) — it covers EVERY Value variant (`Value::String → "wat::core::String"`,
   etc.). Then the `:<T>/<m>` lookup (293.4b) finds the extend-registered callable. The satisfaction re-check at dispatch
   must accept the extend-provided methods (same resolver as the check side).

## Implementation sketch
- Registration: 2nd-arg-is-surface → register each impl as `:<T>/<m>` (sym.functions + check scheme); collision → DuplicateDefine.
- Satisfaction (check.rs:14380): a method-only surface is satisfied by any `T` whose `:<T>/<m>` all resolve — not just Aggregates.
- Dispatch (runtime.rs ~5140): `concrete_type_fqdn = format!(":{}", receiver.type_name())`; then the existing 293.4b `:<T>/<m>` lookup.

## Blast radius (bounded)
`src/runtime.rs` (the 2 extend-type registration arms + the dispatcher receiver extraction), `src/check.rs` (the
register-extend pass + the satisfaction non-aggregate path). NO change to 293.4a/b's parse/dispatch shape; NO
`defprotocol` touch (293.4d). The protocol `extend-type` path stays EXACTLY as is — surfaces get a parallel branch.

## STOP triggers (halt + surface; do NOT improvise)
- **STOP-1 (where `:<T>/<m>` callables register):** if you can't register a foreign `:<T>/<m>` into `sym.functions` the
  way a `defn` does (e.g. the registration pass can't reach sym at that point) — STOP and report the seam.
- **STOP-2 (the `type_name()` FQDN ≠ the extend key):** if a receiver's `type_name()` does not match the `:<T>` the
  user wrote in `extend-type` (e.g. the holon Vector's `Value::Vec` reports `wat::core::Vector` but a user wrote
  `:wat::holon::Vector`) — STOP and surface the mapping question; do NOT guess a normalization. (The probe uses
  `:wat::core::String`, whose `type_name` is unambiguous, to dodge this — but the dispatcher generalization must not
  silently mis-map other types.)
- **STOP-3 (satisfaction goes always-true):** if dropping the Aggregate gate makes EVERY type satisfy EVERY method
  surface (because `resolve_method` is too loose) — STOP. The negative arm (below) must still reject.

## The gate (the disconfirming probe, committed RED)
`tests/types/probe_arc293_4c_extend_type_adapter.rs` + `.wat` — a `:t::Tagged` surface (method `tag`); `extend-type`
teaches `:wat::core::String` to be `:t::Tagged` (constant body 42); `(:t::tag-of "hello")` must satisfy + dispatch.
Verified RED at HEAD (the dispatcher rejects the String receiver). UN-IGNORE it; GREEN at 293.4c.
**ADD a collision arm** (`_dup.wat.bad`): two `extend-type` of the same `:<T>/<m>` (or an `extend-type` colliding with a
real `defn :<T>/<m>`) → `DuplicateDefine` at startup. **ADD a negative arm**: a foreign type NOT extended (no
`:<T>/tag`) passed where `:t::Tagged` is required → rejected. Do NOT touch `probe_arc293_acceptance_demo` (293.4d's gate).

## You are a LEAF
Anchor cwd `/home/watmin/work/holon/wat-rs`; `pwd` first; reject any `.claude/worktrees/` path. Do NOT spawn subagents.
Do NOT commit. Build incrementally. Read every diff end-to-end. Self-verify the EXPECTATIONS scorecard. If a STOP fires
or the work exceeds the brief, halt and report.
