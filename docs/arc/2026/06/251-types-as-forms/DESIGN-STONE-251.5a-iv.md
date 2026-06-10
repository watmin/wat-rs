# DESIGN — Stone 251.5a-iv: `with-children`, the kind-preserving REBUILD

**Status: STRIKE-READY (probe RED at HEAD 2026-06-09). The inverse half of the AST↔walkable bridge.**

## Why

`ast->children` (251.5a-iii) decomposes a `:wat::WatAST` node into a `Vector<:wat::WatAST>`
the `first`/`rest`/`map` vocab walks. The fixer's recursion needs the *inverse* — after a
walk transforms the children, reassemble the node. That is this stone.

## The contract decision (pinned)

`ast->children` is **lossy on kind**: List, Vector, and Set all collapse to a flat `Vec`,
and Map to interleaved k/v. So a naive `children->ast : Vector<WatAST> → WatAST` cannot know
which kind to rebuild — and the breadcrumb's "always build a List" sketch **fails the Honest
question**: it would silently turn a binder `[x :- T]` (a Vector) into a call `(x :- T)`,
corrupting the tree the fixer must preserve byte-faithfully.

The honest primitive is **kind-preserving**, and the template carries the kind:

```
(:wat::core::with-children <template:WatAST> <children:Vector<WatAST>>) -> WatAST
```

→ a node of the **same kind as `template`**, carrying `children`. The round-trip
`(with-children n (ast->children n)) = n` holds for **every** node kind. The corruption
class is made un-expressible (extirpare's top rung), not merely discouraged.

Per-kind rebuild from the template:

| template kind | rebuild |
|---|---|
| `List(_, span)`   | `List(children, span)` |
| `Vector(_, span)` | `Vector(children, span)` |
| `Set(_, span)`    | `Set(children, span)` |
| `Map(_, span)`    | re-pair `children` into `(k, v)` pairs; **odd count → error** (honest) |
| leaf (Symbol/Keyword/literal) | children must be empty → clone the template; **non-empty → error** |

Span: preserve the template's span (write-forms reflows anyway; preserving it costs nothing
and keeps diagnostics honest). The children arrive as `Value::Vec` of `wat__WatAST`; any
element not a `wat__WatAST` → `TypeMismatch`.

## Out of scope = rejected

- A general `ast-kind` discriminant / `Vector?`/`Map?`/`Set?` predicates — node *recognition*
  is the NEXT bridge piece (for deciding *whether* to rewrite a head), separate from *rebuild*.
- Symbol construction (`symbol/from-string`) for the keyword→symbol head swap — also next.
- The `fix-source` transform itself — 251.5a-v, after recognition lands.

## The three rooms (mirror `ast->children`)

1. **`src/edn_shim.rs`** — new `pub fn eval_with_children` directly after `eval_ast_children`
   (ends line 353). Two-arg: arity-check 2, `eval(&args[0])` = template, `eval(&args[1])` =
   children. Match-and-rebuild per the table.
2. **`src/runtime.rs:3312`** — new producer dispatch arm after the `ast->children` arm:
   `":wat::core::with-children" => return crate::edn_shim::eval_with_children(args, list_span, env, sym).map_err(Into::into),`
3. **`src/check.rs:16539`** — new `env.register` after the `ast->children` sig:
   `params: [Path(":wat::WatAST"), Parametric{head:"wat::core::Vector", args:[Path(":wat::WatAST")]}]`,
   `ret: Path(":wat::WatAST")`.

## Gate

- `cargo test --release --test probe_arc251_stone5a_with_children` → 2/2 green (RED at HEAD).
- `cargo build --release` clean; `cargo test --release --lib` 950/0/1 floor held.
