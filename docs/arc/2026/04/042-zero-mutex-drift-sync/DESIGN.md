# Arc 042 — wat-rs/docs/ZERO-MUTEX.md drift sync

**Opened:** 2026-04-24.
**Status:** notes on disk; one tiny implementation slice + INSCRIPTION.
**Scope:** wat-rs/docs/ZERO-MUTEX.md only. One file, one arc.

## Why this arc exists

`wat-rs/docs/ZERO-MUTEX.md` is the concurrency-architecture
document — the three-tier story (immutable / thread-owned /
program-owned), the case-by-case translation of "would-reach-for-
Mutex" patterns, the empirical claim. 482 lines. Last touched
pre-`5b5fad8` at commit `7b47ddc` (2026-04-21 — the
`wat/std/program → wat/std/service` rename).

**Drift only.** The concurrency story didn't move under arcs
028-037 (those are about config + namespace + algebra surface,
not threading model). Survey returns zero retired-form
occurrences from the standard audit set. The three-tier framing,
HandlePool discipline, spawn/send/recv/select primitives, and
empirical claim are all current.

## What's broken

Three references to `:wat::std::service::Cache<K,V>` (lines
191, 297, 307) — that program moved to wat-lru via arc 013, and
the wat namespace promoted via arc 036. The current path is
`:wat::lru::CacheService<K,V>`.

That's it. Architectural prose is solid throughout. No retired
forms, no stale config setters, no namespace migrations beyond
this one path.

## Out of scope

- `:wat::std::service::Console` (line 183) stays — Console
  didn't move; it still ships at that path.
- Other audit-set docs (`CLAUDE.md` in lab repo). Each gets its
  own arc.

## Why this is honest

The doc's central claim — *zero Mutex, by construction* — is
unaffected by which namespace the cache program lives in. The
three updates are mechanical surface alignment, not substantive
architectural change. Smallest possible arc.
