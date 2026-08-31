# BRIEF — the session ceiling's zero point belongs to the session

Move the memory ceiling's zero point off the thread and onto the session, so a second
`compile-all` cannot forgive the first session everything it has staged. Read `DESIGN.md` beside
this file first — its ★ ONE CONTRACT DECISION constrains what you may claim in the doc you leave
behind, and it is the part of this strike most easily got wrong while every test passes.

## Read in order, and why

1. **`src/alloc_counter.rs:113-165`** — `SESSION_ORIGIN`, `mark_session_origin`, `session_bytes`.
   The whole defect is in these three. Read the doc blocks in full: they state the assumption, name
   the line that moves, and rule the over-count direction safe. **You are keeping two of those
   three statements and replacing one.**
2. **`src/rete/kernel/arm.rs:1205`** — the one `mark_session_origin()` call, inside `arm-session`.
   The session's identity is available here; find how `ARM_TABLE` keys its entries
   (`network_identity`) and use the same key rather than inventing one.
3. **`src/rete/kernel/session.rs:1404,1420`** — `session_ceiling_breach(sym)` and its insert-door
   dress `check_insert_ceiling`. Both need to know **which** session they are judging.
4. **`src/rete/kernel/fire/delta.rs:682`** and **`src/rete/kernel/insert.rs:194,235`** — the three
   call sites. Each already holds the session; confirm that before threading anything.
5. **`strike-session-origin/probe.rs.txt`** → append to
   `tests/rete/probe_arc278_fixpoint_round_cap.rs` (its `run()` helper is what the probe uses).
   **`probe.wat.txt`** → `tests/rete/probe_arc278_session_ceiling_second_session.wat`.

## Implementation sketch

```rust
thread_local! {
    static SESSION_ORIGINS: RefCell<FxHashMap<u64, usize>> = /* … */;
}
pub fn mark_session_origin(id: u64) { /* insert, do not clobber a foreign id */ }
pub fn session_bytes(id: u64) -> usize { /* per-id origin; unmarked marks itself */ }
```

`session_ceiling_breach(sym, id)` and `check_insert_ceiling(sym, span, staged, id)` carry the id
down. Note `alloc_counter.rs`'s own warning about the hot path: `SESSION_ORIGIN` was
`const`-initialised deliberately because it is touched from the insert ceiling check, and a lazily
initialised `thread_local!` allocates on first touch. **Whatever you replace it with is on that
same path** — say in your report what it costs.

## Blast radius

Six files, listed with line numbers in DESIGN. If you find a seventh, STOP rather than widening.

## STOP triggers

1. **If the three call sites do NOT already hold the session, STOP** and surface it before
   threading an id through a signature that has no business carrying one.
2. **If keying by network identity cannot distinguish two sessions in the corpus — e.g. two
   sessions compiled from the same network share an id — STOP.** That is a real property of
   `network_identity` and it would make the key wrong; report it rather than picking a second key.
3. **If registering the import door costs more than one call, STOP** and leave it. DESIGN admits it
   only at that price.
4. **If you find yourself writing that the fix separates two sessions' allocations, STOP.** It does
   not. See the ★ decision.

## The mutation proof

- **The probe** — RED today, GREEN after. Free.
- **The control arm is the counter-proof and is NOT a formality.** `verdict("control") == REFUSED`
  proves the ceiling is live at that workload; without it, a green probe arm is indistinguishable
  from a ceiling that stopped firing for some *other* reason. **Run it and name its result
  separately.** ⚠ The orchestrator's previous strike named a counter-proof that could not have
  failed; do not accept this one on its name either — check that the control would go red if the
  ceiling were disabled.
- **Then break it deliberately:** make `mark_session_origin` clobber regardless of id, confirm the
  probe reddens, restore.

## What to leave behind in the doc

The ★ decision, in the module's own voice: the origin now follows the session, the reading still
includes other sessions' bytes on the same thread, that over-count refuses **early** which the
module already rules safe, and **a per-session origin is not a per-session allocator.** Keep the
sentence that named this hole; it earned its place.
