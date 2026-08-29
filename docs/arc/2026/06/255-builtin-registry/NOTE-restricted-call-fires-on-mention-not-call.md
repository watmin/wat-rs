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
does not justify a stone on its own.

## ⚠ What the first version of this NOTE got wrong, and why it matters

I verified the **mechanism** (the walker has no head-position test — true, and I read the source) and
then published the **conclusion** (the reflection surface is unreachable) without testing it. The
conclusion was one MCP call away and I never made the call. A confirmed mechanism is not a confirmed
claim: the walker really does fire on mentions, and "therefore reflection is unreachable" simply does
not follow from it. I also inherited the rider's framing — they hit the error inside a `defn`, which
is the only place a `.wat` program can be — and generalised their true observation into a false one.
`[[feedback_verifying_the_mechanism_is_not_verifying_the_claim]]`
