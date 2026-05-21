# BRIEF — Arc 216 Stone 216.6 — Process-tier HolonRepresentable cascade validation

**Stone:** verify the cascade is real end-to-end at the process tier. After arc 216 stones 216.1/216.2/216.3 added `HolonRepresentable` impls for `HashSet<T>`, `Vec<T>`, `HashMap<K, V>`, the existing process-tier IPC path at `src/comms/process.rs` should round-trip these collections through `Sender<T>::send` → tagged-EDN over pipe → `Receiver<T>::recv` without any further substrate work. This stone writes the probes that prove it.
**Type:** Sonnet Mode A.
**Time budget:** 45-75 min target; 90 min STOP.
**Depends on:** Stones 216.1 (`b478ff4` HashSet HolonRepresentable), 216.2 (`e4a63ed` Vec HolonRepresentable), 216.3 (`fdc5031` HashMap HolonRepresentable), 216.5d (`ef7e0c6` antidote complete; substrate impeccable).
**Unblocks:** Stone 216.7 (INSCRIPTION + arc closure).

## Substrate target (verified)

The process-tier cascade lives entirely in `src/comms/process.rs`:

- `Sender<T: HolonRepresentable>::send` at `src/comms/process.rs:144-160`
  - Line 160: `let ast = value.to_holon_ast();` — the HolonRepresentable invocation
  - Then `edn_shim::write_holon_ast_tagged` → newline-framed bytes → `libc::write`
- `Receiver<T>::recv` at `src/comms/process.rs:647-654`
  - Line 654: `T::from_holon_ast(&ast_arc)` — the reverse invocation

The trait + collection impls:
- `trait HolonRepresentable` at `src/comms/mod.rs:90`
- `impl HolonRepresentable for HashSet<T>` at `src/comms/mod.rs:142` (Stone 216.1)
- `impl HolonRepresentable for Vec<T>` at `src/comms/mod.rs:190` (Stone 216.2)
- `impl HolonRepresentable for HashMap<K, V>` at `src/comms/mod.rs:291` (Stone 216.3)

Existing test pattern at `tests/comms/process.rs` (Stone C String round-trip — header notes this is "the wire chain `T → HolonAST → tagged EDN string → newline-framed bytes → libc::write → io_uring Read → bytes → EDN → HolonAST → T`"):

```rust
let (tx, rx) = pair::<String>().expect("pair");
tx.send("hello".to_string()).expect("send");
let got = rx.recv().expect("recv");
assert_eq!(got, "hello");
```

After arc 216 stones, the same shape compiles for `pair::<HashMap<...>>()`, `pair::<Vec<...>>()`, `pair::<HashSet<...>>()`. This stone writes the probes.

## Pre-flight verified

- Stones 216.1/216.2/216.3 SHIPPED — HolonRepresentable impls present
- Stone 216.5d SHIPPED — substrate impeccable; `hashmap_key` deleted; `impl Hash for Value` is canonical
- All 16 prior probe suites GREEN at commit `ef7e0c6`
- The existing `tests/comms/process.rs` String round-trip pattern is the template

## Working dir + constraints

- `/home/watmin/work/holon/wat-rs/`
- Branch: `arc-170-gap-j-v5-deadlock-state`
- Linux only; Zero Mutex; no `--no-verify`

## Your scope

### Part A — Probe file

Write a new probe file at `tests/probe_arc216_stone6_process_collection_roundtrip.rs` with ~9 tests using the existing `tests/comms/process.rs` pattern:

1. **Probe 1** — `pair::<HashMap<String, String>>()`: send a 2-entry map, recv, assert equal
2. **Probe 2** — `pair::<HashSet<String>>()`: send a 3-element set, recv, assert equal
3. **Probe 3** — `pair::<Vec<String>>()`: send a 3-element vec, recv, assert equal (order preserved)
4. **Probe 4** — Nested: `pair::<HashMap<String, Vec<String>>>()` round-trips
5. **Probe 5** — Nested: `pair::<Vec<HashSet<String>>>()` round-trips
6. **Probe 6** — Triple nested: `pair::<HashMap<String, Vec<HashSet<String>>>>()` round-trips
7. **Probe 7** — Empty collection: empty HashMap round-trips as empty
8. **Probe 8** — FIFO with collection payloads: three sends, three recvs, ordering preserved
9. **Probe 9** — Compile-time HolonRepresentable check: `fn assert_holon_representable<T: wat::comms::HolonRepresentable>() {}` invoked with each collection type; the fact this compiles is the proof

### Part B — Test audit

Before writing probes, run:

```
grep -rn HolonRepresentable tests/comms/
grep -rn pair tests/comms/process.rs
```

These show the existing pattern. Mirror it; don't invent new patterns.

After writing probes:

```
grep -rn capture.*fail tests/
grep -rn not HolonRepresentable tests/
```

If any existing test asserts collection-capture failure (pre-arc-216 state), surface it. Flip is in scope; surfacing first is the right discipline.

### Part C — SCORE doc

Write `docs/arc/2026/05/216-collections-as-holons/SCORE-STONE-216.6.md` matching EXPECTATIONS row count.

## NOT your scope

- INSCRIPTION + closure — Stone 216.7
- Any substrate change — the cascade should "just work" given existing impls
- Refactoring `tests/comms/process.rs` or `src/comms/*` — the substrate is settled
- Touching anything in arc 215 / 214 / 213

## STOP triggers

- **STOP-1: cascade fails at runtime.** If `pair::<HashMap<...>>()` compiles but the round-trip fails (send succeeds + recv returns wrong value), STOP and surface the failure mode. This would mean a substrate gap not yet known.
- **STOP-2: probe substitution.** If a probe in the matrix fails because the substrate doesn't support what it tests, STOP. Do NOT substitute a different type. Surface to orchestrator.
- **STOP-3: existing test asserts collection-capture failure.** Surface the test path + assertion. Orchestrator decides flip-vs-update.
- **STOP-4: any existing probe regresses** — surface; do not push through.
- **STOP-5: 90 min elapsed.**

## Verification

One per line:

```
cargo build --release
cargo test --release --test probe_arc216_stone6_process_collection_roundtrip -p wat
cargo test --release --test probe_arc216_stone5c_hashmap_native_storage -p wat
cargo test --release --test probe_arc216_stone5b_hashset_native_storage -p wat
cargo test --release --test probe_arc216_stone5a_value_hash -p wat
cargo test --release --test probe_verify_hashset_of_vector_gap -p wat
cargo test --release --test probe_arc216_stone4_predicate_composition -p wat
cargo test --release --test probe_arc216_stone3_hashmap_roundtrip -p wat
cargo test --release --test probe_arc216_stone2_vector_roundtrip -p wat
cargo test --release --test probe_arc216_stone1_hashset_roundtrip -p wat
cargo clippy --release -- -D warnings
```

## When you finish

Report: pass count out of EXPECTATIONS row count, deltas, verification summary, elapsed time, any tests surfaced via STOP-3.

Don't commit. Orchestrator commits after review.
