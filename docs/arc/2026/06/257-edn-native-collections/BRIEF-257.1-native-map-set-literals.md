# BRIEF — Strike 257.1: native `WatAST::Map` + `WatAST::Set` value literals

Read `DESIGN.md` (same dir) first — it carries the full context and the 8 pinned
contract decisions. This brief is the room map for the FIRST strike only.

## The work (one paragraph)
wat's AST has native `List` + `Vector` but eagerly desugars `{…}`/`#{…}` at parse
time into `(:wat::core::HashMap …)` / `(:wat::core::HashSet …)` constructor-call
Lists. Introduce first-class **`WatAST::Map(Vec<(WatAST, WatAST)>, Span)`** and
**`WatAST::Set(Vec<WatAST>, Span)`** nodes so map/set **value literals** parse to
native nodes and evaluate to the same `Value::wat__std__HashMap` /
`Value::wat__std__HashSet` they do today. **Destructure is OUT OF SCOPE for this
strike** — leave `WatAST::StructPattern` and the symbol-head brace dispatch exactly
as they are; they are eliminated in the NEXT strike (257.2). The typed constructor
verbs `:wat::core::HashMap` / `:wat::core::HashSet` (explicit `(… :K :V k v)`) also
STAY. After this strike: map/set literals are native nodes; the full workspace stays
green; behavior is unchanged.

## Rooms (read in order; `file:line` from the lair study)
1. `src/ast.rs:34` `enum WatAST` — add `Map(Vec<(WatAST, WatAST)>, Span)` and
   `Set(Vec<WatAST>, Span)`. Then the machinery, each of which has a `Vector` arm to
   mirror: `span()` (~124), `children()` (~208 — **Map flattens pairs to
   `[k,v,k,v,…]`**, Set yields elements), `variant_name()` (~229 → `"map"` / `"set"`),
   `impl Hash` (~276). Add `WatAST::map(pairs)` / `WatAST::set(items)` test
   constructors near `struct_pattern()` (~163) if helpful.
2. `src/parser.rs` — `parse_map_literal_body` (~483) returns `WatAST::Map(pairs, span)`
   instead of the `HashMap`-List; `parse_hashset_literal_body` (~549) returns
   `WatAST::Set(items, span)`. **Keep the `LBrace` content-shape dispatch (~276): a
   symbol-head brace still routes to `parse_struct_destructure_body` /
   `parse_hash_destructure_body` → `StructPattern`** (untouched). Only the
   MapLiteral / HashSet arms change their return node.
3. `src/check.rs` — `infer` dispatch: add `WatAST::Map` / `WatAST::Set` arms that
   reuse the K/V (and T) unification in `infer_hashmap_constructor` (~12886) /
   `infer_hashset_constructor` (~11499) but **skip the leading type-keyword sentinel
   slots** (a literal carries no `:K :V`; start from a fresh type var). The
   `infer_list` head-keyword arms (4180 HashMap / 4207 HashSet) STAY for the verb.
4. `src/runtime.rs` — `eval` dispatch: add `WatAST::Map` / `WatAST::Set` arms reusing
   `eval_hashmap_ctor` / `eval_hashset_ctor` (`src/collection/eval.rs:984/1040`) minus
   the type-keyword-skip. The `eval_list` verb dispatch (3805/3806) STAYS.
   `watast_to_holon` (~13693): add `Map` → `Bind(Atom(String("Map")), Bundle([Bind(k,v),…]))`
   and `Set` → `Bind(Atom(String("Set")), Bundle([…]))`, matching `from_holon_item`'s
   existing `"Map"` / `"Set"` classifier arms (~11553) so the holon round-trip is symmetric.
5. **Metadata-sniff (8 sites)** — today they detect a `{…}` metadata-map by
   `List head == :wat::core::HashMap`. Now metadata `{…}` parses to `WatAST::Map`.
   Introduce ONE helper `is_metadata_map(&WatAST) -> Option<&[(WatAST,WatAST)]>` (or
   `bool` + accessor) that accepts `WatAST::Map` **and** the legacy `List`-with-
   `:wat::core::HashMap`-head, and call it at all 8 sites: `runtime.rs`
   `try_parse_metadata_map` (~1962) + the def/defn/defenum sniffs (~1874),
   `check.rs::infer_def` (~7666), `closure_extract.rs::walk_defenum_form` (~850),
   `types.rs` defenum metadata (~1675), `function/metadata.rs::peel_metadata_preamble`
   (~20). Grep `:wat::core::HashMap"` for `meta_items.first()` / `list_items.first()`
   patterns to find them all.
6. `src/macros/eval.rs` `validate_pure_total` (~232) — add `WatAST::Map` / `WatAST::Set`
   arms marked pure (recurse into children for purity; literal collections are pure).

## Implementation sketch
```rust
// ast.rs
Map(Vec<(WatAST, WatAST)>, Span),
Set(Vec<WatAST>, Span),
// children(): Map => pairs.iter().flat_map(|(k,v)| [k,v]).collect (or push k,v)
// parser.rs parse_map_literal_body: build Vec<(WatAST,WatAST)> from the alternating
//   items, return WatAST::Map(pairs, open_span). (No HashMap head keyword, no Infer slots.)
// check.rs infer: WatAST::Map(pairs,_) => infer_map_literal(pairs, ...)  // fresh K,V; unify
// runtime.rs eval: WatAST::Map(pairs,_) => eval_map_literal(pairs, ...)  // -> Value::wat__std__HashMap
```

## Blast radius
`src/ast.rs`, `src/parser.rs`, `src/check.rs`, `src/runtime.rs`,
`src/collection/eval.rs` (maybe a literal wrapper), `src/macros/eval.rs`,
`src/types.rs`, `src/closure_extract.rs`, `src/function/metadata.rs`. **No new public
API. Do NOT touch `StructPattern`, the symbol-head destructure parsers, or the
`Value` enum.** Runtime `Value` representation is unchanged.

## STOP triggers (halt + report; do not improvise)
1. If making `{…}` a `Map` node breaks a metadata-map site you cannot find via the
   `is_metadata_map` helper (a 9th sniff site), STOP and report it — don't leave a
   silent head-keyword check that now misses `WatAST::Map`.
2. If `infer`/`eval` for the literal cannot reuse the constructor logic without the
   type-keyword sentinels (the abstraction doesn't factor cleanly), STOP and report
   the shape — do not duplicate the whole constructor body.
3. If any test that exercises destructure (`{x y z}` / `{var :field}`) changes
   behavior, STOP — destructure must stay byte-for-byte on `StructPattern` this strike.

## Gate (the kill — run before declaring done)
- `cargo build --release` clean.
- `cargo test --release --workspace --no-fail-fast` — **full workspace green** (the
  ripple-class gate: a new AST variant must compile every exhaustive match).
- The 257.0 probe (`tests/nursery/probe_arc257_keys_destructure.rs`) stays **RED**
  (destructure is the next strike — confirm it still fails on "list/map in binder
  position", not on a parse/compile error).
- Add 2-3 honest tests in `tests/nursery/probe_arc257_native_map_set.rs`: a `{:k v}`
  literal and a `#{x y z}` literal each `eval` to the right `HashMap`/`HashSet` value,
  and (if a WatAST↔EDN path is reachable) render as a native EDN map/set — no
  `#wat-edn.holon/*` tags.

## Expectations (independent scorecard — fixed before the strike)
| what | command | expected |
|---|---|---|
| nodes compile everywhere | `cargo build --release` | clean |
| full surface green | `cargo test --release --workspace --no-fail-fast` | 0 failures |
| map/set literals native | new `probe_arc257_native_map_set` | pass |
| destructure untouched | `cargo test --test nursery probe_arc234` + struct_destructure | pass (unchanged) |
| probe still RED | `cargo test --test nursery probe_arc257_keys` | 2 failures ("binder" error) |

Runtime estimate: 30–60 min. Trap-door: the `children()` flattening of Map pairs must
be consistent between `span`-walk consumers and the generic recursion, or quasiquote /
free-var walks miss map children.
