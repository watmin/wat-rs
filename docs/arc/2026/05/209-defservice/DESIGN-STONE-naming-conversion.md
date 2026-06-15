# Arc 209 — naming-conversion tooling: PascalCase ⇄ kebab-case (both directions)

> defservice derives kebab fn names from PascalCase op keywords by bare `to-lowercase` — so
> `:GetObject` → `getobject` (wrong), not `get-object`. This stone builds the full bidirectional
> converter and threads the forward direction into defservice. The two directions land on **opposite
> sides of the floor**, exactly as the macro-fence rubric (OP-PLACEMENT.md) predicts. Grounded
> against HEAD. Spec: `docs/PASCAL-KEBAB-CONVERSION.md` (the algorithm + bijection contract).

## The three pieces (each placed by the OP-PLACEMENT rubric)

1. **`:wat::core::string::pascal->kebab`** — **Rust intrinsic**, on `is_pure_total`. The defservice
   macro calls it at expand time → it must be on the floor (the fence can't reach a wat helper).
   Self-contained Rust (boundary-before-each-uppercase, downcase, join with `-`). Mirrors
   `eval_string_to_lowercase` (string_ops.rs) for wiring: eval fn + check scheme (`String->String`)
   + runtime dispatch + `is_pure_total` entry.
2. **`:wat::core::string::to-uppercase`** — **Rust primitive** (the floor op the inverse needs;
   char-case mapping can't be composed in wat). The sibling of `to-lowercase`, completing the pair.
   Same four-site wiring as `to-lowercase`, EXCEPT **not** on `is_pure_total` (no macro calls it).
3. **`:wat::core::string::kebab->pascal`** — **wat helper** (no macro needs it; it composes). Lives
   in a new `wat/string.wat` (added to the `src/stdlib.rs` embed list after `core.wat`). Algorithm
   (per the spec): `split` on `-` → for each segment, `to-uppercase` the first char + keep the rest →
   `concat`. The self-hosting default: wat does its own kebab->pascal, dropping to the floor only for
   the char-case primitive.

## The defservice thread

In `wat/service.wat`, the op-name derivation currently does
`op-lower (:wat::core::string::to-lowercase op-str)` (constructors + methods, ~lines 387/443).
Replace `to-lowercase` with `pascal->kebab` so `:GetObject` → `get-object` (method) and
`get-object-request` (constructor). Records stay PascalCase (`<Op>Request` = concat, no conversion).

## The bijection contract (from the spec)

On the disciplined subset — **one uppercase per word, no consecutive-capital acronyms** (write
`GetUrl`, not `GetURL`) — the two functions are total mutual inverses:
`kebab->pascal(pascal->kebab(x)) == x` and vice versa. The probe asserts the round-trip. (Acronym
discipline is a naming convention, not enforced here; the spec records why no heuristic round-trips
raw acronyms.)

## Rooms (read in order)
1. `src/string_ops.rs` `eval_string_to_lowercase` — the intrinsic model to mirror (×2: pascal->kebab,
   to-uppercase).
2. `src/check.rs` (the `to-lowercase` scheme registration) + `src/runtime.rs` (the `to-lowercase`
   dispatch arm) + `src/macros/eval.rs` `is_pure_total` (add `pascal->kebab` ONLY).
3. `src/stdlib.rs` — the embedded-wat list; add `wat/string.wat` after `core.wat`.
4. `wat/service.wat` ~387/443 — replace `to-lowercase` with `pascal->kebab` in op-lower derivation.
5. `docs/PASCAL-KEBAB-CONVERSION.md` — the algorithms (forward in wat = the Rust logic to port;
   inverse in wat = the kebab->pascal helper verbatim).

## Gate (RED at HEAD → GREEN)
RED probe `tests/probe_arc209_naming_conversion.rs`:
- `(pascal->kebab "GetObject")` → `"get-object"`; `(pascal->kebab "Get")` → `"get"`.
- `(to-uppercase "abc")` → `"ABC"`.
- `(kebab->pascal "get-object")` → `"GetObject"`.
- round-trip `(kebab->pascal (pascal->kebab "GetObject"))` → `"GetObject"`.
- defservice with a multi-word op `:GetObject` → the generated `<svc>/get-object` method +
  `<svc>/get-object-request` constructor resolve and work.
RED at HEAD (all three ops unresolved; defservice gives `getobject`). + lib (zero new) + nursery
(zero new) + workspace compiles.

## Scope / out
- Acronym-run enforcement (reject `GetURL` at defservice expand time) → not here; the convention is
  documented, enforcement is a separate concern if a service ever needs it.
- `to-uppercase` on `is_pure_total` → NOT added (no macro needs it; add only if a future macro does).

## STOP triggers
1. `pascal->kebab` can't be added to `is_pure_total` (the macro can't reach it) → STOP (defservice
   needs it at expand time; that's the whole point).
2. `wat/string.wat`'s load order means `kebab->pascal` can't call the string primitives → STOP
   (the Rust intrinsics should be available before any `.wat` loads; surface if not).
3. Threading `pascal->kebab` into service.wat breaks a single-word op (`Get` → `get` must still
   hold) → STOP.
