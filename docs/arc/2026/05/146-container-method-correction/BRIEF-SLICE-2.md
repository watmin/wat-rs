# Arc 146 Slice 2 — Sonnet Brief — Migrate `length` (canonical first migration)

**Drafted 2026-05-03.** Substrate-informed: orchestrator crawled
`src/runtime.rs:4940-4967` (existing eval_length impl with 3-way
match on Vec/HashMap/HashSet), `src/runtime.rs:2658` (eval dispatch
arm), `src/check.rs:3116` (infer_list dispatch arm), `src/check.rs:7797+`
(infer_length impl), `src/check.rs:11463+` (arc 144 slice 3 TypeScheme
fingerprint registration that this slice retires alongside the handler),
`src/stdlib.rs:28-148` (STDLIB_FILES — where new wat files register;
load order matters), `tests/wat_arc143_define_alias.rs:104-130` (the
slice 6 length canary — load-bearing proof), `wat/list.wat` (precedent
file for arc 146 dispatch declarations alongside core).

FM 9 baseline confirmed: wat_arc146_dispatch_mechanism 7/7,
wat_arc144_lookup_form 9/9, wat_arc143_define_alias 2/3 (length
canary still red — THIS SLICE CLOSES IT).

**Goal:** prove the dispatch mechanism works end-to-end on a real
substrate primitive. Migrate `:wat::core::length` from
hardcoded-handler shape to dispatch-over-per-Type-impls shape.
The slice 6 length canary turning GREEN is the load-bearing
verification.

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

## Required pre-reads (in order)

1. **`docs/arc/2026/05/146-container-method-correction/BRIEF-SLICE-1.md`**
   + **SCORE-SLICE-1.md** — slice 1 shipped the dispatch mechanism;
   note the Q3 decision (dispatch wins over primitives in
   lookup_form precedence) which means dispatch takes effect
   even before retirement.
2. **`docs/arc/2026/05/146-container-method-correction/DESIGN.md`**
   — the audit table identifying which primitives are GENUINELY
   polymorphic (length: YES — 3 arms over Vec/HashMap/HashSet).
3. **`docs/COMPACTION-AMNESIA-RECOVERY.md`** § 12 — foundation
   discipline; this slice is the foundation auditing itself.
4. **`src/runtime.rs:4940-4967`** — `eval_length` impl. Three
   branches, one per container. You'll split this into 3
   standalone per-Type impls.
5. **`src/runtime.rs:2658`** — eval dispatch arm
   `":wat::core::length" => eval_length(...)`. RETIRE.
6. **`src/check.rs:3116`** — infer_list dispatch arm
   `":wat::core::length" => return infer_length(...)`. RETIRE.
7. **`src/check.rs:7797-7843`** — `infer_length` impl. RETIRE
   entire function.
8. **`src/check.rs:11463+`** — arc 144 slice 3's TypeScheme
   registration for `:wat::core::length` (the fingerprint).
   RETIRE — this slice replaces it with the per-Type
   registrations.
9. **`tests/wat_arc143_define_alias.rs:104-130`** — the slice 6
   length canary. Source code calls
   `(:user::my-size (:wat::core::Vector :wat::core::i64 10 20 30))`
   and expects "3". After this slice: GREEN.
10. **`src/stdlib.rs:28-148`** — STDLIB_FILES. You add
    `wat/core.wat` here (load order: BEFORE `wat/runtime.wat` so
    the dispatch is registered before the runtime macros that
    might reference it).
11. **`wat/list.wat`** — example of a top-level wat file with
    one form (alias declaration). Your `wat/core.wat` follows
    similar shape but uses `:wat::core::define-dispatch` instead
    of `:wat::runtime::define-alias`.

## What to ship

### 1. Three per-Type runtime impls in `src/runtime.rs`

NEW functions adjacent to `eval_length` (line 4940 area), then
retire `eval_length` itself:

```rust
fn eval_vector_length(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::core::Vector/length".into(),
            expected: 1,
            got: args.len(),
            span: Span::unknown(),
        });
    }
    let v = eval(&args[0], env, sym)?;
    match v {
        Value::Vec(xs) => Ok(Value::i64(xs.len() as i64)),
        other => Err(RuntimeError::TypeMismatch {
            op: ":wat::core::Vector/length".into(),
            expected: "Vec<T>",
            got: other.type_name(),
            span: Span::unknown(),
        }),
    }
}

// Analogous for eval_hashmap_length and eval_hashset_length —
// each takes one arg, type-checks against its container shape,
// returns :i64.
```

### 2. Three per-Type dispatch arms in `src/runtime.rs`'s eval_list_call

Add three new arms in the eval dispatch (search for where slice 1's
dispatch_keyword_head guard sits — the new arms go in the existing
keyword switch BELOW the dispatch_registry guard):

```rust
":wat::core::Vector/length" => eval_vector_length(args, env, sym),
":wat::core::HashMap/length" => eval_hashmap_length(args, env, sym),
":wat::core::HashSet/length" => eval_hashset_length(args, env, sym),
```

### 3. Three per-Type TypeScheme registrations in `src/check.rs::register_builtins`

Add three new registrations adjacent to the arc 144 slice 3 length
registration (which you'll then DELETE — see step 6 below):

```rust
// Arc 146 slice 2 — per-Type length impls. The dispatch
// :wat::core::length (declared in wat/core.wat) routes to these.
env.register(
    ":wat::core::Vector/length".into(),
    TypeScheme {
        type_params: vec!["T".into()],
        params: vec![TypeExpr::Parametric {
            head: "Vec".into(),
            args: vec![TypeExpr::Var("T".into())],
        }],
        ret: TypeExpr::Path(":i64".into()),
    },
);

env.register(
    ":wat::core::HashMap/length".into(),
    TypeScheme {
        type_params: vec!["K".into(), "V".into()],
        params: vec![TypeExpr::Parametric {
            head: "HashMap".into(),
            args: vec![TypeExpr::Var("K".into()), TypeExpr::Var("V".into())],
        }],
        ret: TypeExpr::Path(":i64".into()),
    },
);

env.register(
    ":wat::core::HashSet/length".into(),
    TypeScheme {
        type_params: vec!["T".into()],
        params: vec![TypeExpr::Parametric {
            head: "HashSet".into(),
            args: vec![TypeExpr::Var("T".into())],
        }],
        ret: TypeExpr::Path(":i64".into()),
    },
);
```

### 4. NEW `wat/core.wat` with the dispatch declaration

```scheme
;; wat/core.wat — :wat::core::* dispatches.
;;
;; Substrate dispatches that route polymorphic-name primitives
;; to per-Type impls. Per arc 146 DESIGN: one entity-kind
;; (dispatch) for genuinely-polymorphic primitives; per-Type
;; impls live in Rust as clean rank-1 schemes.
;;
;; Each declaration uses arc 146's :wat::core::define-dispatch
;; (slice 1).

(:wat::core::define-dispatch :wat::core::length
  ((:wat::core::Vector<T>)    :wat::core::Vector/length)
  ((:wat::core::HashMap<K,V>) :wat::core::HashMap/length)
  ((:wat::core::HashSet<T>)   :wat::core::HashSet/length))
```

### 5. Register `wat/core.wat` in `src/stdlib.rs`

Add to STDLIB_FILES (line 28+) at a position BEFORE `wat/runtime.wat`
(line 137) — so the dispatch is registered before any macro that
might reference it. The cleanest spot: after `wat/edn.wat`
(line 130), before the arc 143 comment block introducing
`wat/runtime.wat`:

```rust
// Arc 146 slice 2 — :wat::core::* dispatches. Routes polymorphic
// primitive names (length, get, etc.) to per-Type impls. Loads
// before wat/runtime.wat so dispatches are visible to any macro
// that might reference them.
WatSource {
    path: "wat/core.wat",
    source: include_str!("../wat/core.wat"),
},
```

### 6. Retire the old length machinery

DELETE the following (the dispatch + per-Type impls replace them):

- **`src/runtime.rs:2658`** — the eval dispatch arm
  `":wat::core::length" => eval_length(args, env, sym),`
- **`src/runtime.rs:4940-4967`** — the `eval_length` function
  (entire body)
- **`src/check.rs:3116`** — the infer_list dispatch arm
  `":wat::core::length" => return infer_length(...)`
- **`src/check.rs:7797-7843`** — the `infer_length` function
  (entire body)
- **`src/check.rs:11463+`** — the arc 144 slice 3 TypeScheme
  registration for `:wat::core::length`. The fingerprint is
  superseded by the dispatch.

After retirement: any incremental-evaluator step paths or other
dispatch sites that referenced `:wat::core::length` directly
need verification — sonnet greps for any remaining direct refs
+ retires them.

### 7. Verification

```
cargo test --release --test wat_arc143_define_alias
```

EXPECTED: 3/3 PASS. The slice 6 length canary
`define_alias_length_to_user_size_delegates_correctly` was 2/3
pre-slice; this slice closes it.

```
cargo test --release --test wat_arc146_dispatch_mechanism    # 7/7 unchanged
cargo test --release --test wat_arc144_lookup_form           # 9/9 unchanged
cargo test --release --test wat_arc144_special_forms         # 9/9 unchanged
cargo test --release --test wat_arc144_hardcoded_primitives  # WATCH — this tests the OLD length scheme
cargo test --release --test wat_arc143_lookup                # 11/11 unchanged
cargo test --release --test wat_arc143_manipulation          # 8/8 unchanged
cargo test --release --workspace
```

**Special attention:** `wat_arc144_hardcoded_primitives` tests
the arc 144 slice 3 fingerprint registrations. After this slice
RETIRES the `:wat::core::length` fingerprint, that test's
length-related assertion may break. Sonnet investigates: if
the test asserts `signature-of :wat::core::length` returns a
specific shape, it now needs to assert the DISPATCH form (or
update the test to query the per-Type primitive instead). If
the test is generic enough it may still pass via the dispatch
path. Surface the diagnostic; orchestrator decides.

```
cargo clippy --release --all-targets
```

No new warnings.

## Constraints

- **EDITS ONLY:** `src/runtime.rs` + `src/check.rs` + `src/stdlib.rs`.
- **NEW FILE:** `wat/core.wat`.
- **NO new test file.** The load-bearing proof is the existing
  slice 6 canary. Do not add test files for slice 2.
- **NO commits, no pushes.**
- **Q3 from slice 1 (dispatch precedence over primitives) is in
  effect** — the dispatch already takes over even if you don't
  retire infer_length. Retire anyway for cleanness.

## Open questions sonnet decides

### Q1 — Order of operations: register dispatch first, then retire? Or retire first?

Both work since cargo build is atomic. Sonnet picks an order that
keeps the workspace compilable at intermediate states (probably:
add new impls + schemes + wat file FIRST; then retire OLD code in
one swept commit-shape).

### Q2 — wat_arc144_hardcoded_primitives length test breakage

If retiring the `:wat::core::length` TypeScheme breaks the arc 144
slice 3 hardcoded_primitives test:
- Option A: update the test to query per-Type (e.g.,
  `signature-of :wat::core::Vector/length`) — preserves the test's
  intent
- Option B: update the test to query the dispatch (which returns
  the dispatch declaration form, not a primitive scheme)
- Option C: delete the broken assertion (the dispatch's existence
  IS the new contract)

Recommend Option A — preserves verification of the per-Type
primitives' schemes are queryable via reflection.

### Q3 — Any other consumers of `:wat::core::length` in tests/wat?

Sonnet greps `:wat::core::length` workspace-wide:
- Any direct call sites in substrate stdlib wat files? Update
  to use `:wat::list::reduce`-style if needed (but length is
  polymorphic so callers should KEEP writing `:wat::core::length`
  — it's the dispatch now)
- Any test files asserting length's specific shape? Update or
  retire per Q2 logic

## What success looks like

1. 3 new per-Type length impls in src/runtime.rs.
2. 3 new dispatch arms in eval_list_call.
3. 3 new TypeScheme registrations in register_builtins.
4. New `wat/core.wat` with the dispatch declaration.
5. `wat/core.wat` registered in stdlib.rs at the correct load
   position.
6. Old infer_length + eval_length + their dispatch arms RETIRED.
7. Old `:wat::core::length` TypeScheme fingerprint RETIRED.
8. **Slice 6 length canary turns GREEN** — wat_arc143_define_alias
   3/3.
9. wat_arc144_hardcoded_primitives investigated + adapted (Q2).
10. All other baseline tests unchanged.
11. clippy clean.

## Reporting back

Target ~250-350 words.

1. **Per-Type impls quoted** — verbatim signatures of the 3 new
   eval_* functions.
2. **Dispatch arms quoted** — the 3 new arms in eval_list_call.
3. **TypeScheme registrations quoted** — the 3 new env.register
   blocks.
4. **wat/core.wat content quoted** — verbatim file content.
5. **Retirement summary** — list each deleted item with
   file:line.
6. **Length canary transition** — confirm 3/3 passes; quote the
   summary line.
7. **wat_arc144_hardcoded_primitives status** — Q2 decision +
   what was updated.
8. **Workspace test totals** — confirm baseline failure profile
   shrinks (length canary closed).
9. **clippy** — no new warnings.
10. **Honest deltas** — anything investigated / adapted.

## Sequencing

1. Read pre-reads in order.
2. Add 3 new per-Type runtime impls + dispatch arms (additive;
   workspace still compiles).
3. Add 3 new TypeScheme registrations (additive).
4. Create `wat/core.wat` + register in stdlib.rs (additive; new
   dispatch becomes visible).
5. Run `cargo test --release --test wat_arc143_define_alias` —
   verify the canary turns GREEN now (because dispatch precedence
   from slice 1's Q3 takes effect even without retirement).
6. Retire old infer_length + eval_length + dispatch arms +
   TypeScheme fingerprint.
7. Re-run length canary — confirm STILL green.
8. Run wat_arc144_hardcoded_primitives — investigate breakage;
   adapt per Q2.
9. Run all other baselines — confirm unchanged.
10. Run workspace tests — confirm shrunk failure profile.
11. Run clippy.
12. Report.

Then DO NOT commit. Working tree stays modified.

## Why this slice matters

This is the PROOF SLICE for arc 146. It demonstrates:
- The dispatch mechanism shipped in slice 1 works end-to-end on
  a real substrate primitive
- The slice 6 length canary that has tracked the
  arc-130-→-arc-143-→-arc-144-→-arc-146 cascade for days finally
  turns green
- The substrate's design coherence is restored for one primitive;
  slices 3-7 follow the same shape for the other genuinely-
  polymorphic primitives (empty?, contains?, get, conj) and the
  pure-rename family (assoc/dissoc/keys/values/concat).

Per § 12 foundation discipline: this slice closes the FIRST chain
link of the cascade. Each subsequent migration slice is shorter
because the mechanism does the heavy lifting.

The discipline this proves: orchestrator + sonnet + brief
discipline produces clean migrations on a substantial substrate
foundation. One primitive at a time; each migration ~10-20 min;
each compounds into the impeccable foundation.
