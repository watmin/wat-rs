# wat-rs arc 049 — newtype value support

**Status:** opened 2026-04-24. Fourth wat-rs arc post-known-good.
Lab arc 023 (exit/trade_atoms vocab) surfaced the gap. PaperEntry
needs three Price fields (entry-price, trail-level, stop-level);
`:trading::types::Price` was declared via `(:wat::core::newtype
:Price :f64)` back in Phase 1.2 but **has never been constructed
or accessed as a value** — the substrate parses the declaration
but never synthesized constructors or accessors.

Same shape gap as arc 048's enum gap. The 058-030 PROPOSAL pinned
newtype as nominal (NOT substitutable for inner; line 538 + 727)
and the FOUNDATION shows the four-form declaration syntax (line
2943-2947) but neither pinned construction or accessor names. We
do that here.

---

## What ships

### Construction — mirrors struct's `/new`

**`(:Type/new <inner>)`** — auto-synthesized constructor.
Signature `(:fn(<Inner>) -> :Type)`. Body invokes the existing
`:wat::core::struct-new` primitive with the type path and the
single arg. Runtime returns `Value::Struct(Arc<StructValue>)` with
`type_name = ":...Type"` + `fields = [inner]`.

```scheme
(:trading::types::Price/new 100.0)
;; ⇒ Value::Struct { type_name: ":trading::types::Price",
;;                   fields: [Value::F64(100.0)] }
```

Same constructor convention as struct's `/new`. The `/` separator
attaches a function to a type path per FOUNDATION line 189
(`/` attaches a function; `::` navigates namespaces; `::new` is
Rust-deps-only).

### Accessor — positional `:Type/0` mirrors Rust's `.0`

**`(:Type/0 <self>)`** — auto-synthesized accessor.
Signature `(:fn(:Type) -> :Inner)`. Body invokes the existing
`:wat::core::struct-field` primitive with index 0.

```scheme
(:trading::types::Price/0 (:trading::types::Price/new 100.0))
;; ⇒ Value::F64(100.0)
```

The `/0` accessor name mirrors Rust's `.0` tuple-struct positional
access. 058-030 PROPOSAL line 538 says newtype "Compiles to Rust:
`struct A(B);`" — a single-field tuple struct whose Rust-native
accessor IS `.0`. Embodying the host language: numeric positional
access, named the same way Rust names it.

Why not `:Type/value` or `:Type/inner`? Both would invent a name
that Rust doesn't have. Newtype's tuple-struct shape gives the
field no name (only an index). Inventing one is mumbling; using
`/0` says exactly what it is — the zeroth (and only) positional
field.

### Nominal distinction — already enforced by the type checker

Today's `expand_alias` (`src/types.rs`) walks `TypeDef::Alias`
only — newtypes pass through unchanged. So
`TypeExpr::Path(":Price")` and `TypeExpr::Path(":f64")` are
distinct types under the unifier; the type-checker rejects
mixing without going through `/new` / `/0`. Zero new check.rs
work needed.

### Atom hashing — distinct from inner via type_name

`Value::Struct` carries `type_name`. Atom's content-addressing
includes the StructValue's full encoding (type_name + fields).
So `(Atom (:Price/new 100.0))` and `(Atom 100.0)` hash to
different vectors — the algebra-level identity preserves the
nominal distinction.

This works for free because we represent newtype values as
`Value::Struct`-of-arity-1. A separate `Value::Newtype` variant
would require parallel hashing paths; reusing `Value::Struct`
gets atom hashing right by reusing the struct path.

---

## Implementation — mirror `register_struct_methods`

`register_newtype_methods(types, sym)` in `src/runtime.rs`,
mirrors `register_struct_methods`'s exact shape:

```rust
pub fn register_newtype_methods(
    types: &TypeEnv,
    sym: &mut SymbolTable,
) -> Result<(), RuntimeError> {
    for (_name, def) in types.iter() {
        let nt = match def {
            TypeDef::Newtype(n) => n,
            _ => continue,
        };
        let nt_type = TypeExpr::Path(nt.name.clone());

        // Constructor: :Type/new(value :Inner) -> :Type
        // body: (:wat::core::struct-new :Type value)
        let constructor_path = format!("{}/new", nt.name);
        let new_func = Function {
            params: vec!["value".into()],
            param_types: vec![nt.inner.clone()],
            ret_type: nt_type.clone(),
            body: Arc::new(WatAST::List(vec![
                WatAST::Keyword(":wat::core::struct-new".into(), Span::unknown()),
                WatAST::Keyword(nt.name.clone(), Span::unknown()),
                WatAST::Symbol(Identifier::bare("value"), Span::unknown()),
            ], Span::unknown())),
            // ... type_params, name, closed_env
        };
        sym.functions.insert(constructor_path, Arc::new(new_func));

        // Accessor: :Type/0(self :Type) -> :Inner
        // body: (:wat::core::struct-field self 0)
        let accessor_path = format!("{}/0", nt.name);
        let zero_func = Function {
            params: vec!["self".into()],
            param_types: vec![nt_type.clone()],
            ret_type: nt.inner.clone(),
            body: Arc::new(WatAST::List(vec![
                WatAST::Keyword(":wat::core::struct-field".into(), Span::unknown()),
                WatAST::Symbol(Identifier::bare("self"), Span::unknown()),
                WatAST::IntLit(0, Span::unknown()),
            ], Span::unknown())),
            // ...
        };
        sym.functions.insert(accessor_path, Arc::new(zero_func));
    }
    Ok(())
}
```

Wired into `freeze.rs` at step 6.7 (after enum methods at 6.5).

---

## What this arc does NOT add

- **New `Value::Newtype` variant.** Reusing `Value::Struct` keeps
  the implementation tight and atom hashing correct without a
  parallel path. If multi-field tuple newtypes ever surface, the
  same struct-with-arity-N representation extends without a new
  variant.
- **New runtime primitive.** `:wat::core::struct-new` and
  `:wat::core::struct-field` already do the work. Newtype is a
  registration-time auto-synthesis; the runtime sees ordinary
  function bodies invoking existing primitives.
- **Match patterns for newtype.** Newtype has no variants —
  match isn't applicable. `:Type/0` is the access surface; if
  the pattern-match shape is ever wanted (`(:Type x) → bind
  inner to x`), it's a follow-up.
- **Generic newtypes.** `(:wat::core::newtype :Wrapper<T> :T)`
  parses today; this arc registers methods only for monomorphic
  newtypes. Parametric newtype methods (auto-instantiation per
  use site) ship when a caller demands it. Lab's three newtypes
  (TradeId, Price, Amount) are all monomorphic.
- **CheckEnv changes.** Type-checker picks up the synthesized
  functions through `from_symbols` automatically; nominal
  distinction was already free via `expand_alias` not walking
  newtypes.

---

## Why `/0` over alternatives

| Candidate | For | Against |
|---|---|---|
| `:Type/0` | Mirrors Rust's `.0` exactly. No invented name. Tuple-struct-honest. | Numeric path segment is novel in wat (struct accessors all named). |
| `:Type/value` | Descriptive. Reads naturally. | Invents a name Rust doesn't have. Namespace-pollutes if multi-field newtype ever lands. |
| `:Type/inner` | Common Rust ecosystem convention for opaque wrappers. | Same invented-name issue. Conflates with "Inner type" vs "the inner value." |
| `:Type/unwrap` | Rust idiom from Option/Result. | Wrong semantics: `unwrap` carries "this might fail" — newtype always has a value. Reject. |

`/0` wins on the "embody the host language" axis. The lexer is
permissive (accepts `/0` in keyword paths today, no parser change
needed). The substrate's `struct-field` primitive already accepts
integer indices.

---

## Sub-fogs

- **(none expected.)** The shape is a clean clone of
  `register_struct_methods`. The type-checker, atom hashing, and
  parser all already accept what we need without modification.

---

## Non-goals

- **058-030 INSCRIPTION addendum.** Ships in a separate edit set
  to the lab's `058-030-types/PROPOSAL.md` once the runtime work
  lands. Same pattern as arc 048's struct-construction
  inscription.
- **Lab callsite migration.** Currently zero; this arc unblocks
  arc 023 to USE Price/Amount/TradeId properly. The migration is
  arc 023's work, not this arc's.
- **USER-GUIDE Forms appendix sync.** Same deferred follow-up
  pattern as arc 048.
