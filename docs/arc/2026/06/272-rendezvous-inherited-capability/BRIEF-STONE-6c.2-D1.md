# BRIEF — 6c.2 (D1): the connect gate verifies the stamped minter pid; AddressWire is a record

**Supersedes** the positional-vector wire in `DESIGN-STONE-6c §6c.2` and the old `BRIEF-STONE-6c.2.md`.
Records now round-trip on the wire (234.7a base + 234.7b holon), so the address capability's portable
form is a **registered base record**, encoded/decoded by the ONE general record path (no hand-build).

**The work:** close the connect leg of the arc-272 capability channel. The minter stamps its own
`getpid()` into the address at autobind; it rides the capability **by value** as a record field; the
connect gate verifies `answerer.uid == my_euid AND answerer.pid == address.minter_pid` (symmetric with
accept). Then retract the false "unguessable name ⇒ lineage-proven" claims.

Wire shape (legal EDN): `#wat-edn.cap/address #wat.kernel/SocketAddressWire {:minter-pid 4242 :name [1 2 3 4 5]}`
— outer cap tag = ocap gate (decode only off the trusted door); inner record tag = the heterogeneous-
honest record (`tag_from_type_path(":wat::kernel::SocketAddressWire")` → `wat.kernel/SocketAddressWire`).

---

## Section A — the wire (waist evolution + record registration + codec)

### A1. Register the record (a wat file)
Add a base record near the kernel/spawn records (find the loaded wat file that hosts kernel record/struct
defs — `wat/spawn.wat` hosts `:wat::spawn::Bound`; confirm it loads AFTER `wat/Record.wat`, which
`:wat::Record::def` requires — STOP if load order is wrong):
```
(:wat::Record::def :wat::kernel::SocketAddressWire
  [minter-pid <- :wat::core::i64
   name       <- :wat::core::Vector<:wat::core::i64>])
```
(`name` is the raw UDS bytes as a vector of i64 0..=255 — wat has no byte scalar; ground the exact
`Value` vector variant a `:wat::core::Vector<:wat::core::i64>` evaluates to, used in A3.)

### A2. Thread `types` into the capability waist (`src/capability/registry.rs`)
The `CapCodec` `encode`/`decode` fn signatures gain a `&crate::types::TypeEnv` param (records need the
registry to encode/decode by field name). One-time contract bump — after it, the waist is frozen again.
- `pub encode: fn(&RustOpaqueInner, &TypeEnv) -> Option<OwnedValue>`
- `pub decode: fn(&OwnedValue, &TypeEnv) -> Result<Value, EdnReadError>`
- `encode_in` / `decode_in` / `encode_capability` / `decode_capability` thread `types` through.
- Call sites pass `types`: `edn_shim.rs:2527` (`encode_capability(inner, types)` — `value_to_edn_with`
  already has `types`) and `edn_shim.rs:1858` (`decode_capability(name, body, types)` — the enclosing
  `tagged_to_value` has `types`; pass it). Update the `waist_proof` test codecs (toy token) to the new
  signature (they can ignore `types`).

### A3. Rewrite `address_codec` to D1 (`src/capability/registry.rs`)
- **encode** `|inner, types|`: downcast to `Address`, `portable_form()` → `(minter_pid, name_bytes)`;
  `None` if no portable form (thread-tier). Build a `Value::wat__Record { class_fqdn:
  Arc::new("wat::kernel::SocketAddressWire".into()), struct_form: Arc::new(vec![Value::i64(minter_pid as i64),
  <name_bytes as the Vector<i64> Value>]) }`, then `Some(value_to_edn_with(&record, Some(types)))`. The
  result is `#wat.kernel/SocketAddressWire {:minter-pid … :name …}`; `encode_in` wraps it in the
  `#wat-edn.cap/address` cap tag (the existing `Tagged(wat-edn.cap, …)` wrap — unchanged).
- **decode** `|body, types|`: `edn_to_value(body, Some(types))` → expect `Value::wat__Record` with
  `class_fqdn == "wat::kernel::SocketAddressWire"`; read the two fields by index (struct_form[0] =
  minter_pid i64, struct_form[1] = name vector → bytes `Vec<u8>` with the `0..=255` / non-empty /
  ≤107 checks, routed through `cap_decode_error`); reconstruct `Address::from_socket_name_bytes(bytes,
  minter_pid as i32)`, re-box under `ADDRESS_TYPE_PATH`. Reject a non-record / wrong-class / bad-field
  body via `cap_decode_error` (keep the attested spanless discipline).
- Update the codec tests (`waist_proof`): the round-trip now carries pid+name as a record; update
  `address_decode_rejects_*` to the record body shape; ADD `address_roundtrips_pid_and_name` asserting
  the reconstructed `SocketAddress` has the right `minter_pid` and `name`.

## Section B — `SocketAddress.minter_pid` + the connect gate (`src/kernel/address.rs`)

- `SocketAddress` gains `pub(crate) minter_pid: i32` (always present; doc it: stamped at autobind,
  checked at connect).
- `Address::from_socket_name_bytes(name: Vec<u8>, minter_pid: i32)` — both callers pass the pid:
  the autobind arm (`src/runtime.rs:18697`) stamps `unsafe { libc::getpid() }`; the codec decode passes
  the wire pid (A3).
- `portable_name_bytes` → `portable_form(&self) -> Option<(i32, Vec<u8>)>` returning `(minter_pid, name)`
  (one downcast; `Some` only for `SocketAddress`).
- `SocketAddress::connect` (addr.rs:184): `if !connect_admits(&server, me, self.minter_pid) {` — refusal
  message names the pid mismatch. Rewrite the comment block (163-172): the gate verifies the kernel-
  vouched answerer pid against the stamped minter pid; name-secrecy is NOT relied on.
- `connect_admits(server: &PeerCred, euid: u32, minter_pid: i32) -> bool` →
  `CommsPolicy::OnlyThisPeer { pid: minter_pid }.admits(server, euid)`. Rewrite the `#[cfg(test)] mod`
  test → `connect_admits_exact_pid_admitted_wrong_pid_or_uid_refused` (exact pid ok; same-uid wrong-pid
  refused; wrong-uid refused).

## Section C — the policy rung (`src/capability/policy.rs`)

- ADD `OnlyThisPeer { pid: i32 }` to `CommsPolicy`; `admits` arm `peer.uid == my_euid && peer.pid == *pid`.
- ANNIHILATE `AnyOfMyUser` (its sole consumer was `connect_admits`): remove the variant, its `admits`
  arm, its doc paragraph, and the `any_of_my_user_*` unit test. Keep the `'a` lifetime (used by
  `OnlyMyPeers`). Rewrite the module/enum docs present-tense for the two live rungs (`OnlyMyPeers`,
  `OnlyThisPeer`); DROP the `these-gids` / wat-predicate "future rungs" prose (the exigere round-6 L1).
- Replace the `any_of_my_user_*` test with `only_this_peer_admits_exact_pid_refuses_wrong_pid_and_wrong_uid`.

## Section D — retractions (the false "unguessable ⇒ lineage-proven" claim)

Re-anchor on: *the SO_PEERCRED uid+pid checks ARE the security; the autobind name is an exclusive-bind
rendezvous token, not a secret.* Sites: `policy.rs` (AnyOfMyUser doc goes with the variant);
`address.rs:163-172` connect comment + `connect_admits` doc; `runtime.rs` autobind arm (18636/18663
"unguessable") + recv'/select' decode comments (~23845/24307); `comms/process.rs:178` (soften
"unguessable abstract name" → "kernel-minted, exclusive-bind, not a chosen name" — the collision/squat-
freedom is TRUE; only the secrecy implication drops).

## Section E — probes

- `tests/probe_arc272_6c2_pid_gate.rs` (NEW) — pure unit on the public `CommsPolicy::OnlyThisPeer` rung
  with synthesized `PeerCred` (no socket, no fork): exact-pid same-uid → admitted; same-uid wrong-pid →
  REFUSED; right-pid wrong-uid → refused. (RED at HEAD = `OnlyThisPeer` doesn't exist.) NO multi-process
  IO probe — we test OUR gate logic, not the kernel's SO_PEERCRED honesty.
- `probe_arc272_6a_capability_handoff` (regression) — the child autobinds (pid stamped), hands the
  `Address'` up, parent dials the LIVE child → answerer pid == stamped pid → admitted. Must stay GREEN.
  The codec now carries pid+name as a record across the wire.

## Blast radius
`src/capability/registry.rs` (waist sig + address_codec + tests), `src/capability/policy.rs` (rung +
annihilate), `src/kernel/address.rs` (minter_pid + gate + ctor + portable_form + test), `src/runtime.rs`
(autobind stamp + comment retractions), `src/comms/process.rs` (one comment), `src/edn_shim.rs` (2 call
sites pass `types`), one wat file (the record reg), `tests/probe_arc272_6c2_pid_gate.rs` (new). NO change
to the accept gate (`OnlyMyPeers`, listener.rs), `ThreadAddress`, or the spawn surface.

## STOP triggers
1. If the wat file hosting the record reg does NOT load after `wat/Record.wat`, STOP and report the
   correct home.
2. If `value_to_edn_with` on the built `Value::wat__Record` does NOT yield named keys (`:minter-pid`,
   `:name`) — e.g. the record type didn't register or `class_fqdn` form is wrong — STOP and report.
3. If the `Value` vector variant for `:wat::core::Vector<:wat::core::i64>` is ambiguous, STOP and report
   how vector values are actually built — do not guess `Value::Vec` vs `Value::Vector`.
4. If removing `AnyOfMyUser` reveals a consumer other than `connect_admits`, STOP.

## Verify (run + report each verbatim)
Baseline: lib 928 passed / 36 failed.
1. `cargo build --release -p wat 2>&1 | tail -5`
2. `cargo test --release -p wat --test probe_arc272_6c2_pid_gate 2>&1 | grep "test result"`
3. `cargo test --release -p wat --lib only_this_peer connect_admits waist_proof 2>&1 | grep "test result"`
4. `cargo test --release -p wat --test probe_arc272_6a_capability_handoff 2>&1 | grep "test result"`
5. `cargo test --release -p wat --test probe_arc272_autobind_listener 2>&1 | grep "test result"`
6. `cargo test --release -p wat --test probe_arc234_7a_base_record_roundtrip --test probe_arc234_7b_holon_record_roundtrip 2>&1 | grep "test result"`  (record round-trip regression)
7. `cargo test --release -p wat --lib -- --test-threads=1 2>&1 | grep "test result"`  (≥928 passed; failed == 36)
8. `grep -rn "AnyOfMyUser" src/`  (no matches)
9. `grep -rn "unguessable" src/`  (no security-inference uses remain)

Commit nothing — the orchestrator weighs the diff and commits on green.
