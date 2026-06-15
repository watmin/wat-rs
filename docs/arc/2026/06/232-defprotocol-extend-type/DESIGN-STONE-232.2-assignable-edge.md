# Arc 232 Stone 232.2 — the satisfaction edge: a `:P`-typed param accepts an extender

> 232.1 registered the forms. 232.2 makes the protocol a usable BOUND: `extend-type :T :P`
> registers a subtype-parent edge `T → P`, so a `:P`-typed parameter accepts any `T` that extends
> `P`. NO change to `assignable` or `is_subtype` — they already consult the edge. NO method dispatch
> (232.3). Grounded against HEAD `203ece72`. Single-receiver locked.

## The grounded finding (why this stone is tiny)

The RED probe at HEAD-after-232.1 fails with `TypeMismatch { expected: ":t::Greeter", got:
":t::Robot" }` — **not** "unknown type :t::Greeter". So:
- The annotation `:P` already resolves (no TypeDef registration needed for the annotation).
- The ONLY gap is the satisfaction edge: `is_subtype(:t::Robot, :t::Greeter)` is false.

And `register_subtype` (types.rs:446-449) **explicitly allows edges from unregistered names** —
*"the hierarchy is orthogonal to the TypeDef registry... mirrors Clojure's hierarchy being
independent of what the tags ARE."* So no `TypeDef::Protocol` is needed. `assignable` (check.rs:13566)
already returns true when `is_subtype(ap, ep)` holds, and `is_subtype` (types.rs:3076) walks the
multi-parent `subtype_edges` graph. The edge is the whole stone.

## Contract decision (the one change)

In `splice_type_decls` (types.rs:1489 — the TypeEnv-population pass over `do`/`let` children), add
an `:wat::core::extend-type` arm that:
1. extracts `(type_name, protocol_name)` (via `parse_extend_type_form`, or a lightweight head read),
2. calls `env.register_subtype(type_name, protocol_name, span)`,
3. **keeps the form** in `new_children` (does NOT strip it — unlike a TypeDef decl) so the 232.1
   CheckEnv (`collect_splice_defs_ctx`) and runtime (`register_runtime_defs`) passes still see it.

That's it. `assignable`/`is_subtype` unchanged; multi-parent graph already supports a record having
both `:wat::Record` (from recordtype) and `:P` (from extend-type) as parents.

## Rooms (read in order)

1. `src/types.rs:1489-1544` `splice_type_decls` — the do/let child loops; add the extend-type arm.
2. `src/types.rs:450-467` `register_subtype` — the edge registrar (allows unregistered parents;
   cycle-rejecting). The recordtype precedent calling it is types.rs:416.
3. `src/runtime.rs` `parse_extend_type_form` (232.1; pub(crate)) — reuse to get the names.
4. `src/check.rs:13566` `assignable` + `src/types.rs:3076` `is_subtype` — READ ONLY (confirm no
   change needed; the edge flows through).

## Gate (RED at HEAD → GREEN after)

- **Positive (the gate):** `tests/probe_arc232_2_protocol_assignable.rs` (written, RED-verified) — a
  `[g <- :t::Greeter]` fn called with a `:t::Robot` (which extend-types Greeter) type-checks →
  returns 99. RED at HEAD (`TypeMismatch Robot vs Greeter`). GREEN after the edge.
- **Negative (anti-over-reach):** add to the probe a should-FAIL case — a record that does NOT
  extend the protocol, passed where `:P` is required, must still be a `TypeMismatch` (use a
  `startup_from_source(...).is_err()` assertion in a second `#[test]`, or a `wat::check` call). This
  proves the edge is precise (only registered extenders satisfy), not a blanket accept.
- lib 916/36 + nursery 895/4 (zero new) + workspace compiles.

## Scope / out (rejected here)

- **Method dispatch** (calling `(greet r 3)`) → 232.3. Still not wired.
- `TypeDef::Protocol` — NOT added (orthogonal-hierarchy rule makes it unnecessary; if a later stone
  needs `:P` reflectable as a TypeDef, that's its own decision).

## STOP triggers (reject — surface; do not improvise)

- `register_subtype` rejects the edge (cycle) for a legitimate extend → STOP (report; a protocol
  extension should never cycle).
- The edge requires changing `assignable`/`is_subtype` → STOP (the DESIGN says it does not; if the
  probe won't go green via the edge alone, surface why).
- You're tempted to add `TypeDef::Protocol` or touch method dispatch → STOP (out of scope).
