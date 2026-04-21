# Namespace principle

`:wat::` and `:rust::` coexist as sibling namespaces. Rust-sourced types
mirror their real Rust paths, fully qualified. No short forms, no
dropped `std::`, no magic.

## What lives under `:rust::`

Every type whose identity comes from a Rust crate:

| wat-level path | Rust path |
|---|---|
| `:rust::std::io::Stdin` | `std::io::Stdin` |
| `:rust::std::io::Stdout` | `std::io::Stdout` |
| `:rust::std::io::Stderr` | `std::io::Stderr` |
| `:rust::std::collections::HashMap<K,V>` | `std::collections::HashMap<K,V>` |
| `:rust::std::collections::HashSet<T>` | `std::collections::HashSet<T>` |
| `:rust::crossbeam_channel::Sender<T>` | `crossbeam_channel::Sender<T>` |
| `:rust::crossbeam_channel::Receiver<T>` | `crossbeam_channel::Receiver<T>` |
| `:rust::lru::LruCache<K,V>` | `lru::LruCache<K,V>` |

Every `:rust::*` type requires a `(:wat::core::use! :rust::...)` declaration
in any file that references it.

## What stays at the wat level (no `:rust::`)

- **Primitives** — `:i64`, `:f64`, `:bool`, `:String`, `:()`.
- **Type constructors** — `:Option<T>`, `:Vec<T>`, `:(A,B,...)`,
  `:Result<T,E>`.
- **Algebra substrate** — `:holon::HolonAST`.
- **Meta-AST** — `:wat::WatAST`.
- **Kernel types** — `:wat::kernel::HandlePool<T>`,
  `:wat::kernel::ProgramHandle<R>`.
- **Wat-stdlib smart constructors / macros / programs** — everything
  under `:wat::std::*`, which may internally use Rust types but
  presents a wat-level contract. Examples:
  - `(:wat::std::HashMap :(K,V) k1 v1 ...)` — wat-level variadic
    constructor that produces a `:rust::std::collections::HashMap<K,V>`.
  - `(:wat::std::LocalCache::new cap)` — wat wrapper over `:rust::lru::LruCache`.
  - `:wat::std::service::Console` — wat-source program using `:rust::std::io::*` handles.

## Why

Honesty. `:rust::std::io::Stdin` tells the reader exactly where to
look in Rust for the definition. When a wat user sees a type, they
know immediately whether they're touching native wat algebra or an
imported Rust dep. User-facing aliases are a user concern; wat-rs's
own namespace stays honest regardless.

## Coexistence

`:wat::` (language-native, stdlib) and `:rust::` (imported from Rust
crates) are siblings in the type-path namespace. A single wat program
references both freely. `wat-rs/wat/std/LocalCache.wat` is canonical:

```
(:wat::core::use! :rust::lru::LruCache)

(:wat::core::define
  (:wat::std::LocalCache::new<K,V> (capacity :i64)
                                   -> :rust::lru::LruCache<K,V>)
  (:rust::lru::LruCache::new capacity))
```

One file, both namespaces, honest identities.
