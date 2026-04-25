# wat-rs arc 051 — SimHash + `:wat::holon::simhash` — INSCRIPTION

**Status:** shipped 2026-04-25. Sixth wat-rs arc post-known-good.
The direction-space rule of BOOK Chapter 36's lattice — the
companion to arc 012's value-axis rule.

Builder direction:

> "i think we need this?..."

> "let's get an arc going"

> "we keep the explicit typed expressions but offer a polymorphic
> one - the users chooses" — *the strictness-is-opt-in framing
> set by arc 050; same shape applies here: classical
> SimHash-with-hamming and the cosine-first substrate's
> SimHash-with-bucketing coexist as two valid stories on top of
> the same primitive.*

Three durables:

1. **`:wat::holon::simhash` ships as a single substrate
   primitive.** Maps any `:wat::holon::HolonAST` to a 64-bit i64
   key. Same input → same key. Cosine-similar inputs share keys
   (or differ in few bits). The position allocator: every AST
   gets one i64 slot in a 2^64 lattice, deterministic by
   construction.
2. **The Atom-basis unification is operationalized.** Per BOOK
   Chapter 36, position atoms (used in Permute / Sequential /
   Bigram / Trigram for positional encoding) and LSH anchor
   vectors are the SAME reserved resource — `Atom(0)..Atom(63)`.
   Arc 051's SimHash projects onto these canonical atoms; the
   double-duty becomes operational.
3. **Bidirectional caches are pure composition over `wat-lru`.**
   The DESIGN's earlier draft framed bidirectional cache as a
   follow-up arc; turns out wat-lru's existing `LocalCache` and
   `CacheService` surfaces (per-thread + CSP) cover both shapes.
   `HashMap<i64, V>` keyed by `simhash(ast)` IS the engram cache.
   No new infrastructure needed in the substrate.

**Design:** [`DESIGN.md`](./DESIGN.md).
**Backlog:** [`BACKLOG.md`](./BACKLOG.md).

5 new integration tests; 610 lib tests preserved + new test
crate green; zero clippy.

---

## What shipped

### Slice 1 — runtime primitive

`src/runtime.rs`:

- New `eval_algebra_simhash(args, env, sym)` — single-arg helper
  modeled on `eval_algebra_dot`'s shape.
  - Eval the arg, `require_holon` to extract HolonAST.
  - `require_encoding_ctx` + `require_dim_router` to find the
    natural d for this AST.
  - Encode the AST to a Vector at d via the existing
    `encode(&holon, &vm, &scalar, &registry)` path.
  - For i in 0..64, construct `HolonAST::Atom(Arc::new(i as i64))`,
    encode to atom-Vector, compute `Similarity::dot(&v,
    &atom_vec)`, and set bit i if `dot > 0.0`.
  - Return `Value::i64(key as i64)`.
- One dispatch arm:
  ```rust
  ":wat::holon::simhash" => eval_algebra_simhash(args, env, sym),
  ```

The implementation reuses everything: `encode` (existing
materialization), `Similarity::dot` (existing primitive),
`HolonAST::Atom(Arc::new(i64))` (existing typed-atom construction),
the per-d Encoders + dim-router (existing). Net new code: ~30
lines + the dispatch arm + the doc comment.

### Slice 2 — type checker scheme

`src/check.rs`:

- One scheme registration alongside `:wat::holon::dot`'s:
  ```rust
  env.register(
      ":wat::holon::simhash".into(),
      TypeScheme {
          type_params: vec![],
          params: vec![holon_ty()],
          ret: i64_ty(),
      },
  );
  ```

`(:fn(:wat::holon::HolonAST) -> :i64)`. Type-checker registers
the signature; callers get type-check on input/output without
extra inference machinery.

### Slice 3 — Integration tests

`tests/wat_simhash.rs` ships **5 tests**:

1. **`simhash_deterministic_same_ast`** — same AST, two calls →
   same i64.
2. **`simhash_atom_zero_stable`** — `(simhash (Atom 0))` is
   stable across calls.
3. **`simhash_same_shape_zero_hamming`** — two structurally
   identical AST values produce the same SimHash key.
4. **`simhash_distinct_atoms_distinct_keys`** — `(Atom "alpha")`
   and `(Atom "beta")` produce different SimHash keys (with
   probability ≈ 1 − 2^(-64)).
5. **`simhash_result_works_in_arithmetic`** — `:i64` result
   plugs into `:wat::core::+` and downstream arithmetic.

A 6th test for `LruCache<i64,V>` composition was drafted but
removed — the wat-rs test crate doesn't register the `wat-lru`
shim at startup. The composition pattern lives in arc 051's
DESIGN documentation; `wat-lru`'s own integration tests will
exercise the pattern when an engram-cache consumer materializes.

### Slice 4 — Doc updates

- This INSCRIPTION.
- `wat-rs/docs/USER-GUIDE.md` Forms appendix gains a row for
  `:wat::holon::simhash`.
- `holon-lab-trading/docs/proposals/.../FOUNDATION-CHANGELOG.md`
  — row documenting wat-rs arc 051 + the lattice's
  direction-space-axis completion.

---

## Resolution of design questions

### Q1 — SimHash variant: **Charikar via Atom(i) basis.** ✓

The honest unification of position atoms and LSH anchors. Each
of the 64 hash bits is `sign(v · Atom(i)_at_d)` where Atom(i)
is the canonical position-marker vector. No separate basis
needed.

### Q2 — Key width: **i64 (64 bits).** ✓

Native wat integer width. 64 projections give a clean
locality-vs-collision tradeoff: cosine-1 → 0 bit difference;
cosine-0 → ~32; cosine--1 → ~64.

### Q3 — Cache integration: **composition over `wat-lru`.** ✓

No new infrastructure. Bidirectional engram caches are:

```scheme
((cache :rust::lru::LruCache<i64,wat::holon::HolonAST>)
 (:rust::lru::LruCache::new 4096))

(:rust::lru::LruCache::put cache (:wat::holon::simhash ast) ast)

(:wat::core::match
  (:rust::lru::LruCache::get cache (:wat::holon::simhash query))
                     -> :wat::holon::HolonAST
  ((Some hit) hit)
  (:None      query))
```

`wat-lru`'s `LocalCache` (per-thread) and `CacheService` (CSP)
already cover the per-thread vs shared design space. No new
substrate type.

### Q4 — Reverse lookup: **inherited from `wat-lru`.** ✓

`:rust::lru::LruCache::get k` returns `:Option<V>`. Consumers
pick `V` based on whether they want one-AST-per-key or full
buckets (`:Vec<HolonAST>`). The substrate doesn't have to
choose; the consumer does at the use site.

---

## Sub-fog resolutions

- **6a — Sign-of-zero rule.** RESOLVED: when `v · Atom(i) ==
  0.0` exactly, output bit i = 0 (off). Documented in the
  helper's doc comment. Pathological boundary case;
  locality-sensitivity property holds.
- **6b — Atom basis allocation cost.** RESOLVED: each call
  encodes 64 small atom-ASTs (`HolonAST::Atom(Arc::new(i as
  i64))`). The atom-vector cache inside `VectorManager`
  memoizes; the 64 vectors are computed once per d and reused
  forever. First-call cost is K=64 atom encodes + K dot
  products at the AST's d. Subsequent calls at the same d use
  the cached atom vectors — only 1 input encode + K dot
  products.
- **6c — Test variance bands.** RESOLVED: test 4 asserts only
  that `key_a ≠ key_b` for distinct atom inputs (true with
  probability 1 − 2^(-64)). No hamming-distance assertions; the
  primitive's locality property is exercised structurally
  rather than statistically.

---

## Count

- New runtime support functions: **1** (`eval_algebra_simhash`).
- New runtime primitives: **0** (reuses `encode`, `Similarity::dot`,
  `HolonAST::Atom`, `EncoderRegistry`).
- New `Value` variants: **0**.
- New SymbolTable / CheckEnv fields: **0**.
- Lib tests: **610 → 610** (unchanged; integration crate covers
  the surface).
- Integration tests: **+5** in `tests/wat_simhash.rs`.
- Lab migration: **0** (no current callers; primitive lands now
  so it's ready when Phase 4 EngramLibrary or any
  content-addressed retrieval surfaces).
- Clippy: **0** warnings.

---

## What this arc did NOT add

- **A new cache type in the substrate.** `wat-lru`'s
  `LocalCache` + `CacheService` cover both per-thread and
  shared (CSP) cache shapes; bidirectional engram caches are
  composition over them, not new infrastructure.
- **`Vec<HolonAST>` reverse-lookup API.** Consumer's choice via
  `LruCache<i64, Vec<HolonAST>>` value type; no substrate-side
  reverse-lookup primitive.
- **Hamming-distance primitive.** Classical SimHash pairs with
  hamming distance for graded similarity. We have `cosine` for
  exact ranking inside a bucket; hamming on i64 keys is one Rust
  call away (`(a ^ b).count_ones()`) if we ever want it as a
  `:wat::core::popcount` / `:wat::core::xor` primitive pair.
  Not needed today.
- **`simhash` on raw `:Vec<f64>`.** Inputs are
  `:wat::holon::HolonAST`; vector materialization is internal.
  Same convention as `cosine` and `dot`.
- **Cross-dim hash agreement.** Same AST at different d
  produces different vectors → different SimHash keys. By
  design — the lattice is per-d, same as the existing forward
  cache.
- **Wider integer keys (i128).** Wat doesn't have native i128
  per arc 050's arithmetic decisions.

---

## The unification operationalized

Pre-arc-051: BOOK Chapter 36 named the lattice with two faces.
Arc 012 shipped the value-axis face (geometric bucketing).
The direction-space face was named-but-not-built — claimed to
share the Atom(integer) basis with position-bound encoding,
but no code reified it.

Arc 051: the direction-space face ships. `simhash` projects
onto `Atom(0)..Atom(63)` — the same atoms that Permute and
Sequential rotate against for positional binding. Position
markers and LSH anchors are operationally one resource. The
unification is now in code, not just prose.

---

## Follow-through

- **Phase 4 EngramLibrary** (when it lands) consumes
  `simhash` as the key-derivation function for content-addressed
  engram lookup. No further substrate work needed; just compose
  `simhash` + `wat-lru`.
- **Hamming primitive** opens its own arc if a real consumer
  surfaces. Not needed today.
- **Cross-d cache architecture** opens its own arc if/when
  multi-d lookup becomes a real concern. The current per-d
  approach matches the existing forward cache and aligns with
  the dim-router's tier-pinning.

---

## Commits

- `<wat-rs>` — runtime.rs (`eval_algebra_simhash` + dispatch
  arm) + check.rs (scheme registration) +
  tests/wat_simhash.rs (5 tests) + DESIGN + BACKLOG +
  INSCRIPTION + USER-GUIDE row.

- `<lab>` — FOUNDATION-CHANGELOG.md (row).

---

*these are very good thoughts.*

**PERSEVERARE.**
