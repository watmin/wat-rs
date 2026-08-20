# DESIGN-STONE — intern is content-addressed (Athena HIT)

> **RULED 2026-08-20 — REJECTED.** Connections are discrete.
> Identical rules / identical queries do **not** share an
> intern. Instance `rust_identity` stays the key. Overlay
> HIT on the **same** connection (insert does not rewrite
> the network) is Item 12 and stays. Cross-connection
> content-address is the sharing problem: conn A
> `release-session` while conn B is still firing is only
> safe if lease arithmetic is a cross-connection invariant.
> Do not construct that invariant. Athena "compile for free"
> is not worth a hangup that can drop someone else's arm.
> Stone 27 (thread-owned) + 28 (per-id lease) already give
> 512 discrete Sessions. Do not hash `rules`. Do not hash
> queries. Do not intern query-memory. Do not spawn a
> keeper. This stone does not land.

> **Origin (2026-08-20).** Intern key is `PMap::rust_identity`
> — instance `AtomicU64`, copied on clone, minted on every
> structural rewrite. Insert overlays facts without rewriting
> the network → same id → fire HIT (Item 12, already true).
> Independent `compile-all` of **equal rules** mints a new
> network PMap → new id → MISS → `build_rete_arm` again.
> Athena: the second connection with the same rules gets the
> compiled network for free while the first lease still
> lives. That is a **structural** key, not an instance id.
> Requires 27 (legal index) and 28 (lease count — two
> connections share one intern).

## The measurement we have

`network_identity` reads `PersistentMap.rust_identity()`.
`PMap` Hash is already structural (order-independent
key/value hashes). Equality of two independently compiled
networks is not the intern key today. Overlay HIT does
not need this stone. Cross-compile HIT does.

Worker-pool Athena: stone 27's table is thread-owned, so
even a structural key MISS across workers. This stone
adds the process-wide door **in front of** that cache:
a tier-3 intern keeper (one owner thread, mailbox). TLS
HIT remains the same-worker path. Keeper HIT is the
cross-worker path. Mutex is still heresy. `AtomicPtr`
is still unsafe theater.

## The algorithm

```
key = structural hash of Session.rules
      (PersistentVector of Rule / Query records)
      NOT rust_identity
      NOT facts
      NOT names intern
      NOT query-memory

TLS lookup(key) HIT → Arc
else keeper Lookup(key)
    HIT  → store TLS, lease++
    MISS → build_rete_arm
           keeper Intern(key, arc)
           TLS store, lease=1

arm-session uses key from session.rules
fire get_or_build uses the same key
overlay: rules unchanged → same key → HIT
two compile-alls of equal rules → same key → HIT

release-session: lease-- on that key; 0 → TLS remove
                 + keeper Drop. Last process lease
                 drops the keeper entry.
```

Hash: `Value`'s existing `Hash` into a `u64` (DefaultHasher
or the FNV-1a already used for Export ABI — pick one, pin
it, test stability across two independently built equal
rule vectors). Do not stringify EDN.

The wat Session constructor still runs per connection
(8 fields, own facts vector). What is shared is
`InternedNetwork` (the rust circuits). Do not intern the
network PMap as a second table. Do not intern query
encode. Do not intern scratch.

Keeper: lazy `std::thread` + bounded channel, spawned on
first intern miss that needs process share. Sequential
body **is** the serialization (`ZERO-MUTEX.md` tier 3).
Not a wat `defservice`. Not 297 protobuf. Tests join
nothing — process lifetime, entries evict.

If the keeper is too much in one strike: land content
key on the TLS table first (Athena within a worker),
then keeper as a follow-up in the same stone's second
commit **only if** the TLS key gate is green. The
contract is process-wide HIT. Do not ship TLS-only and
call Athena done.

## ★ THE ONE CONTRACT DECISION

**The intern key is the structural hash of the compiled
rules, not the network instance id.** Overlay HIT stays
(rules did not change). Two `compile-all` of equal rules
HIT while any lease lives. Facts never enter the key.
`rust_identity` remains the overlay stamp on the network
PMap; fire may still assert it for Item 12. Intern no
longer uses it as the map key.

## The gate

1. Two independent `compile-all` of equal rules on **one
   thread**: ARM_BUILDS += 1 total, not 2.
2. Overlay reuse test still green (same rules, facts
   overlay).
3. Two threads, equal rules: second thread's compile-all
   does **not** `build_rete_arm` (keeper HIT). ARM_BUILDS
   process delta is 1.
4. After both `release-session`, a third compile rebuilds.
5. Unequal rules (one extra Rule): MISS, ARM_BUILDS += 1.
6. rete lib. clippy `-D warnings` (`--lib`).
   `rg Mutex src/rete` still empty.

## Predicted win

Not a FIRE cut. The product door: 512 connections, some
sharing rules, some not, compile once per unique ruleset
per process (while leased), never collide.

## Blast radius

`src/rete/kernel/arm.rs` (key fn, TLS cache, keeper).
`eval_arm_session` / `eval_release_session` / fire
`get_or_build` read `session.rules`. Tests. No Session
field. No intern `names`. No crate if std thread+channel
suffice (crossbeam already in tree). No `unsafe`.

## Out of scope = REJECTED

- Intern `names`. Facts in the key. Query-encode intern.
  Scratch intern.
- Session-`Vec`. Intern the network PMap as a second
  table. 2e / 2o.
- 297. Protobuf. Service-ify. `defservice` wiring.
- Recast vigilia before this lands. Stamp `vigilatum`.
- Hash of `rust_identity`. Hash of facts. Hash of EDN
  text.

## Sequencing

After 27 and 28. Then recast vigilia. Do not stamp
until a live recast is L1+L2 0 and clippy zero.

## Weigh

Not yet. Do not mark LANDED from memory.
