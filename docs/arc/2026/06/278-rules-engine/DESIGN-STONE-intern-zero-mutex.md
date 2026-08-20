# DESIGN-STONE — intern index is thread-owned (ZERO-MUTEX)

> **Origin (2026-08-20).** The 512-session protocol is one Session
> per connection, compile once, fire once, query N, disconnect.
> N workers run bespoke rete side by side. `InternedNetwork` is
> already tier 1 (`Arc`, frozen after `build_rete_arm`). The
> **index** that names those Arcs is
> `OnceLock<Mutex<HashMap<u64, Arc<InternedNetwork>>>>` in
> `src/rete/kernel/arm.rs`. That Mutex is the only Mutex in
> `src/rete/`. `docs/ZERO-MUTEX.md` is the law: OnceLock is
> allowed; Mutex is not. This stone deletes the Mutex. It does
> not change the intern **key** (still `PMap::rust_identity`).
> It does not drop entries. It does not service-ify.

## The measurement we have

`rete_arm_lookup` / `rete_arm_intern` take
`.lock().unwrap_or_else(into_inner)` on every compile and
every fire `get_or_build`. The circuits never mutate under
the lock — only the map of id → Arc does. Contention is
the cache door, not Session corruption. Overlay HIT
(`fire_rules_reuses_arm_across_fire_and_insert_overlay`)
is already true on one thread. Cross-thread rete is legal
today only because the Mutex serializes the door.

A worker pool of 20, 512 pinned connections: each connection
lives on one thread. Those threads must compile and fire
without taking a process lock and without writing each
other's intern.

## The algorithm

```
thread_local! {
    TABLE: RefCell<FxHashMap<u64, Arc<InternedNetwork>>>
}

lookup(id)  → TABLE.with | m.get(id).cloned()
intern(id)  → TABLE.with | m.insert(id, arc)
get_or_build → lookup HIT / else build outside the map,
               then intern. Same as today, no lock.
```

`RefCell` is same-thread. Cross-thread access cannot happen.
No `ThreadOwnedCell` unless the map itself travels as a
Value — it does not. No keeper thread. No `AtomicPtr`.
No `RwLock`. No `arc-swap` crate.

`build_rete_arm` stays **outside** the map borrow (today it
is already outside the Mutex). Do not hold `RefCell` across
compile.

Sequi: the arm table stays `ambient-context`. The rune
moves from process Mutex to thread-owned map. Census
`ARM_BUILDS` stays `performance-counter`.

## ★ THE ONE CONTRACT DECISION

**The intern index is thread-owned.** N threads compiling
and firing never take a lock and never see each other's
table. `InternedNetwork` remains `Arc`. The Session stays
8 fields. Fire HIT after `compile-all` still holds **on
the thread that armed**. Overlay HIT still holds (same
thread, same `rust_identity`).

Athena share across workers is stone
`DESIGN-STONE-intern-content-address.md`, not this stone.
Eviction is `DESIGN-STONE-intern-eviction.md`, not this
stone.

## The gate

1. `rg Mutex src/rete` is empty.
2. `fire_rules_reuses_arm_across_fire_and_insert_overlay`
   still green. ARM_BUILDS 1 for first fire/compile, 0 for
   second fire and overlay fire (same thread).
3. New test: N worker threads each compile-all + fire +
   fire on a private Session. Per thread, ARM_BUILDS delta
   is 1. No deadlock. Threads do not share intern HIT
   (instance `rust_identity` differs; even equal rules
   MISS across threads until stone 29).
4. rete lib.
5. clippy `-D warnings` (`--lib`).

## Predicted win

Not a FIRE cut. The product door: 512 pinned sessions on
N workers compile and fire without a process lock. Duplicate
`build_rete_arm` across workers of equal rules is accepted
until stone 29.

## Blast radius

`src/rete/kernel/arm.rs` (`arm_table`, lookup, intern,
`get_or_build`). Tests in `kernel/tests.rs`. Sequi comment
on the table. No Session field. No `.wat`. No crate. No
`unsafe`. No new intern **table** (same map, different
owner). STOP-3 of `BRIEF-arm-at-compile.md` ("New intern
table") is 297 — not this.

## Out of scope = REJECTED

- Mailbox intern keeper (tier 3). That is the Athena
  cross-worker door in stone 29.
- `AtomicPtr` / `arc-swap` snapshot publish (unsafe theater
  or a crate for a door that is not on the token loop).
- `RwLock` (still a lock).
- Eviction. Content hash. Session-`Vec`. Intern `names`.
- Intern on insert. 2e / 2o. 297. Fact insertion.
- Service-ify rete. `defservice` `-on-connect`.
- Recast vigilia. Stamp `vigilatum`.

## Sequencing

1. This stone — Mutex out.
2. `DESIGN-STONE-intern-eviction.md` — last lease drops
   the entry. Needs a table that can drop.
3. `DESIGN-STONE-intern-content-address.md` — structural
   key + process-wide HIT. Needs a legal index.
4. Recast vigilia. Do not stamp until 0+0.

## Weigh (2026-08-20) — LANDED

`rg Mutex src/rete` empty. Overlay reuse green.
`intern_index_thread_owned_workers_do_not_collide` green
(8 workers, 8 instance ids, second fire HIT per thread).
rete lib **100** passed. clippy `-D warnings` (`--lib`)
silent.

Not a FIRE cut. Next is 28 (eviction).
