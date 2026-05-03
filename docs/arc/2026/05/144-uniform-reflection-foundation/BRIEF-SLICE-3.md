# Arc 144 Slice 3 — Sonnet Brief — TypeScheme fingerprints for hardcoded callables

**Drafted 2026-05-03.** Substrate-informed: orchestrator crawled
`src/check.rs:3036-3082` (the 15 hardcoded callable dispatch arms
in `infer_list`), `src/check.rs:7761-7807` (`infer_length` —
polymorphic over Vec / HashMap / HashSet), `src/check.rs:8728+`
(`register_builtins` — where TypeScheme registrations live), arc
143 slice 1's `lookup_callable`/`lookup_form` precedent that
already calls `CheckEnv::with_builtins().get(name)` for the
Primitive variant. Confirmed slice 1's lookup_form discovers any
TypeScheme registered in `register_builtins` automatically — no
runtime dispatch changes needed.

**Goal:** register TypeScheme "callable-fingerprints" for the 15
hardcoded callables in `register_builtins` so `lookup_form` finds
them and returns `Binding::Primitive`. The hardcoded `infer_*`
handlers continue to do real type-checking; the schemes are
fingerprints capturing arity + return type for reflection's sake.
Closes the slice 6 length canary as a side-effect (the canary is
slice 4's load-bearing verification row).

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

## Required pre-reads (in order)

1. **`docs/arc/2026/05/144-uniform-reflection-foundation/DESIGN.md`**
   — slice 3 description.
2. **`docs/arc/2026/05/144-uniform-reflection-foundation/SCORE-SLICE-2.md`**
   — what slice 2 shipped (special-form registry). Slice 3 is
   independent; doesn't touch slice 2's surface.
3. **`src/check.rs:3036-3082`** — the 15 hardcoded dispatch arms
   (Vector, Tuple, HashMap, HashSet, string::concat, assoc, concat,
   dissoc, keys, values, empty?, conj, contains?, length, get).
4. **`src/check.rs:8728+`** — `register_builtins` body. The new
   TypeScheme registrations go here. Sonnet should read 200-300
   lines of existing registrations to absorb the conventional
   pattern.
5. **`src/runtime.rs:6315+`** — slice 1's `lookup_form`. Verify
   the CheckEnv path (3rd branch) calls
   `CheckEnv::with_builtins().get(name)` — which means new
   registrations in `register_builtins` are discovered AUTOMATICALLY.
6. **`src/runtime.rs:6029-6056`** — `type_scheme_to_signature_ast`
   helper from arc 143 slice 1. The signature head it synthesizes
   for a registered primitive uses `_a0`, `_a1`, ... param names
   and the registered return type.

## What to ship

### 1. The 15 TypeScheme registrations in `register_builtins`

For each hardcoded callable, add a TypeScheme registration in
`src/check.rs::register_builtins` (line ~8728+). The schemes are
FINGERPRINTS — they capture arity + return type so reflection
finds them. The hardcoded `infer_*` handlers continue to do real
type-checking (no removal; slice 3 is purely additive).

Use the existing pattern:

```rust
env.register(
    ":wat::core::length".into(),
    TypeScheme {
        type_params: vec!["T".into()],
        params: vec![TypeExpr::Var("T".into())],   // 1-arg, polymorphic
        ret: i64_ty(),
    },
);
```

Reuse helper functions like `i64_ty()` / `string_ty()` /
`bool_ty()` / `holon_ty()` / `library_ty()` that already exist in
register_builtins (read 100-200 lines around 8728+ to absorb the
conventions).

#### Per-primitive scheme guidance

| Primitive | Type params | Params | Return | Notes |
|---|---|---|---|---|
| `:wat::core::length` | `[T]` | `[:T]` | `:i64` | polymorphic; honest sentinel |
| `:wat::core::empty?` | `[T]` | `[:T]` | `:bool` | polymorphic |
| `:wat::core::contains?` | `[T,K]` | `[:T, :K]` | `:bool` | polymorphic over container + key/element |
| `:wat::core::get` | `[K,V]` | `[:HashMap<K,V>, :K]` | `:Option<V>` | sonnet verifies actual signature; HashMap-specific or polymorphic |
| `:wat::core::conj` | `[T]` | `[:Vector<T>, :T]` | `:Vector<T>` | sonnet verifies actual signature |
| `:wat::core::assoc` | `[K,V]` | `[:HashMap<K,V>, :K, :V]` | `:HashMap<K,V>` | sonnet verifies |
| `:wat::core::dissoc` | `[K,V]` | `[:HashMap<K,V>, :K]` | `:HashMap<K,V>` | sonnet verifies |
| `:wat::core::keys` | `[K,V]` | `[:HashMap<K,V>]` | `:Vector<K>` | sonnet verifies |
| `:wat::core::values` | `[K,V]` | `[:HashMap<K,V>]` | `:Vector<V>` | sonnet verifies |
| `:wat::core::concat` | `[T]` | `[:Vector<T>, :Vector<T>]` | `:Vector<T>` | 2-arg fingerprint (variadic-acceptance noted in comment) |
| `:wat::core::string::concat` | `[]` | `[:String, :String]` | `:String` | 2-arg fingerprint (variadic-acceptance noted) |
| `:wat::core::Vector` | `[T]` | `[:T]` | `:Vector<T>` | 1-arg fingerprint (variadic — comment notes runtime accepts 0+ args) |
| `:wat::core::Tuple` | `[T]` | `[:T]` | `:Tuple<T>` | 1-arg fingerprint (variadic) |
| `:wat::core::HashMap` | `[K,V]` | `[:K, :V]` | `:HashMap<K,V>` | 2-arg fingerprint (variadic — runtime accepts K-V pair sequence) |
| `:wat::core::HashSet` | `[T]` | `[:T]` | `:HashSet<T>` | 1-arg fingerprint (variadic) |

**Audit:** sonnet reads the actual `infer_*` handler for each (line
numbers in pre-reads) to confirm the type-param + param + return
shape. Where the brief's table differs from the actual handler,
PREFER THE HANDLER — the brief is the orchestrator's pre-flight
estimate.

**Variadic comment.** For each variadic constructor, add a Rust
comment above the registration explaining the fingerprint
limitation:

```rust
// :wat::core::Vector — variadic at runtime (accepts 0+ args).
// TypeScheme registers a 1-arg fingerprint per arc 144 slice 3
// limitation: TypeScheme has no variadic shape today. Hardcoded
// dispatch in check.rs:3038 handles real type-checking.
env.register(":wat::core::Vector".into(), TypeScheme { ... });
```

### 2. NO changes to runtime.rs or to the hardcoded `infer_*` handlers

Slice 3 is purely additive to `register_builtins`. The hardcoded
handlers stay; the new schemes don't replace them. `lookup_form`'s
CheckEnv path (slice 1) discovers the new schemes automatically.

### 3. Tests

NEW `tests/wat_arc144_hardcoded_primitives.rs` with 6+ tests:

1. **`signature_of_length_returns_some`** — `signature-of
   :wat::core::length` returns Some; AST is `(<head> (:_a0 :T) -> :i64)`.
2. **`signature_of_empty_q_returns_some`** — same shape for empty?.
3. **`signature_of_vector_returns_some`** — Vector constructor (1-arg
   fingerprint).
4. **`signature_of_hashmap_returns_some`** — HashMap constructor (2-arg
   fingerprint).
5. **`signature_of_get_returns_some`** — get with Option<V> return.
6. **`signature_of_conj_returns_some`** — conj.
7. **(BONUS)** `signature_of_concat_returns_some` — Vector concat.
8. **(BONUS)** `signature_of_string_concat_returns_some` — string concat.

Tests follow `tests/wat_arc144_lookup_form.rs` (slice 1's) shape.

### 4. The slice 6 length canary

After this slice, `tests/wat_arc143_define_alias.rs::define_alias_length_to_user_size_delegates_correctly`
should PASS — `:wat::core::length` is now reachable via
`lookup_form`, the macro emits the alias, the alias delegates.

Sonnet should run the slice 6 test specifically and report the
transition. If it STILL fails after the registration, surface the
diagnostic — there may be additional plumbing needed (e.g., the
type_scheme_to_signature_ast helper may not handle the polymorphic
single-Var T param shape correctly).

### 5. Workspace verification

```
cargo test --release --test wat_arc144_hardcoded_primitives    # new tests pass
cargo test --release --test wat_arc144_special_forms           # slice 2 unchanged
cargo test --release --test wat_arc144_lookup_form             # slice 1 unchanged
cargo test --release --test wat_arc143_lookup                  # arc 143 unchanged
cargo test --release --test wat_arc143_manipulation            # 8/8 unchanged
cargo test --release --test wat_arc143_define_alias            # NOW 3/3 (length canary turns green)
cargo test --release --workspace                                # baseline failure profile
```

After this slice, the slice 6 length canary should turn green —
the WORKSPACE FAILURE PROFILE shrinks (only the in-flight
CacheService.wat-induced wat-lru fail remains; that's arc 130's
territory).

```
cargo clippy --release --all-targets
```

No new warnings.

## Constraints

- **EDIT `src/check.rs`:** add 15 new TypeScheme registrations in
  `register_builtins` (line ~8728+). NO changes to the hardcoded
  `infer_*` handlers. NO changes to dispatch arms (slice 3 is
  purely additive to the registration table).
- **NEW test file:** `tests/wat_arc144_hardcoded_primitives.rs`.
- **No edits to `src/runtime.rs` or `src/special_forms.rs` or
  `src/macros.rs` or `src/freeze.rs`.**
- **No commits, no pushes.**

## What success looks like

1. 15 new TypeScheme registrations in `register_builtins`, each
   with a Rust comment naming the dispatch site + (for variadic)
   the fingerprint limitation.
2. `lookup_form` returns Some(Binding::Primitive) for each newly-
   registered name.
3. `signature-of` returns Some with the registered shape.
4. `body-of` returns :None (existing Primitive arm behavior).
5. New `tests/wat_arc144_hardcoded_primitives.rs` with 6+ tests;
   ALL pass.
6. **`define_alias_length_to_user_size_delegates_correctly` PASSES.**
   The slice 6 length canary turns green.
7. ALL OTHER arc 143 + arc 144 tests still pass.
8. `cargo test --release --workspace` failure profile shrinks
   (only CacheService.wat-induced wat-lru fail remains).
9. `cargo clippy --release --all-targets` no new warnings.

## Reporting back

Target ~250-350 words.

1. **Per-primitive scheme summary** — table or list naming each of
   the 15 registrations + verbatim TypeScheme construction for ONE
   representative (length, since it's the load-bearing canary).
2. **Audit deltas** — for each primitive whose actual signature
   differs from the brief's table, name the delta + the
   `infer_*` line evidence.
3. **lookup_form verification** — confirm the CheckEnv path is
   indeed being consulted (no runtime.rs changes needed).
4. **Length canary transition** — confirm
   `define_alias_length_to_user_size_delegates_correctly` PASSES;
   quote the cargo test summary line.
5. **New test results** — 6+ tests pass; quote the summary.
6. **Workspace test totals** — confirm baseline failure profile
   shrinks.
7. **clippy** — no new warnings.
8. **Honest deltas** — anything you needed to investigate / adapt.

## Sequencing

1. Read pre-reads in order.
2. Audit the 15 hardcoded `infer_*` handlers (file:line in pre-reads)
   to confirm each one's actual type-param + param + return shape.
3. Find `register_builtins` (check.rs:8728+) + read 100-200 lines of
   existing registrations to absorb the helper conventions
   (`i64_ty()`, `string_ty()`, etc.).
4. Add the 15 new TypeScheme registrations following the audit + the
   conventions.
5. Create `tests/wat_arc144_hardcoded_primitives.rs` with 6+ tests.
6. Run `cargo test --release --test wat_arc144_hardcoded_primitives` —
   confirm new tests pass.
7. **Run `cargo test --release --test wat_arc143_define_alias` —
   confirm the length canary turns green (3/3 now).**
8. Run slice 1 + slice 2 + arc 143 baseline tests — confirm zero
   regression.
9. Run `cargo test --release --workspace` — confirm shrunk failure
   profile.
10. Run `cargo clippy --release --all-targets` — confirm clean.
11. Report.

Then DO NOT commit. Working tree stays modified.

## Why this slice matters

Slice 3 closes the LAST coverage gap in `lookup_form`'s 5-variant
union. After this slice, the user's principle "nothing is special —
`(help :if) /just works/`" holds across ALL known forms in the
substrate:

- UserFunction ✅ (slice 1)
- Macro ✅ (slice 1)
- Type ✅ (slice 1)
- SpecialForm ✅ (slice 2)
- Primitive — was 90% covered; this slice adds the 15 hardcoded
  callables that bypass the TypeScheme registry today.

Slice 4 verifies the full surface + becomes the closure-prep slice
(originally separate; per slice 3's load-bearing length-canary
test, slice 4 may simplify to just the verification + INSCRIPTION
prep).

The arc 130 stepping stone is one cascade hop closer:
`:reduce` (arc 143) → `:length`/`Vector/len` (this slice + arc 130)
→ arc 130 RELAND completes → arc 109 v1 closes (after arc 130 +
arc 145 also close).
