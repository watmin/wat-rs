# BRIEF — intern index is thread-owned (ZERO-MUTEX)

## The work

Delete the intern Mutex. The index lives in a
`thread_local` `RefCell<FxHashMap<u64, Arc<InternedNetwork>>>`.
Lookup / intern / get_or_build keep the same contract on
the calling thread.

## Read in order

1. `docs/ZERO-MUTEX.md` — three tiers. Mutex is heresy.
2. `DESIGN-STONE-intern-zero-mutex.md`.
3. `src/rete/kernel/arm.rs` `arm_table` / lookup / intern.
4. `CURRENT-STATE-annihilate-interpretation.md` Item 12.

## Sketch

```
thread_local TABLE
lookup  = get cloned Arc
intern  = insert
build   = outside the RefCell
```

## STOP

1. **STOP-1** — `RwLock` / `AtomicPtr` / `arc-swap` /
   intern keeper thread. Those are not this stone.
2. **STOP-2** — intern `names` / facts in `bind_pool` /
   Session-`Vec` / 2e / 2o.
3. **STOP-3** — 297. Fact insertion. A second intern table.
   Service-ify. Recast vigilia. Stamp `vigilatum`.

## Done

- `rg Mutex src/rete` empty.
- Overlay reuse green. 8-worker test green.
- rete lib 100. clippy `-D warnings` silent.

Leave dirty.
