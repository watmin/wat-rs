# DESIGN-STONE C0b.2e-i-0 — extract `EdnRepresentable` (decomplect the holon crutch)

> Foundational prerequisite for the Peer unification (C0b.2e-i). `HolonRepresentable` is a crutch: it
> mandates EDN-compliance but drags the holographic encoding-IR (`to_holon_ast` → `HolonAST` →
> `write_holon_ast_TAGGED`) along, so the comms wire demands "holon-representable" when the honest need
> is **plain EDN**. Builder: *"we've been using holon as a crutch in a bunch of places … we just need
> edn compliance."* This stone extracts the plain-EDN contract, impls it for `Value`, and loosens the
> comms wire onto it. It does NOT touch the peer yet (that's C0b.2e-i) — pure decomplection.

## Where we are (grounded, read this session)

- `trait HolonRepresentable: Send + 'static` (`comms/mod.rs:110`) = `to_holon_ast`/`from_holon_ast`
  (the holographic IR) + `to_wire`/`from_wire` (the EDN string; default = `write_holon_ast_tagged(
  to_holon_ast())` — TAGGED; `String` overrides to passthrough).
- Impls (all `comms/mod.rs`): `String` (`:154`), `HashSet` (`:205`), `Vec` (`:253`), `HashMap`
  (`:354`), tuples 2-5 (`:434`-`:515`). The collections build the wire by RECURSING `to_holon_ast`
  (39 call sites) → `HolonAST` IR → tagged EDN. So `HolonAST` is the encoding-IR.
- The comms wire is bound on `HolonRepresentable`: `comms::process` `Sender`/`Receiver`/`Select`/
  `Clone`/`Debug` (`process.rs:245,295,451,485,756`); `CommSender`/`CommReceiver` are the trait
  contracts (`mod.rs:620,641`). ~47 `: HolonRepresentable` bound sites across 4 files
  (comms/mod.rs, comms/process.rs, edn_shim.rs, kernel/peer.rs).
- `Value` is NOT `HolonRepresentable` → can't be a wire `T` today; the `send'`/`recv'` socket arms
  pre-encode `Value`→EDN at the arm via `value_to_edn` (the plain-EDN escape, the
  contract-not-encoding scar).
- Plain-EDN codec exists: **`value_to_edn_string(v: &Value) -> String`** (`edn_shim.rs:2088`, built on
  `value_to_edn_notag` — PLAIN, no tags) + **`edn_to_value(...)`** (`:898`). `WireError`
  (`comms/mod.rs:702`).

## The contract decision (pinned)

**Split the conflated trait into supertrait + subtrait:**
```rust
pub trait EdnRepresentable: Send + 'static {           // the EDN wire contract (was HolonRepresentable's to_wire/from_wire)
    fn to_wire(&self) -> String;
    fn from_wire(s: &str) -> Result<Self, WireError> where Self: Sized;
}
pub trait HolonRepresentable: EdnRepresentable {       // EDN + the holographic IR
    fn to_holon_ast(&self) -> holon::HolonAST;
    fn from_holon_ast(ast: &holon::HolonAST) -> Result<Self, WireError> where Self: Sized;
}
```
- `to_wire`/`from_wire` MOVE to `EdnRepresentable` (required — no default, since the supertrait can't
  see `to_holon_ast`). Each existing `HolonRepresentable` impl gains an explicit `impl
  EdnRepresentable` carrying its current `to_wire`/`from_wire`: `String` → passthrough (its current
  override); collections → the current tagged default made explicit
  (`write_holon_ast_tagged(&self.to_holon_ast())` / `from_wire` via `read_holon_ast_tagged` +
  `from_holon_ast`). **No behavior change** — tagged stays tagged.
- **`impl EdnRepresentable for Value`** — `to_wire` = `value_to_edn_string(self)` (PLAIN EDN, no
  tags); `from_wire` = `edn_to_value(parse(s))`. `Value` does NOT impl `HolonRepresentable` (it is not
  a holographic value; it is a plain wat value that serializes as plain EDN).
- **Rebound the comms wire on `EdnRepresentable`** — `comms::process` `Sender`/`Receiver`/`Select`/
  `Clone`/`Debug` + `CommSender`/`CommReceiver` bounds: `T: HolonRepresentable` → `T: EdnRepresentable`.
  Backward-compatible (every `HolonRepresentable` is `EdnRepresentable`); additive (`Value` now
  qualifies).

**Four questions — minimal (this stone) vs broad (the full crutch sweep):**
- This stone = **minimal**: extract the trait, impl for `Value`+`String`, split the collection impls,
  rebound the comms wire. Contained to the 4 comms-subsystem files; backward-compatible; unblocks
  C0b.2e-i. Obvious/Simple/Honest/Good-UX all hold.
- **Broad** (reclassify every remaining `HolonRepresentable` *bound* to `EdnRepresentable` where only
  EDN is needed; decide whether the collection *wire* should be plain too) = a follow-on decomplection
  sweep the realization opens. NOT this stone (named, tracked, not deferred-as-vague).

## The gate (refactor + new capability)

A refactor + a primitive addition — no pre-committed wat RED (the disconfirming fact is structural:
`Value: !EdnRepresentable` at HEAD). The gate:
1. **New capability (ships with the impl):** a Rust round-trip test — a `Value` sent over a
   `comms::process` `Sender<Value>` → `Receiver<Value>` returns the same `Value` (plain-EDN wire).
   Won't compile at HEAD (`Value: !EdnRepresentable`); GREEN after. (Place in `tests/comms/process.rs`
   or a `#[cfg(test)]` unit test — `Value`/comms reachable.)
2. **Regression (the decomplect changed no behavior):** the existing socket-wire probes green —
   `probe_arc209_c0b2b_socket_peer` / `c0b2c` / `c0b2d` / `c0b3a0` (String wire, unchanged); any
   existing collection-wire test green (tagged unchanged).
3. Full nursery serial 895/4 (baseline only) + full workspace test surface compiles (the
   `HolonRepresentable`→`EdnRepresentable` rebound is a cascade — compile every binary).

## Files touched

`src/comms/mod.rs` (trait split + the impl splits: String/Vec/HashSet/HashMap/tuples gain explicit
`EdnRepresentable` impls; `Value` impl added — or `Value`'s impl lives in the value layer importing
the trait), `src/comms/process.rs` (rebound the 5 bound sites), `src/edn_shim.rs` /
`src/kernel/peer.rs` (rebound any `HolonRepresentable` bounds that only need EDN). NO peer struct
change (that's C0b.2e-i). NO runtime arm change yet (the `send'`/`recv'` arms still pre-encode until
C0b.2e-i moves encoding into the socket comms via `Sender<Value>`).

## STOP triggers (rejection — ship nothing, report)

1. **STOP-1 (coherence):** if splitting the trait hits a coherence wall (e.g. a blanket
   `impl<T: HolonRepresentable> EdnRepresentable` collides with the concrete `Value`/`String` impls) —
   STOP, report. Use explicit per-type `EdnRepresentable` impls (no blanket); do not force a blanket
   that breaks the concrete impls.
2. **STOP-2:** if `Value`'s plain-EDN `from_wire` (`edn_to_value`) needs context (a SymbolTable / type
   env) not available at the comms layer — STOP, report (the wire decode must be context-free or carry
   what it needs).
3. **STOP-3:** if a `HolonRepresentable` bound being rebound to `EdnRepresentable` actually USES
   `to_holon_ast` (genuinely holographic, not a crutch) — STOP, report that site (keep it
   `HolonRepresentable`).

## Out of scope = rejected

- **The Peer merge** — C0b.2e-i (this only makes `Value` wire-capable + loosens comms). **The broad
  bound-reclassification sweep** — a follow-on (named above). **Retiring the collection
  `HolonRepresentable` impls** — not here (they stay; EDN via the supertrait).

## The deadlock contract carries

Pure type-contract refactor; no transport/lifecycle change. [[feedback_vended_primitives_never_deadlock]]
[[feedback_contract_not_encoding]] is the lesson this stone structurally enforces (the wire contract
is EDN, never the holon-tagged IR).
