# NOTE — `walk_for_restricted_call` fires on a MENTION, and that is CORRECT

> ⛔⛔⛔ **RETRACTED AND REWRITTEN THE SAME DAY, 2026-08-28.** The first version of this NOTE claimed
> *"the entire reflection surface is unreachable for restricted verbs from ordinary `.wat`"* and
> framed the walker as a name/behaviour gap wanting a fix. **That claim was FALSE and the framing
> was backwards.** The builder pushed on it — *"or… is this actually honest as no one is allowed to
> call these but the kernel?"* — and two MCP calls settled it. What follows is the measured truth.
> The original is not preserved because it was never inscribed; what it got wrong is recorded below,
> which is the part worth keeping.

## The measured behaviour

`src/check.rs:1430` raises on any `WatAST::Keyword` naming a restricted verb, in **any** position,
recursing through every child. There is no head-position test. **That part of the original NOTE was
right, and it is not the interesting part.**

The restriction is a **caller-prefix** check — it compares `enclosing_fn` against the whitelist. So
what actually gates is *whether there is an enclosing fn at all*:

```
;; TOP LEVEL — no enclosing fn, nothing fires. ANSWERS IN FULL.
(:wat::core::render-doc :wat::kernel::spawn-thread)
  → ":wat::kernel::spawn-thread\n\n`(… prog init-fn post-spawn-fn)` → `:wat::kernel::Thread<R,S>` …"

;; INSIDE A fn — enclosing_fn is `:user::probe`, which is not `:wat::kernel::`. BLOCKED.
(:wat::core::defn :user::probe [] -> :wat::core::String
  (:wat::core::render-doc :wat::kernel::spawn-thread))
  → DefRestrictedCallerNotAllowed { :enclosing-fn ":user::probe" :prefixes [":wat::kernel::"] }
```

Both measured through the MCP, 2026-08-28. The check pass runs in **both** cases — the difference is
`enclosing_fn`, not whether checking happened.

**⇒ The REPL and the MCP are NOT incomplete.** Interactive reflection over every restricted verb
works today. What is blocked is a *program* reflecting on one, because a `.wat` program body always
lives inside a `defn`. P5-a's rider hit exactly that and reasonably read it as a wall.

## Why the mention rule is LOAD-BEARING — and more so now than when it was written

A keyword in hand can be handed to `:wat::core::apply`:

```wat
(:wat::core::apply :wat::kernel::spawn-thread […])   ;; the verb is in ARGUMENT position
```

Only the mention rule stops this. Restrict the check to head position and `apply` becomes a
laundering path around every capability wall in the table below.

★ **And the premise has STRENGTHENED, not expired.** When this walker was written, `apply` reached a
small minority of the registry. Arc 255's own O-iv sweep is what widened it — 81+ ALGEBRA doors and
counting. The `apply`-laundering argument is more true today than the day the rule shipped. This is
the rare case where a ruling's premise grew INTO the ruling rather than out of it.
`[[feedback_a_rulings_premise_expires_but_the_ruling_stands]]`

## The population — NINE verbs, TWO mechanisms

Rust-side `#[restricted_to(<fqdn>, <prefix>)]`, all to `:wat::kernel::`:

```
:wat::kernel::spawn-thread          src/kernel/spawn.rs:452
:wat::kernel::spawn-process         src/kernel/spawn.rs:538
:wat::io::IOWriter/from-fd          src/io.rs:1278
:wat::io::IOReader/from-fd          src/io.rs:1318
:wat::kernel::close                 src/runtime.rs:25584
```

wat-side `{:restricted-to […]}` metadata-map on `def`/`defn`:

```
:wat::kernel::write-fd-raw          wat/kernel/services/stdio.wat:362   [:wat::kernel:: :wat::test::]
:wat::kernel::flood-stdout-raw      wat/kernel/services/stdio.wat:376   [:wat::kernel:: :wat::test::]
:wat::kernel::str-double            wat/kernel/services/stdio.wat:384   [:wat::kernel:: :wat::test::]
(one more)                          wat/spawn.wat:338                   [:wat::spawn:: :wat::test::]
```

⚠ **Two mechanisms, one enforcement point.** The Rust attribute drains through an inventory channel
into the same `binding_metadata[":restricted-to"]` the wat-side form writes (`src/restriction_entry.rs`,
`src/freeze/env.rs:268`). Anyone auditing the restricted set must ask BOTH — a census of one is half
a census. Note also that the Rust-side five do **not** whitelist `:wat::test::` while the wat-side
four do; that asymmetry is unexamined and is the one live question this NOTE leaves open.

## What is actually left, and it is small

The diagnostic is excellent — it names the callee, the enclosing fn, the whitelist, the prefix-vs-exact
semantics, and two concrete remedies. The only residue is that the internal fn is called
`walk_for_restricted_call` and the error variant is `DefRestrictedCallerNotAllowed`, while what is
enforced is *mention*. That is a naming nit on an internal symbol, **not a capability gap**, and it
does not justify a stone on its own. The real question is the design fork below.


## ⛔ THE DESIGN FORK — and why the obvious answer is ALREADY DEAD

The builder's frame: *"reflection should be able to SEE, but not RUN."* Right frame. The current
rule conflates two questions into one syntactic test — *may you HOLD this name?* and *may you CALL
this verb?* — and holding is restricted only because holding currently implies calling.

### ⛔ OPTION C — "move the wall from the mention to the call door" — REFUTED, AND NOT FOR THE FIRST TIME

Stop restricting who may hold the keyword; restrict who may CALL it, at `apply`/dispatch. It reads
like the extirpare answer — *don't forbid holding the key, make holding it confer nothing.*

**IT WAS TRIED AND DEFEATED BEFORE. THE MENTION RULE IS WHAT REPLACED IT.** The attack, in the
builder's own words (2026-08-28), and re-measured against the live tree the same day:

```wat
(:wat::core::defn :user::launder [] -> :wat::core::nil
  (:wat::core::let [f :wat::kernel::spawn-thread] nil))   ;; bind the restricted name to a local
;; → DefRestrictedCallerNotAllowed at col 76 — the MENTION, in BINDING position, before any call.
```

Bind the restricted name to a local and the call head is no longer the restricted keyword — it is a
local symbol. A door that asks *"which verb is being called"* sees `f`. To see through it you must
trace where `f`'s value came from, and **value-flow tracing is undecidable in general.**

★ **THE LOAD-BEARING PROPERTY IS DECIDABILITY, NOT POSITION.** The mention rule is not a coarse
approximation of the real check that a smarter check could refine. It is *the only syntactically
decidable form of the check*, and it is syntactic precisely because a value can be rebound. Any
proposal that moves the wall off the syntax and onto the value is this option again in new clothes.
`[[feedback_a_rejected_option_returns_in_new_clothes]]`

⚠ **And note how it would have escaped a census.** The version of C proposed on 2026-08-28 asked for
"a complete enumeration of every path that turns a keyword into a call" as its first stone. That
census would have MISSED `let` — because `let` does not call anything. **The predicate named the
wrong act; running it more carefully would not have saved it.**

### ⛔ OPTION A — "let a `@Category Reflection` verb receive restricted keywords" — REFUTED

The axis exists (closed enum in `wat/runtime-meta.wat`, mirrored to Rust; 16 verbs carry it), so it
is tempting. **Honest = NO.** `Category` classifies *what the computation IS*, not *what it does with
its arguments*, and `src/intrinsic/witness.rs:64` already records this exact error being made and
corrected: *"'takes a fn' is a signature property, while `Category` classifies what the computation
IS; mixing those axes is the error that produced `Ambient`."* Nothing about being Reflection-
categorised stops a verb from calling its argument.

### ✅ OPTION B — the ARGUMENT declares it: *this position is READ, never invoked* — the only survivor

A per-`@arg` marker naming the position as reflective. The walker's rule becomes: a restricted
keyword may appear in a **declared reflective argument position**, and nowhere else.

**Why it survives the `let` attack that killed C:** B does not relax the syntactic rule — it keeps
it and punches a **named, enumerable hole** in it. A declared argument position is as syntactically
visible as a call head. `(:wat::core::let [f :wat::kernel::spawn-thread] …)` still fires, because
`let`'s binding position is not a declared reflective position and never will be. **C relaxes the
rule and rebuilds elsewhere; B keeps the rule and carves it. Same decidability class as today.**

Obvious YES · Simple YES · Honest YES · Good UX YES.

### ⛔ B DOES NOT TOUCH USER `{:restricted-to […]}` — IT IS WHAT MAKES USER RESTRICTIONS REFLECTABLE

Builder's question, 2026-08-28: *"does this mean user defs with restricted-to can or cannot be
allowed? … you called it kernel only."* Two different things were run together. Separated:

- **Declaring `{:restricted-to […]}` on your own def** — UNCHANGED, and unrestricted. Any user
  writes `(def :my::secret {:restricted-to [:my::app::]} …)` today and under B. **And under B their
  doc tooling starts working**: `(render-doc :my::secret)` from inside a program passes, because the
  hole is in `render-doc`'s ARGUMENT POSITION, not in the restriction. **The reflective declaration
  lives on the RECEIVING verb, never on the restricted def.** User restrictions get strictly more
  usable; that IS option B, not an exception to it.
- **Declaring an argument position REFLECTIVE** — the only thing "kernel-only" was ever about, and
  it was the wrong rung. See below.

### The laundering vector, and the rung above "kernel-only"

`src/runtime.rs:10650` — *"Both are valid apply heads"*: `apply` accepts a `Value::wat__core__keyword`
as well as a `Value::wat__core__fn`. So if a user could mark their own passthrough reflective:

```wat
(:wat::core::defn :my::launder [f {reflective}] …  (:wat::core::apply f []))
;; (:my::launder :wat::kernel::spawn-thread) passes the mention check at the CALL SITE, and inside
;; `my::launder` the body mentions only the parameter `f` — nothing fires. Applied anyway.
```

**"Only the kernel may declare it" is the CONVENTION rung.** The no-form rung is available and
cheaper: **a reflective position delivers an OPAQUE VERB HANDLE, not the keyword** — a value that is
not a valid apply head. The substrate already has this shape (`:wat::kernel::Thread`, `Peer`, the IO
handles). Then holding it confers nothing, no value-flow tracing is needed, and **the marker is
safely USER-declarable** — which it must be, or user doc tooling is second-class again.

★ **THIS IS WHERE OPTION C's INSTINCT WAS RIGHT AND ITS SUBJECT WAS WRONG.** C tried to make *a
user-written keyword* harmless, which requires tracing where the value came from — which is why
`let` killed it. This mints *an inert thing* at the boundary instead. **The value that crosses was
never a keyword, so there is nothing to trace.** Same instinct — make holding it confer nothing —
applied to a value the system CONSTRUCTS rather than a name the user WROTE. Record this distinction:
it is the difference between the dead option and the live one, and they read alike.

The reflective set on the kernel side stays small and named (`render-doc`, `show-source`,
`metadata-of`, `signature-of-defn`, `examples`).

★ **And it is the SAME SHAPE as P5-b**, which is about giving `@arg` a per-argument subject for
`@yields`. Whoever draws this should read P5-b first; they are one mechanism serving two rules.

### Not drawn, and not mine to open

This is a **capability-model** change, not a builtin-registry stone. It does not belong in arc 255
and it is not on THE ROAD (step 1 is homing; totality is step 6). **Opening an arc is the builder's
ruling.** Nothing is on fire: all nine restricted verbs reflect correctly at the REPL and the MCP
today, and only a `.wat` *program* is blocked.

## ⚠ What the first version of this NOTE got wrong, and why it matters

I verified the **mechanism** (the walker has no head-position test — true, and I read the source) and
then published the **conclusion** (the reflection surface is unreachable) without testing it. The
conclusion was one MCP call away and I never made the call. A confirmed mechanism is not a confirmed
claim: the walker really does fire on mentions, and "therefore reflection is unreachable" simply does
not follow from it. I also inherited the rider's framing — they hit the error inside a `defn`, which
is the only place a `.wat` program can be — and generalised their true observation into a false one.
`[[feedback_verifying_the_mechanism_is_not_verifying_the_claim]]`
