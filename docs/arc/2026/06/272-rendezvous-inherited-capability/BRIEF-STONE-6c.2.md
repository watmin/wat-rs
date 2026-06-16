# BRIEF — 6c.2: the connect gate verifies the stamped minter pid

**The work (one paragraph).** Close the connect leg of the arc-272 capability channel by making the
connect gate verify the answerer's kernel-vouched pid against a pid the minter stamped into the address
at autobind. Today the connect gate checks only euid (`CommsPolicy::AnyOfMyUser`); a same-uid process at
any pid is admitted. After this strike the gate checks `answerer.uid == my_euid AND answerer.pid ==
address.minter_pid` — symmetric with the accept gate. The minter pid is the minter's own `getpid()`
(perfect knowledge), stamped at autobind, carried with the `Address'` capability by value across the
`wat-edn.cap/address` wire (no global set, no mutex). Then retract the now-false "the autobind name is
unguessable ⇒ the answerer is lineage-proven" claims, since the pid — not name-secrecy — is the security.

Read the pinned contract first: `docs/arc/2026/06/272-rendezvous-inherited-capability/DESIGN-STONE-6c-capability-channel-trust.md`
§ "6c.2 — THE STRIKE" (the seven pinned decisions). This brief implements exactly those decisions.

---

## Rooms (read in order; each with why)

1. `src/capability/policy.rs` (whole file, 92 lines) — the powerbox enum. You will ADD a rung
   `OnlyThisPeer { pid: i32 }` and REMOVE `AnyOfMyUser` (its sole consumer moves to the new rung).
2. `src/kernel/address.rs:131-320` — `SocketAddress` (gains `minter_pid`), its `connect` (passes the
   pid), `Address::from_socket_name_bytes` / `portable_name_bytes` / the `connect_admits` seam / the
   `#[cfg(test)] mod tests`. This is the heart of the strike.
3. `src/capability/registry.rs:96-238` — `address_codec` (the wire: encode reads `portable_form`, the
   body becomes a 2-element `[pid, [name…]]` vector; decode reconstructs both) + the codec tests.
4. `src/runtime.rs:18689-18699` — the autobind arm: stamp `libc::getpid()` into
   `Address::from_socket_name_bytes(name_bytes, …)`.
5. `src/runtime.rs:23842-23852` and `:24302-24314` — the recv'/select' decode comments to re-anchor.
6. `src/comms/process.rs:174-185` — soften the "unguessable" word in the autobind primitive doc.
7. `tests/probe_arc272_6a_capability_handoff.rs` (regression guard — read, do NOT edit; confirm it
   stays green).

---

## Implementation sketch (the strike path — fill it, do not invent the shape)

### A. `src/capability/policy.rs`

Add the rung to the enum (after `OnlyMyPeers`), remove `AnyOfMyUser`:

```rust
pub enum CommsPolicy<'a> {
    /// Admit iff the peer runs as me (euid match) AND its pid is one of mine (a lineage-set member).
    /// The accept gate's posture — it holds an allow-set of the pids it spawned.
    OnlyMyPeers { lineage: &'a HashSet<i32> },
    /// Admit iff the peer runs as me (euid match) AND its pid is EXACTLY this one — the connect gate's
    /// posture. The dialer has perfect knowledge of the single pid that minted the capability it
    /// holds (stamped into the `Address'` at autobind); it checks for that one identity, not a set.
    OnlyThisPeer { pid: i32 },
}
```

`admits`:
```rust
CommsPolicy::OnlyThisPeer { pid } => peer.uid == my_euid && peer.pid == *pid,
```

- The `'a` lifetime is still needed by `OnlyMyPeers`, keep it.
- Replace the `any_of_my_user_*` unit test with `only_this_peer_admits_exact_pid_refuses_wrong_pid_and_wrong_uid`:
  exact pid + my uid → admitted; **same uid, WRONG pid → refused**; right pid, wrong uid → refused.
- Module doc (policy.rs:1-14, 19-26): rewrite present-tense for the TWO live rungs (`OnlyMyPeers`,
  `OnlyThisPeer`). DROP the `these-gids` / "a wat `fn(PeerCred) -> bool` predicate" *future-rung* prose
  (the exigere round-6 L1) — name only what exists. Keep the narrow-waist law statement (adding a rung
  extends the language; the `admits` contract never changes).

### B. `src/kernel/address.rs`

- `SocketAddress` (line 131): add field `pub(crate) minter_pid: i32,` with a doc line (the pid of the
  process that minted this autobind address — stamped at bind, checked at connect).
- `Address::from_socket_name_bytes` (line 255): signature becomes
  `pub fn from_socket_name_bytes(name: Vec<u8>, minter_pid: i32) -> Self` → constructs
  `SocketAddress { name, minter_pid }`.
- `portable_name_bytes` (line 266): rename to `portable_form` returning `Option<(i32, Vec<u8>)>`:
  ```rust
  pub(crate) fn portable_form(&self) -> Option<(i32, Vec<u8>)> {
      self.inner.as_any_ref().downcast_ref::<SocketAddress>()
          .map(|s| (s.minter_pid, s.name.clone()))
  }
  ```
  Update the doc comment to say it returns `(minter_pid, name_bytes)`.
- `SocketAddress::connect` (line 184): `if !connect_admits(&server, me, self.minter_pid) {`. Update the
  error message: the refusal is now "the answerer's pid {server.pid} is not the minter pid
  {self.minter_pid} (or euid mismatch)". Rewrite the comment block (163-172): the connect gate verifies
  the kernel-vouched answerer pid against the pid the minter stamped — name-secrecy is NOT relied upon.
- `connect_admits` (line 225): signature
  `pub(crate) fn connect_admits(server: &PeerCred, euid: u32, minter_pid: i32) -> bool` →
  `CommsPolicy::OnlyThisPeer { pid: minter_pid }.admits(server, euid)`. Update its doc.
- The `#[cfg(test)] mod tests` (line 291): rewrite `connect_admits_*` →
  `connect_admits_exact_pid_admitted_wrong_pid_or_uid_refused`: build `PeerCred{pid, uid, gid}`;
  assert exact (pid==minter, uid==euid) → true; (uid==euid, pid != minter) → **false**;
  (pid==minter, uid != euid) → false.

### C. `src/capability/registry.rs` — `address_codec` (line 96)

- `encode`: read `portable_form()` → `(pid, bytes)`; emit
  `OwnedValue::Vector(vec![ OwnedValue::Integer(pid as i64), OwnedValue::Vector(bytes.into_iter().map(|b| OwnedValue::Integer(b as i64)).collect()) ])`.
- `decode`: expect `OwnedValue::Vector(items)` with `items.len() == 2`; `items[0]` an `Integer` in
  `i32::MIN..=i32::MAX` (→ `minter_pid`); `items[1]` an `OwnedValue::Vector` of the name bytes — apply
  the EXISTING empty / over-long (≤107) / `0..=255` checks to THAT inner vector. Reconstruct
  `Address::from_socket_name_bytes(bytes, minter_pid as i32)`. Route every rejection through
  `cap_decode_error` (the attested spanless helper) — keep that discipline.
- Tests (waist_proof mod):
  - Update `address_decode_rejects_overlong_name` and `address_decode_rejects_empty_name` to the
    2-element wire shape (wrap the byte vector as `items[1]`, with a valid `Integer` pid at `items[0]`).
  - ADD `address_roundtrips_pid_and_name`: build an `Address::from_socket_name_bytes(vec![1,2,3,4,5], 4242)`,
    encode through `encode_in(&[address_codec()], inner)`, decode through `decode_in`, assert the
    reconstructed `SocketAddress` has `minter_pid == 4242` and `name == [1,2,3,4,5]`. (You can downcast
    via the public path used in `a_second_capability_rides_the_same_waist`.)
  - Add a `address_decode_rejects_wrong_arity` (a 1- or 3-element outer vector → rejected).

### D. `src/runtime.rs` autobind arm (line 18697)

```rust
make_rust_opaque(ADDRESS_TYPE_PATH, Address::from_socket_name_bytes(name_bytes, unsafe { libc::getpid() })),
```
Add a one-line comment: perfect knowledge — the minter stamps its own pid; the connect gate verifies it.
Also soften the arm's "unguessable" wording (18636/18663) → "kernel-minted, exclusive-bind (no chosen
name → no collision/squat)"; the security is the pid check, not name secrecy.

### E. Comments-only retractions

- `src/runtime.rs:23845-23848` and `:24307-24310` (recv'/select'): the "connect'd peer reached an
  unguessable autobind capability ⇒ the answerer IS the minter" clause → "a connect'd peer is
  pid-verified at the gate (answerer.pid == the stamped minter pid); the autobind name is an
  exclusive-bind rendezvous token, not a secret." Keep the rest (inherited handle / self-peer / accept'
  legs) intact.
- `src/comms/process.rs:178`: "unique, unguessable abstract name" → "unique, kernel-minted abstract
  name (no chosen name → nothing to collide with or squat)". The collision/squat-freedom is true; only
  the secrecy implication is dropped.

### F. The disconfirming probe — `tests/probe_arc272_6c2_pid_gate.rs` (NEW)

Pure unit-level, public API only (`wat::capability::CommsPolicy`, `wat::comms::process::PeerCred` —
both `pub`; confirm the re-export path compiles, adjust the `use` if the crate exposes them elsewhere).
NO socket, NO fork.

```rust
//! Arc 272 6c.2 — the connect gate's pid check, proven on OUR policy logic (synthesized creds; no IO).
//! RED at HEAD: `OnlyThisPeer` does not exist — the compile error names exactly the gap (the policy
//! language cannot express "only this one pid"). GREEN after 6c.2. We do NOT multi-process-test the
//! kernel's SO_PEERCRED honesty (that is an axiom, not our code) — see feedback_dont_test_the_substrates_honesty.
use wat::capability::CommsPolicy;
use wat::comms::process::PeerCred;

fn cred(pid: i32, uid: u32) -> PeerCred { PeerCred { pid, uid, gid: 0 } }

#[test]
fn connect_gate_admits_exact_minter_pid_and_refuses_a_same_uid_rebind() {
    let me: u32 = 1000;
    let minter_pid = 4242;
    let policy = CommsPolicy::OnlyThisPeer { pid: minter_pid };
    // The live minter at the stamped pid, my user → admitted.
    assert!(policy.admits(&cred(minter_pid, me), me), "the live minter is admitted");
    // SAME user, a DIFFERENT pid (the death-then-rebind attacker) → REFUSED. This is the edge
    // name-secrecy could never close; the stamped-pid check closes it by construction.
    assert!(!policy.admits(&cred(9999, me), me), "a same-uid process at another pid is refused");
    // Right pid, WRONG user → refused at the euid floor.
    assert!(!policy.admits(&cred(minter_pid, me + 1), me), "another user's process is refused");
}
```

---

## Blast radius

`src/capability/policy.rs`, `src/kernel/address.rs`, `src/capability/registry.rs`,
`src/runtime.rs` (autobind arm + two comment blocks), `src/comms/process.rs` (one comment),
`tests/probe_arc272_6c2_pid_gate.rs` (new). NO new types beyond the one enum rung. NO change to the
accept gate (`OnlyMyPeers`, listener.rs), to `ThreadAddress`, or to the spawn surface.

## STOP triggers (halt and surface — do not improvise)

1. If `CommsPolicy` / `PeerCred` are NOT reachable from `tests/` via `wat::capability::` /
   `wat::comms::process::` (the probe will not compile against the public surface), STOP and report the
   actual public path — do not make types `pub` to force it.
2. If removing `AnyOfMyUser` reveals a consumer OTHER than `connect_admits` (grep
   `AnyOfMyUser` across `src/`), STOP — the contract assumed it was the sole consumer.
3. If `libc::getpid()` is not already used in `src/runtime.rs` (it is — see `address.rs`/`process.rs`
   precedent) and the import shape is unclear, STOP rather than adding a new dependency path.
4. If the existing `address_decode_rejects_*` tests cannot be mechanically lifted to the 2-element wire
   shape because the decode validation is structured differently than this brief assumes, STOP and
   report the actual decode structure.

## Method

Make the changes; run the commands in the Expectations doc; report each row's real result against your
own re-run. Commit nothing — the orchestrator weighs the diff and commits on green.
Prior comparable shapes to copy: the existing `connect_admits` unit test (address.rs:302) and the
`a_second_capability_rides_the_same_waist` codec test (registry.rs:170).
