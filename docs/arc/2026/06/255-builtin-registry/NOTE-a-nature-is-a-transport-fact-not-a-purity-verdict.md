# NOTE — a `Nature` is a TRANSPORT fact; `accessor_meta` reports it as a PURITY verdict

> Measured 2026-08-31 while ruling the struct pair. **No row, nothing drawn here** — this records a
> conflation found by measurement so the arc that re-rules arc 293.W inherits the evidence.

## The two axes, and the line that answers one with the other

```
Nature::Struct   "this VALUE cannot cross a wire"   — a TRANSPORT property of the DATUM
Purity           "same input ⇒ same output"          — a property of the VERB
```

`src/rete/purity.rs`, `accessor_meta`:

```rust
return Some(OpMeta { pure: a.nature.is_pure(), deterministic: true, total: true });
```

`Nature::is_pure()` (`src/types.rs:223`) is `!matches!(self, Nature::Struct | Nature::Peer)`, and its
own doc gives the reason: *"a struct holds a live resource"* / *"a peer holds a live channel"*. The
containment wall states the ground plainly — arc 293.W, quoted from the runtime error:

> *"A struct **cannot be reconstructed from EDN bytes across a comms boundary**; a record or holon
> holding a struct field could never cross — it must not exist."*

★ **That is a serialization claim.** It is true, and it is not a claim about whether *fetching* a
field is a function.

## What the measurement showed

Builder, 2026-08-31: *"the values structs can hold can be impure — a resource is not data.. there's
no EDN form to transmit a file descriptor over the wire.... but a function to say 'give me this
field's value' is constant when applied to a constant input?"*

Measured, out-of-tree, against a `defstruct` holding a LIVE `Lru` handle plus a plain `i64`:

```
plain field: read twice, equal?          true
handle len when FIRST read:              0
handle len via the SECOND read:          2
handle len via the FIRST read, now:      2     <- SAME OBJECT, both names
```

The handle read *before* two mutations and the handle read *after* report the same length: the
projection handed back the identical thing both times. **Nothing about the read varied.** What moved
was behind the handle, and it moved for both names because there is one object.

`eval_struct_field`'s body agrees: evaluate the receiver → match `Value::Aggregate` → bounds-check →
`Ok(inner.fields[index].clone())`. No `Mutex`, no `RefCell`, no `borrow`, no `apply_function`.

## The cost this is imposing TODAY

It is not hypothetical. A fence refuses a constant function, and says the wrong thing about it:

```
(:wat::rete::where (:wat::rete::i64::> (:u::Conn/fd ?c) 3))
  => "compile-condition: where expr is not pure — ':u::Conn/fd' is not pure"
```

`:u::Conn/fd` fetches an `i64` out of a struct. It is a constant function of a constant input. The
fence rejects it for a reason that has nothing to do with purity, and a reader is told the function
is impure when it is not.

## ⛔ Why admitting it is SAFE — the walk already guards the real thing

The worry ("a fence could get a live resource into its hands") was measured and does not survive:

```
(:wat::cache::Lru::len (:wat::cache::Lru::new 4))  inside a where
  => "where expr is not pure — ':rust::cache::Lru::len' is not pure"     REFUSED
(:wat::core::= h1 h2)  on two handles
  => TypeMismatch: "expected matching comparable pair, got :rust::cache::Lru"  REFUSED
```

A handle **in hand** inside a fence is inert. Every verb that could *do* anything with it is refused
one step later by the recursive walk — which is where the effect actually lives. The projection was
never the hazard.

## What a re-ruling would have to decide (NOT decided here)

Whether `accessor_meta` keeps a single `pure:` verdict derived from the holder's nature, or reports
the verb's own purity and leaves transport to the wall that already enforces it (the containment rule
refused a record holding a struct field on its own, with no help from `accessor_meta`).

⚠ **This NOTE rules nothing.** Arc 293.W owns the wall. What is settled is narrower and measured: the
purity axis is currently being answered with a serialization fact, and the two are not the same
question. `[[feedback_a_predicate_can_be_wrong_in_both_directions]]`
