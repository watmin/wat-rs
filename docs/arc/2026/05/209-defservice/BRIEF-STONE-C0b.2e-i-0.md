# BRIEF — Stone C0b.2e-i-0: extract `EdnRepresentable` (decomplect the holon crutch)

**Executor:** Shadowdancer (sonnet). **Anchor:** `/home/watmin/work/holon/wat-rs/` (verify `pwd`;
`.claude/worktrees/` illegal; `git -C /home/watmin/work/holon/wat-rs`). Design:
`DESIGN-STONE-C0b.2e-i-0-edn-representable.md` (read it fully). Do NOT commit — the Inquisitor weighs.

## The work in one paragraph

Split `HolonRepresentable` into a plain-EDN supertrait `EdnRepresentable` (`to_wire`/`from_wire`) +
`HolonRepresentable: EdnRepresentable` (the holographic `to_holon_ast`/`from_holon_ast`). Add
`impl EdnRepresentable for Value` (PLAIN EDN via `value_to_edn_string`/`edn_to_value`). Loosen the
comms wire bounds from `HolonRepresentable` to `EdnRepresentable`. This makes `Value` a legal wire `T`
(unblocking the C0b.2e-i Peer merge) and stops the comms wire demanding the holographic IR — with
**zero behavior change** (every `HolonRepresentable` is still `EdnRepresentable`; tagged stays tagged).
NO peer struct change, NO runtime arm change (those are C0b.2e-i).

## Read in order (the rooms)

1. `src/comms/mod.rs:110` `trait HolonRepresentable` + `:130`-`:139` (the defaulted `to_wire`/
   `from_wire`) — the split point.
2. `src/comms/mod.rs:154` (`String` impl — `to_wire` passthrough override), `:205`/`:253`/`:354`
   (`HashSet`/`Vec`/`HashMap`), `:434`-`:515` (tuples) — each currently `impl HolonRepresentable`;
   each must gain an explicit `impl EdnRepresentable` carrying its `to_wire`/`from_wire`.
3. `src/comms/mod.rs:620` `CommSender<T>` + `:641` `CommReceiver<T>` (the trait contracts) +
   `src/comms/process.rs:245,295,451,485,756` (the `T: HolonRepresentable` bound sites) — rebound to
   `EdnRepresentable`.
4. `src/edn_shim.rs:2088` `value_to_edn_string(v: &Value) -> String` (PLAIN EDN; built on
   `value_to_edn_notag`) + `:898` `edn_to_value` — the `Value` impl's codec.
5. `src/comms/mod.rs:702` `WireError` — the error type both traits use.
6. `src/kernel/peer.rs:330` (doc says "must implement `HolonRepresentable` for the EDN wire") + any
   `HolonRepresentable` bound in `edn_shim.rs`/`peer.rs` — rebound to `EdnRepresentable` if it only
   needs EDN (STOP-3 if it genuinely uses `to_holon_ast`).

## Implementation sketch (fill the shape; the compiler cascade is your guide)

**(A) Split the trait (`comms/mod.rs`).**
```rust
pub trait EdnRepresentable: Send + 'static {
    fn to_wire(&self) -> String;
    fn from_wire(s: &str) -> Result<Self, WireError> where Self: Sized;
}
pub trait HolonRepresentable: EdnRepresentable {
    fn to_holon_ast(&self) -> holon::HolonAST;
    fn from_holon_ast(ast: &holon::HolonAST) -> Result<Self, WireError> where Self: Sized;
}
```

**(B) Split each existing impl** — keep its current wire behavior, just move `to_wire`/`from_wire` into
an `EdnRepresentable` impl, leaving `to_holon_ast`/`from_holon_ast` in the `HolonRepresentable` impl:
```rust
// String — the passthrough it already does:
impl EdnRepresentable for String { fn to_wire(&self){ self.clone() } fn from_wire(s){ Ok(s.to_string()) } }
impl HolonRepresentable for String { fn to_holon_ast(&self){…} fn from_holon_ast(…){…} }   // unchanged
// Vec<T> / HashSet / HashMap / tuples — the tagged default, now explicit:
impl<T> EdnRepresentable for Vec<T> where T: HolonRepresentable {
    fn to_wire(&self) -> String { crate::edn_shim::write_holon_ast_tagged(&self.to_holon_ast()) }
    fn from_wire(s) -> Result<Self,WireError> { let ast = crate::edn_shim::read_holon_ast_tagged(s)?; Self::from_holon_ast(&ast) }
}
impl<T> HolonRepresentable for Vec<T> where T: HolonRepresentable { fn to_holon_ast(&self){…} … }  // unchanged body
```
(Confirm the exact current `to_wire`/`from_wire` per type and move it verbatim. NO behavior change.)

**(C) `impl EdnRepresentable for Value`** — PLAIN EDN, no holon tags. Place it where orphan rules
allow + layering is clean (the value layer importing `comms::EdnRepresentable`, or `comms/mod.rs`
importing `Value` — match how `String`'s impl is sited). `Value` does NOT impl `HolonRepresentable`.
```rust
impl EdnRepresentable for Value {
    fn to_wire(&self) -> String { crate::edn_shim::value_to_edn_string(self) }
    fn from_wire(s: &str) -> Result<Self, WireError> {
        crate::edn_shim::edn_to_value(/* parse s */).map_err(|e| WireError::new(format!("Value from_wire: {e}")))
    }
}
```
(Confirm `edn_to_value`'s exact signature/parse path — STOP-2 if it needs a SymbolTable/type-env not
available at comms.)

**(D) Rebound the comms wire** — `T: HolonRepresentable` → `T: EdnRepresentable` at the `CommSender`/
`CommReceiver` impls + `comms::process` `Sender`/`Receiver`/`Select`/`Clone`/`Debug` bounds. Then
`cargo build` and follow the cascade: every site that breaks is a bound to rebind (EdnRepresentable
unless it calls `to_holon_ast` → keep HolonRepresentable, STOP-3).

**(E) Gate test (ships with the impl)** — a `Value` round-trips over `comms::process`:
```rust
// build a comms::process pair as Sender<Value>/Receiver<Value>, send a Value, recv it, assert equal
// (mirror an existing comms::process round-trip test, but T = Value instead of String)
```
Won't compile at HEAD (`Value: !EdnRepresentable`); GREEN after.

## Blast radius (bounded)

`src/comms/mod.rs` (trait split + impl splits + maybe Value impl) + `src/comms/process.rs` (rebound) +
`src/edn_shim.rs`/`src/kernel/peer.rs` (rebound EDN-only bounds) + the gate test. NO peer struct
change. NO runtime `send'`/`recv'` arm change. NO new wat surface.

## STOP triggers (rejection — ship nothing, report)

1. **STOP-1 (coherence):** trait split hits a blanket-vs-concrete coherence wall — STOP, report; use
   explicit per-type impls, never a blanket that breaks `Value`/`String`.
2. **STOP-2:** `Value::from_wire`/`edn_to_value` needs a SymbolTable/type-env unavailable at comms —
   STOP, report.
3. **STOP-3:** a rebound site actually calls `to_holon_ast` (genuinely holographic) — STOP, report;
   keep it `HolonRepresentable`.

## The gate

`cargo build --release` clean (the rebound cascade), then:
```
cargo test --release -p wat --test comms <the new Value-round-trip test name> -- --test-threads=1
cargo test --release -p wat --test nursery probe_arc209_c0b2b_socket_peer probe_arc209_c0b2c probe_arc209_c0b2d -- --test-threads=1
cargo test --release -p wat --test probe_arc209_c0b3a0_self_peer
cargo test --release -p wat --test nursery -- --test-threads=1            # 895 passed / 4 failed (baseline)
cargo test --release --workspace --no-run                                 # full surface compiles (the rebound cascade)
```
Report the exact `test result:` line for each + any STOP/honest delta. Do NOT commit.
