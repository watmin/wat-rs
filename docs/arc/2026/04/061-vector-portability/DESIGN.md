# Arc 061 — `:wat::holon::Vector` (vector portability)

**Status:** shipped 2026-04-26. See `INSCRIPTION.md` for the
canonical post-ship record. Three deltas from the spec worth
flagging:

1. **Audit shrunk the scope.** Arc 052 already shipped
   `:wat::holon::Vector`, `:wat::holon::encode`, and the polymorphic
   `cosine` over (HolonAST, Vector) pairs. The DESIGN's "what's
   missing" table was drafted before that audit. Genuinely-new
   ops reduced to two: `vector-bytes` and `bytes-vector`.
2. **Polymorphism, not sibling verb (Q-revisit).** The DESIGN
   listed `vector-coincident?` as a separate verb. Shipped
   `:wat::holon::coincident?` polymorphism instead, mirroring
   arc 052's `cosine` shape. Same dispatch infrastructure
   (`pair_values_to_vectors`); one fewer surface name to learn.
3. **2-bit-per-cell packing, not 1-bit (Q3 revisit).** The DESIGN
   assumed bipolar `{-1, +1}` vectors; the substrate's encoding
   produces ternary `{-1, 0, +1}` (per
   `holon-rs::deterministic_vector_from_seed`'s `rng % 3`).
   1-bit packing would lose information; 2-bit-per-cell preserves
   the ternary semantics. At d=10000: 2504 bytes total (4-byte
   dim header + 2500 data bytes).

**Predecessor:** arc 057 (typed Holon leaves) — closed the algebra
under itself; arc 058/059/060 — small-shape additions that complete
the substrate's surface as consumer needs surface them.

**Consumer:** experiment 009 (cryptographic-substrate) demonstrates
the substrate's directed-graph + universe-isolation properties via
T1–T6. The verification protocol works END-TO-END within the
substrate today, but vectors are not first-class portable artifacts
— they're computed on-demand inside `cosine` / `coincident?` calls,
materialized briefly, then discarded. Importing a vector from
outside the system, or transmitting one to another party, requires
substrate work. This arc lifts the encoded vector into a wat-level
type with serialize/deserialize and operations that consume it
directly.

**Book reference:** Chapters 61 (adjacent infinities — seed as
universe selector), 62 (axiomatic surface), 64 (TBD — directed
evaluation; drafted as part of the same session that drafted this
arc). The cryptographic substrate has been articulated in the
book; this arc completes the substrate's surface so the
articulation is implementable.

Builder direction (2026-04-26, mid-arc-002):

> we need to describe whatever arc work in wat-rs to enable this...
> how do we lift vectors out into a thing we can wield and pass
> around?...

> i think we must entertain importing an existing vector into the
> system... that vector is coupled to some universe... if you don't
> know which universe you cannot do work....

> we can reduce the transmission down to .... you need to know the
> key (seed)... the program.. and the vector.. if you don't have
> all three you can't do work?...

The three-factor verification UX (V + K + F) needs:
- **K (seed)** — already exists as config-time selection
- **F (program)** — already exists as `:wat::holon::HolonAST`
- **V (vector)** — *missing as a portable wielded type*

This arc adds V.

---

## What's already there (no change needed)

| Surface | Status |
|---------|--------|
| `:wat::holon::HolonAST` (structural form, universe-independent) | shipped |
| `:wat::holon::Atom`, `Bind`, `Bundle`, `Permute`, `Thermometer`, `Blend` (algebra primitives) | shipped |
| `:wat::holon::cosine` (HolonAST × HolonAST → f64) | shipped |
| `:wat::holon::coincident?` (HolonAST × HolonAST → bool) | shipped |
| `:wat::config::set-global-seed!` (universe selection, config-time) | shipped |
| `:wat::config::global-seed` (read current seed) | shipped |
| `:wat::test::run-hermetic-ast` (per-universe forking) | shipped |

The substrate already has the encoding pipeline (HolonAST → vector
under current seed). What's missing is exposing the encoded vector
as a first-class wat value with its own type and operations.

## What's missing (this arc)

| Op | Signature |
|----|-----------|
| `:wat::holon::Vector` | new type — encoded vector as a wieldable value |
| `:wat::holon::encode` | `:wat::holon::HolonAST → :wat::holon::Vector` (encode under current universe) |
| `:wat::holon::vector-cosine` | `:wat::holon::Vector × :wat::holon::Vector → :f64` |
| `:wat::holon::vector-coincident?` | `:wat::holon::Vector × :wat::holon::Vector → :bool` |
| `:wat::holon::cosine` (extended) | accept `(HolonAST, Vector)` and `(Vector, HolonAST)` — encodes the HolonAST first |
| `:wat::holon::vector-bytes` | `:wat::holon::Vector → :Vec<u8>` (serialize) |
| `:wat::holon::bytes-vector` | `:Vec<u8> → :Option<wat::holon::Vector>` (deserialize, fail on bad bytes) |

Seven additions. The mixed-cosine extension is the **verification
primitive**: importer encodes their HolonAST under their universe
and takes cosine to the imported vector. Same universe → meaningful;
different universe → noise.

---

## Decisions to resolve

### Q1 — Type name and namespace

Going with `:wat::holon::Vector` — same namespace as the algebra
operations that produce/consume it.

Alternative considered: `:wat::holon::EncodedVector` (more explicit,
but verbose). Alternative: `:wat::kernel::Vector` (substrate-level,
similar to channel types). The `:wat::holon::*` choice keeps the
type co-located with the operations that produce it.

### Q2 — Should encoding take an explicit seed parameter?

Two options:

- **(a)** `(:wat::holon::encode <holon-ast>) → Vector` — uses current universe's seed
- **(b)** `(:wat::holon::encode <holon-ast> <seed>) → Vector` — explicit seed override

**Recommended: (a).** The substrate already uses config-time seed
selection (`:wat::config::set-global-seed!`); per-call seed override
is unusual and would complicate the encoding pipeline's internal
caching. Multi-universe encoding within one process is a future
extension (see "open questions").

### Q3 — Serialization format

The encoded vector at d=10000 with bipolar values is 10000 i8's.
Packing options:

- **f64 per dim** — 80000 bytes. Wasteful for bipolar vectors.
- **i8 per dim** — 10000 bytes. Simple.
- **1 bit per dim (bipolar)** — 1250 bytes. Optimal for bipolar.

**Recommended: 1-bit-per-dim packed.** Format:
- Bytes 0–3: dim as `u32` little-endian (validation header)
- Bytes 4–: packed bits, LSB-first within each byte
- Total at d=10000: 1254 bytes per vector

This matches Kanerva's bipolar VSA assumption (d entries, each ±1).
For substrates that move to f64 dims (continuous encoding), a
separate format flag would land in a future arc; this one targets
the current bipolar substrate.

### Q4 — Should `bytes-vector` validate against current universe's `dim`?

Pro: catches dim mismatch (vector encoded at d=4096 imported into a
d=10000 universe).
Con: dim is per-config; an ad-hoc cross-universe dim mismatch should
fail the cosine operation gracefully (returns noise, not panic).

**Recommended: validate dim header on deserialize.** Reject (return
`:None`) if the header dim doesn't match current universe's dim.
The cryptographic protocol already handles other mismatch (wrong
seed → noise cosine); dim mismatch is a different category — it's a
structural error the substrate should report cleanly.

### Q5 — Should the Vector carry universe metadata (the seed it was encoded under)?

Pro: makes "wrong universe" detectable as an error rather than a
noise-level mismatch.

Con: defeats the cryptographic protocol — the whole point is that V
+ K are separate factors; bundling K into V makes it a one-factor
system.

**Recommended: NO universe metadata.** The vector is bytes + dim
header. The universe (seed) is the receiver's responsibility to
know. This is the substrate's principled position on the
three-factor verification UX.

### Q6 — Operations on Vectors that don't exist for HolonASTs

Should there be `unbind` for Vectors? `bundle` for Vectors?

**Recommended: NO for v1.** HolonAST is the composition surface;
Vector is for transmission. Composing Vectors directly bypasses the
algebra's structural identity (HolonAST has Hash + Eq; encoded
Vectors don't have a meaningful Hash/Eq beyond byte-equality).

If a future use case surfaces (e.g., distributed compute that
operates on Vectors directly without round-tripping to HolonAST),
a separate arc adds vector-side composition.

### Q7 — Idempotency of `encode`

`encode(holon-ast)` under the same universe should produce the same
Vector deterministically (per the replay-determinism property
demonstrated by experiment 009 T4). `vector-bytes` should be
deterministic (same Vector → same bytes).

**Recommended: deterministic by construction.** The
EncoderRegistry's universe seed determines the encoding; same input
+ same seed → same bytes. Test: encode the same HolonAST twice in
one process; assert byte-equality of the serialized form.

---

## What ships

One slice. Pure additions; non-breaking. Existing HolonAST-based
APIs unchanged.

- New Value variant: `Value::wat__holon__Vector(Arc<EncodedVector>)`
- 7 new substrate primitives (above)
- Type registration: `:wat::holon::Vector` in the substrate's type
  registry
- Tests inline in `src/runtime.rs::mod tests` (matching arcs
  058/059/060 convention)
- Doc: USER-GUIDE update under existing § "Holon" — add a "Vectors"
  subsection

Estimated effort: 200–300 lines of Rust + ~80 lines of inline tests
+ doc updates. Single commit. Mirrors arcs 058/059/060's small-
addition shape.

---

## Open questions

- **Multi-universe co-existence within one process**: a future arc
  could add a `with-seed` form that scopes encoding to a specific
  seed. Would let one process inhabit multiple universes
  simultaneously without forking. Not needed for v1; the hermetic-
  fork pattern (experiment 009) covers the cross-universe case
  adequately.
- **Vector → HolonAST inversion**: probably impossible in general
  (the directed-graph property — scratch arc 002 beat 1 — says
  decode is unbounded). If wanted, a separate "best-fit" decoder
  primitive that uses cleanup to find the nearest HolonAST atom.
- **Vector arithmetic**: should Vectors support `+`/`-` directly, or
  only via `Bind`/`Bundle` on their HolonAST sources? Recommended:
  only via HolonAST sources for v1.
- **Cryptographic-quality serialization**: the recommended bipolar
  packing is space-efficient but doesn't have AEAD or signing. A
  later arc could add `:wat::crypto::*` primitives that wrap
  Vectors with HMAC or signatures.

## Slices

One slice. Single commit. Pattern matches arcs 058/059/060.

## Consumer follow-up

After this arc lands, experiment 009 can grow a T7 step:
- Encode form F under seed_42 in one hermetic child → write Vector
  bytes to stdout
- Read those bytes in the parent (or a second hermetic child)
- Re-encode F under seed_42 in the verifier
- Use mixed cosine (HolonAST × Vector) to verify
- Demonstrate: with seed_42 the verification works; with seed_99
  the cosine is noise

T7 would close the loop on the universe-bound-vectors beat (scratch
arc 002 beat 4) by showing the cryptographic transmission protocol
end-to-end with actual byte-level vector handoff.
