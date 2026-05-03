# Arc 146 Slice 3 — Sonnet Brief — BUNDLED dispatch migrations: empty? + contains? + get + conj

**Drafted 2026-05-03.** Substrate-informed: orchestrator crawled
all 4 retirement targets:
- `eval_empty_q` (runtime.rs:5087-5117) + `infer_empty_q` (check.rs:7669) + dispatch arms (runtime.rs:2666 + check.rs:3113) + arc 144 slice 3 fingerprint
- `eval_contains_q` (runtime.rs:5977) + `infer_contains_q` (check.rs:7808) + dispatch arms (runtime.rs:2687 + check.rs:3115) + fingerprint
- `eval_get` (runtime.rs:5698) + `infer_get` (check.rs:7283) + dispatch arms (runtime.rs:2681 + check.rs:3122) + fingerprint
- `eval_conj` (runtime.rs:4848) + `infer_conj` (check.rs:7722) + dispatch arms (runtime.rs:2651 + check.rs:3114) + fingerprint

FM 9 baseline: all green except the in-flight CacheService.wat
noise (ignored). Slice 2 closed the length canary; the dispatch
mechanism is PROVEN end-to-end for one primitive.

**Goal:** migrate the remaining 4 GENUINELY POLYMORPHIC primitives
in ONE bundled sweep — same shape as slice 2, repeated 4 times.
The pattern is mechanical now. Slice 2's substrate-completion
deltas are FIXED so this slice should run cleaner per migration.

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

## The 4 migrations

| Primitive | Container arms | Per-Type impl names | Notes |
|---|---|---|---|
| `empty?` | Vector / HashMap / HashSet | `Vector/empty?` / `HashMap/empty?` / `HashSet/empty?` | uniform verb; 3 arms; returns `:bool` |
| `contains?` | Vector / HashMap / HashSet | `Vector/contains?` / `HashMap/contains-key?` / `HashSet/contains?` | **MIXED VERBS** — HashMap tests KEY membership (different verb); Vector + HashSet test ELEMENT membership |
| `get` | Vector / HashMap | `Vector/get` / `HashMap/get` | 2 arms (HashSet's "get-by-equality" IS just contains?); RETURN TYPE varies — Vec returns `:Option<T>`, HashMap returns `:Option<V>` |
| `conj` | Vector / HashSet | `Vector/conj` / `HashSet/conj` | 2 arms (HashMap uses assoc; no conj); each returns container type |

**Why mixed verbs for contains?** Pass-through dispatch routes by
TYPE PATTERN, not by impl-name uniformity. The dispatch declares
3 arms; each arm's impl can have whatever verb makes semantic
sense. Caller writes `(:contains? c x)`; dispatch picks the arm
based on `c`'s type; the impl's verb (`contains?` vs
`contains-key?`) is internal.

## Required pre-reads (in order)

1. **`docs/arc/2026/05/146-container-method-correction/SCORE-SLICE-2.md`**
   — slice 2's pattern + the 4 substrate-completion deltas
   (eval_dispatch_call substrate-impl fallback; infer_dispatch_call
   arg-side type-var instantiation; stdlib step 4a vs user step
   6b; signature-of synthesis). These are FIXED — your migrations
   inherit a complete substrate.
2. **`docs/arc/2026/05/146-container-method-correction/BRIEF-SLICE-2.md`**
   + DESIGN — the migration pattern.
3. **`src/runtime.rs:4940-...`** (post-slice-2 — the NEW per-Type
   length impls + their dispatch arms; mirror this shape for
   each of the 4 primitives).
4. **`src/check.rs::register_builtins`** — find the slice-2-added
   per-Type length schemes; add the new per-Type schemes near
   them. Find the arc 144 slice 3 fingerprint section (around
   line 11800-12010) — RETIRE 4 fingerprints (empty?, contains?,
   get, conj).
5. **`wat/core.wat`** — slice 2's length declaration. You APPEND
   4 more dispatch declarations.
6. **`tests/wat_arc144_hardcoded_primitives.rs`** — slice 2's Q2
   Option A pattern (`:wat::core::length` test was updated to
   query `:wat::core::Vector/length`). You apply analogous
   updates for empty?, contains?, get, conj if their tests
   reference the `:wat::core::*` fingerprints that this slice
   retires.

## What to ship

For EACH of the 4 primitives, the slice-2 pattern repeats:

### A. Per-Type runtime impls (in src/runtime.rs)

Mirror slice 2's `eval_vector_length` shape — inner helper
taking pre-evaluated Value + outer eval wrapper.

```rust
// empty?: 3 impls (Vector/HashMap/HashSet → :bool)
fn eval_vector_empty_q(args, env, sym) -> Result<Value, RuntimeError>
fn eval_hashmap_empty_q(args, env, sym) -> Result<Value, RuntimeError>
fn eval_hashset_empty_q(args, env, sym) -> Result<Value, RuntimeError>
// (each with inner helper for substrate-impl fallback per slice 2 Delta 1)

// contains?: 3 impls with MIXED verbs
fn eval_vector_contains_q(args, env, sym)            // Vec<T> :T -> :bool
fn eval_hashmap_contains_key_q(args, env, sym)       // HashMap<K,V> :K -> :bool
fn eval_hashset_contains_q(args, env, sym)           // HashSet<T> :T -> :bool

// get: 2 impls (Vec/HashMap; return type VARIES)
fn eval_vector_get(args, env, sym)                   // Vec<T> :i64 -> :Option<T>
fn eval_hashmap_get(args, env, sym)                  // HashMap<K,V> :K -> :Option<V>

// conj: 2 impls
fn eval_vector_conj(args, env, sym)                  // Vec<T> :T -> :Vec<T>
fn eval_hashset_conj(args, env, sym)                 // HashSet<T> :T -> :HashSet<T>
```

### B. Per-Type dispatch arms in dispatch_keyword_head

Add 10 new arms total (3+3+2+2):
```rust
":wat::core::Vector/empty?"        => eval_vector_empty_q(args, env, sym),
":wat::core::HashMap/empty?"       => eval_hashmap_empty_q(args, env, sym),
":wat::core::HashSet/empty?"       => eval_hashset_empty_q(args, env, sym),
":wat::core::Vector/contains?"     => eval_vector_contains_q(args, env, sym),
":wat::core::HashMap/contains-key?" => eval_hashmap_contains_key_q(args, env, sym),
":wat::core::HashSet/contains?"    => eval_hashset_contains_q(args, env, sym),
":wat::core::Vector/get"           => eval_vector_get(args, env, sym),
":wat::core::HashMap/get"          => eval_hashmap_get(args, env, sym),
":wat::core::Vector/conj"          => eval_vector_conj(args, env, sym),
":wat::core::HashSet/conj"         => eval_hashset_conj(args, env, sym),
```

### C. Per-Type TypeScheme registrations in register_builtins

Add 10 new registrations, mirroring slice 2's Vector/length /
HashMap/length / HashSet/length pattern. Place them adjacent to
slice 2's length registrations.

For `get`: return type uses the container's element/value type:
```rust
env.register(":wat::core::Vector/get".into(), TypeScheme {
    type_params: vec!["T".into()],
    params: vec![vec_of(t_var()), i64_ty()],
    ret: option_of(t_var()),
});
env.register(":wat::core::HashMap/get".into(), TypeScheme {
    type_params: vec!["K".into(), "V".into()],
    params: vec![hashmap_of(k_var(), v_var()), k_var()],
    ret: option_of(v_var()),
});
```

(Use existing helpers like `vec_of`, `hashmap_of`, `option_of`,
`i64_ty`, `bool_ty` — they exist in register_builtins. Check the
slice 2 length registrations for the canonical pattern.)

### D. Append 4 dispatch declarations to wat/core.wat

```scheme
(:wat::core::define-dispatch :wat::core::empty?
  ((:wat::core::Vector<T>)    :wat::core::Vector/empty?)
  ((:wat::core::HashMap<K,V>) :wat::core::HashMap/empty?)
  ((:wat::core::HashSet<T>)   :wat::core::HashSet/empty?))

(:wat::core::define-dispatch :wat::core::contains?
  ((:wat::core::Vector<T>    T) :wat::core::Vector/contains?)
  ((:wat::core::HashMap<K,V> K) :wat::core::HashMap/contains-key?)
  ((:wat::core::HashSet<T>   T) :wat::core::HashSet/contains?))

(:wat::core::define-dispatch :wat::core::get
  ((:wat::core::Vector<T>    :wat::core::i64) :wat::core::Vector/get)
  ((:wat::core::HashMap<K,V> K)                :wat::core::HashMap/get))

(:wat::core::define-dispatch :wat::core::conj
  ((:wat::core::Vector<T>  T) :wat::core::Vector/conj)
  ((:wat::core::HashSet<T> T) :wat::core::HashSet/conj))
```

(Slice 1's pass-through arms take a type pattern PER ARG — for
2-arg primitives like contains?/get/conj, the pattern includes
both arg types. Verify slice 1's pattern grammar accepts the
`(:Container<X> X)` shape — slice 2 only used 1-arg patterns.)

### E. Retire all 4 sets of old machinery

For each primitive (empty?, contains?, get, conj):
- DELETE the eval dispatch arm in src/runtime.rs (line 2651, 2666, 2681, 2687)
- DELETE the eval_* function (line 4848, 5087, 5698, 5977 — entire body each)
- DELETE the infer_list dispatch arm in src/check.rs (line 3113, 3114, 3115, 3122)
- DELETE the infer_* function (line 7283, 7669, 7722, 7808 — entire body each)
- DELETE the arc 144 slice 3 TypeScheme fingerprint (in the section around line 11800-12010 — find each `env.register(":wat::core::empty?".into(), ...)` etc. and delete its block + leading comment)

### F. Update arc 144 hardcoded_primitives tests if needed

Per slice 2's Q2 Option A: tests that asserted
`:wat::core::empty?` / `:get` / `:contains?` / `:conj` returns
specific scheme shapes need updating to query the per-Type
primitive (e.g., `:wat::core::Vector/empty?`).

Sonnet greps `tests/wat_arc144_hardcoded_primitives.rs` for
references; updates each to the analogous per-Type query.

### G. Verification

```
cargo test --release --test wat_arc144_hardcoded_primitives  # 17/17 with updates
cargo test --release --test wat_arc146_dispatch_mechanism    # 7/7 unchanged
cargo test --release --test wat_arc144_lookup_form           # 9/9 unchanged
cargo test --release --test wat_arc144_special_forms         # 9/9 unchanged
cargo test --release --test wat_arc143_lookup                # 11/11 unchanged
cargo test --release --test wat_arc143_manipulation          # 8/8 unchanged
cargo test --release --test wat_arc143_define_alias          # 3/3 unchanged
cargo test --release --workspace
```

Workspace failure profile: same as post-slice-2 (only
CacheService.wat noise).

```
cargo clippy --release --all-targets   # 40 → 40 (no new warnings)
```

## Constraints

- **EDITS:** src/runtime.rs (10 new fns + 10 dispatch arms; 4 fns + 4 arms RETIRED), src/check.rs (10 new TypeScheme; 4 fns + 4 arms + 4 fingerprints RETIRED), wat/core.wat (4 dispatch declarations APPENDED), tests/wat_arc144_hardcoded_primitives.rs (Q2 Option A updates as needed).
- **NO new test files.** The dispatch mechanism is proven by slice 1's existing tests; the migrations are mechanical applications of the proven shape.
- **NO commits, no pushes.**

## Open questions sonnet decides

### Q1 — Slice 1's pass-through pattern grammar with multi-arg arms

Slice 2 only declared 1-arg dispatches (length). This slice
declares 2-arg dispatches (contains?, get, conj). Verify slice
1's parse_define_dispatch_form (or equivalent) accepts arms with
multiple type-pattern entries: `((:Container<X> X) :impl)`.

If parser doesn't accept this shape: surface clean diagnostic;
that's a slice 1 substrate-completion gap that needs filling
BEFORE this slice can ship.

### Q2 — `get` return type unification

`:Vector/get` returns `:Option<T>`; `:HashMap/get` returns
`:Option<V>`. The dispatch's per-arm return types differ.
Verify slice 1's infer_dispatch_call correctly returns the
matched arm's specific return type (not a unified Option of
something).

If unification fails: surface; slice 1 substrate-completion gap.

### Q3 — `contains?` dispatch arms with mixed impl verbs

Verify slice 1's parser doesn't require uniform impl naming
across arms. If it does (e.g., expects all impls to share a
common suffix matching the dispatch's name): surface; sonnet
adapts the dispatch declaration shape.

Recommended: each arm declares its own impl freely; slice 1's
parser should already support this since impl_name is just a
keyword path.

## What success looks like

1. 10 per-Type runtime impls (3 empty? + 3 contains? + 2 get + 2 conj).
2. 10 per-Type dispatch arms in dispatch_keyword_head.
3. 10 per-Type TypeScheme registrations in register_builtins.
4. 4 dispatch declarations appended to wat/core.wat.
5. 4 sets of old machinery RETIRED (eval_* + infer_* + dispatch arms + arc 144 fingerprints).
6. arc 144 hardcoded_primitives tests updated per Q2 Option A.
7. ALL baseline tests still pass.
8. `cargo test --release --workspace` failure profile unchanged.
9. clippy 40 → 40, no new warnings.

## Reporting back

Target ~400-500 words.

1. **Per-Type impls + dispatch arms** — list each primitive's per-Type impls + dispatch arms; quote ONE representative for each primitive (10 total impls; one verbatim per primitive = 4 verbatim).
2. **TypeScheme registrations** — quote 1 representative.
3. **wat/core.wat additions** — quote all 4 declarations verbatim.
4. **Retirements** — list each retired item with file:line.
5. **arc 144 hardcoded_primitives updates** — name each test updated.
6. **Test totals** — confirm baselines unchanged.
7. **clippy** — no new warnings.
8. **Decisions on Q1-Q3** — name what was found / chosen.
9. **Honest deltas** — anything investigated / adapted.

## Sequencing

1. Read pre-reads in order.
2. Add 10 new per-Type runtime impls + dispatch arms (additive).
3. Add 10 new TypeScheme registrations (additive).
4. Append 4 dispatch declarations to wat/core.wat.
5. Run `cargo test --release --test wat_arc146_dispatch_mechanism`
   — confirm slice 1 tests still pass with the new dispatches
   registered.
6. Verify each migration end-to-end: pick one of the 4 primitives;
   manually call (e.g., `(:wat::core::empty? (:wat::core::Vector
   :wat::core::i64))` should return `true`) — confirm dispatch
   route works.
7. Retire all 4 sets of old machinery.
8. Re-run all baselines.
9. Update arc 144 hardcoded_primitives tests as needed (Q2
   Option A pattern).
10. Run workspace tests + clippy.
11. Report.

Then DO NOT commit. Working tree stays modified.

## Why this slice matters

This is the BUNDLED MIGRATION — 4 of the remaining 5 dispatch-
migration primitives in one sweep. After this slice + slice 4
(rename family) + slice 5 (closure), arc 146 closes; all 10
originally-violating primitives are properly defined; the
substrate has ONE type-system model (schemes + dispatches);
every defined symbol is queryable at runtime via lookup_form.

The user's finish line: **every defined symbol can be queried
at runtime.** This slice closes 4/5 of the remaining gap.

Per § 12 foundation discipline: rhythm. The mechanism is proven;
the pattern is mechanical; ship the migrations; foundation
strengthens with each.
