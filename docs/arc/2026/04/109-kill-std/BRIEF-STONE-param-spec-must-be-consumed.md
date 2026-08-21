# BRIEF — arc 109: a type declaration must CONSUME its param-spec

A type declaration that names a type parameter no member type ever mentions is illegal. This is a
**wall**, and the corpus already obeys it — 38 parametric type declarations across `wat/`,
`wat-scripts/` and `tests/`, zero violations. It is a ratchet on existing practice.

```clojure
(defrecord ns/R :- [T] [])                                  ILLEGAL — T consumed by nothing
(defenum   ns/E :- [I O] … [First :- [I] …])                ILLEGAL — O consumed by nothing
(defrecord ns/R :- [T] [x :- T])                            legal
(defrecord ns/R :- [T] [x :- (wat.type/Vector :- [T])])     legal — consumed by NESTING
```

Design: `DESIGN-STONE-a-param-spec-must-be-consumed.md`. **Read its middle section** — it records
that an unused param is NOT vacuous (it still discriminates types), so this wall deliberately trades
away a capability rather than forbidding a no-op. That framing belongs in the diagnostic's tone.

## ★ Both halves already exist. This stone is wiring, not invention.

**The walker** — `crate::runtime::collect_free_type_vars` (`src/runtime.rs:4217`). It already walks
`Path` (via `is_type_var_path`, the VAR TEST at `:4189`), and recurses through `Parametric { args }`,
`Fn { args, ret }` and `Tuple(elements)`. **That recursion is the whole reason this stone is safe**:
consumption through nesting — `[x :- (Vector :- [T])]` — is handled by machinery that already
exists and is already exercised.

⛔ **Do NOT write a second walker.** Its current signature is function-shaped:

```rust
pub(crate) fn collect_free_type_vars(param_types: &[TypeExpr], ret_type: &TypeExpr) -> Vec<String>
```

Add a sibling entry point that takes just a slice, and have the existing one delegate to it, so the
inner `walk` stays the single implementation:

```rust
pub(crate) fn collect_free_type_vars_in(types: &[TypeExpr]) -> Vec<String>
```

**The door** — `parse_type_decl` (`src/types.rs:3700`) returns the built `TypeDef` and has exactly
**three** call sites (`types.rs:3357`, `:3564`, `:3585`). All six `TypeDef` variants —
`Aggregate`, `Enum`, `Newtype`, `Alias`, `Union`, `Surface` — carry `type_params`. So the check runs
ONCE, on the returned `TypeDef`, not seven times in seven parsers.

## The work

At `parse_type_decl`'s return, before handing the `TypeDef` back:

1. Gather every member `TypeExpr` the def reaches — `Aggregate` fields · `Enum` variant fields ·
   `Newtype` inner · `Alias` body · `Union` members · `Surface` fields.
2. `collect_free_type_vars_in(&members)` → the consumed set.
3. Every entry in `type_params` must appear in that set. The first that does not is the error.

Empty `type_params` ⇒ nothing to check, no branch needed.

## The diagnostic

`RVINA ERVDIT` — name the param, the declaration, and the fix:

> `type parameter "O" is declared but never used — every parameter in a type declaration's
> param-spec must be consumed by a field, variant, or body type. Remove it from the param-spec, or
> use it.`

## STOP triggers

1. **STOP-1** — if a legitimate NESTED consumption is rejected (`[x :- (Vector :- [T])]`), STOP.
   That is the one way this wall goes wrong quietly, and it would fire hardest on exactly the forms
   this arc is introducing. `collect_free_type_vars` already handles it; if your wiring does not,
   the wiring is wrong, not the walker.
2. **STOP-2** — if you cannot reach every member type from a `TypeDef` variant without adding a new
   accessor to that variant, STOP and report which. Reading the def should not require reshaping it.
3. **STOP-3** — do not touch `defn` / function signatures. A function's unused param stays legal;
   the caller supplies it. Type declarations only.
4. **STOP-4** — if the floor goes red, STOP and report the declarations rather than fixing them. The
   corpus scan says zero violations, but that scan is a regex that cannot tell code from string
   literals. **A real violation is a finding about the scan, not a chore.**

## Blast radius

`src/runtime.rs` (one sibling entry point, no new walker) · `src/types.rs` (one check at
`parse_type_decl`'s return, one new error kind if the existing `MalformedDecl` does not fit).
No `.wat`. No macro. No function signatures.

## How this lands

You are a rider. **Text edits only.** Do not run cargo, build, commit, stash, or revert. The
orchestrator builds, floors and clippies centrally, once.

This stone is Rust-only, so `./target/release/wat --check` on a scratch file reflects the LAST
BUILD and will not show your edit — the staleness warning is expected. Trace by reading.

Report: the diff; the exact diagnostic text; how you reach each variant's member types; and any
declaration in the corpus you believe violates the rule.
