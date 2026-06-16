# DESIGN + BRIEF — 258.5b: kill `-> :T` on recv'/select' (the no-ascription path is the only path)

## Override of 258.5c

`DESIGN-STONE-258.5 §258.5c` kept the `recv' -> :T` ascription as a "genuine-ambiguous seed" for
values that exit straight to Rust with no wat consumer to infer from. **That concession is overridden.**
`-> :T` in a non-return position is *illegal* — the arrow is a fn return annotation only. There is no
seed, because the case it was reserved for is not actually ambiguous:

**The EDN wire is self-describing.** Post-234.7, every value crosses tagged — records as
`#wat.kernel/Foo {…}`, structs/enums tagged, scalars typed. So `decode_trusted_wire(edn, sym.types())`
reconstructs the *exact* `Value` from the wire's own tags + the type registry — it needs no declared
target type. The `-> :T` branch was only doing **coercion** the self-describing wire makes redundant.

So every `recv'`/`select'` value gets its type from one of two places, never a declared arrow:
1. a **wat consumer** constrains it → inference flows back (258.5a, `connect'` unifies its arg); or
2. it **exits to the wire / to Rust** → the self-describing tags reconstruct it.

## The kill (full annihilation)

### A. `recv'` eval (`src/runtime.rs`)
- Remove the `target_ty` parse (the 3-arg `[peer, "->", ":T"]` detection, ~23665-23666) and the
  `match &target_ty { Some(ty) => edn_to_typed_value(...), None => ... }` (~23816-23859). `recv'` is
  **1-arg** (`peer`) only; the decode is ALWAYS:
  `crate::edn_shim::decode_trusted_wire(&edn_str, sym.types().map(|a| a.as_ref()))`.
- Delete the now-dead `edn_to_typed_value` recv' call + the 3-arg arity acceptance.

### B. `select'` eval (`src/runtime.rs:24310`)
- Thread the registry into the (already no-ascription) decode:
  `decode_trusted_wire(&edn_str, sym.types().map(|a| a.as_ref()))` (was hardcoded `None`).

### C. check-side (`src/check.rs`)
- `infer_recv_prime` (~11002): drop the `-> :T` (3-arg) handling — `recv'` is 1-arg; its type flows
  from the consumer (258.5a). A `recv' peer -> :T` form must be **REJECTED** with a clear error: the
  `-> :T` arrow is a function-return annotation only, illegal on `recv'` (point the user at the
  no-ascription form; the type flows from the consumer or the wire).
- `select'` (~4932): same — no `-> :T` requirement; reject it if present.
- Confirm 258.5a's consumer-inference still covers the constrained cases (the `connect'`/`recv'` unify).

### D. callers (tests)
- Migrate every `recv' -> :T` / `select' -> :T` caller to the no-ascription form (the Explore sweep
  found 2: `tests/kernel/peer_verb_round_trip_process.rs:45`,
  `tests/kernel/probe_arc214_beta_forms_server.rs:61`). They round-trip via the self-describing wire.
  Grep `recv'.*->`/`select'.*->` across `src/ wat/ wat-tests/ tests/` for the full set — STOP if there
  are materially more than the 2 expected (means a consumer relies on coercion we haven't accounted for).
- Do NOT touch `readln` / `match` / `Option/expect` — those are other 258 clusters.

### E. doc
- Record the override in `DESIGN-STONE-258.5.md` (§258.5c): no seed; the self-describing wire does the
  inference; `recv'`/`select'` are arrow-free.

## Probe-first (RED→GREEN)

`tests/probe_arc272_6c2_record_ipc_derisk.rs` (already written, untracked) is the disconfirming probe: a
plain base record minted in a forked child, sent over the self-peer, `recv'`d **with no `-> :T`** by the
parent, fields read across the fork. **RED at HEAD** (`recv'` decode passes `None` for types →
`NoTypeRegistry`). **GREEN after** B/A. This empirically proves the self-describing wire reconstructs a
record with no arrow — and catches any "second layer" beyond the `None` (the orphan's `expected a
SocketAddressWire record` hint). Run it FIRST, confirm RED, then implement, then GREEN.

## STOP triggers
1. If a `recv'`/`select'` value genuinely cannot be reconstructed from the self-describing wire (a case
   where the tags/scalar typing are insufficient), STOP and report it — that is the real seed case and
   contradicts the override; do not silently re-add `-> :T`.
2. If dropping `infer_recv_prime`'s `-> :T` handling breaks a 258.5a consumer-inference path, STOP.
3. If the `recv'`/`select'` arrow caller sweep finds materially more than the 2 expected, STOP + report.

## Blast radius
`src/runtime.rs` (recv' eval branch removal + select' types-thread), `src/check.rs` (recv'/select'
arrow rejection), 2 test callers migrated, `DESIGN-STONE-258.5.md` (override note), the de-risk probe.
NO change to `readln`/`match`/`Option-expect`, the accept gate, or capability codecs.

## Verify (run + report each)
Baseline: lib 928 passed / 36 failed.
1. `cargo build --release -p wat 2>&1 | tail -5`
2. `cargo test --release -p wat --test probe_arc272_6c2_record_ipc_derisk 2>&1 | grep "test result"`  (was RED; now passed)
3. `cargo test --release -p wat --test probe_arc272_6a_capability_handoff 2>&1 | grep "test result"`  (capability over no-ascription recv' — must stay GREEN)
4. `cargo test --release -p wat --lib -- --test-threads=1 2>&1 | grep "test result"`  (≥928 passed; failed == 36)
5. `cargo test --release -p wat --test peer_verb_round_trip_process --test probe_arc214_beta_forms_server 2>&1 | grep "test result"` (migrated callers — if these aren't standalone targets, run the nursery/kernel group and report)
6. `grep -rn "recv'.*->\|select'.*->" src/ wat/ wat-tests/` — no `-> :T` ascriptions remain on recv'/select' (test seeds migrated)

Commit nothing — the orchestrator weighs the diff and commits on green.
