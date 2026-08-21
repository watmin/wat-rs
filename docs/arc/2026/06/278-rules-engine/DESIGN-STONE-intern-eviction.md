# DESIGN-STONE — intern eviction on last lease

> **Origin (2026-08-20).** Item 12: a Weak intern died when
> fire returned, so the table holds a strong `Arc` for
> process lifetime. The 512-session protocol ends with
> on-disconnect deprovision. Hangup drops the Session
> value; the intern entry stays until the thread (today:
> the process) exits. 512 connections become a museum of
> every ruleset ever compiled. This stone makes the intern
> a **lease**. Last lease drop removes the entry. Session
> stays 8 fields. Requires stone 27 (thread-owned table).

## The measurement we have

`InternedNetwork` is not a Session field. `compile-all`
calls `arm-session` → `rete_arm_get_or_build` → strong
insert. Fire clones the Arc for the pass and drops it
when fire returns. Nothing decrements the table. Weak
without an owner is the hole Item 12 already burned.

The protocol:

```
on-connect → Session
install-rules × N
compile        → arm-session leases 1
insert × N
fire-rules     → HIT, no lease change
query × N
on-disconnect  → release-session; lease 0 → drop intern
```

Overlay insert shares `rust_identity`. Overlay is the
same connection, not a second lease. Two connections
with distinct instance ids are two entries until stone
29 hashes the rules.

## The algorithm

```
entry = { arm: Arc<InternedNetwork>, leases: usize }

arm-session / get_or_build MISS:
    intern(id, arm, leases=1)

arm-session HIT:
    leases += 1

fire get_or_build HIT:
    return Arc; leases unchanged

(:wat::rete::release-session session) → session
    leases -= 1
    if 0: remove id
    session value unchanged (8 fields)

TLS drop (thread end) drops whatever remains.
```

`release-session` is a public unprimed wat Fn. Rust is
`$native`. Oracle is identity (no intern to drop). Do
not put an intern handle on the Session. Do not Weak
the table — the lease **is** the owner count.

Double `arm-session` on one connection without release
is two leases (protocol compiles once; tests that arm
twice must release twice). Fire-only tests that never
arm still intern at first fire with leases=1; they leak
until thread end unless they call `release-session`.
The service always arms at compile and releases at
hangup.

## ★ THE ONE CONTRACT DECISION

**The intern lives as long as a lease lives, not as long
as the process.** `arm-session` takes a lease.
`release-session` drops one. At zero the entry is gone.
The next compile/fire on that id rebuilds. Session bytes
do not carry the Arc. We do not revive Weak-without-owner.

Athena two connections / one intern is stone 29's key
plus this lease count. Until then, two instance ids are
two entries and two leases.

## The gate

1. New test: compile-all, fire HIT, `release-session`,
   next `get_or_build` / fire MISS, ARM_BUILDS += 1.
2. Two Sessions, same instance id (overlay): release the
   connection Session once; overlay is not a second
   lease. (Overlay shares id; only compile leased.)
   Document: overlay fire after release of the armed
   Session rebuilds — do not release mid-connection.
3. Two distinct Sessions (two compile-alls): release
   one; the other still HIT.
4. `fire_rules_reuses_arm_across_fire_and_insert_overlay`
   still green (no release in that test).
5. rete lib. clippy `-D warnings` (`--lib`).

## Predicted win

Not a FIRE cut. The product door: `release-session` deprovisions.
Session Drop does not (stone 29: no intern handle on Session).
A hangup that only drops the Value leaks the lease until thread end.
`wat/query.wat` `:stop` already calls `release-session`.
512 connections that hang up without that call keep 512 interned networks.

## Blast radius

`src/rete/kernel/arm.rs` (entry = Arc + lease).
`eval_release_session` next to `eval_arm_session`.
`runtime.rs` dispatch. `check.rs` TypeScheme.
`purity.rs` completeness if the op is fenced.
`wat/rete.wat` public `release-session`. Oracle identity
or omit (oracle has no intern). Tests in `kernel/tests.rs`.
No 9th Session field. No Weak table. No `unsafe`.

## Out of scope = REJECTED

- Session-`Vec`. Intern `names`. Facts in `bind_pool`.
- Content hash (stone 29). Intern keeper (stone 29).
- `defservice` `-on-disconnect` wiring. Service-ify.
- 297. Fact insertion. Recast vigilia. Stamp `vigilatum`.
- Drop intern on fire return (the Weak hole).

## Sequencing

After 27. Before 29. Then recast vigilia.

## Weigh (2026-08-20) — LANDED

`intern_release_drops_arm_and_next_fire_rebuilds` green
(lease 1 after compile; fire HIT no extra lease; release
removes; next fire ARM_BUILDS += 1).
`intern_release_one_session_leaves_the_other` green.
`intern_overlay_is_not_a_second_lease` green (overlay fire
after release rebuilds).
`intern_release_session_wat_mouth_drops_the_lease` green.
Overlay reuse still green. rete lib **104**. clippy
`-D warnings` (`--lib`) silent. `rg Mutex src/rete` empty.

Session still 8 fields. Next is 29 (content-address).
