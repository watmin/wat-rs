# DESIGN — STONE G: a wire `send` refuses a value that cannot cross, at the SENDER

**Drawn 2026-09-24.** The one real hole left under `RULING-purity-is-parametric.md`.

## The defect, measured in a real spawned process

A child process opens `(self-peer (Box :- [T]) i64)` inside a GENERIC fn and is called with an `Lru`
handle. The compile-time wire wall treats `T` as pure, so it passes (`check=0`). Then:

```
wire_pure    (T = i64)   parent receives  "#t/Box {:x 42}"                              the wire works
wire_handle  (T = Lru)   parent receives  RecvOutcome.Lost:
  "recv EDN decode failed: src/edn/render.rs:3587:19: unsupported substrate tag
   #rust.cache/Lru has a bare-nil body — retired (arc 278 A.0); unit variants are `#tag {}`"
```

Nothing crosses — the promise holds in the end. It fails in the WRONG PLACE and says the WRONG THING:

1. **Late and on the far side.** The sender encodes the handle as `opaque_nil` (correct per arc 294)
   and ships it; only the RECEIVER learns, as a `Lost`. (What the sender's `SendOutcome` was is
   unmeasured — the probe discarded it. Measure it first.)
2. **The message describes stale syntax, not the event.** "retired (arc 278 A.0); unit variants are
   `#tag {}`" — nothing says an impure value was sent over a wire.
3. **It locates in `src/edn/render.rs:3587`** — the F-006 family.

## The choke point

`src/kernel/message.rs`, the process-peer arm of `send` (~`:212-245`):
`value_to_edn_with(&payload_val, …)` then `bundle.peer.send(edn_str)`. By the time it ships, any opaque
is already `nil`. ⚠ The child's `self-peer` may be a DIFFERENT peer type path from `PROCESS_PEER_TYPE_PATH`
— find every arm of `send` (and `try-send`) that encodes to EDN for a wire.

## The ONE contract decision — ask the WRITER, never mirror it

The refusal is "this value contains something the EDN writer would render as `opaque_nil`." That set
must come from the writer itself, not from a parallel list:

- ⛔ NOT stone E's `value_is_hashable` / `key_eligibility()` — `NeverAKey` calls EVERY `RustOpaque`
  impure, but a Wire `Address` is an opaque that DOES cross (it encodes as `SocketAddressWire`, via
  `crate::capability::encode_capability`). Mirroring `key_eligibility` would refuse legal sends.
- ✅ A strict encode: the wire path encodes with a mode in which reaching `opaque_nil` is an error
  instead of a `nil`. Everywhere else (printing, `:wat::edn::write`) keeps `opaque_nil` UNCHANGED —
  arc 294 `BRIEF-294.i:15`: *"That is correct and final."* Only the WIRE refuses, because only the
  wire promises the value comes back whole.

## How it refuses

**Raise, at the sender, with the user's span.** The same arm already raises for programmer misuse
(its timer case: *"a genuine programmer misuse (wrong peer kind), so it still raises"*), and
`list_span` — the user's call — is in hand there. Sending an impure value over a wire is exactly a
misuse the checker could not see. The message names what happened: an impure value (and its type)
was sent over a wire peer, which carries only pure data.

⚠ Why not a `SendOutcome` variant: `Sent/Closed/Stopped/Lost` all describe the CHANNEL; none describes
a bad VALUE, and reusing `Lost` would repeat the receiver's dishonesty on the sender's side. A new
variant would make every exhaustive `send` match in the corpus change. The misuse-raises precedent in
the same arm is the fit.

## Out of scope — REJECTED

- The EDN writer's `opaque_nil` in non-wire paths (by design, arc 294).
- The compile-time wall's generic bypass itself (type-parameter bounds are the builder's call).
- The RECEIVER's misleading "retired (arc 278 A.0)" message: after this stone the sender refuses
  first, so a nil-bodied substrate tag reaching a receiver means a peer outside this runtime sent it.
  Its wording is worth a line in the report, not a change here.
