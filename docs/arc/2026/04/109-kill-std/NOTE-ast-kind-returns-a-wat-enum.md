# NOTE (arc 109 vocabulary) — `ast-kind` must return a wat ENUM, not a Rust-flavoured String

**Filed 2026-07-15 (builder catch, mid arc-294 item 9a floor drive).** Queued
deliberately, NOT built: the floor is being driven to the ONE allowed failure and
this strike RAISES the floor by design before lowering it (the rete-wall pattern,
64 → 76 → 52). Run it as its own clean strike from a green floor.

## The catch (builder's words, 2026-07-15)

Reviewing a new shape-wall the orchestrator had just added to `defservice`:

> *"huh... `_durable-shape (:wat::core::if (:wat::core::= (:wat::core::ast-kind
> durable-fields) "vector")` ... should... that be an enum.. not a string?.."*

and then the mechanism:

> *"wait... we already have enums?... can we just return a wat-defined enum?....
> we can make the rust side return a wat-defined enum and then hand off to a
> sonnet to address the burning heretics calling out to the shadowdancers?"*

Yes on both counts. This note is the queue entry.

## The flaw (grounded)

`eval_ast_kind` (`src/edn_shim.rs`) is a `match` over the `WatAST` variant set
emitting **string literals**:

```rust
let kind = match ast {
    WatAST::IntLit(..)    => "int",
    WatAST::Keyword(..)   => "keyword",
    WatAST::Vector(..)    => "vector",
    WatAST::List(..)      => "list",
    // … 13 total: int float rational bigint bool string nil keyword symbol list vector set map
};
```

So `:wat::core::ast-kind -> :wat::core::String`. **A closed sum, flattened to prose.**
In a types-mandatory language that is a hole:

- `(= (ast-kind x) "vecter")` **compiles and silently never fires** — the exact
  silent-failure class this arc has spent itself extirpating (the rete wall, the
  bare-positional wall). A discriminant typo must not be expressible.
- **No exhaustiveness.** A kind-match cannot be checked for coverage; add a
  `WatAST` variant and nothing screams at the consumers.
- It re-encodes `WatAST`'s variant set as strings in ~46 places instead of naming
  it ONCE.

## The strike

1. **Name the set once**, in `wat/core.wat` (fieldless variants — the syntax is
   live, cf. `(:wat::core::defenum :h::MixDir :wat::enum::Pure :Up :Down)`):

   ```wat
   (:wat::core::defenum :wat::core::AstKind :wat::enum::Pure
     :Int :Float :Rational :BigInt :Bool :String :Nil
     :Keyword :Symbol :List :Vector :Set :Map)
   ```

2. **Rust returns the wat-defined enum.** `eval_ast_kind` yields
   `Value::Enum(EnumValue { type_path: ":wat::core::AstKind", variant_name: "Vector",
   fields: vec![] })`, and `ast-kind`'s declared return type becomes
   `:wat::core::AstKind`. **This is proven** — arc 294 built exactly this shape for
   the (since-reverted) crash sentinel: Rust minting a wat-typed `Value::Enum` with a
   `type_path` the registry knows.

3. **The heretics enumerate themselves.** Every `(= (ast-kind x) "vector")` becomes
   `AstKind` vs `String` — a LOCATED type error. **~46 sites** at filing time, the
   heavy consumers being `wat/fix.wat` (the codemod itself) and `wat/deporder.wat`,
   plus `core.wat`, `Record.wat`, `bracket.wat`, and the two `service.wat` shape-wall
   sites added 2026-07-15. Hand the census to a sonnet: the wall names each heretic
   by span; the fix is mechanical (`"vector"` → `:wat::core::AstKind::Vector`).
   RVINA ERVDIT — the system educates the caller.

4. **Gate:** whole-floor differential, ZERO NEW failures once the census is done;
   the floor spike between (2) and (3) is BY DESIGN.

## Why it belongs to 109 (kill-std)

A Rust `&'static str` standing in for a wat sum type is std-flavour leaking across
the boundary: the discriminant's identity lives in Rust string literals rather than
in the wat type system. Returning a wat-defined enum moves the truth to the wat
side and lets the checker enforce it — the arc's thesis, applied to the macro
surface's own reflection primitive.

## Scope note

`ast-name` (returns the node's name as a `String`) is genuinely a string and is NOT
part of this. Only the KIND — the closed discriminant — moves to an enum.
