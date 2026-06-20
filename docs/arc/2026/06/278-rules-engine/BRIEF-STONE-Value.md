# BRIEF — Stone Value: `:wat::core::Value` universal subtype-top

**Executor:** one **sonnet** Shadowdancer. **No sub-agents. No `git`. No worktrees.** Do not run
`./target/release/wat` (orchestrator-only); `cargo test` is yours.

## The work (one paragraph)
`:wat::core::Value` is the universal subtype-top: every type `<: Value` (UP is free), but a `Value` is NOT
assignable where a specific type is wanted (DOWN is checked). The entire change is **one root rule** in
`is_subtype`. The directional acceptance `assignable` (`src/check.rs:13962`) already does everything else — it
consults `is_subtype` first, then falls to `unify` — so adding `is_subtype(_, Value) → true` makes UP free
while DOWN stays rejected (for any specific `sup ≠ Value`, the rule is skipped, the parents-walk finds no edge,
`unify` fails). Then **un-ignore the three disconfirm asserts** in the already-committed probe; the whole probe
(6 tests) must go green.

## Read in order (the rooms)
1. `src/types.rs:3142` — `pub fn is_subtype(sub, sup, env)`. The insertion point is **right after the reflexive
   `if sub == sup { return true }` block, before `let mut visited = …`**. This is the ONLY production edit.
2. `src/check.rs:13962` — `pub(crate) fn assignable(...)` — **READ ONLY, do not edit.** Confirm for yourself
   that it calls `is_subtype(ap, ep, types)` before falling to `unify`. This is why the root rule alone
   delivers up-free / down-checked. (Build-step #1, already verified — you are re-confirming, not changing.)
3. `tests/probe_arc278_value_universal_top.rs` — the committed RED probe. The contract. The three
   `#[ignore = "RED until STONE-Value lands …"]` attributes come OFF (the three disconfirm asserts:
   `up_i64_is_subtype_of_value`, `up_string_is_subtype_of_value`, `widen_record_value_field_accepts_i64_and_string`).
   The three discipline asserts (`down_*`, `narrow_*`) are already live — leave them.

## Implementation sketch (fill it; do not invent the shape)
In `src/types.rs`, inside `is_subtype`, immediately after the reflexive check:

```rust
    if sub == sup {
        return true; // reflexive
    }
    // Arc 278 Stone-Value — :wat::core::Value is the universal subtype-top: every type
    // <: Value. UP is free (this rule); DOWN stays checked — for any specific `sup ≠ Value`
    // this rule is skipped, the parents-walk finds no edge, and `assignable`'s fall-through
    // `unify(Value, T)` fails. No registration (Value is recognized as an opaque Path
    // already; a TypeDef::Struct would wrongly synthesize a constructor — Value is
    // un-constructible). assignable (check.rs:13962) does the directional rest.
    if sup == ":wat::core::Value" {
        return true;
    }
    let mut visited = …  // (unchanged)
```

Then in `tests/probe_arc278_value_universal_top.rs`, delete the three `#[ignore = …]` lines (only those three).

## Blast radius (bounded)
- `src/types.rs` — the one root rule (≈6 lines incl. comment). **Nothing else in this file.**
- `tests/probe_arc278_value_universal_top.rs` — remove 3 `#[ignore]` attributes. No assertion changes.
- **NO** `src/check.rs` edit. **NO** `wat/rete.wat` edit. **NO** registration in `register_builtin_types`. **NO**
  new types, traits, or functions.

## STOP triggers (halt and surface — do not improvise)
1. **STOP if `assignable` (`check.rs:13962`) does NOT call `is_subtype` before `unify`** for Path/Path
   acceptance. The down-rejection rests on this; if the routing differs from the brief, surface it — do not
   add the rule blind.
2. **STOP if any `down_*` or `narrow_*` discipline assert turns RED** after your change. That means UP leaked
   into DOWN (`Value` became a loose any) — the stone has failed its core invariant. Do not "adjust" the test;
   surface it.
3. **STOP if making the probe green requires touching `check.rs`, `wat/rete.wat`, or registering a type.** The
   grounding says the root rule alone suffices (the HEAD error is a constructor-arg `unify` failure, not an
   unknown-type error). If that proves false, the scope assumption is wrong — surface it, do not expand scope.
4. **STOP if any of the four floors regress** beyond the documented baseline (see EXPECTATIONS). A new failure
   is a real regression — surface it.

## Prior comparable (copy the shape)
- The probe itself, `tests/probe_arc278_value_universal_top.rs` (committed `58cd8c91`) — the worked reference.
- `DESIGN-STONE-Value-universal-top.md` (committed, this arc) — the full contract + the grounded reversals
  (no registration / no check.rs edit / re-type→P12).
- `is_subtype` reflexive + parents-walk shape is already in `src/types.rs:3142` — you are adding one branch to
  an existing function, not authoring a new one.
