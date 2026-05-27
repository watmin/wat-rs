# Arc 237 Stone 237.7 — RESHAPED: collection ops → `∀T` intrinsics; kill `define-dispatch`

**Supersedes the original 237.7** ("arc-146 Dispatch → defclauses"). The defclause migration was
disproven this session; the honest shape is intrinsics. Authority: memory
`project_intrinsic_boundary` + the three RED investigation probes
(`probe_arc237_s7_defclause_container_dispatch`, `..._raw_dispatch_surface`, `..._any_necessity`).

## Why the reshape (proven, not chosen)

- The collection ops dispatch on **generic container heads** (`:Vector<T>`, `:HashMap<K,V>` —
  element-agnostic). A defclause clause cannot express that: `unify(Vector<i64>, Vector<T>)` fails
  (`T` is a nominal `Path`, not a var), and there is **no universal binding** to fall back on —
  `:Any` is banned (closed universe, 058-030), generics are Rust-only.
- Therefore the collection ops **must be Rust intrinsics** — the proven `:wat::core::type` shape
  (`∀T. T -> X`, internal `match` on the raw `Value`). The "accept any value" privilege lives in
  the kernel `∀T`, never a user-facing type. `define-dispatch` (the `DispatchRegistry` entity-kind)
  is the redundant fat service; its routing policy collapses into ordinary builtins.
- **Raw-value dispatch, holonic opt-in** (four-questions verdict): `(length [1 2 3])` operates on the
  raw value via `:wat::core::type`-style inspection; `is-Vector?`/holon-space is NOT involved.

## Scope boundary (what this is NOT)

- The **intrinsic/substrate vocabulary** + the `:wat::kernel` → `:wat::linux`/`:wat::chan`/`:wat::io`
  reorg are **arc 109** (task #565), NOT here. This thread touches only `:wat::core::*` and needs
  **zero renaming**. "Intrinsic" is how we *describe* the ops; no namespace churn.
- The `DispatchRegistry` entity is NOT deleted yet — arithmetic (`+'2` etc.) still tenants it.
  Registry deletion lands after 237.8 evacuates arithmetic to defclause.

## Slicing

- **237.7a** — `length` as the recipe-prover. Register `:wat::core::length` (`∀T. T -> :i64`, mirror
  `eval_type` at runtime.rs:16119); delete the `define-dispatch :length` decl (core.wat:12). Leaves +
  registry stand. Behavior-preserving (probe is a regression guard); mechanism-change verified by grep.
- **237.7b** — the rest: `empty?` (`->bool`), `contains?` (`(coll,elem)->bool`), `get`
  (`∀T. (coll,key) -> :Option<T>`), `conj` (`∀T. (coll,elem) -> coll`), `assoc` (multi-impl:
  HashMap + Record — dispatch on raw type, type-preserving). Each mirrors 237.7a's recipe.
- **237.7c** — (after 237.8) delete `DispatchRegistry`/`dispatch.rs` + remaining `define-dispatch` decls.

## Doctrine (the intrinsic boundary — for the eventual 109 doc)

> If a verb needs `∀T`/accept-any/raw-`Value` inspection/an irreducible machine op, it **must be
> computed in the substrate** — an **intrinsic**. wat (the surface) orchestrates intrinsic calls;
> it cannot author them. The boundary is **expressiveness** (the closed type universe), not isolation.
