# BRIEF — Stone C0b.2e-i-a: the comms trait-object foundation

**Executor:** Shadowdancer (sonnet). **Anchor:** `/home/watmin/work/holon/wat-rs/`
(verify `pwd` first; operate only here; use `git -C /home/watmin/work/holon/wat-rs`).
Design: `DESIGN-STONE-C0b.2e-i-a-comm-trait-foundation.md` (read it fully). The probe
is already on disk and RED-verified at HEAD. Do NOT commit — the Inquisitor weighs.

## The work in one paragraph

Add three things to `src/comms/`, all additive: a named enum `ReactorClass { InMemory,
Fd }`; a `CommReceiver` method `reactor_class(&self) -> ReactorClass` (thread →
`InMemory`, process → `Fd`); and concrete recovery via `CommReceiver: Any` +
`as_any(&self) -> &dyn Any`. This makes the existing probe
`probe_arc209_c0b2eia_boxed_comm_both_tiers` compile and pass. `CommSender` is
untouched; `close` is untouched (the traits are already object-safe). No struct
changes, no runtime/checker changes.

## Read in order (the rooms)

1. `src/comms/mod.rs:806` `trait CommSender<T>` + `:827` `trait CommReceiver<T>` — the
   contracts. `ReactorClass` goes near here; `CommReceiver` gains the supertrait + two
   methods. (`CommSender` needs nothing.)
2. `src/comms/thread.rs:178` `impl<T: Send + 'static> CommReceiver<T> for Receiver<T>` —
   the in-memory impl: `reactor_class → InMemory`, `as_any → self`.
3. `src/comms/process.rs:484` `impl<T: EdnRepresentable> CommReceiver<T> for Receiver<T>`
   — the fd impl: `reactor_class → Fd`, `as_any → self`. (`process.rs:399` shows the
   existing `poll_fd` for orientation — you do NOT need to touch it.)
4. `tests/comms/trait_object.rs` — the gate probe (already written). It is the exact
   shape your additions must satisfy; mirror its expectations.

## Implementation sketch (fill the shape)

**(A) `src/comms/mod.rs` — near the trait defs:**
```rust
/// Which wait-primitive demuxes a receiver in `select'` — Stone C0b.2e-i-a.
/// `InMemory` = parked-thread crossbeam-select (no fd). `Fd` = kernel fd-poll
/// (io_uring). A closed enum on a fixed axis (two wait primitives; a third OS
/// poller is still `Fd`); the growing remote-transport axis lives in the impls,
/// every one fd-backed → `Fd`.
pub enum ReactorClass { InMemory, Fd }
```
```rust
pub trait CommReceiver<T>: std::any::Any {            // + Any supertrait
    // … existing recv / len / close UNCHANGED …
    /// The wait-primitive class `select'` groups this receiver under.
    fn reactor_class(&self) -> ReactorClass;
    /// Recover the concrete receiver (the i-b `select'` reactor bridge).
    fn as_any(&self) -> &dyn std::any::Any;
}
```

**(B) `src/comms/thread.rs` (in the `CommReceiver` impl):**
```rust
fn reactor_class(&self) -> crate::comms::ReactorClass { crate::comms::ReactorClass::InMemory }
fn as_any(&self) -> &dyn std::any::Any { self }
```

**(C) `src/comms/process.rs` (in the `CommReceiver` impl):**
```rust
fn reactor_class(&self) -> crate::comms::ReactorClass { crate::comms::ReactorClass::Fd }
fn as_any(&self) -> &dyn std::any::Any { self }
```

Derive on `ReactorClass` whatever the probe's `matches!` needs (nothing extra) — keep it
minimal; add `#[derive(Debug, Clone, Copy, PartialEq, Eq)]` so it is ergonomic for i-b.

Then `cargo build` and follow the compiler: the `Any` supertrait requires each impl's
`Receiver<T>` be `'static` — both impls already bound `T` to `'static` (thread: `Send +
'static`; process: `EdnRepresentable` ⊇ `Send + 'static`), so `self: &dyn Any` coerces.

## Blast radius (bounded)

`src/comms/mod.rs` (enum + two trait methods + one supertrait) + `src/comms/thread.rs`
(two method bodies) + `src/comms/process.rs` (two method bodies). The probe already
exists. Nothing else.

## STOP triggers (rejection — ship nothing, report)

1. **STOP-1:** the `Any` supertrait trips object-safety or collides with an existing
   supertrait — STOP, report (the design expects it clean).
2. **STOP-2:** a `CommReceiver` impl exists beyond `thread`/`process` and lacks
   `'static` (cannot satisfy `Any`) — STOP, report. (Grep `impl.*CommReceiver` to
   confirm; design expects only the two.)
3. **STOP-3:** `process::pair::<Value>()` / `thread::pair::<Value>()` absent or
   `Value: !EdnRepresentable` — STOP, report (would contradict i-0).

## The gate

```
cargo build --release
cargo test --release -p wat --test comms probe_arc209_c0b2eia_boxed_comm_both_tiers -- --test-threads=1
cargo test --release -p wat --test comms -- --test-threads=1           # all comms green (regression)
cargo test --release -p wat --test nursery -- --test-threads=1         # 895 passed / 4 failed (baseline)
cargo test --release --workspace --no-run                              # full surface compiles
```
Report the exact `test result:` line for each + any STOP/honest delta. Do NOT commit.

## Prior comparable (copy the shape)

`BRIEF-STONE-C0b.2e-i-0.md` — the immediately-prior comms-trait stone (the EDN split).
Same surface (`src/comms/{mod,thread,process}.rs` + a `tests/comms` probe), same
additive-and-recompile-cascade shape.
