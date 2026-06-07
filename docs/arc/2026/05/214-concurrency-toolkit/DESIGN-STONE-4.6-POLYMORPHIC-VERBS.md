# Stone 214.4.6 — polymorphic peer verbs (`send'` / `recv'` / `try-recv'` / `close'` / `select'`)

> The peer types (4.4) and the spawn dispatcher (4.5) exist; peers are
> `Value::RustOpaque(":wat::kernel::Thread'")` / `(":wat::kernel::Process'")`.
> 4.6 gives wat code the verbs to drive them. After 4.6, Slice 5 migrates the
> old `:wat::kernel::send`/`recv`/`select`/`close` callers onto these primes and
> Slice 6 retires `typed_channel`.

## The dispatch mechanism — DECIDED: defclause (the blessed recipe, NOT a hand-rolled match, NOT multimethod)

This is precisely what defclause is FOR. Grounded against the live template
(`wat/core.wat:58`, the `:wat::core::+` defclause from 237.8b):

```
(:wat::core::defclause :wat::core::+
  [x <- :wat::core::i64  y <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::+ x y))
  [x <- :wat::core::f64  y <- :wat::core::f64] -> :wat::core::f64 (:wat::core::f64::+ x y))
```

A polymorphic wat **defclause** whose per-concrete-type clauses route to per-type
Rust **primitives** (`:i64::+` / `:f64::+`, keyword-head builtins at
`runtime.rs:3450/8046`). The clause MATCHER dispatches via `assignable`
per-position on the concrete arg type (`check.rs ~5281`). No hand-rolled match.

The peer verbs are the SAME shape — monomorphic per concrete peer type
(`project_dispatch_clause_vs_intrinsic`: clause = monomorphic; `send'`/`recv'`/
`try-recv'`/`close'` have invariant returns and no type-var flow → CLAUSE):

```
(:wat::core::defclause :wat::kernel::send'
  [p <- :wat::kernel::Thread'   v <- <payload-ty>] -> :wat::core::nil (:wat::kernel::Thread'/send  p v))
  [p <- :wat::kernel::Process'  v <- <payload-ty>] -> :wat::core::nil (:wat::kernel::Process'/send p v))
```

Why this and not my first draft: a "2-arm `type_path` match inside the eval fn"
REINVENTS the clause matcher by hand — the scattered string-matching the
substrate (and arc 255) exists to abolish. typed_channel.rs:33 rightly calls
*multimethod* over-engineered here; the answer between "hand-match" and
"multimethod" is **defclause** — lighter than multimethod, declarative (not
hand-rolled), already the stdlib's own arithmetic mechanism. **Four questions:**
Obvious ✅ (reads like `:wat::core::+`) · Simple ✅ (reuses the existing
entity-kind; zero new dispatch machinery) · Honest ✅ (the substrate's blessed
mechanism, marked at `wat/core.wat`; dig-before-assert) · Good UX ✅. The
breadcrumb's "multimethod" shorthand and my "hand-match" draft are BOTH
superseded by defclause.

## Two load-bearing prerequisites the defclause shape demands

1. **Peer `RustOpaque` must report its specific type as `declared_type_name`.**
   Today `type` of any `RustOpaque` → `:rust::opaque` (`runtime.rs:5220`). For the
   clause matcher's `assignable` to pick the `Thread'` vs `Process'` clause, a
   peer value must report `:wat::kernel::Thread'` / `:wat::kernel::Process'` as
   its declared type (route `declared_type_name` through the `RustOpaque.type_path`
   for these). This is THE bridge that lets defclause dispatch over opaque peers.
2. **Wat-level type registration of `:wat::kernel::Thread'` / `Process'`** (4.4
   deferred it to here, peer.rs:33-37) — now load-bearing: the clause param types
   must name registered types.

## The load-bearing asymmetry — the Value↔EDN bridge

The two peers carry DIFFERENT wire types (from 4.5, grounded):
- **Thread′** = `ThreadOwnedCell<Thread<Value, Value>>` — sends/recvs `Value`
  **directly** (in-process Arc sharing; no serialization).
- **Process′** = `ThreadOwnedCell<ProcessPeerBundle{Process<String,String>}>` —
  wire is **EDN String**: `send'` must encode `Value → EDN String`
  (`value_to_edn` + `wat_edn::write`); `recv'`/`try-recv'` must decode
  `EDN String → Value` (`read_edn`). The `HolonRepresentable for Value` impl in
  `spawn.rs` already defines this encoding.

So each verb's two arms are NOT symmetric: thread arm is a pass-through; process
arm wraps the call in encode/decode. This asymmetry is the real content of 4.6
(the dispatch is trivial; the bridge is the work).

## Proactive split

- **4.6a — the four uniform verbs as defclauses over per-peer primitives.**
  - **Leaves (Rust keyword-head primitives, the per-type impls — mirror `:i64::+`):**
    `:wat::kernel::Thread'/send|recv|try-recv|close` (downcast `Thread<Value,Value>`,
    call the 4.4 method, Value pass-through) and
    `:wat::kernel::Process'/send|recv|try-recv|close` (downcast `ProcessPeerBundle`,
    bridge Value↔EDN via the `spawn.rs` `HolonRepresentable for Value` impl). 8 primitives.
  - **Polymorphic surface (wat defclauses in `wat/kernel.wat` or sibling):** `send'`,
    `recv'`, `try-recv'`, `close'`, each with a `Thread'` clause and a `Process'`
    clause routing to the matching leaf. The clause matcher does the dispatch.
  - Ships a complete, useful peer surface, built the same way as stdlib arithmetic.
- **4.6b — `select'`.** Heterogeneous multiplex over N peers (return the first
  ready + its value). Harder: thread peers use crossbeam select; process peers
  use the io_uring `comms::process::Select` (POLL_ADD+POLLHUP). Mixed thread+
  process selection is the genuinely new design — give it its own stone on the
  settled 4.6a foundation. (Stepping-stone test: YES — 4.6a's downcast+bridge
  helpers are exactly what 4.6b reuses; building them first shrinks 4.6b.)

## Open detail — the payload param type (resolve at strike)

The clause param types are concrete (`:wat::core::i64` in the arithmetic template).
The PEER arg (`p`) is concrete (`Thread'`/`Process'`) and carries the dispatch.
The PAYLOAD arg (`v`) on `send'` must accept ANY wat value. wat has no top-level
∀-generics (`project_dispatch_clause_vs_intrinsic`), so `v <- :T` is not a free
var. Resolve at strike: dispatch on `p` only and give `v` the substrate's
universal/any payload type (grep for an existing "any" type or the payload type
the legacy `:wat::kernel::send` declares), OR confirm the clause matcher accepts a
permissive payload position. This is the one genuinely-open shape question; the
dispatch mechanism (defclause) is settled. (Type registration of
`:wat::kernel::Thread'`/`Process'` is prereq #2 above — load-bearing for the
clause param types, not a separate step.)

## Cadence

4.6a: sub-DESIGN (this) → FM-2-bis probe (a wat-level thread-peer round-trip via
`send'`/`recv'`/`close'`, RED at HEAD — the verbs don't exist) + a process-peer
round-trip probe (integration, `--test-threads=1`) → BRIEF + EXPECTATIONS →
spawn sonnet → SCORE vs own re-run → commit + push. Then 4.6b (`select'`).

## Ownership / consume semantics note

`close'` CONSUMES the peer (thread: `close()`→join; process: `close()`→wait,
both take `self`). But the peer is held behind `Arc<ThreadOwnedCell<…>>` inside a
`Value::RustOpaque` — `with_ref` gives `&T`, not owned `T`. `close'` therefore
needs an owning path (e.g. `with_owned`/take, or an interior `Option<T>` the cell
can move out of once). Resolve in 4.6a's strike: confirm the `ThreadOwnedCell`
API for a move-out-once, or model close via a `&self` shutdown that drops the
endpoints without consuming (sending EOF) + a separate wait. Grounded decision at
strike time against `custodia.rs`.
