# DESIGN — a service stops instead of crashing

Builder, ruling the crusade's next frontier:
*"i'm not convinced we need services to be restarting themselves.. (yet)... i'm more convinced that
services shouldn't be allowed to crash, only gracefully shutdown...."* — and, on my framing:
*"'terminal failure' is an odd term.... we have a strong erlang influence..."*

**Rulings taken: D1-a + D1-c · D2-a · D3-a.** This stone is **D1-a**. D1-c (migrating the arms for
diagnosis quality) is a LATER stone and is out of scope here — see below.

**Drawn 2026-09-13. NOT STRUCK.**

## WHY — measured, not assumed

`wat-scripts/scratch-pad/probe-a-handler-raise-kills-the-service.wat` (`79179f62f`):

```
a-control=Ok             non-vacuity — the service answers
a-boom   =Lost           the handler called assertion-failed!
b-dial=connect-REFUSED   an INNOCENT second client cannot even connect
```

One `assertion-failed!` inside one op handler destroys the service **for every client**. That is the
shape of all **19** `"redial failed — peer is dead"` arms (6 `circuit.wat` · 7 `sqs.wat` ·
6 `sns-fanout.wat`) and of the ~59 `Malformed` placeholders.

★ It is also, character-for-character, the arc-278 Stone 2 headline —
`probe-arc278-wire-dos-service-killed.wat`: *"A bad caller, malicious or dumb, cannot crash
anything."* Stone 2 built that wall for **one cause** (a malformed request SHAPE, guarded before the
handler runs). A handler that raises **on its own** walks straight through it. This stone generalizes
Stone 2 from one cause to all wat-level raises.

## ⛔ THE ONE CONTRACT DECISION

**An `assertion-failed!` escaping an op handler becomes `Outcome::Stop` carrying the PRE-OP state,
and the `PeerCrashed` broadcast is RETAINED.**

Three halves, each load-bearing:

1. **`Outcome::Stop`, not a new variant.** The serve loop's existing `Stop` arm
   (`wat/service.wat:2203`) already sends the reply, fans `sends`, and returns `nil` — a clean end to
   the serve recursion. The service *leaves*; it does not unwind.
2. **PRE-OP state.** The seam holds the `state` the serve loop passed IN. The handler's transition
   never completed, so there is no half-written state — this is what makes D2-a sound rather than
   merely possible, and it is why the state must come from the seam's argument and **never** from
   anything the panicking body produced.
3. **The broadcast stays.** Clients still get `PeerCrashed` → `Lost`. ⛔ This is the answer to D2-a's
   honesty gap, which the builder was asked about and confirmed: the service exits gracefully **and
   still says it faulted**. A silent clean stop would collapse a failure into a success — the exact
   defect `[[feedback_a_fallback_that_collapses_failures_reports_nothing]]` names.

### ⛔ AND THE SUB-DECISION THAT KEEPS IT HONEST: only an AssertionPayload converts

`catch_unwind` at this seam catches **two** populations:

| payload | disposition |
|---|---|
| `AssertionPayload` (`assertion-failed!` / `raise!`) — a wat-level raise | → graceful `Outcome::Stop` |
| anything else — a genuine Rust panic, i.e. a SUBSTRATE BUG | → `resume_unwind`, **unchanged** |

**Downcast before converting.** Converting an interpreter bug into a tidy shutdown would hide the one
class of failure that must stay loud. This stone does not make "crashes impossible"; it makes **a wat
raise a graceful stop** and leaves a substrate panic exactly as fatal as it is today.

## THE ROUTE — a third seam argument carrying an on-fault function

⛔⛔ **THIS SECTION WAS WRONG TWICE. Read the correction before the route.** The seam does **not**
return an `Outcome`. `serve` is declared `-> :wat::core::nil` (`wat/service.wat:3498`, `:3937`), and
`serve-dispatch-op` sits in its tail position, so the form's type is **`nil`** (or a `TailCall` signal
for the recursion). Each dispatch arm consumes its own `Outcome` **inside** the arm — matching it and
then either recursing into `serve` or returning `nil`. **The `Outcome` never reaches the seam's return
position.** Returning a constructed `Outcome::Faulted` there would put an enum value where `nil`
belongs.

`:wat::kernel::serve-dispatch-op` (`src/runtime.rs:27435`) already catches the panic and already
broadcasts. It then calls `std::panic::resume_unwind` at `:27474`. **One branch changes.** The question
is what it returns instead — and the answer must be `nil`-typed.

**CHOSEN — a third argument: an on-fault function the `defservice` macro emits.**
`(serve-dispatch-op clients body state on-fault-fn)`. Rust downcasts the payload, broadcasts as today,
then applies `on-fault-fn` to `(state, cause)` and returns its result — which is `nil`, ending the
serve recursion cleanly. `apply_function` (`src/runtime.rs:19512`) is the mechanism; the precedent that
the runtime survives catching this payload and keeps evaluating is `src/host/test_runner.rs:301`.

⭑ **Why the on-fault function and not a bare `Ok(Value::Nil)`.** Returning nil alone *would* deliver a
graceful exit with a one-line diff — but it runs no wat code, so D2-a's durable-state projection
cannot happen. The on-fault function is the only shape that both ends the loop cleanly and lets
`(~hibernate-project-name state)` run, and that call already works from macro-emitted code
(`service.wat:2448`). **If projection proves impossible there, the bare-nil shape is the honest
fallback and it is STOP-2 in the BRIEF — to be reported, not silently substituted.**

**`state` is passed BY the seam, taken from the serve fn's own fifth parameter** (`service.wat:2409`).
Rust forwards the same value it received and never constructs one, so the pre-op state is what the
on-fault function gets. Appending the two new arguments keeps `args[0]`/`args[1]` meaning what they
mean today, so `infer_serve_dispatch_op`'s do-style passthrough on `args[1]` is preserved.

**REJECTED — a new `Outcome::Faulted` variant.** Drawn as the route, then refuted by the type above:
the seam cannot return an `Outcome`. ⚠ Recorded because the *blast-radius* reasoning that got it
chosen was independently wrong too, and both errors were mine:

| I claimed | measured | the defect |
|---|---|---|
| 388 `Outcome` matches would red | **1** arm head, `wat/service.wat` | a LINE count conflating match patterns with constructions (387 are constructions) |
| Rust cannot construct a parametric wat-only enum | params are **erased** (`runtime.rs:16170`); `names` is **carried** (`value/value.rs:1181`) | a **zero** from a grep with no positive control |
| the seam returns an `Outcome`, so a new variant lands there | serve is `-> nil`; the Outcome is consumed one level below | the shape was read from the emission site's NEIGHBOURHOOD, never traced to the return position |

**REJECTED — Rust synthesizes the graceful exit itself (bare `Ok(Value::Nil)`).** One line, and it
cannot satisfy D2-a: no wat runs, so nothing projects. Kept as STOP-2's reportable fallback.

## Out of scope = REJECTED

- **The 19 arm migrations (D1-c).** The builder ruled D1-a **and** D1-c, in that order. Once the wall
  stands, those arms stop being load-bearing for crash-safety and become a **diagnosis-quality**
  change — a matched value says more than a raise. A separate stone, and each arm still owes its own
  reading: a template across them is the collapse-failures defect. **Not this stone.**
- **The lifecycle frames.** ⛔ This seam wraps **op-handler bodies ONLY**. `:init`, `:hibernate`,
  `:stop` and the serve loop's own frame unwind past it to `finish_forked_child` /
  `spawn_thread_peer` (that seam's own doc comment says so). So "a service cannot crash" is **NOT**
  what this stone delivers, and the SCORE must not say it does. It delivers: *a raise in an op
  handler is a graceful stop.* I have twice this week called a refusal total when it was not.
- **Self-restart / supervision.** The builder explicitly deferred it (*"not convinced we need
  services to be restarting themselves.. (yet)"*). ⚠ And it is not cheap: addresses are kernel-minted
  autobind (`src/kernel/address.rs:180` — 5 random bytes), so a restarted service comes back at a
  **different** address and every held `Address` is permanently `Refused`. Recovery needs a
  rendezvous story the tree does not have. Arc-shaped, and the builder's ruling to open.

## Blast radius, measured across all carriers

```
wat/service.wat                1 emission site (:2531) + the on-fault defn to emit
src/runtime.rs                 the Err(payload) branch (:27472-:27475) + the arity check (:27441)
src/check.rs                   infer_serve_dispatch_op (:12400) + its dispatch (:4775) — new arity
src/intrinsic/kernel/serve.rs  the #[wat_intrinsic] declaration (:216) + its derivation doc
.wat corpus                    0 — no user-visible form changes
```

## Trap-doors named up front

1. ⛔ **`wat/service.wat` is STDLIB — frozen into the binary at build time.** A Rust change ships
   alongside it, so this is the **STASH-DANCE** case. Read `wat/fix.wat`'s BOOTSTRAP / STASH-DANCE
   header before touching it; do not hand-edit your way out of a non-booting tool.
2. ⛔ **The `state` the seam forwards must be its own argument, never anything the panicking body
   produced.** That is the soundness argument: the handler's transition never completed, so the serve
   fn's fifth parameter is the last valid state. `AssertUnwindSafe` is an assertion, not a proof.
3. ⚠ **This seam runs on EVERY op dispatch.** Emit the on-fault function as a **top-level `defn`**
   (the shape `hibernate-project-def` already uses, `service.wat:760`) and pass its symbol — do NOT
   emit an inline `(fn …)` that allocates a closure per dispatch. The third bijection check cost
   5–9 % of floor time by walking at every expansion; this one would pay at every *message*.
4. ⚠ **The tail-call trampoline.** `serve`'s indefinite recursion depends on the recursive call
   staying in tail position through this wrapper (the seam's own doc says so). The returned value
   must not break that.
5. **Non-`AssertionPayload` panics must still `resume_unwind`.** See the sub-decision above.
6. **`cargo build --release` does not compile tests.** Use `cargo nextest run --release --no-run`.
7. ⚠ **Two stale citations found while drawing this, both in the code you are about to edit.**
   `src/intrinsic/kernel/serve.rs` cites `infer_serve_dispatch_op` at `check.rs:11347`; it is at
   **`:12400`**. And `src/assertion.rs`'s header describes the payload as existing so
   *"`run-sandboxed`'s `catch_unwind`"* can read it — **`run-sandboxed` is DELETED**
   (`wat/test.wat:175`). Verify every line number from the code, not from a comment. The live
   panic→value precedent is `src/host/test_runner.rs:301`.
