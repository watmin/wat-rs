# DESIGN — STONE J: the `wat::holon` family gets the naming treatment

**Drawn 2026-09-24.** Builder: *"holon was the earliest tooling in wat - doesn't surprise me it never
got the naming treatment, we likely never needed to question it - that time is now - get the names
uniform"*.

## The rule, derived from the tree

A `Value` variant is named after its wat type path with `::` written as `__`:
`:wat::core::fn` → `wat__core__fn`, `:wat::kernel::Sender` → `wat__kernel__Sender`,
`:wat::stream::Stream` → `wat__stream__Stream`, `:wat::WatAST` → `wat__WatAST`.

## The seven that break it — every one is `wat::holon`

| type path (from `src/value/value.rs`) | variant today | becomes | `Value::` refs |
|---|---|---|---|
| `:wat::holon::HolonAST` | `holon__HolonAST` | `wat__holon__HolonAST` | 148 in 23 files |
| `:wat::holon::Vector` | `Vector` | `wat__holon__Vector` | 98 in 31 |
| `:wat::holon::Engram` | `Engram` | `wat__holon__Engram` | 16 in 8 |
| `:wat::holon::OnlineSubspace` | `OnlineSubspace` | `wat__holon__OnlineSubspace` | 15 in 9 |
| `:wat::holon::Reckoner` | `Reckoner` | `wat__holon__Reckoner` | 15 in 9 |
| `:wat::holon::EngramLibrary` | `EngramLibrary` | `wat__holon__EngramLibrary` | 13 in 8 |
| `:wat::holon::Hologram` | `Hologram` | `wat__holon__Hologram` | 12 in 8 |

~317 references, ~80 files. Measured at `e47f008b7`: no `Self::<Name>` uses inside `impl Value`.

## ⛔ What does NOT change

- **EDN tag strings.** `Tag::ns("wat.holon", "Vector")`, `"OnlineSubspace"` etc. are the WIRE FORMAT.
  Only Rust identifiers change.
- **`holon::Vector` (the holon-rs type), `OwnedValue::Vector`, `Edn::Vector`, `"wat::core::Vector"`.**
  `Vector` is the dangerous word: rename ONLY the `Value::Vector` variant and its declaration.
- **Arc history.** 88 `.md` files under `docs/` mention `holon__HolonAST`; arc docs are the record of
  what was true when written — leave them. Update only living docs that name the variant as current.
- **Behaviour.** This is a pure rename; zero semantic change.
- The unprefixed PRIMITIVES (`i64`, `String`, `Vec`, `Option`, `Tuple`, `Aggregate`, …) — core types,
  a separate question, not this stone.
