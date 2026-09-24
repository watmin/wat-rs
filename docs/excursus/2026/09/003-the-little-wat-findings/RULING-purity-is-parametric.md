# RULING — generic purity is PARAMETRIC (2026-09-24)

**Builder:** *"we have 4 YES declared - it has been reasoned"* — accepting the four-questions pass
below over the stone F executor's STOP-3 evidence.

## The rule

A generic `:Pure` aggregate or enum is pure **iff its type arguments are pure.** `(Box :- [i64])` is
pure; `(Box :- [Lru])` is an impure type — legal to construct, refused at every boundary that
requires purity (the wire, durable state). This is what `is_pure_type`'s `Parametric` arm already
computes, and what the stdlib is built on.

## Why (four questions, all YES)

- **Obvious** — `Box<T>` is as pure as its `T`; `Vector<T>` and `Option<T>` already work so.
- **Simple** — it is what the checker already does; nothing needs building.
- **Honest** — purity only has to hold where data crosses, and the wire wall refuses
  `(Box :- [Lru])` directly (measured: *"a wire peer … carries only pure data — type (:t::Box :-
  [(:wat::cache::Lru …)]) is not pure"*).
- **Good UX** — `Outcome`, every `<svc>::Status` (Shared `Address`, 293.W.2f), and `PoolMsg` stay
  legal. The rejected alternative ("bounded": every instantiation of a pure type must be pure)
  produced **166,330** census hits on the floor, every one a deliberate stdlib construction, and
  killed every program at startup.

## Consequences

- **Stone F is CLOSED as not-a-defect** in its original framing. `SKETCH-stone-F-the-stashed-two-layer-
  implementation.diff` implements the REJECTED "bounded" reading; kept as a record, not to be applied.
- Stone E's `Holder<Lru>` fixtures are legal under this ruling and stay as they are.
- `wat::service::Outcome` stays `:Pure` — under parametric purity `Outcome<State,…>` is simply impure.

## What remains — the one real hole under this ruling

The purity promise must hold **at the boundaries**. Measured 2026-09-24 at `5b9ca1626`:

```
(self-peer (Box :- [Lru]) i64)              check=1   refused by the wire wall
(self-peer (Box :- [T]) i64), T = Lru       check=0   passes the check  ⛔
```

A generic function reaches `Peer'` with `T` still a type parameter, which the wall treats as pure,
and nothing re-checks when `T` is instantiated to `Lru`. **Whether a runtime wall stops the data
before it crosses is UNMEASURED** — the probe died on "self-peer is only valid inside a spawned
process service", which is not the question. That is the next measurement, and a stone if it holds.

## Also surfaced, and live at HEAD

**the-little-wat F-107** — a constructor's explicit type argument is silently discarded:
`(:t::Box :- [:wat::core::i64] :x "str")` → `#t/Box {:x "str"}`, rc 0. Reproduced 2026-09-24.
