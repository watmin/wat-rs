# FINDING — duplicate map keys are LEGAL in `.wat` source, and the builder ruled they must not be

**Measured 2026-09-20**, `main` @ `1b5ccb783`, on the built binary.

## The builder's ruling

> *"the duplicate key in a map literal makes sense - the user generated code that should be
> considered an error"*

## The measurement

```
$ cat dup.wat
(wat.core/defn u/f [] :- wat.type/i64 (wat.core/length {:a 1 :a 2}))
$ target/release/wat --check dup.wat
(no output — ACCEPTED)
```

**Three readers, two verdicts:**

| reader | `{:a 1 :a 2}` |
|---|---|
| `clojure.edn` / Clojure's reader | ❌ `Duplicate key: :a` |
| `wat-edn` (since 218.7) | ❌ `duplicate key in map literal` |
| **`wat-reader` / the wat checker** | ✅ **accepted** |

So wat source permits what both its own EDN layer and the language it claims compliance with refuse.

## Why it is filed, not struck

- It is **`wat-reader` + the checker**, not `wat-edn`. Different crate, different arc (300/251).
- **Blast radius is UNMEASURED.** At least 2 tracked files carry a duplicate-key literal
  (`tests/collection/probe_arc215_collection_literal_inference.wat`,
  `tests/collection/probe_arc216_stone1_hashset_roundtrip.wat` — both surfaced by `clojure.edn`
  refusing them with `Duplicate key: 1`). Whether those are deliberate probes of duplicate handling,
  and how many non-test sites exist, is not known. ⛔ **Size it before striking it.**
- A checker rule that reds files nobody examined is the shape that makes a stone unreviewable.

## What a stone would need

1. The rule at the right rung — reader or checker; a *reader* refusal matches Clojure and `wat-edn`.
2. Sets too: `#{1 1}` is the same ruling (`A set is a collection of unique values`).
3. A per-file census of duplicate keys/elements in code positions across the tracked corpus.
4. A decision on the 2 known probes: fix the data, or keep them as negative controls under a rune.
