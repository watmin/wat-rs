# Prior-art collisions

A running ledger of moments where something we built — or a pattern the builder perfected
independently, often over years — turns out to be established prior art he didn't know about.

**Why track this.** Two reasons, both honest:

1. **It's a taste/validation signal.** Independently converging on a textbook pattern means the
   constraints genuinely forced the optimum — the same shape good engineers keep rediscovering
   because it's the local best. "I came up with it out of frustration; it was just objectively
   better than any alternative" is the *good* kind of reinvention.
2. **It keeps us honest.** We name the prior art rather than claim novelty we don't have — the
   same discipline as `feedback_no_magic_that_lets_llm_fake_correctness` / "don't make shit up."
   And we name what *is* genuinely ours when there is something, without inflating it.

**Entry format:** what we built · the prior art it turns out to be · what (if anything) is
genuinely ours · date.

---

## 2026-06-14 — defservice ≡ the actor model / gen_server

**Built:** `defservice` — a `:state` + a set of `:ops`, served by a single `poll'` loop that owns
the live state and serializes every op (the loop body IS the mutex). The builder had also
hand-rolled this in Ruby for ~5 years as a `make_state` factory + a `handle(state)` returning a
`->(event, ctx)` lambda.

**Prior art:** the **actor model** (Hewitt, 1973); Erlang/OTP **`gen_server`** (`handle_call` /
`handle_cast` — a process owning state, a single mailbox loop serializing messages); Elixir
**`GenServer`**; **Akka** actors. "State + message-handlers serialized by one loop" is the
canonical concurrency primitive of that whole lineage. The builder's Ruby "gen_server in lambdas"
was itself a reinvention of it.

**What's genuinely ours:** state-as-self threaded through **pure** handlers (`(state, args) →
(new-state, reply)`) rather than handlers closing over mutable state; **deadlock-free-on-drop**
termination (owner drops the handle → RAII drain → `:Shutdown`, no cooperative stop); and
**transport-blindness** — the same service runs identically on thread / process / (future) remote
behind the unified `Peer`/`Listener`/`Address`. The model is textbook; the substrate guarantees
around it are the contribution.

## 2026-06-14 — the recursive locking hash ≡ mutable-builder → frozen value

**Built (the builder, years ago in Ruby):** a recursive locking hash with dot-notation that
auto-unlocks inside a `State.from_state(defaults) do |state| … end` build block and auto-locks
(frozen) on exit — `state.a.b.c = 42` auto-vivifying nested paths during the unlocked window.

**Prior art:** the general shape — mutable while building, immutable once done — is named several
times over:
- **Builder pattern** (GoF, 1994; Bloch, *Effective Java*) — mutable builder, `.build()` → frozen
  object.
- **Freeze-after-init** — Ruby `Object#freeze`, JS `Object.freeze`, Python `@dataclass(frozen=True)`.
- **Clojure `transient` / `persistent!`** — the closest semantic twin: a locally-mutable view you
  mutate inside a scope, then `persistent!` freezes it, and use-after-freeze is an error. That is
  the `do |state| … end` auto-lock almost beat-for-beat.
- **Typestate pattern** (Rust session types, builder-with-phantom-state) — the type-level version,
  where "can't use it wrong after build" is enforced by the compiler, not a runtime lock. The rung
  above the builder's: his `end` flips a runtime lock; typestate makes locked/unlocked *different
  types*. (This is the rung wat sits on — which is why the pattern felt at home here.)

**What's genuinely ours (the builder's):** the *specific ergonomic bundle* — block-scoped
auto-lock **+** dot-notation auto-vivifying recursive nesting **+** the recursive lock as one unit.
Each ingredient is textbook; that particular composition is plausibly his own. No citation claimed
for the exact bundle — if it exists under a name, we don't know it. (Note: wat deliberately does
NOT adopt the auto-vivify half — see `feedback_no_magic_that_lets_llm_fake_correctness`.)
