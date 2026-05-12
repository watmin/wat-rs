# Arc 170 slice 3 — Gap C V2 BRIEF (top-level `do` splice for `def`/`defn`)

**Sonnet.** Complete arc 136's vision: `:wat::core::do` at top level should splice its children as N siblings UNIFORMLY across every substrate pass — not just the partial coverage arc 157 added for `def` legality. User framing 2026-05-12:

> *"let's get (def ...) and by proxy (defn ...) supported in do blocks - i thought we already did this - let's make my beliefs correct"*

The empirical finding (probed earlier this session): `(do (defn x) (defn y))` at top level **fails** with `:my::helper (call head — not a builtin, not a registered function)`. The error is at resolve-time call-head lookup. The substrate has multiple passes; arc 157 covered `do` splicing for the def-legality check (`check.rs:6848`) and arc 136 covered runtime eval (`runtime.rs:2018`), but the resolve pass + function-registration pass don't see-through.

`defn` expands to `(:wat::core::def name (:wat::core::fn ...))` per `wat/core.wat`. So fixing `def` inside top-level `do` automatically fixes `defn`.

## Goal (precise, narrow scope)

Make these two probes pass:

```rust
#[test]
fn probe_do_def_two_vars_visible() {
    let src = r#"
        (:wat::core::do
          (:wat::core::def :my::helper (:wat::core::fn [] -> :wat::core::i64 42))
          (:wat::core::def :my::main (:wat::core::fn [] -> :wat::core::i64 (:my::helper))))
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new())).expect("freeze");
    assert!(world.symbols().get(":my::helper").is_some());
    assert!(world.symbols().get(":my::main").is_some());
}

#[test]
fn probe_do_defn_via_expansion() {
    let src = r#"
        (:wat::core::do
          (:wat::core::defn :my::helper [] -> :wat::core::i64 42)
          (:wat::core::defn :my::main [] -> :wat::core::i64 (:my::helper)))
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new())).expect("freeze");
    assert!(world.symbols().get(":my::helper").is_some());
    assert!(world.symbols().get(":my::main").is_some());
}

#[test]
fn probe_do_def_via_macro_emission() {
    // The actual Phase E use case: defmacro emits a top-level do.
    let src = r#"
        (:wat::core::defmacro
          (:my::probe (body :AST<wat::core::nil>) -> :AST<wat::core::nil>)
          `(:wat::core::do
             (:wat::core::defn :my::helper [] -> :wat::core::i64 42)
             ~body))

        (:my::probe (:wat::core::defn :my::main [] -> :wat::core::i64 (:my::helper)))
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new())).expect("freeze");
    assert!(world.symbols().get(":my::helper").is_some());
    assert!(world.symbols().get(":my::main").is_some());
}
```

(Empirical: all three currently fail. After Gap C: all three pass.)

## Context — what arcs 136 + 157 actually shipped

**Arc 136** (`docs/arc/2026/05/136-core-do-form/INSCRIPTION.md`) shipped `:wat::core::do` as a substrate special form for VALUE-BEARING sequencing in expression position. Four DESIGN amendments; locked shape: variadic, non-finals' types unconstrained (Clojure-faithful), final's type IS the do's type. **Top-level position was not on arc 136's radar** — the use case was "print, compute, return" inside function bodies.

**Arc 157** (added `:wat::core::def`) later bolted on partial top-level recognition: `def` is documented as legal inside top-level `do` (check.rs:715). The check pass `collect_splice_defs_ctx` (check.rs:6848) recurses into top-level `do` to find `def` forms.

**The gap**: arc 157's recognition is partial. Other substrate passes (resolve, register_defines, etc.) don't see-through. This is incompleteness, not deliberate design — the comprehensive uniform splicing was never explicitly designed.

Gap C closes the door on this. Arc 136's vision was Clojure's `do` semantics; arc 157 extended for `def` legality; Gap C makes it consistent everywhere.

## Required reading IN ORDER

1. **`docs/arc/2026/05/136-core-do-form/INSCRIPTION.md`** — the original do form arc; understand the value-bearing design
2. **`docs/arc/2026/05/157-core-def-form/INSCRIPTION.md`** — the def arc that added partial top-level recognition
3. **`src/check.rs:6848`** — `collect_splice_defs_ctx` `do` arm (existing pattern for splice recognition)
4. **`src/check.rs:715`** — error message documenting `def` inside top-level `do`
5. **`src/runtime.rs:2018-2023`** — `register_runtime_defs` `do` arm (runtime-eval splicing)
6. **`src/resolve.rs`** (and any other resolve-pass file) — sonnet locates the call-head resolution pass; that's where the gap manifests
7. **`wat/core.wat`** `defn` defmacro — confirm defn expands to def-of-fn

## Implementation path

### Phase 1 — Reproduce the failure

Write the three probe tests from above. Run them. Confirm they fail with the resolve-time error. This is the failing baseline; the fix's load-bearing verification is making these pass.

### Phase 2 — Locate every pass that consumes top-level forms

Grep + read to identify ALL substrate passes that walk top-level forms:
- `register_defines` / `register_stdlib_defines` (runtime.rs)
- `register_struct_methods` / `register_enum_methods` / `register_newtype_methods` (runtime.rs)
- `register_defmacros` / `register_stdlib_defmacros` (macros.rs)
- `register_types` / `register_stdlib_types` (types.rs)
- `resolve_references` (resolve.rs) — for call-head resolution
- Any other top-level-form-consumer

For each, check: does it already recurse into `(:wat::core::do ...)`? If yes (like `register_runtime_defs`), the pattern is the mirror. If no, the pass needs the `do` arm added.

### Phase 3 — Extend each missing pass

Add a `do` arm to each pass that walks top-level forms, mirroring the pattern from `register_runtime_defs:2018`:

```rust
WatAST::List(items, _) if matches!(items.first(),
    Some(WatAST::Keyword(k, _)) if k == ":wat::core::do") => {
    for child in &items[1..] {
        // recurse with the same top-level context
        Self::recurse_call(child, ...);
    }
}
```

Each pass's exact recursion mechanism may differ slightly; match the existing function signature.

### Phase 4 — Verify the three probes pass

Run the probe tests; expect all pass. Then full workspace cargo test.

### Phase 5 — Note Phase E V3 readiness

After Gap C ships, Phase E V3 can re-spawn with confidence: deftest macro emits `(:wat::core::do ~@prelude (:wat::core::defn ~name ...))`; top-level `do` now splices uniformly; the prelude defines + test fn all register.

## Scope (what's IN)

- All top-level-form-consuming passes recurse into `(:wat::core::do ...)` uniformly
- The three probe tests pass
- Workspace stays at 0 failed
- SCORE doc

## Scope (what's OUT)

- Phase E V3 (deftest rewrite) — separate, after this ships
- Phase F retirement of run-sandboxed-* — separate
- Slice 4 destructive reap — separate
- The workspace-wide `define` → `defn` rename — separate arc 109 follow-up; NOT this slice
- `let` at top level splicing (arc 157 also mentioned: "(3) inside a top-level `(:wat::core::let ...)` body") — verify whether same gap exists for let, but DEFER fixing if it does; this slice scopes to `do`
- Other Clojure-bias audit candidates (threading macros, when, for, etc.) — separate

## Ship criteria (6 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | All top-level form-consuming passes identified + extended to recurse into `(:wat::core::do ...)` | grep + SCORE inventory |
| B | Probe 1 (def two vars) passes | cargo test |
| C | Probe 2 (defn two helpers) passes | cargo test |
| D | Probe 3 (defmacro-emitted do) passes | cargo test |
| E | Workspace at 0 failed | full cargo test |
| F | `cargo check --release` green | clean |

**6 rows.**

## Predicted runtime

**45-120 min sonnet.** Identifying the passes is the bulk; each pass's `do` arm is small (~5-10 LOC). Workspace verification.

**Hard cap:** 240 min.

## Constraints (hard)

- DO NOT commit. Orchestrator atomic-commits after scoring verification.
- DO NOT touch deftest macro (Phase E V3 work)
- DO NOT modify Layer 1/2 macros / drivers
- DO NOT retire run-sandboxed-* substrate verbs (Phase F)
- DO NOT touch BareLegacy* walker / spawn.rs / Process<I,O> struct fields
- DO NOT rename `define` → `defn` workspace-wide (separate arc 109 follow-up)
- DO NOT extend `let` top-level splicing in this slice (separate concern if it surfaces)
- DO NOT use deferral language in SCORE — per FM 11
- If extending registration passes causes unexpected breakages, STOP and report (root cause per test)
- Workspace must stay at 0 failed

## Honest delta categories (anticipated)

1. **Complete inventory of passes extended** — list every function that got a `do` arm + the rationale per pass
2. **Resolve pass mechanism** — the most subtle one; how does call-head resolution see-through `do`
3. **Top-level `let` splicing** — does the same gap exist for `let` at top level? Confirm or surface as follow-up (don't fix this slice)
4. **Workspace impact** — any tests that change behavior because of newly-recognized `do` splicing (unlikely; would be a real bug surfaced)
5. **Anything unexpected** — surfaced during workspace verification

## Cross-references

- Arc 136 (the do form): `docs/arc/2026/05/136-core-do-form/INSCRIPTION.md`
- Arc 157 (the def form, added partial top-level do recognition): `docs/arc/2026/05/157-core-def-form/INSCRIPTION.md`
- V2 SCORE that revealed the gap: [`SCORE-SLICE-3-PHASE-E-V2-DEFTEST-REWRITE.md`](./SCORE-SLICE-3-PHASE-E-V2-DEFTEST-REWRITE.md)
- Bias audit (captures findings): [`CLOJURE-BIAS-AUDIT-CANDIDATES.md`](./CLOJURE-BIAS-AUDIT-CANDIDATES.md)
- Pattern to mirror: `src/runtime.rs:2018` (`register_runtime_defs` `do` arm)
- Phase E V3 (next): deftest macro rewrite using the now-consistent top-level `do` splice
- Future arc 136 / arc 157 cross-reference doc update: this Gap C ships as the completion that wraps both arcs' top-level-`do` story
