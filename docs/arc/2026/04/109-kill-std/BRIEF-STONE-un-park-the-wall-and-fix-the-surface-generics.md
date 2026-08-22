# BRIEF — un-park the type-reference wall, and fix what it finds

Two halves, one stone, because **neither is verifiable without the other**: the wall cannot go green
until the `defsurface` bug is fixed, and the bug has no observable symptom without the wall.
Mid-sweep brokenness is expected here; on-disk-committed brokenness is not.

DESIGN: `109/DESIGN-STONE-a-type-reference-must-resolve.md` (RULED D1-A · D2-A · D3-B).
The door it needed now exists: `255/…SCORE-STONE-the-type-registry-holds-the-builtin-types.md`.

## PART A — un-park

**636 lines of correct work live on branch `arc109-type-refs-parked` (`ce8f144a9`).** It shipped
green on its own rows and was parked for ONE reason: it carried a hand-written list of builtin type
names, `known_builtin_leaf_types()` in `src/resolve/type_refs.rs`, because the DESIGN asserted a door
that did not exist at the time.

⚠ **Do NOT merge the branch.** Cherry-pick paths:

```
git checkout arc109-type-refs-parked -- src/resolve/ src/freeze.rs src/freeze/env.rs tests/resolve/
```

Then, and this is the entire point of the rework:

1. **DELETE `known_builtin_leaf_types()`** and its call site.
2. **Re-point the membership query at the registry.** `TypeEnv::contains` now answers for primitives,
   containers and opaques — 23 of the 24 names that list carried, verified by set comparison, zero
   surplus. The `:rust::*` names route through `rust_deps::registry().has_type()` exactly as before;
   leave that path alone.
3. `:wat::core::Never` is the ONE name the registry deliberately refuses. It cannot reach your sweep —
   `check.rs:10662` builds it as an INFERRED expression type (`CheckResult::ok`), and you walk
   DECLARED positions. **If it does reach you, that is STOP-3.**

Everything else on that branch survives unchanged: the `freeze.rs` precedence fix gated on
`ReferenceKind::Type`, `UnresolvedReference.context` as `String`, the registry sweep, all five
fixtures.

## PART B — the bug the wall finds, and its fix

**This is a real, pre-existing defect. You are not fixing your own mess.** It has been invisible
because nothing ever validated that a type name resolves.

`src/types.rs`, `register_types_impl`'s surface-derivation arm:

```rust
if let SurfaceMember::Method { name: op_name, args, ret, .. } = member {   // ← `..` DROPS type_params
    d.push(TypeDef::Alias(AliasDef {
        name: format!("{}::{}/Request", surf.name, op_name),
        type_params: surf.type_params.clone(),                             // ← the SURFACE's params
        expr: request_ty.clone(),                                          // ← mentions the METHOD's
```

`SurfaceMember::Method` carries its own `type_params: Vec<String>` (`src/types.rs:399`). The
destructure discards them and substitutes the surface's, so **a method-level generic becomes a free
variable in the minted alias.** Same shape at `runtime.rs:2018` for the synthesized `::Op`/`::Reply`
variant-constructor `Function`s.

**Measured, with the wall built, on a ONE-LINE innocent program:**

```
:D :I :O :W    "alias body of :wat::spawn::Locus::spawn-runner/Response"
:S :R :Sh :Lu  "alias body of :wat::spawn::Locus::launch/Request|Response"
```

`wat/spawn.wat`'s `spawn-runner<D,I,O,W>` and `launch<S,R,Sh,Lu>` are the methods; `Locus` is the
surface.

**The hypothesis to test, not to assume:** the alias's `type_params` should be the UNION of the
surface's and the method's own. The parked rider reached the same conclusion independently for its
sweep's bound set (*"a method's bound set is the union of both, confirmed necessary by real corpus
usage"*). **Verify it against the disk before applying it to both sites**, and if the union is wrong
for either site, STOP and report what the correct scope is.

## STOP triggers — ship nothing further and report

- **STOP-1 — do NOT silence the wall to make it green.** If you find yourself excluding
  `*/Request`/`*/Response` aliases from the sweep, or adding the free variables to a bound set they
  do not belong to, you are hiding the defect the stone exists to surface. The nine phantoms must
  disappear because the ALIAS became correct, not because the wall stopped looking.
- **STOP-2 — if the union hypothesis is wrong** for either site, STOP. Report what the scope should
  be. Do not pick something that makes the count go to zero.
- **STOP-3 — if `:wat::core::Never` appears in your unresolved output**, my analysis of where it
  lives is wrong. STOP and report where the sweep met it.
- **STOP-4 — if the sweep finds violations that are NEITHER the nine above NOR builtin-name gaps**,
  STOP and report the full list before touching any of them. A genuine unresolvable type in `wat/`
  outranks this stone.

## Acceptance

| # | what | expected |
|---|---|---|
| 1★★ | the nine phantoms are gone | `--check` on a one-line innocent program: **EXIT 0**, no unresolved output |
| 2★★ | ⛔ **and the wall is NOT blind** | a fixture naming `:user::NoSuchType` in an UNCALLED declaration still **EXIT 1** |
| 3★ | the wall names the TYPE, not the caller | the phantom-with-a-caller fixture: no `TypeMismatch` blaming parameter #1 |
| 4 | all five parked fixtures | pass |
| 5 | `known_builtin_leaf_types` is gone | `grep -rn known_builtin_leaf_types src/` → no hits |
| 6 | the alias fix is real | a probe declaring a parametric surface METHOD checks clean, and its minted alias carries the method's params |
| 7 | clippy | 0 under `-D warnings` |

**Row 2 is the row that decides the stone.** Row 1 alone is satisfied by a wall that reports nothing —
which is exactly what "delete the sweep" would achieve. Rows 1 and 2 are only meaningful together:
the wall must be silent where the corpus is correct and loud where it is not.

## Boundaries

- `src/resolve/`, `src/freeze.rs`, `src/types.rs` (the alias arm), `src/runtime.rs` (the `::Op`/`::Reply`
  arm), and `tests/`.
- Do NOT touch `is_resolvable_call_head`'s reserved-prefix exemption for CALL heads. Late but honest,
  and a separate question nobody has asked.
- Do NOT touch `src/value/symbol_table.rs`. It is the narrow waist; it stayed empty last stone and
  should stay empty.
- Do NOT run `scripts/floor.sh` or a full `cargo nextest` — the orchestrator measures centrally.
  Scoped: `binary_id(wat::types)`, `binary_id(wat::resolve)`, `binary_id(wat::services)`.
  ⚠ A scoped run is not the floor; one was 133/133 green today while the floor was red in `wat::lint`.
- Do NOT commit, push, stash, revert or amend.

⚠ **`no_loose_string_assert` has a known FALSE-POSITIVE class**: it flags
`assert!(registry.contains("literal"))` because a text lint cannot tell registry membership from
`String::contains`. If it fires on you, do **not** add a `rune:lint(loose-assert)` — the site is not
loose and the marker would lie. Ask through the door instead
(`sym.registrations(name).contains(RegistryKind::Type)` — an enum argument).

Prefix long commands with `systemd-run --user --scope -q -p MemoryMax=16G -p MemorySwapMax=0 timeout 1800`.
Read exit codes DIRECTLY — never through a pipe, never after a trailing `; echo`.

## Your report

Every acceptance row with verbatim output, rows 1 and 2 especially — and state them together, since
either alone is meaningless. What the union hypothesis turned out to be at each of the two sites, with
the evidence. The complete unresolved list at each stage: after Part A, and after Part B. What
surprised you. Anything you inspected and left alone.
