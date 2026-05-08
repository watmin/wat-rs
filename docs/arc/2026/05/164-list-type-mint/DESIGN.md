# Arc 164 — Mint `:wat::core::List<T>` as a proper LinkedList

**Status:** queued 2026-05-07. Not yet started.

**Gates:** none structurally; arc 163 should close first since it's
retiring the OLD `:wat::core::list` (alias for vec). Arc 164 mints
NEW `:wat::core::List` (capital — proper LinkedList type) on the
cleared vocabulary.

## Background

User direction 2026-05-07: *"we need to introduce rust's
std::collections::LinkedList<T> as a proper :wat::core::List type...
i didn't realize we were overloading what Vec was doing. clojure
uses java's linked list for its source code management - users will
almost always want a vec but list for lisps is like a core
requirement."*

Currently the substrate has:
- `:wat::core::Vector<T>` — backed by `Vec<Value>` (Rust's growable array)
- `:wat::core::list` — was just an alias for vec (being retired in arc 163)

What's missing: a real `:wat::core::List<T>` backed by `std::collections::LinkedList<T>`. Lisps need this. Code-as-data, head/tail decomposition, persistent-list semantics, etc.

## Vocabulary mint (per arc 109 INVENTORY § D — verb-equals-type)

- **Type:** `:wat::core::List<T>` (PascalCase Type position)
- **Constructor verb:** `:wat::core::List` (same name; verb-equals-type per arc 109 slice 1f playbook)
- **Internal storage:** `Value::wat__core__List(Arc<LinkedList<Value>>)`
- **Internal head string:** `"List"` (Parametric { head: "List", args: [T] })

Mirrors the Vec → Vector pattern arc 109 slice 1f settled (capital
type-name + lowercase-or-same-case verb).

## Operations to mint

Core LinkedList ops (Lisp essentials):

| Operation | Wat surface | Semantics |
|---|---|---|
| Construct | `(:wat::core::List :T x1 x2 ... xN)` | Build LinkedList<T> with N elements |
| Cons | `(:wat::core::List/cons x lst)` | Prepend (O(1)) — return NEW list |
| Head | `(:wat::core::List/head lst)` | First element as `Option<T>` |
| Tail | `(:wat::core::List/tail lst)` | All but first as `Option<List<T>>` |
| Length | `(:wat::core::length lst)` | Polymorphic via dispatch (extends arc 146) |
| Empty? | `(:wat::core::empty? lst)` | Polymorphic via dispatch |
| Reverse | `(:wat::core::List/reverse lst)` | New reversed list |
| Conj | `(:wat::core::conj lst x)` | Polymorphic via dispatch — for List, prepend (Lisp semantics: cons not push) |

## Why LinkedList vs other backends

Rust's `std::collections::LinkedList<T>`:
- O(1) push/pop at both ends
- O(1) splice
- Lisp-natural cons/head/tail decomposition

Trade-offs vs `im::List<T>` (HAMT-backed persistent):
- LinkedList is mutable-by-Rust-API (we wrap in Arc for sharing)
- Persistent semantics achieved via "cons returns new Arc" pattern
- LinkedList doesn't structurally share like HAMT, but for Lisp-style code-as-data the shapes are typically small
- Adding `im` as a dep is heavier than std

Decision: start with `std::collections::LinkedList`. If profiling later shows persistent-list structural sharing matters, can swap implementation behind the same wat surface.

## Relationship to Vector

| Use case | Recommended type |
|---|---|
| Sequence of homogeneous data, indexed access | `:wat::core::Vector<T>` |
| Code as data, recursive cons/head/tail patterns | `:wat::core::List<T>` |
| Stack semantics (push/pop one end) | Either; List slightly cheaper at push |
| Iteration | Either |

User-facing guidance in USER-GUIDE: "Vector for data, List for code." Lisps often default to List for everything because their core data structure IS code; wat distinguishes the two intents.

## Substrate work

### Layer 1 — types.rs

Mint `:wat::core::List<T>` parametric:
```rust
env.register_builtin(TypeDef::Primitive(PrimitiveDef {
    name: ":wat::core::List".into(),
    type_params: vec!["T".into()],
}));
```

Or as a typealias to internal `head: "List"` if the typealias pattern is preferred for parametric primitives.

### Layer 2 — runtime.rs

Mint Value variant:
```rust
wat__core__List(Arc<std::collections::LinkedList<Value>>),
```

Mint constructor + ops:
- `eval_list_ctor` (NEW — distinct from existing eval-vector-ctor)
- `eval_list_cons`, `eval_list_head`, `eval_list_tail`, `eval_list_reverse`

### Layer 3 — check.rs

Add `:wat::core::List` to special-form dispatch (mirror `:wat::core::Vector`'s
arm).

### Layer 4 — dispatch (arc 146 polymorphic ops)

Extend `:wat::core::length`, `:wat::core::empty?`, `:wat::core::conj`,
`:wat::core::contains?` etc. arms to include `:wat::core::List<T>`
shape.

### Layer 5 — edn / EDN render

Render List as EDN list `(...)` (parens, not brackets). Vector renders
as `[...]` if we adopt that convention later.

## Slice plan

### Slice 1 — substrate mint

types.rs primitive + runtime.rs Value variant + constructor +
basic ops (cons / head / tail). Tests: round-trip a small List;
cons + decompose.

### Slice 2 — polymorphic ops (extend arc 146 dispatch)

Extend length / empty? / conj / contains? to include List arm.
Tests: each polymorphic op on List.

### Slice 3 — check.rs Vector parallel + USER-GUIDE entry

Type-checker recognizes List as Vector's sibling. USER-GUIDE doc
explaining "Vector for data, List for code." WAT-CHEATSHEET row.

### Slice 4 — closure paperwork

INSCRIPTION + 058 row + cross-references.

## Cross-references

- **Arc 163 slice 3d** — retires the OLD `:wat::core::list` alias.
  Arc 164 mints NEW `:wat::core::List` on the cleared vocabulary
  (different spelling: lowercase verb retired, capital Type minted).
- **Arc 109 slice 1f** — `Vec → Vector` rename precedent. Arc 164
  follows the same verb-equals-type pattern.
- **Arc 146** — dispatch mechanism. List integrates as another
  per-Type impl arm.

## Why arc 164 is the right shape

Four questions:
- Obvious — Lisps need a proper List; substrate had been overloading Vec
- Simple — additive (new type, new variant, new ops); doesn't touch existing Vector
- Honest — names the substrate's missing primitive; distinguishes "data sequence" from "code sequence" intent
- Good UX — Lisp users get the canonical structure; wat doesn't pretend Vec covers the use case

## When this opens

After arc 163 closes (the OLD `:wat::core::list` alias must be
fully retired so the NEW `:wat::core::List` doesn't collide with
transitional scaffolding).
