# Arc 146 Slice 4 — Sonnet Brief — Alias migrations: assoc/dissoc/keys/values/concat

**Drafted 2026-05-03.** Substrate-informed: orchestrator crawled
all 5 retirement targets:
- `eval_assoc` (runtime.rs:5764) + `infer_assoc` (check.rs:7394) + dispatch arms (runtime.rs:2683 + check.rs:3108) + arc 144 fingerprint
- `eval_dissoc` (runtime.rs:5835) + `infer_dissoc` (check.rs:7500) + dispatch arms (runtime.rs:2684 + check.rs:3110) + fingerprint
- `eval_keys` (runtime.rs:5872) + `infer_keys` (check.rs:7563) + dispatch arms (runtime.rs:2685 + check.rs:3111) + fingerprint
- `eval_values` (runtime.rs:5907) + `infer_values` (check.rs:7615) + dispatch arms (runtime.rs:2686 + check.rs:3112) + fingerprint
- `eval_concat` (runtime.rs:5120) + `infer_concat` (check.rs:8047) + dispatch arms (runtime.rs:2682 + check.rs:3109) + fingerprint

FM 9 baseline: all green except CacheService.wat noise. Slice 3
shipped 4 dispatch migrations + 1 substrate completion (stdlib_loaded
fixture).

**Goal:** migrate the remaining 5 SINGLE-IMPL ops via arc 143's
`:wat::runtime::define-alias` mechanism. These ops aren't
polymorphic — assoc/dissoc/keys/values are HashMap-only;
concat is Vector-only. Dispatch is overkill (1-arm); alias is
the architecturally-correct mechanism for "one impl, two names
(short ergonomic + long explicit)."

After this slice + slice 5 (closure), arc 146 closes; every
defined symbol queryable at runtime.

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

## The 5 migrations

| Primitive | Container | Per-Type impl name | Notes |
|---|---|---|---|
| `assoc` | HashMap | `:wat::core::HashMap/assoc` | (HashMap K V) → HashMap |
| `dissoc` | HashMap | `:wat::core::HashMap/dissoc` | (HashMap K) → HashMap |
| `keys` | HashMap | `:wat::core::HashMap/keys` | (HashMap) → :Vec<K> |
| `values` | HashMap | `:wat::core::HashMap/values` | (HashMap) → :Vec<V> |
| `concat` | Vector | `:wat::core::Vector/concat` | (Vec<T> Vec<T>) → Vec<T> — variadic at runtime; 2-arg fingerprint per arc 144 slice 3 limitation |

## Required pre-reads (in order)

1. **`docs/arc/2026/05/146-container-method-correction/SCORE-SLICE-3.md`**
   — slice 3's pattern + the stdlib_loaded fixture completion
   (Delta 3). Substrate is now COMPLETE for both dispatch +
   stdlib registration.
2. **`docs/arc/2026/05/146-container-method-correction/DESIGN.md`**
   slice 1b — the gaze-justified `Dispatch`/`define-dispatch`
   naming.
3. **`wat/list.wat`** — precedent for arc 146 alias declarations.
   `(:wat::runtime::define-alias :wat::list::reduce :wat::core::foldl)`
   is the canonical shape; you mirror it for the 5 ops.
4. **`wat/runtime.wat`** — defines the `:wat::runtime::define-alias`
   macro you'll use.
5. **`src/stdlib.rs:28-148`** — STDLIB_FILES + load order. Note:
   wat/list.wat loads AFTER wat/runtime.wat because list.wat
   USES the alias macro. Your new file follows the same
   constraint.
6. **Slice 2's per-Type impl pattern in src/runtime.rs** —
   inner-helper-plus-eval-wrapper for substrate-impl fallback
   compatibility.

## What to ship

### A. Per-Type runtime impls in src/runtime.rs

5 new substrate primitives. Each with inner helper + eval
wrapper, mirroring slice 2/3 pattern:

```rust
fn eval_hashmap_assoc(args, env, sym) -> Result<Value, RuntimeError>    // (HashMap K V) -> HashMap
fn eval_hashmap_dissoc(args, env, sym) -> Result<Value, RuntimeError>   // (HashMap K) -> HashMap
fn eval_hashmap_keys(args, env, sym) -> Result<Value, RuntimeError>     // (HashMap) -> :Vec<K>
fn eval_hashmap_values(args, env, sym) -> Result<Value, RuntimeError>   // (HashMap) -> :Vec<V>
fn eval_vector_concat(args, env, sym) -> Result<Value, RuntimeError>    // (Vec, Vec) -> Vec  (binary fingerprint; variadic accepted at impl level if needed)
```

Each takes 1-3 args; type-checks against its container shape;
returns the appropriate result type. Logic comes from the
existing eval_* impls being retired (extract the HashMap or
Vector branch).

### B. 5 dispatch arms in dispatch_keyword_head

```rust
":wat::core::HashMap/assoc"   => eval_hashmap_assoc(args, env, sym),
":wat::core::HashMap/dissoc"  => eval_hashmap_dissoc(args, env, sym),
":wat::core::HashMap/keys"    => eval_hashmap_keys(args, env, sym),
":wat::core::HashMap/values"  => eval_hashmap_values(args, env, sym),
":wat::core::Vector/concat"   => eval_vector_concat(args, env, sym),
```

Plus `dispatch_substrate_impl` extension with 5 new entries.

### C. 5 TypeScheme registrations in register_builtins

Add 5 new registrations adjacent to slice 2/3's per-Type
schemes. Use existing helpers (hashmap_of, vec_of, k_var,
v_var, t_var). Example for HashMap/assoc:

```rust
env.register(":wat::core::HashMap/assoc".into(), TypeScheme {
    type_params: vec!["K".into(), "V".into()],
    params: vec![hashmap_of(k_var(), v_var()), k_var(), v_var()],
    ret: hashmap_of(k_var(), v_var()),
});
```

### D. NEW `wat/core-aliases.wat`

```scheme
;; wat/core-aliases.wat — :wat::core::* short-name aliases.
;;
;; Each alias maps a short ergonomic name to its explicit
;; per-Type impl. Per arc 146 DESIGN: single-impl ops are
;; aliases, not dispatches (dispatch is for genuine polymorphism).
;;
;; Both names work; both are honest. The alias machinery (arc
;; 143's :wat::runtime::define-alias) expands at registration
;; time into a delegating user-define.

(:wat::runtime::define-alias :wat::core::assoc   :wat::core::HashMap/assoc)
(:wat::runtime::define-alias :wat::core::dissoc  :wat::core::HashMap/dissoc)
(:wat::runtime::define-alias :wat::core::keys    :wat::core::HashMap/keys)
(:wat::runtime::define-alias :wat::core::values  :wat::core::HashMap/values)
(:wat::runtime::define-alias :wat::core::concat  :wat::core::Vector/concat)
```

### E. Register `wat/core-aliases.wat` in stdlib.rs

Add to STDLIB_FILES AFTER `wat/runtime.wat` (because the file
USES `:wat::runtime::define-alias` from runtime.wat). Place
adjacent to `wat/list.wat` (same constraint):

```rust
// Arc 146 slice 4 — :wat::core::* short-name aliases.
// Uses arc 143's :wat::runtime::define-alias macro so loads
// AFTER wat/runtime.wat. Each alias maps a short name to an
// explicit per-Type impl (assoc → HashMap/assoc, etc.).
WatSource {
    path: "wat/core-aliases.wat",
    source: include_str!("../wat/core-aliases.wat"),
},
```

### F. Retire all 5 sets of old machinery

For each of assoc / dissoc / keys / values / concat:
- DELETE the eval dispatch arm in src/runtime.rs (lines 2682-2686)
- DELETE the eval_* function (entire body — 5 functions)
- DELETE the infer_list dispatch arm in src/check.rs (lines 3108-3112)
- DELETE the infer_* function (entire body — 5 functions)
- DELETE the arc 144 slice 3 TypeScheme fingerprint (in the
  fingerprint section around line 11800-12010 — find each
  `env.register(":wat::core::assoc".into(), ...)` etc. and
  delete the block + leading comment)

### G. arc 144 hardcoded_primitives test status

Per slice 3's experience: tests for these primitives may pass
unchanged via slice 2 Delta 4's `dispatch_to_signature_ast`
inheritance — BUT, that was for DISPATCHES. Slice 4's aliases
expand to USER DEFINES (Function entities, not Dispatch).

`signature-of :wat::core::assoc` post-slice-4 returns the
user-define's signature (the alias-expanded `(:assoc<K,V>
(_a0 :HashMap<K,V>) ...)` head). The test assertions may need
adjustment.

If they fail: update each to query the per-Type primitive
(e.g., `:wat::core::HashMap/assoc`) per slice 2's Q2 Option A
pattern. Surface in report.

### H. Verification

```
cargo test --release --test wat_arc146_dispatch_mechanism    # 7/7 unchanged
cargo test --release --test wat_arc144_lookup_form           # 9/9 unchanged
cargo test --release --test wat_arc144_special_forms         # 9/9 unchanged
cargo test --release --test wat_arc144_hardcoded_primitives  # WATCH — see G above
cargo test --release --test wat_arc143_lookup                # 11/11 unchanged
cargo test --release --test wat_arc143_manipulation          # 8/8 unchanged
cargo test --release --test wat_arc143_define_alias          # 3/3 unchanged
cargo test --release --workspace
```

Workspace failure profile: same as post-slice-3 (only
CacheService.wat noise).

```
cargo clippy --release --all-targets   # 40 → 40 (no new warnings)
```

## Constraints

- **EDITS:** src/runtime.rs (5 new fns + 5 dispatch arms; 5 fns + 5 arms RETIRED), src/check.rs (5 new TypeScheme; 5 fns + 5 arms + 5 fingerprints RETIRED), src/stdlib.rs (1 new WatSource entry).
- **NEW FILES:** wat/core-aliases.wat.
- **NO new test files.** Verification is via existing baseline tests + arc 144 hardcoded_primitives behavior.
- **NO commits, no pushes.**

## Open questions sonnet decides

### Q1 — alias file naming + load order

Brief recommends `wat/core-aliases.wat` adjacent to `wat/list.wat`
in load order. If sonnet finds another shape works better
(e.g., extending wat/list.wat to also hold core aliases), report
+ adapt. Constraint: must load AFTER wat/runtime.wat.

### Q2 — concat variadic semantics

`:wat::core::concat` was variadic (0+ Vec args). The per-Type
`:wat::core::Vector/concat` may need to mirror or simplify to
binary. Mirror existing eval_concat's runtime arity acceptance.

### Q3 — arc 144 hardcoded_primitives test breakage

If `signature_of_assoc/dissoc/keys/values/concat` tests break
due to alias-expansion-vs-dispatch shape difference, apply
slice 2 Q2 Option A pattern (query per-Type instead of short
name).

## What success looks like

1. 5 per-Type runtime impls (HashMap/assoc, dissoc, keys, values + Vector/concat).
2. 5 dispatch arms in dispatch_keyword_head + dispatch_substrate_impl.
3. 5 TypeScheme registrations in register_builtins.
4. NEW wat/core-aliases.wat with 5 define-alias declarations.
5. wat/core-aliases.wat registered in stdlib AFTER wat/runtime.wat.
6. Old machinery RETIRED: 5 eval_* + 5 dispatch arms + 5 infer_* + 5 dispatch arms + 5 arc 144 fingerprints (25 retirements total).
7. Both short + long names work end-to-end (verify by manually checking that `(:keys some-map)` and `(:HashMap/keys some-map)` both type-check + run identically).
8. ALL baseline tests pass (with arc 144 hardcoded_primitives possibly Q3-updated).
9. clippy clean.

## Reporting back

Target ~250-350 words.

1. **Per-Type impls** — list 5 + quote 1 representative.
2. **TypeScheme registrations** — quote 1.
3. **wat/core-aliases.wat content** — quote verbatim.
4. **Retirements** — list each with file:line.
5. **arc 144 hardcoded_primitives status** — passed/updated/etc.
6. **Test totals** — confirm baselines.
7. **clippy** — quote count.
8. **Q1-Q3 decisions** — name choices.
9. **Honest deltas** — anything investigated/adapted.

## Sequencing

1. Read pre-reads in order.
2. Add 5 new per-Type impls + dispatch arms (additive).
3. Add 5 new TypeScheme registrations (additive).
4. Create wat/core-aliases.wat + register in stdlib.rs.
5. Verify dispatch works: `cargo test --release --test
   wat_arc146_dispatch_mechanism` + ensure no new failures.
6. Retire all 5 sets of old machinery.
7. Run all baselines.
8. Update arc 144 hardcoded_primitives tests if Q3 fires.
9. Run workspace + clippy.
10. Report.

Then DO NOT commit. Working tree stays modified.

## Why this slice matters

This is the LAST migration slice. After this slice + slice 5
(closure paperwork), arc 146 closes. The substrate has:
- 6 entity kinds: UserFunction, Macro, Primitive, SpecialForm,
  Type, Dispatch
- Every defined symbol queryable at runtime via lookup_form
- 10/10 originally-violating primitives properly defined
  (length + 4 dispatch + 5 alias)
- Hardcoded `infer_*` handlers for these primitives RETIRED

User's finish line achieved: every defined symbol queryable at
runtime.

Per § 12 foundation discipline: rhythm at scale. Slice 1
proved the mechanism; slice 2 proved a single migration; slice 3
proved bundled migration; slice 4 proves the alias-pattern fit
for single-impl ops. Each cycle compounds; the foundation is
demonstrably impeccable.
