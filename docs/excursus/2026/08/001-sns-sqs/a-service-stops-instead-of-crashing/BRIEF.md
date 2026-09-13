# BRIEF — a service stops instead of crashing

**Read `DESIGN.md` beside this first.** It carries the one contract decision, the sub-decision that
keeps it honest (only an `AssertionPayload` converts), two routes affirmatively rejected with measured
reasons, and seven trap-doors — including the STASH-DANCE, which governs how you edit at all.

## The work, in one paragraph

`:wat::kernel::serve-dispatch-op` already wraps every op-handler body in `catch_unwind` and already
broadcasts `PeerCrashed` to connected clients on a crash; it then calls `std::panic::resume_unwind`
and the service dies for everyone. Add a `:Faulted [cause <- String]` variant to
`:wat::service::Outcome`; in that one Rust branch, downcast the panic payload, and **if it is an
`AssertionPayload`** broadcast as today and return a constructed `Outcome::Faulted[cause]` instead of
resuming. Then give the serve loop's `Outcome` match one new arm that performs the graceful stop —
projecting durable state via `hibernate-project` and ending the serve recursion. Any other panic
payload still resumes, unchanged.

## The rooms

| where | why you are going there |
|---|---|
| `src/runtime.rs:27435` `eval_kernel_serve_dispatch_op_tail` | ⭑ THE SEAM. Read its whole doc header first — it explains why this is the one hook with `clients` reachable and why it must stay in tail position. **The only branch you change is `Err(payload)` at `:27472`–`:27475`.** |
| `src/assertion.rs:54` `AssertionPayload` | the struct to downcast to: `message` / `actual` / `expected` / `location` / call stack. The `cause` string comes from here. |
| `src/host/test_runner.rs:301` | ⭑ THE PROVEN SHAPE — `catch_unwind`, and the loop **continues in the same process** afterwards. The live precedent that the runtime survives catching this payload. |
| `src/value/value.rs:1176` `EnumValue` | what you construct: `type_path` / `variant_name` / `names` / `fields`. ⭑ `names` is **carried, never looked up** (arc 296 G′), so no `src/types.rs` registration is needed. Type params are erased in a runtime `type_path` (`src/runtime.rs:16170`). |
| `wat/service.wat:80` the `Outcome` defenum | where `:Faulted [cause <- :wat::core::String]` goes. |
| `wat/service.wat:2164` the `Outcome::Continue` match arm | ⭑ the ONE match over `Outcome` in the corpus — your new arm's home. Measured: 1 arm head; the other 387 occurrences are constructions and are unaffected. |
| `wat/service.wat:2203` the `Outcome::Stop` arm | the closest sibling shape: it replies, fans `sends`, returns `nil`. Read it to see what a clean serve-loop exit looks like — and note it does **not** project state, which is why `Stop` alone could not satisfy D2-a. |
| `wat/service.wat:2409` `serve-params` | `state` is the serve fn's **fifth parameter**, so your arm reads the pre-op state from scope. This is the soundness argument — see DESIGN trap-door 2. |
| `wat/service.wat:2448` | the precedent for calling `(~hibernate-project-name state)` from macro-emitted code. |
| `wat-scripts/scratch-pad/probe-a-handler-raise-kills-the-service.wat` | the acceptance gate. It prints `b-dial=connect-REFUSED` today; the strike makes the innocent client's call succeed. |

## Implementation sketch

Rust — the `Err` branch, and nothing else:

```rust
Err(payload) => {
    crate::kernel::peer::broadcast_peer_crashed_best_effort(&clients_val);
    match payload.downcast_ref::<crate::assertion::AssertionPayload>() {
        Some(ap) => Ok(Value::Enum(EnumValue {
            type_path: ":wat::service::Outcome".into(),   // params erased at runtime
            variant_name: "Faulted".into(),
            names: Arc::new(vec!["cause".into()]),        // carried, not looked up
            fields: vec![Value::String(/* ap.message, + location if it reads well */)],
        })),
        // NOT a wat raise — a substrate bug. Unchanged.
        None => std::panic::resume_unwind(payload),
    }
}
```

wat — one arm beside the existing two, reading `state` from the serve fn's own parameter:

```wat
((:wat::service::Outcome::Faulted cause)
  ;; graceful stop: project the PRE-OP durable state, then end the serve recursion.
  ;; The client already has PeerCrashed — this arm does not invent a reply.
  <project via (~hibernate-project-name state), then the clean exit Stop takes at :2203>)
```

⭑ **What the arm does with the projection is yours to choose and to state in the SCORE.** The DESIGN
pins only that the state comes from the serve fn's parameter and that the exit is graceful, not an
unwind.

## Blast radius

`wat/service.wat` — one variant on the `Outcome` defenum, one new match arm. `src/runtime.rs` — one
branch. **Arity is unchanged**, so `src/check.rs`'s `infer_serve_dispatch_op` (`:12400`, do-style
passthrough) and the `#[wat_intrinsic]` declaration (`src/intrinsic/kernel/serve.rs:216`) are
untouched — confirm that rather than inheriting it.

⛔ **Three citations in `src/intrinsic/kernel/serve.rs`'s header are stale** — `check.rs:11347`
(→ `:12400`), `runtime.rs:25359` for `apply_function` (→ `:19512`), and its `runtime.rs:33041`/`:33091`
references, which predate the current file. Its *reasoning* is sound and worth reading whole; its
*line numbers* are not. Verify every one from the code.

## STOP triggers

1. **STOP-1 — if a new `Outcome` variant reds more than the one match arm at `:2164`**, STOP and
   report the real count with the matches (`grep -o … | sort -u`, not `grep -c`). The route was chosen
   on that number being 1; if it is not, the route is wrong and I want to know before it ships.
2. **STOP-2 — if `hibernate-project` cannot be called from the new arm**, STOP and say why. Report
   whether the honest shipping shape is a graceful exit **without** projection (what happens today
   minus the crash) — do not quietly ship one and describe the other.
3. **STOP-3 — if `catch_unwind` returning a value breaks `serve`'s recursion**, STOP and report it.
   The trampoline (a wat fn's body evaluated via `eval_tail` inside a plain Rust `loop`) is why this
   hook can exist at all; trading TCO for crash-safety is a different stone and the builder decides it.
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
  `assertion-failed!`) must STILL crash the service. A wall that swallows substrate bugs is worse than
  no wall, and this is the one check that can tell the two apart.

## Shape to copy

`docs/excursus/2026/08/001-sns-sqs/the-gate-outcome-outlives-its-file/` — the last stone that moved an
outcome type across the Rust/wat line, and the source of the "a namespace may legitimately span Rust
and wat" precedent. And `wat-scripts/scratch-pad/probe-arc278-wire-dos-service-killed.wat` — arc 278
stone 2, the same wall for a narrower cause, and the source of this stone's headline.
