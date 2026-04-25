# wat-rs arc 051 — SimHash (direction-space quantization)

**Status:** opened 2026-04-24. Sixth wat-rs arc post-known-good.
Builder direction:

> "i think we need this?..."  *(re: arc 013 SimHash)*

> "let's get an arc going"

**Motivation.** The substrate's quantization architecture has
two axes per BOOK Chapter 36 ("The Lattice"):

1. **Value axis** — per-atom geometric buckets, sized by
   `noise-floor × scale`. Each Thermometer-encoded value snaps
   to one of `2√d` cells per atom. Shipped as **arc 012**
   (geometric bucketing in `wat/encoding/scale-tracker.wat` /
   `scaled-linear.wat`).
2. **Direction space** — sphere-wide locality-sensitive hash
   over composed vectors. Cosine-similar vectors collide to
   the same i64 key (or near keys); orthogonal vectors get
   ~half-bit-distance keys. **THIS arc.**

Without the direction-space rule, the cache architecture is
incomplete — input quantized via arc 012, output stays
unbounded. Phase 4 EngramLibrary (engram-by-vector lookup),
OnlineSubspace cleanup, and any content-addressed retrieval
need this primitive.

**The unification.** BOOK Chapter 36 names that **position
atoms** (used in `Permute` / `Sequential` / `Bigram` /
`Trigram` for positional encoding) AND **LSH anchor vectors**
(needed for content-based retrieval) are the SAME reserved
resource: `Atom(integer)`. Arc 051 formalizes this by using
`Atom(0)..Atom(K-1)` as the canonical SimHash projection
basis. Each Atom serves double duty: position-bound bind and
LSH anchor.

---

## What ships

A single new substrate primitive:

```scheme
(:wat::holon::simhash (h :wat::holon::HolonAST) -> :i64)
```

Materializes `h` to a vector at the current d via the existing
encode path, then SimHashes the vector to an i64 key. Output type
is i64; the K=64 binary projection rule below is the body of the
hash.

(Arc 012's INSCRIPTION used `vec-to-int` colloquially as a
type-shape sketch; the actual primitive ships as `simhash`,
honest to the algorithm name. `vec-to-int` would mislead readers
into thinking it operates on a `:Vec<f64>` input — the input is
`:wat::holon::HolonAST`, materialization is internal.)

**Properties:**
- **Locality-sensitive.** `cosine(a, b) ≈ 1` → identical or
  near-identical i64. `cosine(a, b) ≈ 0` → expected hamming
  distance ≈ 32 of 64 bits. `cosine(a, b) ≈ -1` → expected
  hamming distance ≈ 64.
- **Deterministic.** Same input AST + same dim → same i64
  every time. Atom(i) projection vectors are seeded; SimHash
  is pure arithmetic.
- **Dim-dependent.** Same AST → potentially different i64 at
  different `d`. Call sites should treat the int as
  `(ast, d)`-coupled, same as the existing forward cache key.

**Algorithm — Charikar's SimHash via canonical Atom basis:**

```
Given vector v at dimension d:
  for i in 0..64:
    bit_i = sign(v · Atom(i)_at_d)
  key = pack 64 bits into i64
```

Atom(0)..Atom(63) are reserved as the LSH projection basis —
the same atoms that serve as position markers in Permute /
Sequential. The dot product `v · Atom(i)` is well-defined at
any d (the existing Encoders cache materializes Atom(i)
deterministically per seed).

---

## Decisions resolved

### Q1 — SimHash variant

**Charikar's hyperplane SimHash via Atom(i) basis.**

Alternatives considered:
- MurmurHash3 sliding-window over vector bytes — fast but
  doesn't preserve cosine geometry.
- Random hyperplane SimHash with separately-generated
  hyperplanes — works, but introduces a second basis to
  reserve. Misses the BOOK Chapter 36 unification.
- **Charikar SimHash via Atom(0..K-1)** — cosine-preserving,
  reuses existing canonical basis, lands the unification.

Chosen: Charikar via Atom(i). Each bit is `sign(v ·
Atom(i)_at_d)`. Atom(i)'s vector is already deterministically
materialized by `EncoderRegistry.get(d)`; reuse.

### Q2 — Key width

**i64 (64 bits).**

Tradeoffs:
- 32 bits — denser collisions, smaller cache keys, half the
  projection cost.
- 64 bits — balance of collision rate vs storage. Native i64
  matches wat's primitive integer type. Hamming distances
  range cleanly: 0 (identical), ~32 (orthogonal), 64
  (anti-parallel).
- 128 bits — wat doesn't have native i128 (per arc 050's
  arithmetic decisions).

Chosen: 64. Native int width; Atom(0)..Atom(63) is the
projection basis.

### Q3 — Cache integration

**No new infrastructure. Compose `simhash` over `wat-lru`.**

The `wat-lru` sibling crate (already shipping at
`crates/wat-lru/`) wraps `:rust::lru::LruCache<K,V>` with two
surfaces:

- **`:wat::lru::LocalCache`** — per-thread, `ThreadOwnedCell`
  binding, zero Mutex. For single-thread caches.
- **`:wat::lru::CacheService`** — CSP-style worker thread,
  channel-backed access. For shared caches across threads.

A bidirectional engram cache becomes pure composition:

```scheme
;; Per-thread, single-AST-per-key (most-recent-wins):
((cache :rust::lru::LruCache<i64,wat::holon::HolonAST>)
 (:rust::lru::LruCache::new 4096))

(:rust::lru::LruCache::put cache
  (:wat::holon::simhash ast)   ; SimHash key
  ast)                          ; payload

;; Lookup-by-vector:
(:wat::core::match
  (:rust::lru::LruCache::get cache (:wat::holon::simhash query))
                     -> :wat::holon::HolonAST
  ((Some hit) hit)
  (:None      query))   ; bucket miss; caller decides fallback
```

For full-bucket retrieval (many ASTs per key, ranked by exact
cosine), the value type becomes `:Vec<wat::holon::HolonAST>`
and lookup appends/sorts.

**No special cache type needed in the substrate.** No new
infrastructure decision deferred. wat-lru's two surfaces cover
the per-thread vs shared design space; consumers pick at the
use site.

### Q4 — Reverse lookup return shape

**Inherited from `wat-lru`. `Option<V>` for single-value;
`Vec<V>` for buckets — both from existing wat-lru API.**

`:rust::lru::LruCache::get k` returns `:Option<V>`. Same
pattern works for engram lookup: hit returns `Some(ast)`,
miss returns `None`. For bucketed-many-asts, the value type
is `:Vec<HolonAST>` and the consumer cosine-ranks within the
bucket.

The substrate doesn't need to choose `Option` vs `Vec` —
that's the consumer's decision based on whether they want
one-AST-per-key or many. Both shapes work over the same
`simhash → i64` key derivation.

---

## Implementation sketch

`src/runtime.rs`:

- New `eval_vec_to_int(args, env, sym)` — eval the AST arg,
  ask `EncoderRegistry.get(current_d)` for the encoders at
  the current dimension, materialize the vector via the
  existing `encode` path, then run SimHash.
- New private helper `simhash(vec: &Vector, encoders:
  &Encoders) -> i64` — for i in 0..64: compute `vec ·
  encoders.atom_vector(i)`, set bit i = sign of that dot.
- Dispatch arm in eval `match head` block:
  ```rust
  ":wat::holon::simhash" => eval_vec_to_int(args, env, sym),
  ```

`src/check.rs`:

- New scheme registration:
  ```rust
  env.register(
      ":wat::holon::simhash".to_string(),
      TypeScheme {
          type_params: vec![],
          params: vec![holon_ty()],
          ret: i64_ty(),
      },
  );
  ```

`src/vm_registry.rs`:

- (Possibly) add a helper on `Encoders` to fetch `Atom(i)` as
  a vector at d. Likely already exists via the atom-vector
  cache; reuse.

---

## Tests

`tests/wat_simhash.rs`:

1. **Determinism.** Same AST, two `vec-to-int` calls → same
   i64.
2. **Same-input identity.** `(vec-to-int (Atom 0))` returns a
   stable i64; calling twice returns the same.
3. **Cosine-preservation upper bound.** Two orthogonal-by-
   construction holons (e.g., `(Atom :alpha)` and
   `(Atom :beta)`) → expected hamming distance close to 32 (±
   noise band). Test asserts in [20, 44] for d=10000 (the
   variance bound for K=64).
4. **Cosine-preservation lower bound.** Two cosine-near-1
   holons (e.g., the same AST, or AST with epsilon-perturbed
   leaves) → small hamming distance.
5. **Anti-parallel.** `(Atom :alpha)` vs negated/orthogonalized
   form → expected hamming distance close to 64 (or to 32+;
   depending on what "anti-parallel" means in {-1, 0, +1}^d
   substrate).
6. **i64 type.** Return value's type is `:i64`; passes through
   arithmetic with `:wat::core::+` etc.

Test budget: ~6 cases covering the core properties.

---

## Sub-fogs

- **6a — Sign-of-zero corner case. RESOLVED: bit = 0 (off).**
  When the dot product comes out exactly 0 (rare; only when
  positive and negative contributions cancel exactly), the
  output bit is 0 ("no positive signal here"). Documented in
  the `simhash` helper's doc comment. Affects only pathological
  cases at the boundary; locality-sensitivity property holds.
- **6b — Atom basis allocation cost.** Each `vec-to-int` call
  needs Atom(0)..Atom(63) as vectors. The existing Encoders
  has an atom-vector cache. Verify that reading 64 atom
  vectors is cheap (cached) and doesn't force re-materialization.
- **6c — Test variance bands.** SimHash's hamming distance
  for cosine ≈ 0 is expected ≈ 32 with binomial variance √16
  ≈ 4. Test assertions should use a [22, 42] or wider band
  to avoid flakes. Decide based on test runs.

---

## What this arc does NOT add

- **A new cache type in the substrate.** `wat-lru` already
  ships LocalCache + CacheService; bidirectional engram caches
  are composition (`simhash` for keys + LruCache for storage +
  cosine for in-bucket ranking). No new infrastructure.
- **Cross-dim SimHash agreement.** Same AST at different d
  produces different vectors and therefore different SimHash
  keys. The bidirectional cache's key would be `(SimHash, d)`
  same as the existing forward cache. Documented behavior, not
  a defect.
- **Wider integer keys (i128).** Native wat doesn't have i128.
- **Atom-other-than-i64 LSH anchors.** Only `Atom(integer)`
  family is reserved. String-atom or composite-atom basis
  isn't part of the unification.
- **`simhash` on raw `:Vec<f64>`.** The primitive operates on
  `:wat::holon::HolonAST`. Materialization is internal. Direct
  vector input would require exposing `Vector` as a user-tier
  type, which the project explicitly defers (per arc 023's
  PaperEntry design — substrate caches make raw Vector exposure
  unnecessary).

---

## Non-goals

- **Lab adoption.** No callsites today; ship when Phase 4
  EngramLibrary opens. The primitive lands now so it's ready.
- **058-* INSCRIPTION addendum.** Arc 051 is foundational
  algebra (the lattice's direction-space rule). It probably
  warrants a FOUNDATION-CHANGELOG entry; whether a sub-
  proposal addendum is needed depends on whether SimHash gets
  its own 058-NNN slot. For now: FOUNDATION-CHANGELOG row
  only.
- **USER-GUIDE update.** Add row in the Forms appendix when
  the arc lands.
