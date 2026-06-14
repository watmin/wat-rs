# REALIZATIONS — arc 209 (defservice)

Disciplines and insights named while building defservice. Each entry dated, grounded
against the disk. (Project convention: `REALIZATIONS.md` per arc dir.)

---

## 2026-06-13 — The braid that made defservice trivial (build the floor; keep the bridge; flip once)

**The observation, drawing Stone C.1.** The whole counter service, written as `defservice`
(surface option A, inline bodies), is ~10 lines. The *same* service hand-rolled —
`crates/wat-lru/wat/lru/CacheService.wat` — is **530 lines**: protocol enums, `HandlePool`,
pair-by-index reply routing, `loop-step`, per-op client helpers, the spawn wiring. defservice
doesn't shrink that work; it *generates* it. Why is this suddenly easy?

**The grounded answer — a convergence, not a sequence.** Two honesty-pursuits ran in the same
era and met at defservice. (Chronology matters, and the tidy story gets it backwards: the
concurrency rebuild *predates* the clojure-migration promotion — so this is a braid of two
strands, not a single "we went clojure, which forced a concurrency pause" line.)

- **Strand 1 — honesty of FORM (EDN + faithful Clojure).** Arc 213 (ship a wat program as EDN
  over the wire) surfaced the non-EDN abuses: struct-destructure (odd-arity map), `::`-keyword
  call-heads, `/`-in-keyword. Builder's call (251 DESIGN status header, 2026-06-09): *"the wat
  invented forms… were a bridge to get us here… we go for parity."* → arc 251 (types-as-forms),
  + arc 257 (EDN-native Map/Set). Forms that can't cross the wire as clean EDN, or can't be
  faithful Clojure, get **retired or rebuilt — not migrated**.

- **Strand 2 — honesty of CONCURRENCY (deadlock-free).** defservice was first designed
  (May 2026) against concurrency tooling that was then shelved. The month-long deadlock
  annihilation rebuilt the substrate beneath it:
  - arc **170** (2026-05-09) — program entry points; the annihilation begins.
  - arc **214** (2026-05-18) — `typed_channel` dies; the unified transport-blind `Peer`.
  - arc **249** (2026-06-04) — threading reborn as wat; total-pure macro engine (the tooling
    `defservice` itself is written in).
  - arc **259** (2026-06-11) — `spawn-program'` (the host-type defclause).
  - + arc 209's own C0b campaign — `Peer`/`Listener`/`Address` unified, deadlock-free `poll'`,
    the `SO_PEERCRED` gate.

  The regrounded design says it plainly: *"the rebuild… produced exactly the idealized tooling
  defservice's design assumed it would have to hand-roll."* (`DESIGN-REGROUNDED-2026-06-12.md`.)

**The convergence.** defservice sits at the intersection of the two strands: it can't be written
honestly in the bridge surface, **and** it can't run on the old tooling. So it waited (209
reactivated 2026-06-12). When both strands landed, the 530-line hand-roll collapsed into a
~10-line declaration whose macro *expansion* is those 530 lines — the op enum, the `poll'`
dispatch loop, the client wrappers, the start fn.

**The discipline (the actual lesson).** Arc 251 — the clojure surface cutover — is *deliberately
parked.* We build defservice + the concurrency tooling **first, in current `:wat::` syntax**, and
defer the surface flip to one coordinated cutover (the mechanisms are reader-agnostic; e.g. an
env-fn `(app/beta-fn)` falls out free on cutover, unchanged mechanism). You do not migrate a
surface onto foundations that are not real yet. You build the floor, keep the bridge-forms until
it holds, then flip once. **Pausing to address what can't migrate or can't run yet — defservice,
the concurrency tooling — is *why* the migration will be clean and *why* defservice is trivial.**

**Cross-references:**
- `DESIGN-REGROUNDED-2026-06-12.md` — "the rebuild produced exactly the idealized tooling."
- `docs/arc/2026/06/251-types-as-forms/DESIGN.md` — the parity call + the "bridge" framing.
- `crates/wat-lru/wat/lru/CacheService.wat` — the 530-line hand-rolled reference defservice generates.
- `DESIGN-STONE-C.1-defservice-skeleton-op-enum.md` — the surface (option A) this entry was drawn against.

---

## 2026-06-14 — defservice is the gen_server you've been hand-rolling (capture → threading is the whole point)

**The recognition (builder, looking at the C.1 surface).** *"we implemented a ruby pattern i've
been using for like 5 years."* The pattern:

```ruby
def make_state(args...)
  State.new do |state|
    state.handle = handle(state)   # bind deps; attach the handler to the state
  end
end

def handle(state)
  ->(event, ctx) { ... use state to process event with ctx ... }   # a lambda closing over state
end
```

State + a set of event-handlers bound to it — a gen_server / actor, hand-rolled in Ruby lambdas.
It maps onto defservice almost line-for-line:

| Ruby pattern | defservice |
|---|---|
| `make_state(args…)` — build state, bind deps | `:state <T>` + its init/env-fn (the 0-arg constructor, run child-side, that wires deps) |
| `state.handle = handle(state)` | `:ops` — the handlers, bound to the state |
| `->(event, ctx)` | an op `(:Op [s <- :State …args] -> ret body)` (`event` = the op + args; `ctx` = the client peer) |
| `state.handle.(event, ctx)` | a client calling `(ns/App/Op handle …args)` |

**Why this matters (the design working).** The surface matches a shape a working engineer already
reaches for — and per the reach-stumble doctrine, an LLM/engineer instinctively reaching for a
tool IS the design spec. The substrate surfacing the exact pattern the builder has trusted for
five years is among the strongest signals the surface is right. This is the *human* half of "the
braid" (see the 2026-06-13 entry): honesty-of-form isn't only EDN/Clojure faithfulness — it's
landing on shapes practitioners already trust by hand.

**Where it diverges — and the divergence is the whole point.** The Ruby handler **closes over**
`state` (lexical capture; `handle(state)` captures by reference, processing mutates in place).
Fine single-threaded; the moment two threads call `state.handle`, you race on shared mutable
state and locking is *on you*. defservice does NOT capture: it is **state-as-self** — the single
`poll'` loop owns the one live state and **threads it explicitly** through each handler
(`[s <- :State …] -> (:Tuple :State …)`); every handler is a pure transform `(state, args) →
(new-state, reply)`. No capture, no mutation, no lock. The single loop serializing every op **is**
the mutex, by construction (Rust `&mut self` / Haskell `s→(s,a)` / Erlang `handle_call`). So
defservice is the pattern the builder already trusts — minus the "remember to lock the captured
state" footgun, because there is nothing captured to race on. The concurrency-safety moved from
*discipline* into *structure*.

**Cross-references:** `DESIGN-REGROUNDED-2026-06-12.md` § "What is preserved" pt 1 (state-as-self
= the mutex); the reach-stumble doctrine (`feedback_reach_stumble_is_the_signal`).
