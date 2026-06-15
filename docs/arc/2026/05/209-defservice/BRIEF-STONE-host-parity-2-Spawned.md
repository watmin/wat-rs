# BRIEF — Stone host-parity-2: the `:wat::kernel::Spawned` handle marker

Executor: Shadowdancer (sonnet). Anchor: `/home/watmin/work/holon/wat-rs/` (verify `pwd`; ONLY here;
ignore `.claude/worktrees/`). Run `cargo` PLAINLY (no setsid/timeout). Trust your own clean build over
rust-analyzer (its mid-edit snapshots lie). **Do NOT commit — the Inquisitor weighs.** Full rationale:
`DESIGN-STONE-host-parity-2-Spawned.md` (this dir).

## Work in one paragraph
Mint `:wat::kernel::Spawned` — a typesub/`derive` marker (NO methods, NOT a protocol) — by deriving
the two spawn handles onto it in `wat/spawn.wat`, then retype defservice's `Handle.handle` field from
`Thread'<Op,Reply>` to the marker. Three edits (spawn.wat derives, service.wat retype, a probe rename
to avoid a name clash). All the machinery (`derive`, parametric bounds) already shipped — this just
uses it.

## Room 1 — `wat/spawn.wat` (add the derives)
Near `Bound`/`ServiceEvent` (the `:wat::kernel::` decls), add:
```wat
;; Spawned — the owner-side spawn-handle marker (typesub/derive axis; no methods). Thread'/Process'/
;; future-remote derive it so the host-agnostic Handle field + Host/spawn return can bind any of them.
;; Lifecycle = close'/join (intrinsics). A new transport's handle joins with one more `derive`.
(:wat::core::derive :wat::kernel::Thread'  :wat::kernel::Spawned)
(:wat::core::derive :wat::kernel::Process' :wat::kernel::Spawned)
```

## Room 2 — `wat/service.wat` (retype the Handle field)
The `handle-fields` quasiquote (~526) is:
```wat
handle-fields `[handle <- ~thread-ty addr <- ~addr-ty]
```
Change the handle field to the marker (a plain keyword, no type args):
```wat
handle-fields `[handle <- :wat::kernel::Spawned addr <- ~addr-ty]
```
The `thread-ty` local (~101, built via `keyword/from-string` + `string::concat` to
`wat::kernel::Thread'<…>`) is now **dead** — grep `thread-ty` to confirm it's used ONLY in
`handle-fields`, then remove its `let` binding. Update the Handle-record comment block (~519-525) so
`handle <- :wat::kernel::Thread'<…>` reads `handle <- :wat::kernel::Spawned` (+ a word on why: the
host-agnostic marker). If `thread-ty` turns out to be used elsewhere → STOP and report.

## Room 3 — `tests/probe_arc209_handle_protocol.rs` (rename to avoid the clash)
This probe declares `(:wat::core::defprotocol :wat::kernel::Spawned …)` + `extend-type … :wat::kernel::Spawned`
+ method `spawned-tag`, INLINE (it was the exploratory 267 probe). That name now collides with the new
stdlib marker. **Rename every `:wat::kernel::Spawned` in THIS file to `:t::Spawnable`** (the protocol,
the two extend-types, the param bound, the method-dispatch head). It stays a valid arc-267 regression
test (a protocol bound over the opaque `Thread'`); only the name changes. Do NOT touch its logic.

## Gate (run all; report verbatim from YOUR runs)
```
cargo test --release -p wat --test probe_arc209_spawned_marker -- --test-threads=1       # 1 passed (RED→GREEN)
cargo test --release -p wat --test probe_arc209_c3_defservice_client_face -- --test-threads=1   # passes (Handle.handle now :Spawned, holds a Thread')
cargo test --release -p wat --test probe_arc209_handle_protocol -- --test-threads=1       # passes (renamed to :t::Spawnable)
cargo test --release -p wat --test probe_arc237_derive_verb                               # passes (derive unbroken)
cargo test --release -p wat --test probe_arc267_parametric_extend_type                    # passes (267 unbroken)
cargo test --release -p wat --lib -- --test-threads=1                                     # zero NEW vs baseline 917/36
cargo test --release -p wat --test nursery -- --test-threads=1                            # zero NEW vs baseline 895/4
cargo test --release --workspace --no-run                                                 # compiles
```

## STOP triggers (REJECT — surface; do not improvise)
1. `derive` of an opaque kernel type (`Thread'`/`Process'`) into `:Spawned` doesn't register / the
   probe stays RED → STOP; report (the derive verb + 267 should make this work — surface the gap).
2. `thread-ty` is used somewhere other than `handle-fields` → STOP (the retype's blast radius was
   mis-mapped; report the other use).
3. The C.3 client-face probe goes red because a `Thread'` no longer satisfies the `Handle.handle`
   field → STOP (that would mean the marker bound doesn't accept the concrete handle — a real gap).
4. Renaming the handle_protocol probe surfaces deeper coupling to `:wat::kernel::Spawned` → STOP.

## Blast radius
`wat/spawn.wat` (+~4 lines), `wat/service.wat` (1 field + remove the dead `thread-ty` local + a
comment), `tests/probe_arc209_handle_protocol.rs` (a rename). NO Rust changes. NO change to
`derive`/`is_subtype`/the 267 arm/`Bound`/the process tier of `listener'`. The probe
`tests/probe_arc209_spawned_marker.rs` is already committed.

## Return
Report: the spawn.wat derives (file:line); the service.wat retype + the removed `thread-ty` (file:line)
+ confirm it was used only in handle-fields; the handle_protocol rename; every gate command's counts
from YOUR runs; any honest delta. If a STOP fires, STOP and report. Do NOT commit.
