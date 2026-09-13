# BRIEF — a service stops instead of crashing

**Read `DESIGN.md` beside this first.** It carries the one contract decision, the sub-decision that
keeps it honest (only an `AssertionPayload` converts), two routes affirmatively rejected with measured
reasons, and seven trap-doors — including the STASH-DANCE, which governs how you edit at all.

## The work, in one paragraph

⛔⛔ **A PRIOR REVISION OF THIS BRIEF WAS WRONG — if you are holding one that says "add
`:Faulted` to `:wat::service::Outcome`", discard it.** The seam cannot return an `Outcome`: `serve` is
declared `-> :wat::core::nil` (`wat/service.wat:3498`, `:3937`) and `serve-dispatch-op` sits in its
tail position, so each dispatch arm consumes its own `Outcome` **inside the arm** and the seam's value
is `nil` (or a `TailCall` for the recursion). See DESIGN §THE ROUTE for the full correction.

`:wat::kernel::serve-dispatch-op` already wraps every op-handler body in `catch_unwind` and already
broadcasts `PeerCrashed` to connected clients on a crash; it then calls `std::panic::resume_unwind`
and the service dies for everyone. Give the form **two more arguments** — the serve loop's `state` and
an on-fault function the macro emits — and in that one Rust branch downcast the payload; **if it is an
`AssertionPayload`**, broadcast as today, then `apply_function` the on-fault fn to `(state, cause)` and
return its result, which is `nil` and ends the serve recursion cleanly. The on-fault function is where
D2-a's durable-state projection happens. Any other panic payload still resumes, unchanged.

## The rooms

| where | why you are going there |
|---|---|
| `src/runtime.rs:27435` `eval_kernel_serve_dispatch_op_tail` | ⭑ THE SEAM. Read its whole doc header first — it explains why this is the one hook with `clients` reachable and why it must stay in tail position. **The only branch you change is `Err(payload)` at `:27472`–`:27475`.** |
| `src/assertion.rs:54` `AssertionPayload` | the struct to downcast to: `message` / `actual` / `expected` / `location` / call stack. The `cause` string comes from here. |
| `src/host/test_runner.rs:301` | ⭑ THE PROVEN SHAPE — `catch_unwind`, and the loop **continues in the same process** afterwards. The live precedent that the runtime survives catching this payload. |
| `src/runtime.rs:19512` `apply_function` | how Rust applies a wat function. ⚠ `serve.rs`'s derivation cites it at `:25359` — **stale**; `:19512` verified. |
| `wat/service.wat:2531` | the single emission site — where `~@serve-op-arms` is spliced into the seam, and where the two new arguments go. |
| `wat/service.wat:3498` + `:3937` | ⭑ READ THESE FIRST. `(defn ~serve-name ~serve-params -> :wat::core::nil ~serve-body)` — the proof that the seam's value is `nil`-typed and why an `Outcome` cannot be returned there. |
| `wat/service.wat:2203` the `Outcome::Stop` arm | the clean-exit shape, one level BELOW the seam: it replies, fans `sends`, returns `nil`. Note it does **not** project state — which is why a bare nil return cannot satisfy D2-a. |
| `wat/service.wat:760` `hibernate-project-def` | the shape for emitting a **top-level** helper `defn` from the macro. Use it — see STOP-6. |
| `wat/service.wat:2409` `serve-params` | `state` is the serve fn's **fifth parameter**, so your arm reads the pre-op state from scope. This is the soundness argument — see DESIGN trap-door 2. |
| `wat/service.wat:2448` | the precedent for calling `(~hibernate-project-name state)` from macro-emitted code. |
| `wat-scripts/scratch-pad/probe-a-handler-raise-kills-the-service.wat` | the acceptance gate. It prints `b-dial=connect-REFUSED` today; the strike makes the innocent client's call succeed. |

## Implementation sketch

Rust — the `Err` branch, and the arity check above it:

```rust
Err(payload) => {
    crate::kernel::peer::broadcast_peer_crashed_best_effort(&clients_val);
    match payload.downcast_ref::<crate::assertion::AssertionPayload>() {
        // a wat raise → hand it to the macro's on-fault fn; its result is nil and
        // ends the serve recursion cleanly.
        Some(ap) => apply_function(on_fault_fn, vec![state_val, cause_from(ap)], sym, span),
        // NOT a wat raise — a substrate bug. Unchanged.
        None => std::panic::resume_unwind(payload),
    }
}
```

wat — emit the helper at top level beside `hibernate-project-def`, then append two arguments at `:2531`:

```wat
;; emitted ONCE per service, not per dispatch
(:wat::core::defn ~on-fault-name [state <- ~state-ty-ann  cause <- :wat::core::String]
  -> :wat::core::nil
  ;; project the PRE-OP durable state, then end the loop. The client already holds
  ;; PeerCrashed — this does not invent a reply.
  <… (~hibernate-project-name state) …>)

(:wat::kernel::serve-dispatch-op ~peers-only-expr
  (:wat::core::match (:wat::kernel::retag-op op …) ~@serve-op-arms)
  state
  ~on-fault-name)
```

⭑ **Append, don't reorder.** Keeping `args[0]`/`args[1]` as they are today preserves
`infer_serve_dispatch_op`'s do-style passthrough on `args[1]`, which is the property that makes the
type change small.

## Blast radius

`wat/service.wat` — one emission site plus one emitted top-level `defn`. `src/runtime.rs` — one branch
plus the arity check. `src/check.rs`'s `infer_serve_dispatch_op` (`:12400`) and the
`#[wat_intrinsic]` declaration (`src/intrinsic/kernel/serve.rs:216`) BOTH take the new arity — the
passthrough on `args[1]` is what you preserve, not the arity.

⛔ **Three citations in `src/intrinsic/kernel/serve.rs`'s header are stale** — `check.rs:11347`
(→ `:12400`), `runtime.rs:25359` for `apply_function` (→ `:19512`), and its `runtime.rs:33041`/`:33091`
references, which predate the current file. Its *reasoning* is sound and worth reading whole; its
*line numbers* are not. Verify every one from the code.

## STOP triggers

1. **STOP-1 — if the seam's return value turns out NOT to be `nil`-typed**, STOP and report what it
   actually is. The whole route rests on `serve` being `-> :wat::core::nil` (`:3498`, `:3937`); the
   previous revision of this stone was refuted on exactly this point, so re-derive it rather than
   trusting this sentence.
2. **STOP-2 — if `hibernate-project` cannot be called from the new arm**, STOP and say why. Report
   whether the honest shipping shape is a graceful exit **without** projection (what happens today
   minus the crash) — do not quietly ship one and describe the other.
3. **STOP-3 — if `catch_unwind` returning a value breaks `serve`'s recursion**, STOP and report it.
   The trampoline (a wat fn's body evaluated via `eval_tail` inside a plain Rust `loop`) is why this
   hook can exist at all; trading TCO for crash-safety is a different stone and the builder decides it.
4. **STOP-4 — if a genuine (non-`AssertionPayload`) panic cannot be distinguished**, STOP. Converting
   a substrate bug into a tidy shutdown is the one outcome this stone must not produce.
6. **STOP-6 — if the on-fault function cannot be emitted at TOP LEVEL** and an inline `(fn …)` is the
   only shape that works, STOP and report the per-dispatch cost before shipping it. This seam runs on
   **every message**; the last wall that walked at every expansion cost 5-9 % of floor time.
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
  `assertion-failed!`) must STILL crash the service. A wall that swallows substrate bugs is worse than
  no wall, and this is the one check that can tell the two apart.

## Shape to copy

`docs/excursus/2026/08/001-sns-sqs/the-gate-outcome-outlives-its-file/` — the last stone that moved an
outcome type across the Rust/wat line, and the source of the "a namespace may legitimately span Rust
and wat" precedent. And `wat-scripts/scratch-pad/probe-arc278-wire-dos-service-killed.wat` — arc 278
stone 2, the same wall for a narrower cause, and the source of this stone's headline.
