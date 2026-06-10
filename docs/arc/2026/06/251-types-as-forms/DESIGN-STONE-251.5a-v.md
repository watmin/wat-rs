# DESIGN — Stone 251.5a-v: node recognition + construction (the last bridge piece)

**Status: STRIKE-READY target. After this, `fix-source` is writable entirely in wat.**

## Why

`ast->children` (decompose) + `with-children` (rebuild) let a wat transform WALK and
REASSEMBLE a form tree. But the role-inversion needs to (a) RECOGNIZE a node's kind (recurse
a list's head specially; leave a data keyword alone), (b) READ a Symbol/Keyword node's text
(to invert `:wat::core::map` → `wat.core/map`, `<-` → `:-`), and (c) CONSTRUCT the inverted
node. Those three verbs are this stone. They are thin wrappers over `WatAST` (construction
precedent: `WatAST::Symbol(Identifier::bare(s), Span::unknown())`; `ident.as_str()` reads a
symbol; `WatAST::Keyword(String, Span)` stores the leading `:` verbatim).

## The contract (pinned) — text in, text out, VERBATIM

Four primitives, all under `:wat::core::`, all mirroring the spine producers (edn_shim fn +
runtime dispatch arm + check sig):

| primitive | sig | behaviour |
|---|---|---|
| `ast-kind`     | `:wat::WatAST -> :wat::core::String` | total discriminant: `"list" "vector" "set" "map" "symbol" "keyword" "int" "float" "bool" "string" "nil"` |
| `ast-name`     | `:wat::WatAST -> :wat::core::String` | the node's stored token text VERBATIM — Symbol → bare (`"<-"`, `"wat.core/map"`); Keyword → `:`-prefixed (`":wat::core::map"`, `":-"`). Error on any non-named node (list/vector/map/set/literal). |
| `symbol-node`  | `:wat::core::String -> :wat::WatAST` | `WatAST::Symbol(Identifier::bare(s), unknown)` |
| `keyword-node` | `:wat::core::String -> :wat::WatAST` | `WatAST::Keyword(s, unknown)` — **requires `s` to start with `:`** (else MalformedForm; a keyword token without its sigil is not round-trippable) |

**Round-trip identities** (the honesty contract): `(symbol-node (ast-name sym)) = sym` and
`(keyword-node (ast-name kw)) = kw`. The kind CHANGE (keyword head → symbol) is explicit in
WHICH constructor the transform calls — never hidden in the accessor.

`ast-kind` returns a `String`, not a keyword value — deliberately: this is an internal fixer
primitive, and a String dodges keyword-Value construction (Simpler); the transform reads it
once per node as `(= (ast-kind n) "list")`.

## Out of scope = rejected

- The `fix-source` transform itself (251.5a-vi — now UNBLOCKED once this lands).
- Per-kind bool predicates (`Vector?`/`Map?`/…) — `ast-kind` subsumes them; `List?` already
  exists and stays (used by prior probes).
- Construction of List/Vector/Map/Set nodes from scratch — `with-children` over a template
  already covers structural rebuild; the transform never mints a fresh collection node.

## The three rooms (mirror `ast->children` / `with-children`)

1. **`src/edn_shim.rs`** — four new `pub fn`s after `eval_with_children`: `eval_ast_kind`,
   `eval_ast_name`, `eval_symbol_node`, `eval_keyword_node`. All single-arg except none;
   `ast-kind`/`ast-name` take a `:wat::WatAST` (match `Value::wat__WatAST`); `symbol-node`/
   `keyword-node` take a `Value::String`.
2. **`src/runtime.rs`** (after the `with-children` arm ~3313) — four producer dispatch arms.
3. **`src/check.rs`** (after the `with-children` register ~16567) — four TypeScheme registers.

## Gate

- `cargo test --release --test probe_arc251_stone5a_recognition` → green (RED at HEAD).
- `cargo build --release` clean; full suite: only the 4 known nursery deadlock-reds.
