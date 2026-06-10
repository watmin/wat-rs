# DESIGN — Stone 251.4b: `(ann-form expr type)` expression ascription

**Status: STRIKE-READY (drawn 2026-06-10). Probe RED at HEAD before build. Sonnet-built.**

Predecessor: 251.4a (`:-` annotation arrow). Sibling: 251.4c (`:->` fn-type arrow). A NEW
evaluation form, not an arrow swap — check + runtime + registry.

## The form (core.typed parity)

`(ann-form e T)` ascribes type `T` to expression `e`: at check time it asserts `e`'s inferred
type is assignable to `T` and the form's type becomes `T`; at runtime it evaluates `e` and
returns its value unchanged (the type is erased — ascription has no runtime effect). Core.typed's
`ann-form` is exactly this: a checked, type-erased identity. The keyword head is
`:wat::core::ann-form`; the symbol surface `(wat.core/ann-form e T)` resolves via the
251.1b normalize-layer (dual-read, free).

## Semantics (the contract)

- **Check** (`src/check.rs` `infer_list`, ~3659 — add a `":wat::core::ann-form"` arm):
  arity 2 = `[expr, type]`. Parse the type slot with `parse_type_node` (so it accepts the
  keyword `:wat::core::i64`, the `wat.type/i64` symbol, AND the `(wat.type/Vector i64)` form —
  the 251.2a/251.3a surfaces, for free). Infer `expr` → S. Require `S` assignable to `T`
  (the existing assignability/subtype check — same one defn-body-vs-return-type uses). On
  mismatch → a `CheckError` (TypeMismatch-class). Result type = `T`.
- **Runtime** (`src/runtime.rs` `dispatch_keyword_head` ~3291 — add `":wat::core::ann-form" =>
  eval_ann_form`): evaluate `expr` → value; return the value. The type slot is erased (no
  runtime effect). Tail position: the inner `expr` may itself be tail-evaluated, but ann-form
  has no tail-recursive structure of its own — evaluate the expr and return. (If a tail/step
  variant is mechanically required by the dispatch shape, mirror the simplest existing form;
  ann-form adds no new control flow.)
- **Registry** (`src/special_forms.rs` `build_registry`): `insert(&mut m,
  ":wat::core::ann-form", &["<expr>", "<type>"])`.
- **Resolver**: `:wat::core::ann-form` is under the reserved `:wat::` prefix → resolves with no
  change. The symbol form `wat.core/ann-form` normalizes to it (251.1b).

## The probe (RED at HEAD)

`tests/probe_arc251_stone4b_ann_form.rs`:
- **C01 (RED→GREEN):** `(:wat::core::ann-form 41 :wat::core::i64)` in i64-return position
  type-checks AND evaluates to `41`. RED at HEAD (`:wat::core::ann-form` is not a registered
  form → unresolved/check error).
- **C02 (load-bearing — the ascription actually CHECKS):** `(:wat::core::ann-form 42
  :wat::core::String)` is REJECTED (42 is i64, not String) → `checks().is_err()`. Proves
  ann-form is a CHECKED ascription, not a pass-through no-op.
- **C03 (type-slot integration):** `(:wat::core::ann-form 41 wat.type/i64)` checks clean —
  the type slot accepts the `wat.type/` surface (251.2a), confirming `parse_type_node` reuse.

## Out of scope (named)

- `:->` fn-type arrow → **251.4c**.
- The corpus adopting `ann-form` / symbol-head migration → the unified **251.5** sweep.
