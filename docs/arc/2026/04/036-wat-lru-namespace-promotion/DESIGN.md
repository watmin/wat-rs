# Arc 036 — wat-lru namespace promotion

**Status:** opened 2026-04-23. Cave-quest from the builder's
eye: `:user::wat::std::lru::*` reads noisy. A first-party
crate deserves a first-party name.

**Motivation.** wat-lru lives in `wat-rs/crates/wat-lru/` —
same repo, same author, same release cadence, same review
discipline as baked stdlib. Arc 013 shipped it under
`:user::wat::std::lru::*` alongside the community convention
(CONVENTIONS.md: `:user::*` = community wat crates + user
app code). That conflates two different tiers:

- **First-party extension crates** — workspace members of
  wat-rs. Author-authored, co-released, reviewed in the same
  repo.
- **Third-party community crates** — external repos, added to
  consumer `Cargo.toml`s, independent release cadence, no
  shared review.

The builder's taste named the problem directly:

> i hate the lru cache name... i think we need these as
> `:wat::lru::*`

---

## The mechanism is already in place

Confirmed via `src/freeze.rs:362-368`. Every registration
path has two arms:

| Tier | Registers via | Reserved-prefix gate |
|---|---|---|
| Baked stdlib (compile-time `include_str!`) | `register_stdlib_*` | **BYPASSES** |
| Installed dep sources (via `install_dep_sources`) | `register_stdlib_*` | **BYPASSES** |
| User source | `register_*` | **ENFORCES** |

`stdlib_forms()` concatenates baked + installed into one
stream, both flow through the stdlib-tier path. A
workspace-member crate that registers under `:wat::*` already
works today — the mechanism doesn't distinguish baked-stdlib
from installed-dep-stdlib. The only barrier has been
convention.

---

## The new rule

`:wat::*` prefix — reserved for:
1. Baked stdlib (compile-time, included in wat-rs binary).
2. **Workspace-member crates of wat-rs** — `crates/wat-*/`.

`:user::*` prefix — user code + third-party community crates
(not workspace members; added to consumer `Cargo.toml`s from
crates.io or external git sources).

The rule is operational, not a feature: *being in wat-rs's
workspace IS the bless signal.* Workspace membership means
co-authored, co-reviewed, co-released. Anyone can fork and
add their own `crates/wat-foo/` to their workspace, but that
workspace is theirs — not this one.

---

## The rename

wat-lru's surface shrinks from five segments to three:

| Before | After |
|---|---|
| `:user::wat::std::lru::LocalCache<K,V>` | `:wat::lru::LocalCache<K,V>` |
| `:user::wat::std::lru::LocalCache::new` | `:wat::lru::LocalCache::new` |
| `:user::wat::std::lru::LocalCache::put` | `:wat::lru::LocalCache::put` |
| `:user::wat::std::lru::LocalCache::get` | `:wat::lru::LocalCache::get` |
| `:user::wat::std::lru::CacheService<K,V>` | `:wat::lru::CacheService<K,V>` |
| `:user::wat::std::lru::CacheService/loop` | `:wat::lru::CacheService/loop` |

"wat-lru" as a crate name → `:wat::lru::*` as the namespace.
The crate's identity and its namespace converge. No more
double-nesting "::wat::std::" inside a user-tier prefix.

---

## Why not `:wat::std::lru::*`

Tempting — `:wat::std::*` is where baked stdlib services live
(`:wat::std::service::Console`). But `:wat::std::*` carries
"expressible in wat from the core substrate" semantics
(CONVENTIONS.md's § Core vs stdlib rubric). wat-lru wraps a
Rust crate (`lru = "0.12"`) — it's a Rust-backed
thin-wrapper, not a pure-wat composition.

The honest shape is **one prefix per workspace-member crate**:
`:wat::lru::*` for wat-lru, `:wat::sqlite::*` for future
wat-sqlite, `:wat::redis::*` for wat-redis. Each crate owns
its own sub-namespace at the `:wat::*` root, parallel to the
Rust-backed baked stdlib.

---

## Non-goals

- **No mechanism change.** The bypass is already there. Only
  the CONVENTIONS.md wording and wat-lru's declared paths
  change.
- **No baking wat-lru into wat-rs.** The crate stays
  externalized per arc 013's transitive-composition proof
  (wat-rs root has zero dep on wat-lru). A consumer that
  doesn't declare `deps: [wat_lru]` doesn't get the
  `:wat::lru::*` surface.
- **No historical rewrites.** Arc 013's INSCRIPTION, DESIGN,
  and BACKLOG keep their original `:user::wat::std::lru::*`
  language — they record the decision that was current then.
  Arc 036 supersedes that choice; the audit trail preserves
  both.
