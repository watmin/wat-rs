# wat-rs arc 061 — Vector portability — INSCRIPTION

**Status:** shipped 2026-04-26. One slice, one commit, ~1 hour of
focused work — well under DESIGN's 200–300 LOC estimate because
arc 052 had already shipped most of what the DESIGN's "what's
missing" table proposed.

Builder direction (2026-04-26, mid-arc-002):

> "we need to describe whatever arc work in wat-rs to enable this...
> how do we lift vectors out into a thing we can wield and pass
> around?..."

> "we can reduce the transmission down to .... you need to know the
> key (seed)... the program.. and the vector.. if you don't have all
> three you can't do work?..."

V (vector) joins K (seed) and F (program / HolonAST) as a portable
wieldable type. The three-factor verification UX has its bytes
contract.

---

## What shipped

| Slice | Module | LOC | Tests | Status |
|-------|--------|-----|-------|--------|
| 1 | `src/runtime.rs` — `eval_holon_vector_bytes` (Vector → `Vec<u8>`, 4-byte dim header + 2-bit-per-cell ternary packing); `eval_holon_bytes_vector` (`Vec<u8>` → `Option<Vector>`, dim-header validation); `eval_algebra_coincident_q` rewritten to use `pair_values_to_vectors` (polymorphic over HolonAST × Vector pairs in any combination, mirroring arc 052's cosine shape); 2 dispatch arms. `src/check.rs` — `infer_polymorphic_holon_pair_to_bool` (sibling of the f64 variant; same shape, `:bool` return); `coincident?` moved to the early-return polymorphism arm; fixed scheme retired; `vector-bytes` / `bytes-vector` schemes registered. `docs/USER-GUIDE.md` — surface-table rows updated; the old fixed `coincident?` row collapsed into the polymorphic one. | ~270 Rust + ~10 doc | 8 new (round-trip, deterministic, short-input rejected, truncated rejected, coincident? Vector × Vector, coincident? mixed Vector × HolonAST, vector-bytes arity, bytes-vector arity) | shipped |

**wat-rs unit-test count: 643 → 651. +8. Workspace: 0 failing.**

Build: `cargo build --release` clean. `cargo test --release` (workspace-wide per arc 057's `default-members`): 0 failures.

---

## Architecture notes

### Audit before implementation

The DESIGN's "what's missing" table was drafted before its author
audited what arc 052 already shipped. Three of the seven proposed
additions were already in place:

| DESIGN proposed | Reality (arc 052) |
|----|----|
| `:wat::holon::Vector` type | already shipped — `Value::Vector(Arc<holon::Vector>)` |
| `:wat::holon::encode` | already shipped — `eval_holon_encode` |
| `:wat::holon::vector-cosine` | redundant — `cosine` already polymorphic |
| `:wat::holon::cosine` (extended) | already shipped (arc 052's polymorphism) |
| `:wat::holon::vector-coincident?` | redundant — see "Polymorphism, not sibling verb" below |
| `:wat::holon::vector-bytes` | NEW (this arc) |
| `:wat::holon::bytes-vector` | NEW (this arc) |

The audit shrunk the work to two genuinely-new ops + one
polymorphism extension. Same shape as arcs 058/059/060.

### Polymorphism, not sibling verb (`coincident?`)

The DESIGN proposed `:wat::holon::vector-coincident?` as a sibling
verb. After audit, the principled call is to extend the existing
`:wat::holon::coincident?` polymorphism — mirroring arc 052's
choice for `cosine`. Two reasons:

1. The coincident-floor arithmetic doesn't depend on which of
   HolonAST / Vector the inputs are; the substrate's
   `pair_values_to_vectors` already normalizes both shapes to
   `(holon::Vector, holon::Vector)` at the same d.
2. A consumer holding both an AST and a Vector shouldn't have to
   pick between two verbs. The polymorphic verb says "I want
   coincidence, here are my inputs, figure it out."

`infer_polymorphic_holon_pair_to_bool` is a direct copy of arc
052's `_to_f64` variant; only the return type differs. The
fixed-scheme registration retired in favor of the polymorphic
dispatch arm.

### Wire format — 2-bit per cell, dim header

The DESIGN's 1-bit-per-dim packing assumed bipolar `{-1, +1}`
vectors. The substrate's encoding produces ternary `{-1, 0, +1}`
(per `holon-rs::deterministic_vector_from_seed`'s `rng % 3`;
`Primitives::bundle` ties at zero). 1-bit-per-dim would lose
information; encoding ambient through a serialize-deserialize
round-trip would shift the cosine.

Shipped: 2-bit-per-cell packing.

```
bytes 0..4   : dim as u32 little-endian  (validation header)
bytes 4..end : packed 2-bit cells, 4 cells per byte, LSB-first
```

Cell encoding:
- `0b00` → `0`
- `0b01` → `+1`
- `0b10` → `-1`
- `0b11` → reserved (rejected on decode as corrupt input)

At d=10000: 4-byte header + 2500 data bytes = **2504 bytes**
total. 5x denser than i8-per-dim, ~2x sparser than 1-bit-per-dim
but preserves substrate semantics.

### `:None` on structural failure

`bytes-vector` returns `:Option<Vector>` with `:None` on:
- input shorter than 4-byte dim header
- dim-header doesn't materialize an encoder at the ambient
  router (cross-dim transmission — structural failure)
- data length doesn't match `ceil(dim/4)` bytes
- any cell decodes to the reserved `0b11` pattern

Same posture as arc 056's `from-iso8601` / `:wat::core::string::to-i64`
— failure is a binary outcome from the caller's perspective; the
Result discipline doesn't pay for itself when the only meaningful
consumer reaction is "fall back."

### No universe metadata in the bytes

Per DESIGN Q5: the seed is the receiver's responsibility to know.
The bytes carry only `(dim, cells)` — no seed, no universe tag.
This preserves the three-factor V + K + F verification UX: V is
just the projection; K is owned by the protocol; F is owned by
the receiver. Bundling K into V would defeat the cryptographic
discipline.

### Determinism

Same encoding → same bytes (test `vector_bytes_deterministic`).
The substrate's deterministic encoder + the deterministic packing
together produce a stable wire format. Test `vector_bytes_round_trip_recovers_vector`
confirms cosine ≈ 1.0 after round-trip (byte-perfect recovery,
not noise tolerance).

---

## What this unblocks

- **Lab experiment 009 T7** — encode F under seed_K in one
  hermetic child, write Vector bytes to stdout; read in the
  parent or a verifier child; re-encode F under seed_K; mixed
  cosine validates. Demonstrates the cryptographic transmission
  protocol end-to-end with actual byte-level vector handoff.
- **Future cross-process / cross-machine** — Vector bytes ride
  any transport (file, socket, queue). The dim header is the
  substrate's structural compatibility check.
- **Engram libraries (Phase 4)** — engrams that store learned
  pattern vectors get a portable persistence shape.
- **Distributed compute** — a worker can ship its computed
  Vector to a coordinator without round-tripping through the
  HolonAST source (saves re-encoding when the coordinator
  doesn't have the source).

---

## What this arc deliberately did NOT add

Reproduced from DESIGN's "What this arc does NOT add":

- **Multi-universe co-existence within one process.** Future
  `with-seed` form scoping encoding to a specific seed. The
  hermetic-fork pattern (experiment 009) covers cross-universe
  cases adequately for now.
- **Vector → HolonAST inversion.** Probably impossible in
  general (arc 002's directed-graph property: decode is unbounded);
  if wanted, a separate "best-fit" decoder uses cleanup.
- **Vector arithmetic** (`+`/`-` on Vectors directly). For v1,
  composition stays on the HolonAST side; Vector is for
  transmission and direct cosine.
- **Cryptographic-quality serialization** (AEAD, signing).
  Future `:wat::crypto::*` arc when a consumer surfaces.

---

## The thread

- **2026-04-26 (mid-arc-002)** — directed-evaluation arc surfaces
  the V transmission gap: vectors don't have a portable form.
- **2026-04-26 (DESIGN)** — proofs lane drafts the arc; six new
  ops proposed (later reduced to two after audit).
- **2026-04-26 (this session)** — audit identifies arc 052 already
  shipped most of the proposed ops; pivot to polymorphism +
  serde. Slice 1 ships in one commit: vector-bytes + bytes-vector
  + coincident? polymorphism + 8 inline tests + USER-GUIDE rows
  + this INSCRIPTION.
- **Next** — experiment 009 grows T7 with byte-level vector
  handoff; the verification protocol works end-to-end across
  process boundaries.

PERSEVERARE.
