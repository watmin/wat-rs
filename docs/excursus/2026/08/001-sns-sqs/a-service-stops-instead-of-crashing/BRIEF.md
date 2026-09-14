# BRIEF — a service stops instead of crashing (v3)

**Read `DESIGN.md` beside this first.** It carries the revised contract (CONTINUE, not exit), the
three-party rule, and the measured history of two dead routes — including the orchestrator's refute
that was wrong and cost a working strike. ⛔ **If you are holding v1 or v2 of this brief, discard it.**

## What changed since your last strike, in three lines

1. Your wrap-the-scrutinee instinct was **right**; the refute that made you discard it was the
   orchestrator's error, and its premise held only at the *original* wrap site. Restore that shape.
2. The contract is now **continue serving from pre-op state**, not exit — the builder ruled it on the
   networking framing, and your v2 measurement independently refuted exit.
3. **New and load-bearing:** the owner must be told. Your two floor reds were right and are the reason.

## The work, in one paragraph

Move `serve-dispatch-op`'s wrap from the whole op-dispatch match to **each public handler body (the
`Outcome` match scrutinee)**, where the form's type is `Outcome`. Add `:Faulted [cause <- String]` to
`:wat::service::Outcome` and `:Faulted [cause <- String]` to the generated `<S>::Status`. In the seam's
`Err(payload)` branch, downcast: an `AssertionPayload` broadcasts `PeerCrashed` as today and returns
`Outcome::Faulted[cause]`; any other payload still `resume_unwind`s. The serve loop's new `Faulted` arm
sends `Status::Faulted[cause]` up `self`, then **recurs with the serve fn's own `state`** — so the
failed op is a no-op and the service keeps serving.

## The rooms

| where | why you are going there |
|---|---|
| `src/runtime.rs:27472`–`:27475` the `Err(payload)` branch | the only Rust you change. Read the function's doc header first. |
| `src/assertion.rs:54` `AssertionPayload` | what to downcast to; `message` is the `cause`. |
| `src/value/value.rs:1176` `EnumValue` | what you construct. `names` is **carried, never looked up** (arc 296 G′) and params are **erased** (`src/runtime.rs:16170`), so no `src/types.rs` registration is needed. |
| `wat/service.wat:80` the `Outcome` defenum | `:Faulted [cause <- :wat::core::String]`. |
| `wat/service.wat:1600` `status-enum-def` | the owner's channel — add `:Faulted [cause <- :wat::core::String]` beside `:Started`/`:Stopped`/`:Hibernated`/`:PeersAllowed`/`:PeersDenied`. |
| `wat/service.wat:1526`–`:1556` | the `status-*-kw` minting pattern to copy for `status-faulted-kw`. |
| `wat/service.wat:2160` (your v2 emission) | where the wrap goes — on the handler body, NOT the whole dispatch. |
| `wat/service.wat:2409` `serve-params` | `state` is the serve fn's **fifth parameter**. The `Faulted` arm reads pre-op state from there; Rust never holds a state. |
| `wat/service.wat:2448` | the exemplar for sending a `Status` up `self` — `(send self (~status-hibernated-kw (~hibernate-project-name state)))`. Copy this shape for the fault notice. |
| `tests/services/probe_arc278_recv_outcome_wall.{rs,wat}` | ⭑ the two reds. Their contract CHANGES: a handler panic no longer kills the service, so the owner can no longer observe `Lost [true]`. Re-draw them to assert the owner sees `Status::Faulted` carrying the cause. **Re-draw, do not patch to fit** — see STOP-2. |

## Implementation sketch

```rust
Err(payload) => {
    crate::kernel::peer::broadcast_peer_crashed_best_effort(&clients_val);
    match payload.downcast_ref::<crate::assertion::AssertionPayload>() {
        Some(ap) => Ok(Value::Enum(EnumValue {
            type_path: ":wat::service::Outcome".into(),   // params erased at runtime
            variant_name: "Faulted".into(),
            names: Arc::new(vec!["cause".into()]),        // carried, not looked up
            fields: vec![Value::String(ap.message.clone())],
        })),
        None => std::panic::resume_unwind(payload),       // substrate bug — unchanged
    }
}
```

```wat
;; the wrap moves INWARD — onto the handler body only
(:wat::kernel::serve-dispatch-op ~peers-only-expr
  (:wat::core::let ~arm-let-bindings ~body))

;; the new arm, beside Continue and Stop
((:wat::service::Outcome::Faulted cause)
  (:wat::core::do
    ;; the OWNER learns — this is the hole the two floor reds exposed
    <send (~status-faulted-kw cause) up `self`, facing the SendOutcome arms>
    ;; keep serving from the serve fn's OWN pre-op state
    (~serve-name self l selectables next-id state)))
```

## Blast radius

`wat/service.wat` — 2 enum variants, 1 kw mint, 1 wrap move, 1 new arm. `src/runtime.rs` — one branch.
`tests/services/probe_arc278_recv_outcome_wall.*` — 2 expectations re-drawn.
Arity stays **2**, so `src/check.rs:12400` and `src/intrinsic/kernel/serve.rs:216` revert to untouched.

⛔ **`src/intrinsic/kernel/serve.rs`'s header carries three stale citations** — `check.rs:11347`
(→ `:12400`), `runtime.rs:25359` (→ `:19512`), and `runtime.rs:33041`/`:33091`. Reasoning sound, line
numbers not.

## STOP triggers

1. **STOP-1 — measure the `Status::Faulted` blast radius BEFORE writing.** A new variant reds every
   exhaustive match on `<S>::Status`. Expected: the macro's own matches plus ~2 in `circuit.wat`. If it
   is larger, STOP and report it **with the matches** (`grep -o … | sort -u`, never a bare count) — the
   orchestrator got a blast radius wrong by 388× on this very stone.
2. **STOP-2 — if re-drawing the two wall tests would WEAKEN what the owner learns**, STOP. They exist
   because a mute owner was already ruled unacceptable once; the new assertion must be at least as
   strong (a cause carried, not merely an event observed). Patching them to pass is the one outcome
   this stone must not produce.
3. **STOP-3 — if a genuine (non-`AssertionPayload`) panic cannot be distinguished**, STOP.
4. **STOP-4 — if `catch_unwind` returning a value breaks `serve`'s recursion**, STOP and report it.
5. **STOP-5 — internal arms (`-run`, `-tick`) are OUT OF SCOPE.** They dispatch through a
   `SelfOutcome` branch (`wat/service.wat:2034`) that this stone does not wrap. ⚠ Say so in the SCORE:
   **the publisher's `-run` crash is NOT fixed by this stone.** That is the next stone, not this one.
6. **STOP-6 — `circuit.wat` / `sqs.wat` / `sns-fanout.wat` are OUT OF SCOPE.** The 19 `redial failed`
   arms and the ~59 placeholders are D1-c.

## Verify

- `cargo build --release` **and** `cargo nextest run --release --no-run`.
- `./scripts/floor.sh`, read the **Summary line**, never a piped exit code.
- ⭑ The acceptance gate: `./target/release/wat wat-scripts/scratch-pad/probe-a-handler-raise-kills-the-service.wat`
  → the `b-` line must report a **served call**.
- Happy path: `./scripts/capped.sh --limit 8g ./target/release/wat wat-scripts/fanout/circuit.wat 2000 4 3 8192 true 1000` → `distinct=8000;dup=0`.
- ⭑ **The negative control:** a handler tripping a genuine Rust panic must STILL crash the service.
- ⭑ **The owner control:** drive a handler panic and assert the owner observes the cause. A fault the
  owner cannot see is this stone failing in its most dangerous direction.

## Shape to copy

`wat-scripts/scratch-pad/probe-arc278-wire-dos-service-killed.wat` — arc 278 stone 2, the same wall for
a narrower cause (*"a bad caller, malicious or dumb, cannot crash anything"*), and the source of this
stone's headline.
