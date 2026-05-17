# Arc 206 Slice 3 INSCRIPTION — Telemetry de-dup + honest closure

**Status:** SHIPPED 2026-05-17.

**Forward-correction of slice 2:** Slice 2's INSCRIPTION (`INSCRIPTION.md`, immutable per
`feedback_inscription_immutable`) closed arc 206 prematurely. User review 2026-05-17:
*"arc 206 is falsely closed — new small edition to make telemetry use the correct calls —
the dependency on edn is no longer necessary."* The slice 2 lesson "separate-impl wins
over alias-chain" was **WRONG**. This INSCRIPTION supersedes that lesson.

## What slice 3 retired

| Artifact | What it was | Why it was wrong |
|---|---|---|
| `crates/wat-telemetry/src/shim.rs` | `:rust::telemetry::uuid::v4` Rust shim wrapping `wat_edn::new_uuid_v4` | Duplicated the substrate-core path (`:wat::core::uuid::v4`) for no gain; dragged `wat-edn = { features = ["mint"] }` into telemetry |
| `wat_edn = { features = ["mint"] }` in `crates/wat-telemetry/Cargo.toml` | Cargo dep pulled solely for UUID minting | Arc 206 slices 1+1.5 made the substrate-core path the canonical mint; telemetry no longer needs its own mint dep |
| `pub mod shim;` + `shim::register(builder);` in `lib.rs` | Registration wiring for the retired shim | Wired something that no longer exists |

## What slice 3 shipped

| File | Change |
|---|---|
| `crates/wat-telemetry/Cargo.toml` | Removed `wat-edn = { features = ["mint"] }`; added `uuid = "1"` (for `WorkUnit`'s `uuid::Uuid::new_v4()` direct call) |
| `crates/wat-telemetry/wat/telemetry/uuid.wat` | Rewritten: `:wat::telemetry::uuid::v4` now delegates to `:wat::core::uuid::v4` via `:wat::core::define` alias; backward-compat call sites see no behavior change |
| `crates/wat-telemetry/src/workunit.rs` | `uuid: uuid::Uuid::new_v4().to_string()` — direct call without wat-edn indirection |
| `crates/wat-telemetry/src/lib.rs` | Removed `pub mod shim;` + `shim::register(builder);`; updated doc comments to reflect retirement |
| `crates/wat-telemetry/src/shim.rs` | DELETED |
| `tests/wat_arc206_uuid_substrate.rs` | New test `uuid_v4_edn_roundtrip` (test E) — EDN serialization proof (Mode C gap; see below) |
| `docs/arc/2026/05/206-uuid-substrate-promotion/DESIGN.md` | Slices table updated with slice 3; status set to CLOSED; "separate-impl wins" claim corrected |
| `docs/USER-GUIDE.md` § 11 | Backward-compat note updated to reflect delegation (not separate-impl) |
| `docs/arc/2026/05/206-uuid-substrate-promotion/INSCRIPTION-SLICE-3.md` | This file |
| `docs/arc/2026/05/206-uuid-substrate-promotion/SCORE-SLICE-3.md` | 12-row SCORE (all YES) |

## EDN-serialization-still-works proof

User direction 2026-05-17: *"we need to prove we can still comm uuids over edn
serialization correctly if we haven't."*

Existing coverage proved the wat-edn internal path (`crates/wat-edn/tests/uuid_v4_mint.rs`
— mints via `new_uuid_v4()` + EDN write + EDN read roundtrip). Gap: no test drove the full
chain "`:wat::core::uuid::v4` (new substrate verb, returns `:wat::core::String`) →
`:wat::edn::write` → `:wat::edn::read` → equality assertion."

**Gap was real. Mode C applied. One test added.**

`tests/wat_arc206_uuid_substrate.rs::uuid_v4_edn_roundtrip` closes the gap. It mints via
`:wat::core::uuid::v4`, writes via `:wat::edn::write`, reads via `:wat::edn::read`, and
asserts the roundtripped value equals the original. 5/5 tests in that file pass green.

EDN UUID serialization invariant is proven end-to-end. No gap remains.

## Corrected discipline lesson

**The slice 2 lesson was wrong.** Slice 2 inscribed:

> "Separate-impl vs alias-chain backward-compat: `:wat::core::uuid::v4` and
> `:wat::telemetry::uuid::v4` are SEPARATE independent impls. NOT an alias chain.
> This is the cleaner backward-compat pattern."

That is **inverted**. The alias-chain (wat-side delegation to the substrate verb) IS the
right backward-compat pattern:

- It eliminates the Cargo dep (`wat-edn = { features = ["mint"] }`) from the consumer crate
- It makes the delegation explicit at the wat layer (one line of `.wat`)
- It costs nothing at runtime (`:wat::core::define` alias has the same call path)
- It is honest: the one canonical source of truth is the substrate verb; the telemetry name
  is an alias that says so

The "separate-impl is simpler to reason about" framing was wrong because it ignored the dep
cost and the honesty gap (two independent impls that happen to do the same thing is NOT
simpler — it is duplication).

**Corrected pattern for future backward-compat promotions:** when retiring a Rust shim in
favor of a substrate-core verb, use a wat-side `:wat::core::define` alias that delegates to
the substrate verb. Do NOT keep a separate Rust shim wrapping the same underlying function.

## Verification results

| Check | Command | Result |
|---|---|---|
| git status (5 files) | `git status --short` | 5 files (4 modified, 1 deleted) + untracked `.claude/worktrees/` (harness state, not orchestrator work) |
| wat-telemetry 36/36 | `cargo test --release -p wat-telemetry --no-fail-fast` | 36 passed, 0 failed |
| Workspace baseline | `cargo test --release --workspace --no-fail-fast` | 4 pre-existing failures (lifeline, bogus canary, t6, startup-exit-3); 0 new |
| No `:rust::telemetry::uuid::v4` in crates/wat-telemetry/ | grep | 0 hits (only comments naming the retired artifact) |
| No `wat_edn::*` in crates/wat-telemetry/src/ | grep | 0 hits |

## Arc 206 closure conditions — all met

1. Substrate-core verbs ship: `:wat::core::uuid::v4` + `:wat::core::uuid::v5` DONE (slices 1 + 1.5)
2. Telemetry delegates: `:wat::telemetry::uuid::v4` → `:wat::core::uuid::v4` via `.wat` alias DONE (slice 3)
3. No `wat-edn` dep on `wat-telemetry` crate DONE (slice 3)
4. EDN roundtrip proven DONE (slice 3 Mode C test)

Arc 206 is closed.

---

Arc 206 slice 3 inscribed. 2026-05-17.
