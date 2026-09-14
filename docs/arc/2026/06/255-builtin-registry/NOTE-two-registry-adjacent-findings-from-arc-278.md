# NOTE — two registry-adjacent findings, handed over from arc 278

> **Written 2026-08-26 from branch `grok-rete` (arc 278), for whoever is working arc 255 on
> `main`.** Both findings below were surfaced by wards cast at `wat/gen.wat`, both land squarely
> on arc 255's ground, and **neither was fixed on `grok-rete`** — arc 278 had no mandate over
> this surface, and one of them sits exactly where a pending enum task will land.
>
> Builder's routing: *"main's agent is actively working on registry items... can you add a NOTE
> to arc 255 about this issue.... it'll get it when we merge next."*
>
> Everything below was verified against the disk on `grok-rete` at `78e344bac`. If a line here
> disagrees with `main`, trust `main` — these are arc-278 observations of a surface arc 255 owns.

---

## ① `:wat::core::Option` and `:wat::core::Result` are registered in BOTH type stores — every debug test in the tree is red

**Severity: the whole debug profile.** Release is unaffected, which is why nobody noticed.

### Measured

| profile | result |
|---|---|
| `cargo nextest run --test kernel` (debug) | **16 passed, 569 failed** |
| `cargo nextest run --lib` (debug) | **667 passed, 490 failed** |
| either, `--release` | green |

Every failure is the same arm:

```
panicked at src/types.rs:598:9:
builtin leaf :wat::core::Option already registered as a structured TypeDef
```

### Cause

`TypeEnv` keeps two stores that are meant to be **disjoint** — `types` (names *with* structure)
and `builtin_names` (membership *without* structure). `Option` and `Result` are written into both,
by two sites **inside the same function**, `register_builtin_types`:

1. `src/types.rs:1231` — registered as `TypeDef::Enum` (the 2026-08-05 *"Option and Result ARE
   ENUMS"* block) → lands in `types`.
2. `src/types.rs:2746` — the `BARE_CONTAINER_HEADS` loop calls
   `register_builtin_leaf(format!(":{fqdn}"))` → lands in `builtin_names`.

`register_builtin_leaf`'s first `debug_assert!` exists precisely to keep the two disjoint, so it
fires on **every `TypeEnv` construction**. Only 2 of the 7 heads collide — `HashMap`, `HashSet`,
`Vector`, `PersistentMap`, `PersistentVector` have no `TypeDef` and are genuine leaves.

### Age — this is not new, and not gen's doing

Both halves are present in `src/types.rs` at **`10599eb36`** (2026-08-22, *"The type registry
holds the BUILTIN types — THE DOOR tells the truth for the first time"*), **216 commits** before
this note. A ward cast at `wat/gen.wat` merely tripped over it.

### Why no gate caught it

**Both gates run release.** `scripts/floor.sh:96` is `cargo nextest run --release`;
`.github/workflows/ci.yml:96` is `cargo nextest run --profile ci --release`. A `debug_assert!` is
compiled out of both. The tree holds **13 `debug_assert!` in `src/`** and **no gate has ever
exercised one.**

`wat-rs/CLAUDE.md:66` is explicit that this counts: *"A `debug_assert!` panic is a **real
failure**… 'It's only in debug' is the same dismissal wearing a compiler flag."*

### A fix that works — written, measured, and DELIBERATELY REVERTED

In the `BARE_CONTAINER_HEADS` loop:

```rust
for (_bare, fqdn) in crate::check::BARE_CONTAINER_HEADS {
    let name = format!(":{fqdn}");
    if env.get(&name).is_some() {   // has structure => not a leaf, by definition
        continue;
    }
    env.register_builtin_leaf(name);
}
```

Derived from the registry rather than a hardcoded `Option`/`Result` skip-list, so a head that
gains or loses a `TypeDef` later needs no edit here.

- **Measured with it applied:** debug `--test kernel` → **571 passed, 13 failed, 1 timed out**.
- **Release behaviour is byte-identical.** `contains()` (`src/types.rs:579`) already answers `true`
  for both names through `types`, so the second registration bought no lookup — only the collision.

**NOT APPLIED, by the builder's ruling.** It is arc 255 ground, and there is a **pending task to
change how `Option`/`Result`/enums behave** that lands on exactly these lines. `grok-rete` is
unmodified here. Take this with that task rather than as a separate strike.

### ⚠ Before anyone proposes "just add a debug run to the floor"

With the guard applied, debug **still** failed 13 + 1 — every one at exactly the **5000ms `deftest`
budget** or **nextest's 30s ceiling**. That is budget exhaustion in an unoptimized build, not
defects. A debug gate needs those budgets scaled first, or it is red on day one.

---

## ② The retired-name lint is structurally blind to `.wat` — and the stdlib is now a diagnostic surface

**Severity: L2.** No wrong answer; a gate that cannot see half its own population.

`tests/lint/retired_name_justified.rs:1` states the thesis exactly:

> *"THE RETIRED-NAME LINT — a wat name in a Rust string must be a name a user can type… an
> un-caught site here educates toward a retired vocabulary."*

But its scope is two axes wide, and `.wat` is outside both:

- **Root:** `:223` — `collect_rs(&Path::new(manifest).join("src"), &mut files)`. `src/` only.
- **Extension:** `:79` — `p.extension()... == Some("rs")`. `.rs` only.
- **Shape:** the trailing-`'` family only (its own §"Scope" explains why that is sound *for Rust*:
  a `word'` in a `.rs` file must be inside a string or a comment).

### Why this now matters more than it did

When the lint was written, user-facing wat diagnostics lived in Rust. They no longer only do.
`wat/gen.wat` shipped **633 lines into the stdlib on 2026-08-25**, carrying its own `raise`
strings — and six of them named verbs (`gen-elements`, `gen-such-that`, `gen-one-of`, `gen-nth`,
`gen-record`, `gen-nth-str`) that the file's own header declared retired on promotion. A user
grepping a name from a raise message would find nothing.

Those six are **fixed on `grok-rete`** (`78e344bac`) — but only as **stems**. Nothing gates them.
They can rot again tomorrow and no build will say so, because:

> **the stdlib is a first-class diagnostic surface and no gate reads its user-facing strings.**

### Closure, and why it belongs to 255

Extend `collect_rs` with a second pass over `wat/**/*.wat` string literals, carrying a
per-migration retired-token list (`gen-`, plus the `'` family it already knows), with the same
`rune:lint(...)` escape hatch. Same walk; only the root and the token set change.

It lands here rather than in 278 because arc 255 owns **what a name IS** — the registry, the
taxonomy, promotion-vs-relocation (`NOTE-promotion-is-not-relocation-three-gates-ask-what-kind-of-verb-this-is.md`
is the adjacent precedent). This is the same question one step downstream: *once a name is
retired, what stops the substrate from still teaching it?* Today the answer covers `.rs` and stops.

---

## Provenance

Both findings come from the vigilia against `wat/gen.wat`, recorded in full at
`docs/arc/2026/06/278-rules-engine/GEN-VIGILIA-2026-08-25.md` — ① from `excusare` (2026-08-25,
verified and diagnosed 2026-08-26), ② as `circumspicere` finding 4 (2026-08-26). That document is
arc 278's and will not be on an arc 255 reader's path, which is why they are restated here in full
rather than cross-referenced.
