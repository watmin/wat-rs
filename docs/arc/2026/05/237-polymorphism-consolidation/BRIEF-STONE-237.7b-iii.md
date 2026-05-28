# BRIEF — Stone 237.7b-iii — `:wat::core::conj` reborn as a `∀T` intrinsic with a CUSTOM inference arm

**Mirror 237.7b-ii (`fef2c8d9`) exactly.** Same custom-arm pattern (Tier B);
strictly smaller surface — two diffs from `contains?`:

1. **2 collection types** instead of 3: `Vector<T>+T` and `HashSet<T>+T`. **NO
   HashMap arm** (`conj` does not operate on HashMap; HashMap insertion is
   `assoc`, which is its own slice in 237.7c).
2. **Return = arg0's collection type** (type-preserving) instead of `bool`.
   `conj(Vector<T>, T)` returns `Vector<T>`; `conj(HashSet<T>, T)` returns
   `HashSet<T>` — the unmodified `coll_ty` from the match arm is the return.

The probe `tests/probe_arc237_7b_intrinsic_typing.rs` (7/7) is the regression
contract — its `conj_*` tests (`conj_vector_preserves_collection_type` +
`conj_wrong_element_rejected_at_check`) MUST stay green.

## The work — read `infer_contains` + `eval_contains` first; mirror with the two diffs

1. **src/check.rs** — find `infer_contains` (the proven Tier-B template from 7b-ii;
   `git -C /home/watmin/work/holon/wat-rs show fef2c8d9 -- src/check.rs` shows
   the exact shape). Add a `":wat::core::conj"` arm in the same `infer_list`
   dispatch family (adjacent to `:wat::core::contains?`), calling a new
   `infer_conj` helper. `infer_conj` mirrors `infer_contains` but:
   - Match `reduce(arg0_ty, subst, env.types())` against TWO shapes:
     * `Parametric { head == "wat::core::Vector", args: [X] }` → unify arg1's
       type with `X`; return `apply_subst(&coll_ty, subst)` (the matched
       collection type, type-preserving).
     * `Parametric { head == "wat::core::HashSet", args: [X] }` → same.
     * else → teaching `CheckError::TypeMismatch` (`expected: "Vector<T> or HashSet<T>"`).
   - On unify-fail of arg1 with element-type → push `CheckError::TypeMismatch`
     (same shape as `infer_contains`'s element-mismatch arm).
   - **Return type is `coll_ty` (the matched Parametric), NOT `bool_ty()`** —
     this is the diff from `contains?`.

2. **src/runtime.rs** — add `eval_conj` mirroring `eval_contains`'s shape: arity 2
   → eval arg0 + arg1 → match raw `arg0_val`:
   - `Value::Vec(xs)` → push arg1, return new `Value::Vec` (functional — clone
     + push, don't mutate the input Arc).
   - `Value::wat__std__HashSet(s)` → insert arg1, return new
     `Value::wat__std__HashSet`.
   - else → teaching `RuntimeError::TypeMismatch` (`expected: "Vector<T> or
     HashSet<T>"`).
   Route to existing per-type leaves (likely `vector_conj_inner` /
   `hashset_conj_inner` per the contains? naming pattern — grep them) if they
   exist and read cleanly. Wire dispatch arm next to `eval_contains`:
   `":wat::core::conj" => eval_conj(...)`.

3. **wat/core.wat** — delete the `(:wat::core::define-dispatch :wat::core::conj ...)`
   decl. Leave a one-line tombstone comment ("arc 237 Stone 237.7b-iii —
   `:wat::core::conj` is now a Rust ∀T intrinsic with custom inference arm; see
   `src/check.rs::infer_conj` + `src/runtime.rs::eval_conj`").

4. **KEEP**: per-type leaves (`:Vector/conj`, `:HashSet/conj`), DispatchRegistry,
   `get` define-dispatch decl (237.7b-iv next), arithmetic.

## Scope note — NO List arm (consistency)

`length` + `empty?` + `contains?` (the shipped exemplars) are Vector/HashMap/
HashSet only — NOT `wat__core__List`. Mirror exactly: do **NOT** add a List arm
to `conj`. List coverage is a separate uniform stone. If you feel the urge,
STOP — out of scope.

## Verify

- `cargo build --release -p wat` → 0 errors.
- `cargo test --release --test probe_arc237_7b_intrinsic_typing` → `7 passed; 0 failed`
  (regression guard; `conj_vector_preserves_collection_type` +
  `conj_wrong_element_rejected_at_check` MUST stay green).
- `./scripts/green-gate.sh` → `green-gate: PASS` (lib 834/0 + test-build 0).

## STOP triggers (REJECTION — surface, do not work around)

- If `(:wat::core::conj <coll> <elem>)` won't resolve to the new intrinsic after
  the decl delete (hidden coupling) — STOP.
- If a wrong-elem call (`(conj (Vector :i64 1 2) "x")`) is NO LONGER rejected at
  check post-migration — STOP. The probe will catch it; if
  `conj_wrong_element_rejected_at_check` flips, the custom arm isn't enforcing.
- If `conj`'s return type loses preservation (e.g.
  `(length (conj (Vector :i64 1 2) 3))` no longer type-checks because the
  result isn't typed as `Vector<i64>`) — STOP. The probe's
  `conj_vector_preserves_collection_type` is the canary.
- Any urge to add a HashMap arm (assoc-via-conj), a List arm, touch other ops,
  the registry, or holon-rs — STOP.

## Definition of done

- All 7 probe tests green; lib 834/0; build gate 0; green-gate PASS.
- `wat/core.wat` no longer has `define-dispatch :wat::core::conj`; `src/check.rs`
  has the custom arm + `infer_conj` helper; `src/runtime.rs` has `eval_conj` +
  the dispatch arm wired.
- Only `src/check.rs` + `src/runtime.rs` + `wat/core.wat` touched. NO holon-rs,
  NO List arm, NO HashMap arm in conj, NO other ops, NO registry deletion, NO
  probe edits.
- Write `SCORE-STONE-237.7b-iii.md` (sibling); do NOT commit (orchestrator
  scores + commits).
