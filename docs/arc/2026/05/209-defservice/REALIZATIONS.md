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

**Second pattern, same theme (builder, 2026-06-14).** A recursive locking hash with dot-notation:

```ruby
State.from_state(has_defaults) do |state|   # auto-UNLOCKS
  state.whatever.nested.depth = 42
  state.something = Hash.new(0)
end                                          # auto-LOCKS (frozen thereafter)
```

This is **mutable-builder → frozen value**, enforced by the block boundary. It maps onto the
init/env-fn: build the state with defaults + deps inside the constructor, return the record;
after that it's the loop's threaded, immutable state. But the `end`-auto-lock is a RUNTIME
mechanism *emulating value semantics* — a toggle that freezes a hash so nothing scribbles on it
after build. wat needs no toggle: a record IS a value; "locked after build" is not a flipped bit,
it's the absence of a mutator. The builder hand-rolled, in Ruby, the thing the substrate just is.
**Twice now**: lock-the-captured-state (gen_server) and lock-after-build (the hash) are both
discipline-enforced reconstructions of value semantics + structural serialization — handed over
for free here.

## 2026-06-14 — NO magic auto-hash: typed records are mandatory, because the author may be a confabulating LLM

The dot-notation hash above **auto-vivifies** — `state.a.b.c = 42` conjures the whole nested path,
untyped. **We do NOT build that, and the reason is load-bearing, not aesthetic.** Builder: *"users
must use fully typed records — i got away with it by discipline — llms make shit up and i can't
have a lower tier llm produce code that is forced into a working position."*

A magic auto-vivifying structure **forces any code into a working position**: a wrong field name,
a typo'd nested path, a confabulated key — all silently vivify and the program runs. That
"always sort-of-works" property is exactly what a careful human can ride on discipline, and
exactly what lets a **confabulating (or lower-tier) LLM ship confident garbage** — the structure
bends to whatever was written and masks the mistake. A fully-typed record turns the same mistake
into a **compile error**: a made-up field/path has no constructor, doesn't type-check, cannot
ship. The author — human or LLM — is *forced into correctness by the type system*; it cannot skate
by on a structure that conforms to its confabulation.

This is the **complement of the reach-stumble doctrine** (`feedback_reach_stumble_is_the_signal`):
engineering the substrate for an LLM to operate instinctively means not only handing it the tool
it reaches for, but **denying the tool that lets it fake success.** It is also the extirpare
ladder, climbed to the top: the builder's Ruby safety was a *convention* (discipline — the weakest
rung); the typed record is *a shape the mistake cannot be expressed in* (uncompilable — the top
rung). And it is `Honest` in the four-questions sense — a magic hash papers over the gap between
what was written and what is correct; the typed record makes that gap a hard failure.

It is directly load-bearing for the Inquisitor/Shadowdancer protocol: substrate code is delegated
to sonnet (a lower tier). Typed records are part of *why* a Shadowdancer cannot ship working-
looking-but-wrong code — the human gate catches at the review layer, the types catch at the
substrate layer. The auto-vivify convenience is precisely the affordance that would defeat both,
which is why wat is nominal/ADT and refuses it. Where genuinely-open dynamic nesting is needed,
it is an explicit `HashMap` field — a *declared* choice, never the typed spine bending silently.

**Cross-references:** `DESIGN-REGROUNDED-2026-06-12.md` § "What is preserved" pt 1 (state-as-self
= the mutex); the reach-stumble doctrine (`feedback_reach_stumble_is_the_signal`); the ADT-language
identity (wat is nominal tagged sums, not dynamic maps — `project_typed_clojure_parity_pivot`).

## 2026-06-14 — Prior-art collisions (independent rediscovery = a taste signal)

Both patterns above turned out to be textbook the builder hadn't known by name — one he'd
perfected independently over ~5 years. Worth naming, for two honest reasons: converging on an
established pattern is a *taste signal* (the constraints forced the optimum, the shape good
engineers keep rediscovering), and naming the prior art rather than claiming novelty is the same
discipline as the no-magic entry above (don't-make-shit-up) — while noting what *is* genuinely
ours.

- **defservice ≡ the actor model / gen_server.** State + handlers serialized by one loop is the
  **actor model** (Hewitt, 1973), Erlang/OTP **`gen_server`** (`handle_call`/`handle_cast`),
  Elixir **`GenServer`**, **Akka**. The builder's Ruby "gen_server in lambdas" was itself a
  reinvention. *Ours:* pure state-as-self handlers (not captured mutable state), deadlock-free-
  on-drop termination, transport-blindness across thread/process/remote. The model is textbook;
  the substrate guarantees around it are the contribution.
- **The recursive locking hash ≡ mutable-builder → frozen value.** **Builder pattern** (GoF, 1994;
  Bloch, *Effective Java*); freeze-after-init (Ruby `freeze` / JS `Object.freeze` / Python frozen
  dataclass); **Clojure `transient`/`persistent!`** (the closest twin — mutable-in-scope then
  frozen, with use-after-freeze an error = the `do |state| … end` auto-lock beat-for-beat); and
  the **typestate pattern** (the type-level rung wat sits on). *Ours:* the specific ergonomic
  bundle — block-scoped auto-lock + dot-notation auto-vivify + recursive lock as one unit (each
  ingredient textbook; the composition plausibly the builder's — no citation invented for the
  exact bundle). wat deliberately drops the auto-vivify half (see the no-magic entry).

- **defservice's client wrappers (C.3 — FORWARD-MAP) ≡ reflective facade + partial application.**
  The builder's Ruby `install_closures` — a module that, on `extend`, reflects a client's public
  methods and installs per-method curried closures:

  ```ruby
  module My::S3
    extend MakeClosures
    install_closures(Aws::S3::Client, kwargs: true)   # reflect public methods → install wrappers
  end

  # each method the reflection installs:
  define_method(method_name) do |client, default_kwargs|
    ->(kwargs) do
      client.call(method_name, default_kwargs.merge(kwargs))
    end
  end

  # bind the client + per-call defaults once, then call with the rest:
  State.locked do |state|
    s3 = My::S3.make_client
    state.s3.get_object = My::S3.get_object(s3, bucket: 'foobar')
  end
  obj = state.s3.get_object.call(key: 'burbaz')
  ```

  That is exactly what **C.3** will generate: one
  wrapper per `:op`, closing over the connection (the peer/handle = his `client`), invoked with
  the op's args (`bucket:` = bound per-op defaults; `key:` = the call args). *Prior art:*
  **partial application** (bind client + defaults, return a fn of the rest — Clojure `partial`;
  not strictly currying); **reflective facade / dynamic proxy** (Ruby `define_method`
  metaprogramming; Java `Proxy`; Python `__getattr__`). *What'll be ours:* the wrappers are
  generated from a *declared* `:ops` surface read at macro time (not runtime reflection), each
  closing over a typed, transport-blind `Peer` — the same call site works thread/process/remote.
  (FORWARD-MAP, captured 2026-06-14 per capture-live; confirm against the shipped C.3.)

Habit (`feedback_note_prior_art_collisions`): when we collide with prior art the builder didn't
know — especially a pattern he perfected over years — name it. Real names only (no invented
citations); note the genuinely-novel part.

## 2026-06-14 — FINDING (follow-up stone): macro params demand a type annotation, then discard it

Decomplecting defservice's signature (it typed its params `:wat::holon::HolonAST` — the holon
crutch again, [[feedback_honest_abstraction_decomplect_crutch_open_seam]] — when the macro does
pure syntactic work and the engine binds every param as `Value::wat__WatAST` regardless) surfaced
a deeper smell in the macro-def grammar itself:

- The defmacro grammar **requires** the `name <- :Type` triple per param (a bare `[param]` won't
  parse — parse.rs).
- The type is then **discarded**: `MacroDef.params: Vec<String>` (registry.rs:13); parse.rs:89
  verbatim *"Extract param names only — MacroDef carries names, not types"* → `.map(|(ident,
  _ty)| …)`. Nothing — parser, `validate_pure_total`, or checker — ever reads it.

So a per-param type annotation is **mandatory-then-ignored** — the exact confabulation surface the
no-magic law (this file, 2026-06-14) condemns: a slot you're forced to fill whose contents are
never checked, so a wrong fill (`fqdn <- :wat::core::i64` — a lie; a macro arg is always a
syntactic node) is silently "forced into a working position." Found dogfooding our own law the same
week we wrote it. (It also explains why the `HolonAST → WatAST` decomplect passed every test: the
engine discards the type, so the change is invisible to the engine — but it matters to the *reader*,
which is why it's still the right fix.)

**NOT a bug in the faithful sense** — Clojure/Typed-Clojure macros are forms→forms; you type-check
the *expansion*, not the macro's params. The defect is narrowly the mandatory-then-discarded
annotation.

**Fix (own stone) — DECIDED 2026-06-14 (four-questions + builder's deciding constraint: "there is
exactly one argspec — i'm not debugging a fifth impl"): ENFORCE, do NOT drop.** DROP (name-only
macro params) was the first read — but it FORKS `parse_argspec_triples`, **the sole argspec parser
across fn/defn/defclause/defmacro** (Stone 241.1; argspec/parse.rs:70 — `fixed_params:
Vec<(Identifier, TypeExpr)>`, type *required*) — or it makes that `TypeExpr` optional (the optional
smell, and it weakens `defn` too). A second argspec impl is intolerable, so DROP is dead. ENFORCE:
a validation rule on the SOLE parser's **output** in the defmacro path — every macro param's
declared type must MATCH what actually binds (`:wat::WatAST` for a fixed param; the forms-sequence
type for the rest param), rejected at macro-def time otherwise. This turns the discarded lie into a
**checked truth** (no-magic satisfied by enforcement, not deletion), leaves the one argspec parser
untouched, and makes the holon crutch **unrepresentable** (`<- :wat::holon::HolonAST` is rejected →
must be `:wat::WatAST`; extirpare top rung). The stone also flushes the existing crutch:
`keyword/of` (`head <- :wat::holon::HolonAST`) + any macro with a non-AST/holon param type get
corrected to `:wat::WatAST` (a bounded mechanical sweep, NOT a grammar change). `cond`'s
`& clauses <- :AST<wat::holon::Holons>` rest type is the legit forms-sequence type (the valid-AST-type
set to pin when the stone is drawn).
