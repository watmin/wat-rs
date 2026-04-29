# Arc 092 — `wat-edn` v4 minting — INSCRIPTION

**Status:** shipped 2026-04-29. Same-session prerequisite landed mid-arc-091-slice-2.

`wat-edn` already owned the UUID concept — `Value::Uuid(Uuid)` variant,
`#uuid "..."` parsed and written. But the `uuid = "1"` pin had no
features, so `Uuid::new_v4()` was unreachable. wat-edn could TRANSPORT
UUIDs; it could not MINT them. A parser-with-typed-Uuid that can read
but not construct is missing half the contract.

This arc closes the asymmetry behind a Cargo feature gate. wat-edn
gains `pub fn new_uuid_v4() -> uuid::Uuid` under `features = ["mint"]`,
plus a roundtrip test proving any minted UUID survives wat-edn's own
write/parse cycle. Default-feature consumers never link `getrandom` —
the entropy cost lands only on opt-in.

The immediate beneficiary: arc 091's wat-measure (slice 2). WorkUnit
keys every measurement scope by uuid; wat-measure now consumes wat-edn
for both transport AND minting, no second `uuid` pin in the workspace.

**Design:** [`DESIGN.md`](./DESIGN.md).

---

## What shipped

### Slice 1 — `mint` feature + `new_uuid_v4()` + roundtrip test

`crates/wat-edn/Cargo.toml`:
- New `[features]` table with `default = []` and `mint = ["uuid/v4"]`.
- The `uuid = "1"` pin is unchanged; the `v4` capability flips on only
  through the gate.

`crates/wat-edn/src/lib.rs`:
- `#[cfg(feature = "mint")] pub fn new_uuid_v4() -> uuid::Uuid` — a
  one-line wrapper around `uuid::Uuid::new_v4()`. Returns typed
  `Uuid`, not `String`; callers `.to_string()` for canonical hex or
  `Value::Uuid(_)` for EDN. The doc comment carries a roundtrip
  example exercised as a doctest under `--features mint`.

`crates/wat-edn/tests/uuid_v4_mint.rs` — three integration tests, the
whole file `#[cfg(feature = "mint")]`:

1. `v4_roundtrips_through_write_and_parse` — mint, write, parse, eq.
   The contract proof.
2. `many_v4_mints_are_unique` — 256 mints, `HashSet::len == 256`.
   Smoke screen against constant returns or seed-stuck RNGs (not a
   formal entropy proof).
3. `v4_string_form_is_canonical_hyphenated` — verifies the
   `Uuid::to_string()` output matches EDN's accepted `#uuid` form
   (8-4-4-4-12 hex, hyphenated, lowercase). Belt-and-suspenders
   against future Uuid display drift.

Both lanes verified:
- `cargo test -p wat-edn` (default, no mint) — 36 tests pass; the
  v4 file sits out via cfg.
- `cargo test -p wat-edn --features mint` — 36 + 3 v4-mint tests +
  2 doctests pass.

**Documentation that landed alongside the slice:**

- `crates/wat-edn/README.md` — Cargo.toml example gains a commented
  `features = ["mint"]` line so the gate is discoverable from the
  first thing a new caller reads.
- `crates/wat-edn/docs/USER-GUIDE.md` § "Minting v4 UUIDs" — new
  subsection under §6 "Built-in tags". Explains the gate's purpose
  (parser-only consumers don't pay for entropy init), shows the
  one-call API, points at the roundtrip test.

---

## What's NOT in this arc

- **Direct wat-side surface.** `:wat::edn::uuid-v4` doesn't exist.
  The wat-side surface for v4 minting lives in **arc 091's wat-measure**
  at `:wat::measure::uuid::v4` — that's the consumer arc; wat-edn's
  role here is to provide the Rust building block.
- **v7 / v6 / v1 / NIL UUIDs.** Future small adds; not blocking. v7's
  time-orderable IDs would matter when WorkUnit keys become
  index-keys (arc 093 WorkQuery), not before.
- **Custom RNG injection.** `getrandom`'s default suits the workspace.
  Deterministic-test-entropy or FIPS-mode flavors land in a follow-up
  if a need surfaces.

---

## Surfaced by

User direction 2026-04-29, mid-arc-091-slice-2:

> "do we need wat-edn to provide the uuid dep?... is it wrong?... we
> must have v4 ... wat-edn needs to prove it can operate on v4.."

The question landed when arc 091's slice 2 was about to take an
independent `uuid = { version = "1", features = ["v4"] }` pin in
wat-measure. The cleaner answer: wat-edn already owns
`Value::Uuid`; minting belongs there too. wat-measure consumes wat-edn
for both directions; one uuid pin in the workspace.

> "yes - go write a new arc and make it exist"

Arc 092 confirmed. Numbering: arc 091's DESIGN.md previously named
arcs 092 (WorkQuery) and 093 (circuit.wat) as future siblings.
With 092 taken by this arc, those bumped to 093 and 094 in the same
edit.

---

## Test coverage

Workspace summary: `cargo test --workspace` zero failures across
both feature lanes; `cargo test -p wat-edn` and
`cargo test -p wat-edn --features mint` both green.

| Lane | wat-edn unit | wat-edn integration | uuid_v4_mint | doctests |
|---|---:|---:|---:|---:|
| default | 36 | 0 | (skipped via cfg) | 1 |
| mint    | 36 | (existing tests) | 3 | 2 |

The cfg-gating is the contract: removing the `mint` feature MUST NOT
re-enable the v4 tests (they'd fail to compile without
`new_uuid_v4`). That's the regression-trip wire: any future change
that accidentally puts `new_uuid_v4` outside the gate would surface
as a default-build compile error in the test, not a silent feature
expansion.

---

## Files changed

Substrate:
- `wat-rs/crates/wat-edn/Cargo.toml` — `[features]` table; `mint`.
- `wat-rs/crates/wat-edn/src/lib.rs` — `pub fn new_uuid_v4()` (cfg).

Tests:
- `wat-rs/crates/wat-edn/tests/uuid_v4_mint.rs` — 3 cfg-gated tests.

Documentation:
- `wat-rs/docs/arc/2026/04/092-wat-edn-uuid-v4/DESIGN.md`
- `wat-rs/docs/arc/2026/04/092-wat-edn-uuid-v4/INSCRIPTION.md` (this file)
- `wat-rs/crates/wat-edn/README.md` — features hint.
- `wat-rs/crates/wat-edn/docs/USER-GUIDE.md` — § Minting v4 UUIDs.

Cross-arc:
- `wat-rs/docs/arc/2026/04/091-wat-measure/DESIGN.md` — WorkQuery and
  circuit.wat references bumped from arc 092/093 to 093/094;
  slice-2 deps line updated to `wat-edn (path, features = ["mint"])`;
  the slice-2 path notation was also corrected from
  `:wat::measure::uuid/v4` to `:wat::measure::uuid::v4` per the
  established `::` = free-fn convention (the `/` separator is reserved
  for type-method calls; `:wat::edn::write` is the precedent).
