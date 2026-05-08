# Arc 164 — Mint `:wat::core::List<T>` as a proper LinkedList

**Status:** **SKIPPED 2026-05-08** after due-diligence investigation.
The arc is preserved as a record-of-decision: we looked, we chose not
to mint List<T> right now. Revisit later when (a) the language has
stabilized + the ergonomic surface is settled, AND (b) the performance
angle named below has surfaced as a real bottleneck in real workloads.

## Decision record (2026-05-08)

User direction 2026-05-08:
> *"let's skip on list support for now... i don't know if we actually
> want it... just because the heritage has it... doesn't mean we need
> it?... let's do our due diligence to know better."*
>
> *"leave a hint in there that the perf angle is real - we'll move to
> work on the perf once we think the lang is stable and ergonomic."*

### What the investigation found

**At the AST/substrate layer — Vec is workload-correct. No
awkwardness exists.**

| Pattern checked | Hits |
|---|---|
| `Vec::insert(0, ...)` / `Vec::splice(0..0, ...)` (cons-on-Vec) | 0 |
| `vec![head]` then `extend(tail)` (head/tail synthesis on Vec) | 0 meaningful |
| `split_first()` chains (head/tail decomp) | 1 |
| `WatAST::List(...)` destructures | 258 |
| `WatAST::List(...)` constructions | ~5 |
| Mentions of "linked list" / "cons cell" / "persistent list" in `src/` | 0 |

50:1 destructure-to-construct ratio. The lone meaningful
construction site (`runtime.rs:7700`) uses `Vec::push` (right-append
— Vec's natural workload). Quasiquote / macro-expansion walks
templates by index/match, not head/tail recursion. Heritage
("Lisp uses cons cells") doesn't carry past the substrate's actual
access pattern.

**At the wat user-data layer — there IS one workload signal worth
naming.** `:wat::core::first` / `:second` / `:rest` see 82 usages.
The interesting one is `:wat::core::rest`, currently implemented as
`xs[1..].to_vec()` — an O(N) Vec clone every call. When wat code
does head/tail recursion (e.g. `wat/stream.wat:385-398`'s
`drain-items`, `wat/holon/Sequential.wat:36`), each step clones the
tail; total cost over an N-item batch is **O(N²)**. Cons-cell List
would make it O(N).

**The awkwardness is PERFORMANCE, not ERGONOMICS.** wat-level code
reads naturally; the cost is hidden in `:rest`'s implementation.
And mitigations exist that don't require minting a new type:
1. **Refactor head/tail loops as `foldl` / `foldr`** — wat already
   has fold idiom (40+ usages in `wat/core.wat`); the five
   `:rest`-recursion sites could refactor to fold and avoid the
   O(N²) entirely.
2. **Make `:wat::core::rest` return a Vec view** (Vec + offset/range)
   — O(1) rest, no new type.
3. **Persistent vector** (Clojure PersistentVector / `rpds` /
   `im::Vector`) — structural sharing, O(log N) rest. Reuses the
   Vector vocabulary surface.

### Why we're skipping

- AST: zero signal. Minting List for the AST would be heritage cargo.
- User-data: signal exists but is narrow (5 sites in the entire wat
  source tree) and addressable WITHOUT a new collection type. fold
  refactor is small and uses idioms already established.
- Cost of minting now: a whole new `Value::wat__core__List` variant,
  new constructor, head/tail/cons ops, dispatch arm extensions
  (length / empty? / conj / contains?), USER-GUIDE story, EDN render
  rules. Adds surface area for a problem the existing fold idiom
  already solves.
- Heritage argument alone doesn't earn a new type. *"just because
  the heritage has it doesn't mean we need it."*

### When to revisit (the hint)

The performance angle is real and would re-open this arc. The
trigger conditions:

1. The language and ergonomic surface have settled (we're not
   actively reshaping core forms; foundation discipline holds);
   AND
2. A real workload surfaces where head/tail-recursion `:rest`
   patterns are quadratic in a hot path AND the fold-refactor
   mitigation either doesn't apply or has unacceptable cost; OR
3. wat-level code starts naturally building sequences front-first
   (cons-style construction), which would be a positive signal
   for List<T> as a USER-DATA type, not just a perf workaround.

If/when those land, this arc opens with a concrete workload as
its scope statement. Until then: skipped.

## Cross-references

- This INVESTIGATION lives in this DESIGN; no separate research
  doc shipped (kept lightweight per the user's "record that we
  looked and chose this outcome" direction).
- Arc 163 closure on 2026-05-08 retired the OLD `:wat::core::list`
  (lowercase alias for vec). The vocabulary is clear if/when arc 164
  re-opens.
- Arc 165 closure on 2026-05-08 (`tuple` → `Tuple`) brought container
  heads into uniform PascalCase canonical form; the substrate is now
  ready to receive a new container type cleanly without colliding
  with transitional scaffolding.

---

## Original scope (preserved as historical context)

The sections below were written 2026-05-07 when arc 164 was queued.
Preserved as the original scope statement — they describe the work
that WOULD have shipped if arc 164 had proceeded. No edits beyond
this header; the original framing stands as a snapshot of intent
at write time.

### Background (original)

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
