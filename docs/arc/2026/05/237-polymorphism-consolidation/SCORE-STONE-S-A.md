# SCORE — Stone S-A — mint `:wat::core::subtype?` + `typesub` registry

**Date:** 2026-05-25
**Status:** COMPLETE — 10/10 probe PASS. All scorecard rows green.

---

## Scorecard

| # | Row | Command | Result |
|---|-----|---------|--------|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| grep "^error"` | 0 errors; 108 warnings (pre-existing ceiling) |
| 2 | **S-A probe 10/10 PASS** (LOAD-BEARING) | `cargo test --release --test probe_arc237_sA_hierarchy 2>&1 \| tail -3` | `10 passed; 0 failed` |
| 3 | Lib baseline | `cargo test --release --lib -p wat 2>&1 \| tail -3` | `827 passed; 0 failed` |
| 4 | Stone 237.1 regression | `cargo test --release --test probe_arc237_stone1_typeunion_substrate 2>&1 \| grep "test result:"` | `14 passed; 0 failed` |
| 5 | Stone 237.5 regression | `cargo test --release --test probe_arc237_stone5_conforms 2>&1 \| grep "test result:"` | `12 passed; 0 failed` |
| 6 | Stone 237.6 regression | `cargo test --release --test probe_arc237_stone6_is_predicate 2>&1 \| grep "test result:"` | `10 passed; 0 failed` |
| 7 | holon-rs untouched | STOP-5 | confirmed — zero holon-rs changes |

---

## Final API shape

### New primitive (`:wat::core::subtype?`)

```
(:wat::core::subtype? :ChildType :ParentType) -> :wat::core::bool
```

- Arg 0: type-position keyword (FQDN of child type; NOT evaluated as a value)
- Arg 1: type-position keyword (FQDN of parent type; NOT evaluated as a value)
- Return: `:wat::core::bool` — `true` iff child is-a parent (reflexive + transitive)
- Error contract: unknown type name (not in TypeEnv, not a built-in primitive) → `Err` (bad input, not a negative result)

### New `TypeEnv` field (src/types.rs)

```rust
subtype_edges: HashMap<String, Vec<String>>
```

Child FQDN → list of direct parent FQDNs. Distinct from `typeunion` membership
(which lives in `types: HashMap<String, TypeDef>`). Default-empty.

### New public methods / functions (src/types.rs)

| Function | Purpose |
|----------|---------|
| `TypeEnv::register_subtype(child, parent) -> Result<(), TypeError>` | Cycle-rejecting edge registration |
| `TypeEnv::subtype_parents(name) -> Vec<&str>` | Private helper: direct parent FQDNs |
| `pub fn is_subtype(sub, sup, &TypeEnv) -> bool` | Free fn: reflexive + transitive walk |

### New `TypeError` variant (src/types.rs)

```rust
CyclicSubtype { child: String, parent: String }
```

Fired by `register_subtype` when the new edge would close a cycle.
No `Span` field (edges registered programmatically, not from source forms).
Mirror of `CyclicUnion { name, span }` but simpler (internal call, no source span available).

### New dispatch arm (src/runtime.rs, beside conforms? at ~5291)

```rust
":wat::core::subtype?" => eval_subtype(args, list_span, env, sym),
```

### New private function (src/runtime.rs)

| Function | Purpose |
|----------|---------|
| `eval_subtype` | Arity-2 entry point: parse both args as type keywords, validate both known, call `is_subtype` |

### New `infer_list` arm (src/check.rs, beside conforms? at ~5561)

Special-cased because BOTH args are type-position keywords (the type-keyword-infers-as-Fn
trap fires for any keyword naming a registered constructor). Both args validated as
`WatAST::Keyword`; inference skipped on both. No `_discard` drain needed (unlike
`conforms?`, neither arg is a value expression).

### New TypeScheme (src/check.rs, `register_builtins`)

```rust
env.register(":wat::core::subtype?".into(), TypeScheme {
    type_params: vec![],                                         // no type var — both args are keywords
    params: vec![keyword_ty(), keyword_ty()],                    // :wat::core::keyword × :wat::core::keyword
    ret: bool_ty(),
    rest_param_type: None,
});
```

### Built-in seeded roots (src/types.rs `register_builtin_types`)

- `:wat::holon::Record` registered as opaque zero-field `TypeDef::Struct` (mirrors `:wat::Record`)
- `env.register_subtype(":wat::holon::Record", ":wat::Record")` seeded (privileged path, `expect()` correct — cannot cycle in a fresh registry)

---

## Line count

| File | Pre-stone lines | Post-stone lines | Net added |
|------|-----------------|------------------|-----------|
| `src/types.rs` | 3,649 | 3,773 | +124 (`TypeEnv::subtype_edges` field; `register_subtype`; `subtype_parents`; `CyclicSubtype` variant + Display arm; `is_subtype` free fn; `:wat::holon::Record` registration + subtype edge seed) |
| `src/runtime.rs` | 33,261 | 33,358 | +97 (dispatch arm comment + arm; `eval_subtype` + section banners) |
| `src/check.rs` | 21,187 | 21,256 | +69 (`infer_list` arm for `subtype?` with arity check + both-arg keyword validation + return; TypeScheme registration block) |

Total net: ~290 lines. Within BRIEF's 40–70 min Mode A calibration band.

---

## Cascade depth

**2 rounds.**

1. `src/types.rs` — adds `subtype_edges` field + `register_subtype` + `subtype_parents` + `CyclicSubtype` TypeError variant + Display arm + `is_subtype` free fn + `:wat::holon::Record` registration + root edge seed. New `TypeError::CyclicSubtype` variant forced one exhaustiveness cascade within `types.rs` (the `Display` `match self` arm) — WITHIN the 3 allowed files, no new files.
2. `src/runtime.rs` + `src/check.rs` — dispatch arm + `eval_subtype` + `infer_list` arm + TypeScheme. Builds clean (no new Value variant = no Value exhaustiveness cascade). Probe 10/10 PASS. No further cascade.

No new Value variants, RuntimeError variants (only ArityMismatch + MalformedForm reused), or CheckError variants → zero forced cascade files beyond `types.rs`. STOP-5 not triggered.

---

## Honest deltas

### `:wat::holon::is-Record?` auto-synthesis (expected, noted in BRIEF)

Registering `:wat::holon::Record` as a `TypeDef::Struct` causes `register_type_predicates`
to synthesize `:wat::holon::is-Record?` for it — the same mechanism that synthesizes
`:wat::is-Record?` for `:wat::Record`. This is correct: `:wat::holon::Record` IS a type;
it should have a predicate. Mentioned in BRIEF § Implementation sketch as an expected,
non-surprising delta.

### `CyclicSubtype` has no `Span` field (minor deviation from `CyclicUnion`)

`CyclicUnion` carries a `Span` because it is raised from source-form parsing where a
location is always available. `register_subtype` is called programmatically (no source
form, no location). Accordingly `CyclicSubtype { child, parent }` carries no `Span` —
the Display message is fully descriptive without one. This is an honest deviation;
the error contract is otherwise identical (cycle detected at registration time, rejected).

### `is_subtype` uses `Vec<String>` for stack items (minor)

The BRIEF sketch used `Vec<&str>` for the stack. The implementation uses `Vec<String>`
to avoid borrow-checker lifetime friction with the iterative BFS/DFS over `env.subtype_parents(p)`
(where `p` is owned by the stack itself). The algorithm and semantics are identical.

### `subtype?` `infer_list` arm placed BEFORE `conforms?` arm

The arm for `":wat::core::subtype?"` is placed immediately before the existing
`":wat::core::conforms?"` arm in the `infer_list` match. Both probe 10/10 and
the predecessor probes (237.5 12/12) confirm no ordering sensitivity.

---

## Working tree on return

```
 M docs/arc/2026/05/170-program-entry-points/INTERSTITIAL-CLIFFNOTES.md
 M docs/arc/2026/05/170-program-entry-points/INTERSTITIAL-REALIZATIONS.md
 M src/check.rs
 M src/runtime.rs
 M src/types.rs
?? docs/arc/2026/05/237-polymorphism-consolidation/SCORE-STONE-S-A.md
```

holon-rs untouched. STOP-5 not triggered. DO NOT commit (orchestrator commits).
