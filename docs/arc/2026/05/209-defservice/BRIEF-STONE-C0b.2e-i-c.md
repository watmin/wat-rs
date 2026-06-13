# BRIEF — Stone C0b.2e-i-c: split `select'`→`select'`+`poll'`; `SelectEvent`→`ServiceEvent`

**Executor:** Shadowdancer (sonnet). **Anchor:** `/home/watmin/work/holon/wat-rs/`
(verify `pwd`; operate only here; `git -C /home/watmin/work/holon/wat-rs`). Design:
`DESIGN-STONE-C0b.2e-i-c-select-poll-split.md` (read it fully). Do NOT commit — the
Inquisitor weighs.

## The work in one paragraph

A naming rename+split, no behavior change. `select'` is one verb with two arities/return
types; split it: the 3-arg service-multiplexer form becomes a new verb `poll'` (routing to
the existing 3-arg logic, renamed), and `select'` becomes 1-arg-only (a clear error on 3
args, naming `poll'`). Separately, rename the event sum `SelectEvent<I,O>` → `ServiceEvent<I,O>`
everywhere (same variants, same fields). The 1-arg `select'` fan-in is byte-for-byte
unchanged. Migrate the c0b1b test to `poll'`/`ServiceEvent` — that migration is the proof
`poll'` works.

## Read in order (the rooms)

1. `src/runtime.rs:4546` `":wat::kernel::select'" =>` dispatch arm; `:23451`
   `eval_peer_select_prime` (arity-dispatches 1 vs 3); `:23873` `eval_peer_select_prime_3arg`
   (+ `SELECT_EVENT_TYPE` const + the variant construction).
2. `src/check.rs:4851` `":wat::kernel::select'" =>` dispatch arm; `:10837` `infer_select_prime`
   (routes 3→_3arg); `:10950` `infer_select_prime_3arg`; the 4 `"wat::kernel::SelectEvent"`
   head strings (:10964/10995/11022/11032) + comment :6575.
3. `wat/spawn.wat:48-67` the `SelectEvent<I,O>` defenum (rename head + the doc).
4. `tests/nursery/probe_arc209_c0b1b_select_listener.rs` — the 3-arg caller + `SelectEvent::*`
   match arms (migrate to `poll'` / `ServiceEvent::*`).
5. `src/kernel/mod.rs:41` doc mention of the kernel verbs (add `poll'`).

## Implementation sketch (fill the shape)

**(A) runtime.rs:**
```rust
// add beside the select' arm:
":wat::kernel::poll'" => eval_poll_prime(args, list_span, env, sym),
// rename: fn eval_peer_select_prime_3arg(...) -> fn eval_poll_prime(...)  (body verbatim)
// in eval_peer_select_prime: drop the `if args.len()==3 { ...3arg... }` branch; require
//   args.len()==1, else a RuntimeError naming poll' ("the 3-arg service multiplexer is poll'").
```
**(B) check.rs:**
```rust
":wat::kernel::poll'" => { /* mirror the select' arm, calling infer_poll_prime */ }
// rename: fn infer_select_prime_3arg -> fn infer_poll_prime  (body verbatim)
// in infer_select_prime: drop `if args.len()==3 { return infer_select_prime_3arg(...) }`;
//   args.len()!=1 → a CheckError ("select' takes one peer vector; the 3-arg service
//   multiplexer is poll'").
```
**(C) Rename `SelectEvent` → `ServiceEvent`** in: `wat/spawn.wat` (defenum head + doc), the
runtime `SELECT_EVENT_TYPE` const value + the variant-head construction strings, the 4
check.rs head strings + the comment, and the c0b1b test (`(select' …)`→`(poll' …)`,
`:wat::kernel::SelectEvent::*`→`:wat::kernel::ServiceEvent::*`). Same variants
(`:Shutdown :Connection :Message :Closed :Lost`), same fields — only the type name changes.

Then `cargo build` and follow the compiler.

## Blast radius

`src/runtime.rs`, `src/check.rs`, `wat/spawn.wat`, `tests/nursery/probe_arc209_c0b1b_select_listener.rs`,
`src/kernel/mod.rs`. No `peer.rs`/`spawn.rs`/`comms` change. No new struct/type beyond the
defenum rename.

## STOP triggers (rejection — ship nothing, report)

1. **STOP-1:** a `SelectEvent` reference that is NOT a simple rename target (a load-bearing
   compared string elsewhere) — STOP, report.
2. **STOP-2:** removing `select'`'s 3-arg branch breaks a 1-arg caller — STOP, report.
3. **STOP-3:** `poll'` not recognized as a builtin after adding the dispatch arms — STOP,
   report (the design expects the arm to suffice, mirroring `select'`).

## The gate

```
grep -rn "SelectEvent" src/ wat/ tests/                                  # must be EMPTY (all ServiceEvent)
grep -rn "wat::kernel::poll'" src/runtime.rs src/check.rs                 # present (both arms)
cargo build --release
cargo test --release -p wat --test nursery probe_arc209_c0b1b -- --test-threads=1   # poll'/ServiceEvent GREEN
cargo test --release -p wat --test nursery connection_primitive -- --test-threads=1 # 1-arg select' unchanged
cargo test --release -p wat --test nursery -- --test-threads=1           # 895 passed / 4 failed (baseline)
cargo test --release --workspace --no-run                                # full surface compiles
```
Report each exact `test result:` line + the two grep outputs + any STOP/honest delta. Do
NOT commit.

## Prior comparable (copy the shape)

`BRIEF-STONE-C0b.2e-i-b.md` (the just-shipped Peer collapse — same files runtime.rs/check.rs,
same dispatch-arm + verb-rename patterns, structural-grep gate).
