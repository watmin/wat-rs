# Arc 036 — wat-lru namespace promotion — INSCRIPTION

**Status:** shipped 2026-04-23. Two slices + doc sweep. Zero
substrate change; pure convention + rename. Cave-quest straight
after arc 035, same session.

**Design:** [`DESIGN.md`](./DESIGN.md).
**Backlog:** [`BACKLOG.md`](./BACKLOG.md).

---

## What shipped

wat-lru's wat surface promoted from the community tier to the
first-party tier. The name shrank from five segments to three:

| Before | After |
|---|---|
| `:user::wat::std::lru::LocalCache<K,V>` | `:wat::lru::LocalCache<K,V>` |
| `:user::wat::std::lru::LocalCache::{new,put,get}` | `:wat::lru::LocalCache::{new,put,get}` |
| `:user::wat::std::lru::CacheService<K,V>` | `:wat::lru::CacheService<K,V>` |
| `:user::wat::std::lru::CacheService/{loop,loop-step,get,put}` | `:wat::lru::CacheService/{loop,loop-step,get,put}` |
| `:user::wat::std::lru::CacheService::{Body,ReplyTx,Request,ReqTx,ReqRx}` | `:wat::lru::CacheService::{Body,ReplyTx,Request,ReqTx,ReqRx}` |

The crate name and its namespace now converge: **wat-lru → `:wat::lru::*`**.

## File layout mirrors the symbols

Builder's catch mid-sweep:

> i think their file names should be.. wat/lru/LocalCache.wat
> wat/lru/CacheService.wat -- in the crates' directory? it
> matches their symbols?

Yes. Files moved into `lru/` subdirectories matching the
namespace segment:

| Before | After |
|---|---|
| `crates/wat-lru/wat/LocalCache.wat` | `crates/wat-lru/wat/lru/LocalCache.wat` |
| `crates/wat-lru/wat/CacheService.wat` | `crates/wat-lru/wat/lru/CacheService.wat` |
| `crates/wat-lru/wat-tests/LocalCache.wat` | `crates/wat-lru/wat-tests/lru/LocalCache.wat` |
| `crates/wat-lru/wat-tests/CacheService.wat` | `crates/wat-lru/wat-tests/lru/CacheService.wat` |

The filesystem path now mirrors the namespace path; a reader
can find the source for `:wat::lru::LocalCache` by walking
`crates/wat-lru/wat/lru/LocalCache.wat`. Same discipline that
applies to baked stdlib — `wat/holon/Trigram.wat` hosts
`:wat::holon::Trigram`.

`src/lib.rs`'s `wat_sources()` updated: both `include_str!`
paths and `WatSource::path` identity strings now point at the
new layout.

## Mechanism audit

`src/freeze.rs:362-368` documents the split clearly:

```rust
// 6. Function definitions. Stdlib defines bypass the reserved-
//    prefix gate (they live under :wat::std::* by design); user
//    defines still go through register_defines where the gate
//    blocks mis-namespaced user source.
let _stdlib_function_residue = register_stdlib_defines(stdlib_post_types, &mut symbols)?;
let residue = register_defines(post_types, &mut symbols)?;
```

Both baked stdlib and installed dep sources (via
`install_dep_sources`) flow through `register_stdlib_*` via
`stdlib_forms()`. The reserved-prefix gate bypass is in place
at every registration type (types, macros, defines). Arc 036 is
pure convention: wat-lru was already privileged enough to
register under `:wat::*`; CONVENTIONS.md just hadn't named the
case for workspace-member crates to do so.

## The new rule (CONVENTIONS.md)

The `External wat crates` section rewrote the tier table:

- **`:wat::<crate>::*`** — first-party workspace-member crates
  of wat-rs (`crates/wat-*/`). Co-authored, co-released,
  co-reviewed in this repo. Promoted because workspace
  membership IS the trust signal.
- **`:user::<org>::<name>::*`** — community general-purpose
  crates, external repos, community scope.
- **`:user::<user-app-tree>::*`** — user's own program code.

The retired tier `:user::wat::std::<crate>::*` — the
stdlib-prefix-buried-inside-user marker — retired entirely. It
double-nested the wat tier inside the user tier, a confusion
the rename resolves.

The CONVENTIONS prose added a "Mechanism vs convention"
explainer — anyone reading the table can now see that the
substrate permits `:wat::*` registration from installed deps,
and that the convention (not the mechanism) is what gates who
actually does so.

## Sweep sites (current-state, non-historical)

- `crates/wat-lru/wat/lru/LocalCache.wat` — define paths.
- `crates/wat-lru/wat/lru/CacheService.wat` — define paths +
  typealias paths + macro references.
- `crates/wat-lru/wat-tests/lru/LocalCache.wat` — four test
  bodies.
- `crates/wat-lru/wat-tests/lru/CacheService.wat` — one test
  body, includes outer-nesting typealias references.
- `crates/wat-lru/src/lib.rs` — module doc comment,
  `wat_sources()` paths + include_str! paths, rustdoc example.
- `crates/wat-lru/Cargo.toml` — description string.
- `examples/with-lru/wat/main.wat` — consumer example program
  body + comments.
- `src/test_runner.rs` — module doc comment.
- `docs/CONVENTIONS.md` — external-wat-crates section rewrite.
- `docs/README.md` — arc-013 entry forward-reference.
- `README.md` — Caches section's LocalCache + CacheService
  bullets.

Arc 013's and 015's INSCRIPTION / DESIGN / BACKLOG files stay
untouched — they record the decision that was current at their
shipping. Arc 036 supersedes that choice; the audit trail
preserves both layers.

## Verification

- `cargo test --workspace` — green across all suites including
  wat-lru's own `wat_suite` (5 wat tests) and the with-loader
  example's smoke test.
- `cargo clippy --workspace --all-targets -- -D warnings` —
  zero warnings (arc 035's recovery holds).
- `cargo run -p with-lru-example --release` — prints `hit`;
  the whole pipeline (wat-lru wat_sources → install → freeze →
  register → compose → run) works under the new paths.

## Precedent for future workspace-member crates

Arc 036 establishes the pattern. When a future `wat-sqlite`,
`wat-redis`, or `wat-postgres` crate lands as a workspace
member:

- Wat files live at `crates/wat-<crate>/wat/<crate>/*.wat`
  (filesystem mirrors namespace).
- `wat_sources()` returns paths like
  `wat-<crate>/<crate>/Connection.wat`.
- The wat source declares `:wat::<crate>::*` symbols.
- The crate ships with `wat` as a path dep in the workspace;
  Cargo is the first-line collision defense (crate names
  globally unique on crates.io + unique within the workspace).
- No special `register_stdlib_for_workspace_member` function is
  needed — `install_dep_sources` already routes through the
  stdlib-tier pipeline.

## Count

- Files renamed: 4 (two `wat/` + two `wat-tests/`).
- Current-state docs updated: 5 files.
- Test count: unchanged (all existing tests pass under new
  paths).
- Lib tests: 590 (arc 035 count holds — this arc is paths, not
  code).
- Zero substrate code changes.

---

*these are very good thoughts.*

**PERSEVERARE.**
