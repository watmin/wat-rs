# DESIGN — step 5: annihilate the name (rendezvous is capability-only)

> Opened 2026-06-16. **The real close of the round-3 vigilatum L1**, not the surgical 6c.1 patch
> (which this supersedes — see `DESIGN-STONE-6c-capability-channel-trust.md`). Builder, on weighing the
> deferrals: a deferred *security posture* is a violation. 6c.1 only refused *capabilities* over a
> euid-only connect; it left the **data** path trusting any same-uid process that squats a guessable
> name (cross-program MITM within the user) — which violates "only my peers." The honest close removes
> the squatting surface itself.

## The statement

**All rendezvous is capability-only.** A connection target is an **unguessable autobind `Address'`**
you were *handed* (over the lineage channel) or *minted* (your own). There is no string-name discovery.
Guessable names — `socket-address'`, connect-by-name, the legacy 2-arg `listener'(process) <addr>` —
are **annihilated** (retired, not patched).

## Why this is the security close (and subsumes 6c.1)

The connect side's trust has exactly two honest sources: **the address is unguessable** (you hold it
because lineage handed it to you), or **a kernel pid check** (which needs a process-global lineage set —
a ZERO-MUTEX trap). Guessable names have neither, so they are the whole gap. Remove them and:

- The only way to reach a listener is to **hold its unguessable autobind address**. Abstract UDS names
  are **exclusive-bind** + kernel-minted-random, so the answerer *is* the minter = **lineage-proven**
  (it handed you the address, or it's you). No same-uid process can squat what it cannot name.
- Therefore **every connect channel is lineage-safe for data AND caps** → `recv'`'s "bytes from a
  lineage peer" comment becomes a structural truth, the round-3 L1 dissolves, and policy.rs's "never to
  a stranger" holds for the whole system. No `PeerTrust` bit needed — there are no euid-only connect
  channels left to refuse anything over.
- The accept gate (`OnlyMyPeers`, pid∈lineage) stays — defense-in-depth if an address ever leaks.

## The contract decision (pinned)

**`Address'` is minted only by the kernel (`autobind`); there is no construction from a user string.**
`connect'` / `listener'` take only a handed/minted `Address'` (or autobind in place). The euid floor
(`AnyOfMyUser`) stays on connect as defense-in-depth; the lineage proof IS the capability you hold.

## Four questions

- **Obvious?** YES — "you can only dial an address you were given" is the ocap rule in one sentence; no name registry to reason about.
- **Simple?** YES — it REMOVES a whole verb + a dispatch arm + a constructor; fewer concepts, not more. One rendezvous shape (autobind handoff), not two.
- **Honest?** YES — the same-uid squatting/MITM surface is gone by construction, not documented-around; the door's premise is true on every leg.
- **Good UX?** YES — one way to rendezvous (the 6a handoff); a service author cannot accidentally expose a guessable, squattable name.

## Blast radius (crawled + verified 2026-06-16 — `socket-address'` is TEST-ONLY)

**Rust (4 files) — REMOVE:**
- `runtime.rs`: `eval_socket_address_prime` (18611-18654) + its dispatch arm (4750); the 2-arg legacy
  named arm in `eval_listener_prime` (18760-18782).
- `check.rs`: `infer_socket_address_prime` (10184-10243) + dispatch arm (4974); the 2-arg legacy arm in
  `infer_listener_prime` (10352-10366).
- `kernel/address.rs`: `from_socket_name(String)` (237) — its only caller is the removed eval fn.
- **SURVIVES:** `from_socket_name_bytes` (autobind + the `wat-edn.cap/address` wire decode),
  `portable_name_bytes`, `autobind_listener`, the `Address'`/`SocketAddress` type, the 3-arg autobind
  `listener'(process) :S :R`, `connect'`/`accept'`, `allow'`/`deny'` (re-grounded as lineage-trust).

**Tests — 6 convert onto autobind+handoff, 2 already prove the replacement:**
- DELETE/fold (redundant mechanism probes superseded by `probe_arc272_autobind_listener`): c0b2a, c0b2c, c0b2d.
- REWRITE onto autobind+handoff, **preserving their property proofs**:
  - c0b3aii (poll' service loop) → service autobinds, sends its address up the lineage channel, owner dials it.
  - c0b3bb_bounced (stranger refused) → hand the non-lineage child the **leaked** autobind address explicitly; the accept gate bounces it on `pid∉lineage`. The proof survives — sourced from a capability, not a name.
  - c0b3bb_verbs (`allow'`/`deny'`) → exercise the verbs on an autobind listener.
- KEEP: `probe_arc272_autobind_listener`, `probe_arc272_6a_capability_handoff`.

## The proof (RED → GREEN)

- After removal, every `socket-address'` reference is a **CHECK ERROR** (verb gone) — RED on the old
  tests, which is the signal to convert them. The autobind+handoff tests stay GREEN.
- The converted c0b3bb stranger-bounce proves the gate still refuses a non-lineage peer that holds a
  (leaked) capability — the security property preserved through the annihilation.
- Then re-cast the `src/capability/` vigilatum: circumspicere's connect-leg L1 is gone (no euid-only
  connect channel exists) → converges → **stamp**.

## Decomposition (sub-strikes)

1. **5a — remove the verb + arms** (`socket-address'`, the 2-arg listener arm, `from_socket_name`).
   RED: the 6 name-based tests fail to check. Bound: runtime.rs + check.rs + address.rs.
2. **5b — convert the 6 tests** onto autobind+handoff (delete the 3 redundant mechanism probes; rewrite
   c0b3aii / c0b3bb_bounced / c0b3bb_verbs preserving their proofs).
3. **5c — docs + the recv' comment**: archive the C0b.2d connect-by-name DESIGN/BRIEF as historical;
   update `recv'`'s comment (now "lineage peer" is enforced, not assumed); re-cast the vigilatum + stamp.

## Out of scope (rejected / deferred)

- **6c.2** (per-Address minter-pid stamped + verified at connect) — accepted **deferral**: belt-and-
  suspenders for the narrow *leak-then-minter-dies-then-same-uid-rebind* edge; the capability + euid
  floor + exclusive-bind already close the main surface ([[feedback_dont_build_the_forcing_function]]).
- **cross-uid** — settled (euid is the floor; can't be staged unprivileged, and it'd test the kernel —
  [[feedback_dont_test_the_substrates_honesty]]).

## The bar

One rendezvous shape, capability-only, no guessable surface. The capability home's central claim becomes
true by construction on every leg; then the vigilatum converges and stamps. Pairs
[[project_rendezvous_inherited_capability]] + [[feedback_vended_primitives_never_deadlock]] +
[[feedback_bar_shockingly_well_written]] (annihilate the class, don't annotate it).
