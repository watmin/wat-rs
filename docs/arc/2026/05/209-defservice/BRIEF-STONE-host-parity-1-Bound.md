# BRIEF — Stone host-parity-1: `Bound<S,R>` (listener' thread tier returns a named struct)

Executor: Shadowdancer (sonnet). Anchor: `/home/watmin/work/holon/wat-rs/` (verify `pwd`; ONLY here;
ignore `.claude/worktrees/`). Run `cargo test`/`cargo build` PLAINLY (no setsid/timeout). Trust your
own clean build over rust-analyzer (its mid-edit snapshots go stale and lie). **Do NOT commit — the
Inquisitor weighs.** Full design + rationale: `DESIGN-STONE-host-parity-1-Bound.md` (this dir).

## Work in one paragraph
`listener'`'s thread tier returns a `Tuple<Listener'<S,R>, Address'<S,R>>`. Replace it with a named
parametric struct `:wat::kernel::Bound<S,R> {listener, address}` so callers use `Bound/listener` /
`Bound/address` instead of positional `first` / `second`. Four pieces: (1) mint the `defstruct` in
`wat/spawn.wat`; (2) the check-side return type (`listener_tuple` helper → `Bound<S,R>`); (3) the
runtime value (`eval_listener_prime` thread tier → `Value::Struct`); (4) migrate the three thread-tier
callers off `first`/`second`. **Thread tier ONLY — do not touch the process tier.**

## Rooms (read in order)

1. **`wat/spawn.wat:101-106`** — the `(:wat::core::defenum :wat::kernel::ServiceEvent<I,O> …)` with
   `:Connection [peer <- :wat::kernel::Peer'<I,O>]`. This proves an opaque parametric type resolves as
   a field type in a `:wat::kernel::`-namespaced decl in this file. **Add, right after it (~line 107):**
   ```wat
   ;; ── Bound<S,R> — the listening state minted by (listener' (thread) :S :R) ─────
   ;; A STRUCT, not a record: its fields are non-EDN RustOpaque kernel entities
   ;; (Listener'/Address'). `listener` is the server accept-side; `address` is what
   ;; clients dial via connect'. Replaces the bare Tuple the thread tier returned.
   (:wat::core::defstruct :wat::kernel::Bound<S,R>
     [listener <- :wat::kernel::Listener'<S,R>
      address  <- :wat::kernel::Address'<S,R>])
   ```
   (Model the parametric `<S,R>` syntax on `:wat::lru::State<K,V>` in
   `crates/wat-lru/wat/lru/CacheService.wat:174` — `defstruct Name<P,Q> [f <- :T<P,Q> …]`.)

2. **`src/check.rs:10360-10366`** — the `listener_tuple(s, r)` helper. Change its body to return
   `TypeExpr::Parametric { head: "wat::kernel::Bound".into(), args: vec![s, r] }` (drop the
   `TypeExpr::Tuple`), and **rename it `bound_type`**. Update its 5 call sites — 10270, 10285, 10289,
   10349, 10355 (run `grep -n listener_tuple src/check.rs` to get the live line numbers; rename ALL).
   **Do NOT touch the process-tier return** (`Listener'<S,R>`, the `else if … ProcessOpts` arm
   ~10290-10334) — that helper is not it.

3. **`src/runtime.rs:18722-18756`** — `eval_listener_prime`, the **thread tier** `else` branch.
   It currently ends:
   ```rust
   Ok(Value::Tuple(Arc::new(vec![
       make_rust_opaque(LISTENER_TYPE_PATH, Listener::from_crossbeam(rx)),
       make_rust_opaque(ADDRESS_TYPE_PATH, Address::from_thread(tx)),
   ])))
   ```
   Replace with a `Value::Struct`:
   ```rust
   Ok(Value::Struct(Arc::new(StructValue {
       type_name: ":wat::kernel::Bound".into(),
       fields: vec![
           make_rust_opaque(LISTENER_TYPE_PATH, Listener::from_crossbeam(rx)),
           make_rust_opaque(ADDRESS_TYPE_PATH, Address::from_thread(tx)),
       ],
   })))
   ```
   `StructValue` is already in scope in runtime.rs (precedent: `Value::Struct(Arc::new(StructValue {
   type_name: ":wat::kernel::Thread".into(), … }))` at runtime.rs:20347 — note the **leading colon**
   in `type_name`). The **process tier** (the `if is_process` branch returning a bare
   `make_rust_opaque(LISTENER_TYPE_PATH, …)`, ~18721) is UNCHANGED.

4. **Migrate the FIVE thread-tier callers** off `first`/`second` (mandatory — `first`/`second` break
   on a struct). The full set was swept (`grep -rn "listener'"` across `wat/ tests/ crates/`):
   - **`wat/service.wat:507-516`** — the `start-body` quasiquote. Change
     `~l-sym (:wat::core::first ~pair-sym)` → `~l-sym (:wat::kernel::Bound/listener ~pair-sym)` and
     `~addr-sym (:wat::core::second ~pair-sym)` → `~addr-sym (:wat::kernel::Bound/address ~pair-sym)`.
     Also update the human comment at 488-494 (`l (first pair)` / `addr (second pair)` →
     `l (Bound/listener b)` / `addr (Bound/address b)`).
   - **`tests/probe_arc209_c2_defservice_dispatch.rs:56-57`** — `(:wat::core::first pair)` /
     `(:wat::core::second pair)` → `(:wat::kernel::Bound/listener pair)` / `(:wat::kernel::Bound/address pair)`.
   - **`tests/nursery/probe_arc209_c0b1b_select_listener.rs:74-75`** — same swap.
   - **`tests/probe_arc209_c0b3bb_verbs.rs:54`** — `THREAD_VERB_PROGRAM`: `l (:wat::core::first pair)`
     → `l (:wat::kernel::Bound/listener pair)` (only `first` here; `allow' l` follows — the
     thread-tier-error assertion is UNCHANGED, you're just fixing how `l` is extracted).
   - **`tests/nursery/probe_arc209_c0b1_thread_connection.rs:45-46`** — `first pair`/`second pair`
     → `Bound/listener`/`Bound/address` (green live round-trip).
   - **NOT to migrate:** `tests/nursery/probe_arc209_c0b2a_listener_host_thread_only.rs:43` binds
     `pair` UNUSED (no `first`/`second`); it stays as-is and stays green (an unused Bound binding
     type-checks fine). Leave it.
   - **Final sanity sweep:** re-run `grep -rn "first pair\|second pair\|first ~pair\|second ~pair" wat/ tests/`
     and confirm ZERO remain that target a thread-tier `listener'` result. Report the result.

## Gate (run all; report verbatim from YOUR runs)
```
cargo test --release -p wat --test probe_arc209_bound_listener -- --test-threads=1     # 1 passed (RED→GREEN)
cargo test --release -p wat --test probe_arc209_c2_defservice_dispatch -- --test-threads=1   # passes (migrated)
cargo test --release -p wat --test nursery probe_arc209_c0b1b_select_listener -- --test-threads=1   # passes (migrated)
cargo test --release -p wat --test probe_arc209_c3_defservice_client_face -- --test-threads=1   # passes (start fn uses Bound)
cargo test --release -p wat --test probe_arc209_c0b3bb_verbs -- --test-threads=1         # passes (straggler migrated)
cargo test --release -p wat --test nursery probe_arc209_c0b1_thread_connection -- --test-threads=1   # passes (straggler migrated)
cargo test --release -p wat --test nursery probe_arc209_c0b2a_listener_host_thread_only -- --test-threads=1   # passes (unused pair binding, untouched)
cargo test --release -p wat --lib -- --test-threads=1                                   # zero NEW vs baseline 915/36
cargo test --release -p wat --test nursery -- --test-threads=1                          # zero NEW vs baseline 895/4
cargo test --release --workspace --no-run                                               # compiles
```

## STOP triggers (REJECT — surface; do not improvise)
1. The `defstruct` won't accept `:wat::kernel::Listener'<S,R>` / `:wat::kernel::Address'<S,R>` as field
   types → STOP (this contradicts the grounding — `ServiceEvent`'s `Peer'` field proves it; surface
   the exact error).
2. Rust can't build `Value::Struct` for `:wat::kernel::Bound` (StructDef not registered at the call,
   accessor unresolved) → STOP (the defstruct must load before `listener'` runs; report the error).
3. You find a thread-tier `listener'` caller (one that applies `first`/`second` to the result) NOT
   among the FIVE now listed in room #4 → STOP and report it (blast radius still mis-mapped); do not
   silently migrate beyond the plan. (The five were swept and confirmed; a sixth would be news.)
4. Migrating forces a change to the **process tier** of `listener'`, or to `first`/`second`
   themselves → STOP (process-tier leveling is a later sub-stone; this stone is thread-tier only).

## Blast radius
`wat/spawn.wat` (+~6 lines), `src/check.rs` (1 helper body + rename at 5 sites), `src/runtime.rs`
(1 return expr, thread tier), `wat/service.wat` (2 template lines + a comment), and FOUR test files
(2 lines each, except c0b3bb which is 1 line): `tests/probe_arc209_c2_defservice_dispatch.rs`,
`tests/nursery/probe_arc209_c0b1b_select_listener.rs`, `tests/probe_arc209_c0b3bb_verbs.rs`,
`tests/nursery/probe_arc209_c0b1_thread_connection.rs`. The probe
`tests/probe_arc209_bound_listener.rs` is already committed. NO new types beyond `Bound`; NO
process-tier changes; NO change to `first`/`second`/`Listener'`/`Address'`/`Peer'`; do NOT touch
`c0b2a` (unused `pair` binding stays).

## Return
Report: the `defstruct` (file:line); the `bound_type` body + all 5 renamed call sites; the
`eval_listener_prime` thread-tier return (confirm process tier untouched); each migrated caller
(file:line, old → new); the straggler-sweep result; every gate command's counts from YOUR runs; any
honest delta. If a STOP fires, STOP and report. Do NOT commit.
