# NOTE — the rete firing family has TWO implementations per verb, and homing would add a THIRD

> Measured 2026-08-29 while sizing P6-c-W5c. **No row, nothing drawn** — this records why nine rete
> verbs are not a wave, so the next drawer inherits the measurement instead of discovering it at
> nine sites.

## The shape

`:wat::rete::fire-rules` exists twice, deliberately, and both sites say so:

```wat
;; wat/rete/oracle/fire.wat:405 — "fire-rules — public production verb. Keyword-head calls and this
;; first-class Fn body both reach rust through `$native`."
(:wat::core::defn :wat::rete::fire-rules
  [session <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::rete::fire-rules$native session))
```

```rust
// src/runtime.rs:5616 — "Public `fire-rules` is a wat Fn (first-class). Keyword-head and the
// Fn body both reach rust through `$native`."
":wat::rete::fire-rules$native" | ":wat::rete::fire-rules" => {
    crate::rete::kernel::eval_fire_rules_native(args, list_span, env, sym)
}
```

So the FQDN is served by **one Rust arm shared with its `$native` twin** *and* by **a wat `defn`
that is a first-class `Fn` value**. `(fire-rules s)` in head position takes the arm; `fire-rules`
passed as a value takes the wat Fn, whose body re-enters the arm through `$native`. That is a
design, not an accident — the first-class use is the reason the wat wrapper exists.

**Four verbs share an arm with their `$native` twin** (`grep -cE '":wat::rete::[a-z-]+\$native" \|
":wat::rete::'` → 4): `fire-rules`, `fire-rules-explain`, `fire-once`, `insert-all`. With their
twins and `insert$native` that is the nine-verb firing family.

## Why homing is not mechanical here

`dispatch_keyword_head` consults **the registry first** (`runtime.rs:5246`), before the literal match
and before anything reaches the symbol table. Registering `:wat::rete::fire-rules` as an intrinsic
therefore inserts a **third** implementation ahead of both existing ones.

Three questions no wave should answer by accident:

1. **Does the registry entry shadow the wat `defn` for the first-class use**, or only for head
   position? The whole point of the wat wrapper is that `fire-rules` can be passed as a value.
2. **Should the public verb be homed at all, or only its `$native` half?** Homing `$native` alone
   leaves the public FQDN exactly as it is — which may be the entire answer, and is cheap.
3. **What happens to the shared arm** when one of its two patterns is homed and the other is not?

⚠ **And four of the nine ADD checker debt** (`fire-rules`, `fire-once`, `fire-rules-explain`,
`insert-all` were in the measured debt-adder set), so the wave that takes them also grows a frozen
ledger. Two unrelated costs landing in one stone is how a wave stops being reviewable.

## What this is NOT

It is not a defect. Both sites are documented, the pairing is intentional, and nothing here is
broken. It is a **dual implementation**, and this arc has a memory about exactly that:
`[[feedback_a_slot_with_two_implementations_is_two_slots]]` — verifying that a form works at one
site and shipping "the slot works" is the failure it names.

## The cheap probe whoever draws this should run FIRST

Before designing anything: pass `fire-rules` as a VALUE (not a head) today, and again with only
`$native` homed. If the first-class path is unaffected by homing the native half, question 2 answers
itself and the family reduces to an ordinary wave over the `$native` five.
