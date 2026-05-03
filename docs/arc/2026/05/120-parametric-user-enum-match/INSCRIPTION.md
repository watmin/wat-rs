# Arc 120 — Parametric user-defined enum match — INSCRIPTION

**Status:** shipped 2026-05-01.
**Closure:** 2026-05-03.

---

## What shipped

The substrate's type checker had a latent bug — parametric user-defined enums (e.g. `:wat::lru::Request<K,V>`) failed `:wat::core::match` with:

```
:wat::core::match: parameter scrutinee
  expects :wat::lru::Request;
  got :wat::lru::Request<K,V>
```

The bug had been latent since arc 048 (user-defined enums). Coverage gap masked it: every wat-test, every example, every lab consumer used **zero** parametric user-defined enums. `Option<T>` and `Result<T,E>` are parametric but bypass the buggy code path via dedicated MatchShape variants. Arc 119's `Request<K,V>` reshape was the **first parametric user enum to exist anywhere in the codebase** — and the first to surface the gap.

Fix landed: the MatchShape resolver normalizes parametric scrutinee against the parametric pattern variants. `Request<K,V>` matches its own variants (`Get(probes, reply-tx)`, `Put(entries, ack-tx)`) without the parameter erasure that triggered the mismatch.

## Why this closes cleanly

The bug surfaced from arc 119's reshape work; the fix unblocked arc 119's continuing work. The DESIGN's "Status: shipped 2026-05-01" header captured the work going in; this INSCRIPTION makes the closure official.

No follow-up arcs spawned; no resolved-design-decisions to record beyond the fix itself. Substrate-as-teacher pattern in motion: the new consumer (Request<K,V>) surfaced a gap; the gap got fixed; coverage extended.

## References

- `docs/arc/2026/05/120-parametric-user-enum-match/DESIGN.md` (provenance + fix sketch)
- `docs/arc/2026/04/119-holon-lru-put-ack/DESIGN.md` (the consumer that surfaced the gap)

---

**Arc 120 — closed.**
