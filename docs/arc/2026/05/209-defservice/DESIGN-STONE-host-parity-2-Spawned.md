# DESIGN — Stone host-parity-2: the `:wat::kernel::Spawned` handle marker

> Arc 209 host-parity leg, stone 2 of 5 (`Bound` ✓ → **`Spawned`** → `listener'` leveling → `Host`
> protocol → generic `start`). Builds on the shipped `derive` verb (arc-237 follow-on) + parametric
> protocol bounds (arc 267). Grounded against HEAD `4593be62`. Name by intueri: **`Spawned`** (past
> participle = the lifecycle state you hold; no collision with `Handle`/`Host`/`Peer'`).

## Why

defservice's `Handle.handle` is typed `Thread'<Op,Reply>` (thread-only). For a host-agnostic `start`
(stone 5), the field must hold a thread *or* process *or* future-remote handle. The clojure-faithful
uniform bound is a **typesub/`derive` marker** (Clojure's `isa?` axis — NOT a protocol; the handle
has no honest uniform method, and lifecycle is the existing `close'`/`join` intrinsics).

## What it delivers

In `wat/spawn.wat` (beside `Bound`/`ServiceEvent`/`spawn-program'`):
```wat
;; Spawned — the owner-side spawn-handle marker. Thread'/Process'/future-remote derive it;
;; the host-agnostic Handle field + Host/spawn return are bound by it. Lifecycle = close'/join
;; (intrinsics). A new transport's handle joins with one more `derive`, zero central edit.
(:wat::core::derive :wat::kernel::Thread'  :wat::kernel::Spawned)
(:wat::core::derive :wat::kernel::Process' :wat::kernel::Spawned)
```
And defservice's `Handle.handle` field is retyped to the marker.

## The one contract decision

`:Spawned` is a **non-parametric marker** (a bare hierarchy node), not `:Spawned<I,O>`. The handle is
held for lifetime/RAII; the typed interface is the `addr` (`Address'<Op,Reply>`). The concrete
`Thread'<Op,Reply>` still carries its params and *also* derives the param-less `:Spawned`;
`is_subtype` + the arc-267 `assignable` head-arm make `Thread'<Op,Reply>` satisfy the `:Spawned`
bound. A marker needs **no type declaration** — annotations resolve permissively; the `derive` edges
are the whole mechanism (grounded via the FM-2-bis probe).

## The edits

1. **`wat/spawn.wat`** — the doc + the two `derive` forms (above).
2. **`wat/service.wat`** — `Handle.handle` field: `handle <- ~thread-ty` → `handle <- :wat::kernel::Spawned`
   (the `handle-fields` quasiquote ~526). The `thread-ty` local (~101, built via `keyword/from-string`
   concat) becomes **dead** — remove it (confirm no other use). Update the Handle-record comment block.
3. **`tests/probe_arc209_handle_protocol.rs`** — it declares `(:wat::core::defprotocol :wat::kernel::Spawned …)`
   INLINE (it was the exploratory 267 probe). That name now collides with the stdlib marker. Rename its
   inline protocol to a test-local name (e.g. `:t::Spawnable`) — it stays a valid arc-267 regression
   test (a protocol bound over the opaque `Thread'`); only the name changes.

## Scope / out

- **Process tier of `listener'`** — untouched (still bare `Listener'`); that's stone 3.
- **`Host` protocol / generic `start`** — stones 4/5. This stone only mints the marker + retypes the
  Handle field (which a thread-only `start` already populates with a `Thread'` — proving the marker
  accepts it end-to-end via the C.3 probe).
- **No change** to `derive`/`is_subtype`/the 267 arm/`Bound` — all consumed as-is.

## Probe

`tests/probe_arc209_spawned_marker.rs` (committed RED) — a real `Thread'` from `spawn-program'` flows
to a `:wat::kernel::Spawned`-typed param. RED at HEAD (no derive edge). GREEN once spawn.wat derives
it. The existing C.3 client-face probe must stay green (its `Handle.handle` is now `:Spawned`, holding
a `Thread'`).
