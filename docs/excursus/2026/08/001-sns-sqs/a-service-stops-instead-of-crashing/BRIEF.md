# BRIEF — a service stops instead of crashing

**Read `DESIGN.md` beside this first.** It carries the one contract decision, the sub-decision that
keeps it honest (only an `AssertionPayload` converts), two routes affirmatively rejected with
numbers, and seven trap-doors — including the STASH-DANCE, which governs how you edit at all.

## The work, in one paragraph

`:wat::kernel::serve-dispatch-op` already wraps every op-handler body in `catch_unwind` and already
broadcasts `PeerCrashed` to connected clients on a crash; it then calls `std::panic::resume_unwind`
and the service dies for everyone. Give the form a **third argument** — an on-fault function the
`defservice` macro emits — and on a caught **`AssertionPayload`** downcast, broadcast as today, then
apply that function to the cause and return its `Outcome` instead of resuming. The macro's on-fault
function returns `Outcome::Stop` carrying the **pre-op** state, so the service leaves through the
serve loop's existing clean-exit arm. Any other panic payload still resumes, unchanged.

## The rooms

| where | why you are going there |
|---|---|
| `src/runtime.rs:27435` `eval_kernel_serve_dispatch_op_tail` | ⭑ THE SEAM. Read its whole doc header first — it explains why this is the one hook with `clients` reachable, and why it must stay in tail position. The arity check is at `:27441`; the two branches at `:27468`–`:27476`. |
| `src/host/test_runner.rs:301` | ⭑ THE PROVEN SHAPE — `catch_unwind` around `apply_function`, and the loop continues in the same process afterwards. This is the live precedent that the runtime survives catching this payload. Copy how it calls and how it recovers. |
| `src/assertion.rs:54` `AssertionPayload` | the struct to downcast to: `message` / `actual` / `expected` / `location` / call stack. The cause string you hand the on-fault fn comes from here. |
| `src/runtime.rs:25359` `apply_function` | how Rust applies a wat function — cited by `serve.rs`'s own derivation. |
| `src/check.rs:12400` `infer_serve_dispatch_op` | the type authority (there is no registered `TypeScheme`). Today: `clients` checked for error coverage only, `body`'s type IS the form's type. You add the third arg's type and keep that passthrough. |
| `src/check.rs:4775` | where the inference above is dispatched from. |
| `src/intrinsic/kernel/serve.rs:33`+ | the `#[wat_intrinsic]` declaration and its `:ControlFlow` derivation. The doc argues the classification from the OLD two-arg shape; extend it, don't silently invalidate it. |
| `wat/service.wat:2512` | the single emission site. `state` is in scope — the sibling `ServiceEvent::Closed` arm at `:2519` passes it to `serve-name` one line below. |
| `wat/service.wat:2203` | the `Outcome::Stop` arm your on-fault value lands in: replies, fans `sends`, returns `nil`. Read it so you know what you are handing it. |
| `wat/service.wat:760` `hibernate-project-def` | the shape for emitting a **top-level** helper `defn` from the macro. Use it — see STOP-3. |
| `wat/service.wat:2448` | the precedent for calling `(~hibernate-project-name state)` from macro-emitted code. |
| `wat-scripts/scratch-pad/probe-a-handler-raise-kills-the-service.wat` | the acceptance gate. It prints `b-dial=connect-REFUSED` today; the strike makes the innocent client's call succeed. |

## Implementation sketch

Rust, at the seam — the `Err` branch is the only one that changes:

```rust
Err(payload) => {
    crate::kernel::peer::broadcast_peer_crashed_best_effort(&clients_val);
    match payload.downcast_ref::<crate::assertion::AssertionPayload>() {
        Some(ap) => {
            let cause = Value::String(/* ap.message (+ location, if it reads well) */);
            apply_function(on_fault_fn, vec![cause], sym, /* span */)
        }
        // NOT a wat raise — a substrate bug. Unchanged.
        None => std::panic::resume_unwind(payload),
    }
}
```

wat, in the macro — emit the helper at top level beside `hibernate-project-def`, then pass its symbol:

```wat
;; emitted once per service, NOT per dispatch
(:wat::core::defn ~on-fault-name [state <- ~state-ty  cause <- :wat::core::String] -> ~outcome-ty
  (:wat::service::Outcome::Stop state :wat::core::None <empty sends>))

;; at :2512 — the third argument closes over nothing; state is applied by the seam
(:wat::kernel::serve-dispatch-op ~peers-only-expr
  (:wat::core::match (:wat::kernel::retag-op op …) ~@serve-op-arms)
  ~on-fault-partial)
```

⭑ **How `state` reaches the on-fault function is yours to choose and to state in the SCORE** — a
partial application built at the emission site, a two-arg fn the seam applies with a state it also
receives, or a fourth argument. The DESIGN pins only that it must be the **pre-op** state the serve
loop passed in, never anything the panicking body produced.

**D2-a's projection:** the builder ruled that the durable state is projected on the way out. The
on-fault `defn` is where `(~hibernate-project-name state)` goes, because that call already works from
macro-emitted code (`:2448`). If projecting there proves to change what the `Stop` arm can accept,
that is STOP-2, not a thing to improvise around.

## Blast radius

`wat/service.wat` (1 emission + 1 emitted defn) · `src/runtime.rs` (one branch) · `src/check.rs`
(one inference) · `src/intrinsic/kernel/serve.rs` (one declaration). **No `.wat` corpus change and no
new user-visible form** — confirm that claim rather than inheriting it.

## STOP triggers

1. **STOP-1 — if converting the panic requires touching how `serve` recurses**, STOP and report it.
   The trampoline (a wat fn's body is evaluated via `eval_tail` inside a plain Rust `loop`) is the
   reason this hook can exist at all; a fix that trades TCO for crash-safety is a different stone and
   the builder decides it.
2. **STOP-2 — if `hibernate-project` cannot be called from the on-fault path**, STOP and say why.
   Report whether the honest shipping shape is `Outcome::Stop` **without** projection (a graceful exit
   that drops durable state, which is what happens today minus the crash) — do not quietly ship one
   and describe the other.
3. **STOP-3 — if the on-fault function cannot be emitted at top level** and an inline `(fn …)` is the
   only shape that works, STOP and report the per-dispatch cost before shipping it. This seam runs on
   **every message**; the last wall that walked at every expansion cost 5–9 % of floor time.
4. **STOP-4 — if a genuine (non-`AssertionPayload`) panic cannot be distinguished**, STOP. Converting
   a substrate bug into a tidy shutdown is the one outcome this stone must not produce.
5. **STOP-5 — the 19 `"redial failed"` arms and the ~59 `Malformed` placeholders are OUT OF SCOPE.**
   If you find yourself editing `circuit.wat` / `sqs.wat` / `sns-fanout.wat`, stop: that is D1-c, a
   later stone, and each arm owes its own reading.

## Verify

- `cargo build --release` **and** `cargo nextest run --release --no-run` — the build does NOT compile
  tests, and that gap reddened the floor once already.
- `./scripts/floor.sh`, read the **Summary line**, never a piped exit code.
- The acceptance gate:
  `./target/release/wat wat-scripts/scratch-pad/probe-a-handler-raise-kills-the-service.wat`
- The circuit's happy path unchanged:
  `./scripts/capped.sh --limit 8g ./target/release/wat wat-scripts/fanout/circuit.wat 2000 4 3 8192 true 1000`
  → `distinct=8000;dup=0`.
- ⭑ **A negative control you must write:** a handler that trips a genuine Rust panic (not
  `assertion-failed!`) must STILL crash the service. A wall that swallows substrate bugs is worse
  than no wall, and this is the one check that can tell the two apart.

## Shape to copy

`docs/excursus/2026/08/001-sns-sqs/the-dial-declares-its-peer/SCORE.md` — the last stone that changed
`wat/service.wat` alongside Rust, including how it handled the stdlib freeze. And
`wat-scripts/scratch-pad/probe-arc278-wire-dos-service-killed.wat` — arc 278 stone 2, the same wall
for a narrower cause, and the source of this stone's headline.
