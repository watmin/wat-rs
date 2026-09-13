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

## THE ROUTE — a new `Outcome` variant, and why the other two lose

`:wat::kernel::serve-dispatch-op` (`src/runtime.rs:27435`) already catches the panic and already
broadcasts. It then calls `std::panic::resume_unwind` at `:27474`. **One branch changes, at one
located place.** The question is only what it returns instead.

**CHOSEN — (B) add `:Faulted [cause <- String]` to `:wat::service::Outcome` (`service.wat:80`).**
Rust downcasts the `AssertionPayload`, broadcasts as today, and returns a constructed
`Outcome::Faulted[cause]`. The macro's serve-loop match gains **one arm**, and that arm does the
graceful stop — including D2-a's `(~hibernate-project-name state)` — in wat, where it belongs.

⭑⭑ **Why this is the right shape and not merely a cheaper one: the pre-op state becomes
STRUCTURAL.** `state` is the serve fn's own fifth parameter (`service.wat:2409`), in scope at the
match site. The new arm reads it from there, so Rust never sees a state at all and **cannot pass the
wrong one**. Under the rejected routes, "use the pre-op state" was a discipline in a trap-door list;
here the wrong state has no path to the call. That is the top rung of the ladder rather than a
convention, and it is the whole reason this route is preferred.

Four facts make it cheap, each measured this session:

| fact | where |
|---|---|
| **1 match arm** on `Outcome::Continue` in the whole corpus — the other 387 occurrences are *constructions*, which a new variant does not touch | `grep -o '((:wat::service::Outcome::Continue'` → 1, in `wat/service.wat` |
| type params are **erased** in a runtime `type_path`, so a parametric enum is no harder | `src/runtime.rs:16170`; `service.wat:2007`, `:2288`, `:2517` |
| `EnumValue.names` is **"carried, never looked up"** — an enum value is self-describing, so Rust needs **no `src/types.rs` registration** to build one | `src/value/value.rs:1181` (arc 296 G′) |
| `serve-dispatch-op` keeps **arity 2**; `infer_serve_dispatch_op`'s do-style passthrough (`body`'s type IS the form's type) stays untouched | `src/check.rs:12400` |

**REJECTED — (C) a third seam argument carrying an on-fault function.** Drawn first and replaced.
It works — `apply_function` (`src/runtime.rs:19512`) is the mechanism — but it ripples the arity
through `check.rs`'s inference, the `#[wat_intrinsic]` declaration and the emission site, makes Rust
apply a wat function from inside a `catch_unwind`, and leaves pre-op state as a *convention*. More
moving parts for a weaker guarantee.

**REJECTED — (A) Rust synthesizes `Outcome::Stop` directly.** Smallest diff, and it cannot satisfy
D2-a: the `Stop` arm (`service.wat:2203`) replies, fans `sends` and returns `nil` — it **does not
project durable state**. Projection needs wat-side logic on the fault path, which is exactly what (B)
adds. It would also put service policy in the interpreter.

⛔ **Two numbers in this section's first draft were wrong and are recorded because the reasoning
turned on them.** I rejected (B) at *"388 matches"* — a line count that conflated match patterns with
constructions (real answer: **1**) — and then at *"Rust cannot construct a parametric wat-only
enum"*, which was a **zero from a grep with no positive control**; params are erased and names are
carried, so neither half held. The builder's challenge (*"that's legal, right?"*) is what reopened it.

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
wat/service.wat                Outcome defenum (:80, +1 variant) + 1 serve-loop match arm (:2164)
src/runtime.rs                 the Err(payload) branch ONLY (:27472-:27475)
src/check.rs                   0 — arity unchanged, passthrough inference untouched
src/intrinsic/kernel/serve.rs  0 — declaration unchanged
.wat corpus                    0 constructions break; a new variant only reds MATCHES, and there is 1
```

## Trap-doors named up front

1. ⛔ **`wat/service.wat` is STDLIB — frozen into the binary at build time.** A Rust change ships
   alongside it, so this is the **STASH-DANCE** case. Read `wat/fix.wat`'s BOOTSTRAP / STASH-DANCE
   header before touching it; do not hand-edit your way out of a non-booting tool.
2. ⛔ **Read `state` from the serve fn's own parameter at the match site — never from anything the
   panicking body produced.** Under the chosen route this is structural (Rust never holds a state),
   so the trap-door is only this: **do not "improve" it by passing state through the seam.** That
   reintroduces the mistake the shape currently forbids. `AssertUnwindSafe` is an assertion, not a
   proof, and the pre-op state is what makes it sound.
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
