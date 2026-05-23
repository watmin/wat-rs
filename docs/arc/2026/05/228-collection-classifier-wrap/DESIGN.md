# DESIGN — Arc 228 — Substrate collection classifier-wrap (Map/Set/Vector/List as classified instances)

> **SPAWN-BLOCK STATUS (2026-05-22 late, post-arc-225-doctrine):** Arc 228 is spawned by arc 225 per `feedback_spawn_block_winding`. Surfaced during the typed-entities doctrine dialogue that landed during arc 225's BRIEF v3 reshape. Per the discipline:
> - **Arc 228 BLOCKS arc 225's INSCRIPTION**
> - **Arc 228's spawn children: arc 230** (substrate variant retirement) spawns from arc 228 closure
> - The chain: arc 220 ← arc 221 ← arc 224 ← arc 225 ← arc 228 ← arc 230 ← arc 226 ← arc 227

**Opened:** 2026-05-22 (post-compaction, after intueri bridge-naming cast)
**Branch:** `arc-170-gap-j-v5-deadlock-state`
**Depends on:** Arc 225 (bridge naming + Atom narrow + verb family) closes first — needs clean `:wat::holon::Atom` constructor + `:wat::holon::to-holon` / `:wat::holon::from-holon` semantics + clean `:wat::holon::to-wat` / `:wat::holon::from-wat` renames.

## Mission

**Collections are classes. Their instances are classified.** Map, Set, Vector, List become substrate-recognized classifier-wrapped forms at the holon-algebra layer:

```
{:foo 42}  →  (Map {:foo 42})  →  (Bind (Atom "Map") (Bundle (Bind k v) ...))
#{1 2 3}   →  (Set 1 2 3)      →  (Bind (Atom "Set") (Bundle ...))
[1 2 3]    →  (Vector 1 2 3)   →  (Bind (Atom "Vector") (Bundle (Bind 0 _) (Bind 1 _) ...))
(1 2 3)    →  (List 1 2 3)     →  (Bind (Atom "List") (Bundle 1 2 3))
```

Per the typed-entities doctrine landed 2026-05-23 evening: *typed entities are bound with their type and data-form*. Collections are typed entities; their substrate encoding follows the universal pattern `(Bind (Atom <ClassName>) (Atom <data>))` — except the data is a Bundle composition not a single Atom.

## Triggering observation

User-articulated 2026-05-22 post-compaction:

> *"228 - yes - maps, sets, vectors, lists are all classes and instance of them will be (bind class instance-data)"*

The typed-entities doctrine's "user-defined types are unlimited via classifier-wrap" insight applied to the BUILT-IN collection family. Type-queryability emerges:

```
(is-Map? x)     ≡  similarity(x's outermost classifier-atom, "Map" prototype-atom)
(is-Set? x)     ≡  similarity(x's outermost classifier-atom, "Set" prototype-atom)
(is-Vector? x)  ≡  similarity(x's outermost classifier-atom, "Vector" prototype-atom)
(is-List? x)    ≡  similarity(x's outermost classifier-atom, "List" prototype-atom)
```

Type-checking becomes VSA similarity. Discrete dispatch replaced by continuous measurement.

## Scope (high-level; stones to be defined)

1. **Substrate constructor verbs** at holon layer:
   - `:wat::holon::Map` — `Vec<(HolonAST, HolonAST)>` → `Bind(Atom("Map"), Bundle(Bind pairs))`
   - `:wat::holon::Set` — `Vec<HolonAST>` → `Bind(Atom("Set"), Bundle(items))`
   - `:wat::holon::Vector` — `Vec<HolonAST>` → `Bind(Atom("Vector"), Bundle(positional Bind pairs))`
   - `:wat::holon::List` — `Vec<HolonAST>` → `Bind(Atom("List"), Bundle(sequential items))`
   - Each a Pascal-Case constructor matching the variant-name family
2. **`:wat::holon::to-holon` extension** for Value-tier collections: when input is `Value::wat__std__HashMap` / `HashSet` / `Vec` / `Tuple` / `wat__core__List`, produces the corresponding classifier-wrapped composition (not bare Bundle as currently)
3. **`:wat::holon::from-holon` extension** for classifier-wrapped collections: recognizes `Bind(Atom("Map"), ...)` and decodes to `Value::wat__std__HashMap`; similar for Set/Vector/List
4. **Type-extraction helper** (preparation for arc 226): given a HolonAST, extract the outermost classifier-atom if present (returns `Option<String>`)
5. **Test cascade** — substrate-as-teacher; cargo errors guide the consumer sweep across wat-side + Rust-side callers
6. **INSCRIPTION** — closes arc 228; unblocks arc 225 INSCRIPTION (when other arc 225 spawn children also close)

## What this arc does NOT do

- Touch holon-rs substrate variants (arc 230 retires Symbol/Keyword/Tag/Nil; arc 228 stays at wat-rs verb-layer + composition-encoding work)
- Mint user-facing type-declaration syntax (`(deftype MyType ...)` — that's arc 227's territory)
- Implement `(is-X? value)` type predicates (arc 226's territory)
- EDN-form constructors at wat-surface (`{...}` / `[...]` / etc. parser-level minting — arc 222's territory; arc 222 USES arc 228's substrate semantics)

## Stones (sketched; ratified at BRIEF time)

| Stone | Scope | Estimate |
|---|---|---|
| 228.1 | Substrate constructor verbs (Map/Set/Vector/List Pascal-Case) | 60-120 min |
| 228.2 | to-holon/from-holon collection extensions | 60-90 min |
| 228.3 | Consumer sweep (wat-side + Rust-side callers) | substrate-as-teacher cascade |
| 228.4 | INSCRIPTION + USER-GUIDE + cross-references | 30-60 min |

Total estimate: 3-5 hours sonnet across the stones.

## Cross-references

- arc 225 DESIGN.md — parent arc; bridge naming + Atom narrow
- arc 222 DESIGN.md — sibling arc; EDN-form named constructors at wat-surface depend on this arc's substrate semantics
- arc 230 DESIGN.md — spawn child; variant retirement built on this arc's collection-wrap pattern
- arc 226 DESIGN.md — spawn child chain; type predicates use this arc's classifier-extraction helper
- [[typed-entities-doctrine]] memory — the doctrine driving this arc
- INTERSTITIAL § 2026-05-23 evening — full doctrine narrative
- INTERSTITIAL § 2026-05-22 post-compaction — timeline + bridge naming dialogue
- `feedback_spawn_block_winding` — parentage discipline
