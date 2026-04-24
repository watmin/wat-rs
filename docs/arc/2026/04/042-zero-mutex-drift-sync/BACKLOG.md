# Arc 042 — BACKLOG

Two slices. The implementation is three Edits.

---

## Slice 1 — `:wat::std::service::Cache` → `:wat::lru::CacheService`

**Status: ready.**

Three occurrences in ZERO-MUTEX.md, each in a different prose
context:
- Line 191 — Tier 3 substrate examples bullet describing the
  L2 caching program.
- Line 297 — Mutex-translation case "I have a complex cache with
  multiple readers and writers." → tier 3 template reference.
- Line 307 — same case, second mention.

All three: `:wat::std::service::Cache<K,V>` →
`:wat::lru::CacheService<K,V>`. The descriptive prose around each
ref still reads correctly under the new name.

## Slice 2 — INSCRIPTION + cross-references

**Status: obvious in shape.**

- `INSCRIPTION.md`.
- `docs/README.md` arc index extended.
- 058 FOUNDATION-CHANGELOG row in lab repo.

---

## Cross-cutting

- Verification: grep for `service::Cache` after slice 1 — should
  return zero.
- Commit per slice.
