# BRIEF — Stone 237.7c — `:wat::core::assoc` as a polymorphic ∀T intrinsic spanning HashMap + Record

**The records-doctrine slice flagged at `DESIGN-STONE-237.7b.md:96`.** Mirror
the Tier-B custom-arm recipe (7b-ii `fef2c8d9` / 7b-iii `2d3259ae` / 7b-iv
`fad1c1c6`) with ONE new shape: matching arg0 against the umbrella
`:wat::Record` Path (not Parametric) and accepting free `∀T` at arg2.

`:wat::core::assoc` today is a HashMap-only `define-alias` (`wat/core.wat:50`).
`:wat::Record/assoc` (`src/runtime.rs:17129`, arc 234.3b `e91860ee`) already
accepts BOTH base and holonic records via Liskov and preserves flavor (base
early-return rebuilds struct only; holonic fallthrough rebuilds BOTH struct +
holon in lockstep — the PARITY invariant). HashMap arm uses the existing
`hashmap_assoc_inner` (`src/runtime.rs:11410`). All runtime parts already exist;
this stone wires the umbrella intrinsic that dispatches into them.

Full ground-truth crawl + design rationale: `DESIGN-STONE-237.7c.md`.
Disconfirming probe: `tests/probe_arc237_7c_assoc_polymorphic.rs` (committed
`9420e850`). The 4 green probe rows are the regression contract; the 2
`#[ignore]`'d rows are the post-stone contract — YOU REMOVE the `#[ignore]`
annotations as part of this sweep.

## The work

### 1. `src/check.rs` — `infer_assoc` helper + dispatch arm

Mint `fn infer_assoc` near `fn infer_get` (`src/check.rs:12513`). Mirror
`infer_get`'s structure with these differences:

- **Arity 3** (collection + key/field + new-value).
- Match `reduce(arg0_ty, subst, env.types())` against TWO shapes:
  - `TypeExpr::Parametric { head, args: targs } if head == "wat::core::HashMap"` →
    - `let key_ty = targs.first().map(...).unwrap_or_else(|| fresh.fresh())`
    - `let val_ty = targs.get(1).map(...).unwrap_or_else(|| fresh.fresh())`
    - Unify arg1 with `key_ty` (K); unify arg2 with `val_ty` (V).
    - Return `apply_subst(&coll_ty, subst)` (type-preserving HashMap<K,V>).
  - `TypeExpr::Path(p) if p == ":wat::Record"` →
    - Unify arg1 with `TypeExpr::Path(":wat::core::keyword".into())`.
    - **arg2 is free ∀T — DO NOT unify it with anything.** (The runtime
      enforces field-type stability per `eval_record_assoc`; check-time field
      narrowing is deferred per arc 232.1 — same as `:wat::Record/assoc` does
      today.)
    - Return `TypeExpr::Path(":wat::Record".into())` (umbrella — flavor is a
      runtime property per Liskov).
  - else → teaching `CheckError::TypeMismatch` with
    `expected: "HashMap<K,V> or :wat::Record"`.
- Add the dispatch arm in `infer_list` adjacent to `:wat::core::get`:
  `":wat::core::assoc" => infer_assoc(...)`.
- Register a fallback `:wat::core::assoc` plain ∀ TypeScheme in
  `register_builtins` near the length/get ∀T entries (mirror 7b-iv exactly —
  the custom arm overrides at infer_list dispatch; the scheme is the fallback
  rank-1 form for the env.get path).

### 2. `src/runtime.rs` — `eval_assoc` + dispatch arm

Mint `fn eval_assoc` near `fn eval_get`. Mirror `eval_get`'s structure:

- Arity 3 check → teaching `RuntimeError::ArityMismatch`.
- Eval arg0 + arg1 + arg2 (preserve span discipline).
- Match raw `arg0_val` (use `eval_inner(&args[0], env, sym)?.value_owned()` per
  the existing pattern):
  - `Value::wat__std__HashMap(_)` → route to `hashmap_assoc_inner(&container, &k, &v)`.
    (Inspect the existing dispatch arm at `src/runtime.rs:5822` for the call
    shape; you may use `eval_hashmap_assoc(args, list_span, env, sym)` directly
    — same outcome, less re-eval.)
  - `Value::wat__Record { .. } | Value::wat__holon__Record { .. }` → delegate
    to `eval_record_assoc(args, list_span, env, sym)`. That function already
    handles BOTH flavors (base early-return arm at 17150–17215, holonic
    fallthrough at 17218+).
  - else → teaching `RuntimeError::TypeMismatch` with
    `expected: "HashMap<K,V> or :wat::Record"`.
- Wire the dispatch arm in `eval_list` next to `:wat::core::get`:
  `":wat::core::assoc" => eval_assoc(args, list_span, env, sym)`.

### 3. `wat/core.wat` — HARD CUT the alias

Delete line 50: `(:wat::runtime::define-alias :wat::core::assoc   :wat::core::HashMap/assoc)`.

Tombstone comment (mirror the 7b family pattern at lines 31–39): "arc 237 Stone
237.7c — `:wat::core::assoc` is now a Rust ∀T intrinsic with custom inference
arm spanning HashMap + Record; see `src/check.rs::infer_assoc` +
`src/runtime.rs::eval_assoc`."

### 4. `tests/probe_arc237_7c_assoc_polymorphic.rs` — un-ignore Record rows

Remove the `#[ignore = "..."]` annotations on:
- `assoc_base_record_returns_base_record_struct_only`
- `assoc_holonic_record_returns_holonic_record_parity_preserved`

After your substrate edits, both must PASS (the post-stone contract).

### 5. KEEP unchanged

- Per-Type leaves (`:wat::core::HashMap/assoc`, `:wat::Record/assoc`) — schemes
  + eval functions + dispatch arms.
- `hashmap_assoc_inner` + `eval_record_assoc` — the workhorses; you call into
  them, don't modify them.
- `DispatchRegistry` — deletion is post-237.8 (arithmetic) territory.
- All other `define-alias` decls (`dissoc`, `keys`, `values`, `concat`) — they
  stay as aliases for now (HashMap-only by their nature; promotion to Record is
  out of arc 237.7c's scope per DESIGN).
- All arithmetic / comparison / time-arith decls — 237.8 territory.

## Scope

- Edits in `src/check.rs` + `src/runtime.rs` + `wat/core.wat` + `tests/probe_arc237_7c_assoc_polymorphic.rs` only.
- NO holon-rs. NO touch of `eval_record_assoc` body. NO touch of
  `hashmap_assoc_inner` body. NO touch of per-Type leaf schemes. NO
  `DispatchRegistry` deletion. NO touch of other alias decls. NO touch of
  arithmetic.
- The probe edits are: un-ignore only. Do NOT change the test bodies, do NOT
  add new tests, do NOT change the 4 already-green rows.

## Verify (RAW commands — no wrapper scripts)

Run these as SEPARATE simple commands, one per line:

- `cargo build --release -p wat` → 0 errors
- `cargo test --release --test probe_arc237_7c_assoc_polymorphic` → `6 passed; 0 failed; 0 ignored` (post-un-ignore)
- `cargo build --release --tests --workspace` → 0 errors (test-build gate)
- `cargo test --release --lib -p wat` → `834 passed; 0 failed` (lib baseline)

Do NOT invoke `./scripts/green-gate.sh` — wrapper scripts get denied; use the
four raw commands above.

## STOP triggers (REJECTION — surface, do not work around)

- If you find yourself modifying `eval_record_assoc` or `hashmap_assoc_inner`
  bodies → STOP. The intrinsic dispatches INTO them; their semantics are
  load-bearing and out of scope.
- If the HashMap arm unifies arg2 with K instead of V (the K-vs-V trap from
  7b-ii) → STOP.
- If the Record arm tries to unify arg2 with anything (it's free ∀T — DO NOT
  unify it) → STOP. Field-type stability is the runtime's job, not the
  check's.
- If `:wat::Record` doesn't reduce to `TypeExpr::Path(":wat::Record")` and
  you're tempted to add an `else if let TypeExpr::...` fallback → STOP and
  surface. The umbrella IS the Path; that's how `:wat::Record/assoc`'s param
  unifies today (`src/check.rs:20019`).
- If you find a runtime path where holonic-record assoc loses flavor (becomes
  base after assoc) → STOP. That would break the parity invariant. The
  existing `eval_record_assoc`'s holonic arm rebuilds both forms; routing
  through it preserves flavor automatically.
- If the un-ignore'd Record rows are still red after your work → STOP and
  surface; do NOT mark them `#[ignore]` again to "make the build green."
  Honest red beats dishonest green.
- Any urge to mint dissoc/keys/values polymorphism, touch other ops, the
  registry, or holon-rs → STOP.

## Definition of done

- All 6 probe tests green; test-build 0 errors; lib 834/0.
- `wat/core.wat` no longer has the `:wat::core::assoc` alias line; the
  tombstone comment is in its place.
- `src/check.rs` has `fn infer_assoc` + dispatch arm + `:wat::core::assoc`
  fallback TypeScheme registration.
- `src/runtime.rs` has `fn eval_assoc` + dispatch arm.
- The probe's two `#[ignore]` annotations are removed.
- Only the four scoped files touched. NO holon-rs, NO per-Type leaf
  modifications, NO HashMap arm K-vs-V swap, NO Record arm arg2 unification,
  NO other alias retirement, NO registry deletion.
- Write `SCORE-STONE-237.7c.md` (sibling); do NOT commit (orchestrator scores
  + commits).
