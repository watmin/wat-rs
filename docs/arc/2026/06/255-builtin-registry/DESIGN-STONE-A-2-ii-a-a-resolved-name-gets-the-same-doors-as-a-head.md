# DESIGN — STONE A-2-ii-a: a name resolved through the environment gets the SAME DOORS as a head

> Found in A-2-ii's pre-flight, 2026-08-30, before any brief was written. **A-2-i shipped a
> capability with an asymmetry inside it**, and the asymmetry blocks two live substrate sites.

## The finding — measured, one probe, two rows

```clojure
;; the accessor as a direct HEAD
(pure? '(fn [a <- :probe::R] -> :i64 (:probe::R/sk a)))            -> true

;; the SAME accessor, reached through an env binding
(let [k :probe::R/sk]
  (pure? '(fn [a <- :probe::R] -> :i64 (k a))))                    -> false
```

**The same name gets two different answers depending on how it was reached.** That is not a
missing feature; it is the classifier contradicting itself.

## Why

`head_ok` tries **four doors** for a head string, in order:

```
constructor_meta  ->  accessor_meta  ->  sym.has_function/classify_fn  ->  intrinsic_meta  ->  deny
```

`classify_closure` (A-2-i) resolves a binding to a `Value::wat__core__fn(Arc<Function>)` and, on a
`FunctionBody::Native`, consults **`intrinsic_meta` alone** on `f.name`. A record field accessor is
recognised by `accessor_meta` (`src/rete/purity.rs:884` — *"a Record/HolonRecord accessor is pure ∧
deterministic, a Struct accessor is impure"*), a door the resolved path never opens.

★ **The name is the same string either way.** `head_ok` knows `R/sk` is a pure accessor; the
resolved path asks a narrower question and gets a narrower answer.

## THE INVARIANT THIS STONE ESTABLISHES — and it is the real deliverable

> **Reach-independence: the classifier's verdict on a name depends on the NAME, never on how the
> name was reached.**

A head and a binding that resolve to the same named callable must classify identically, on every
axis. This is a *correctness property*, testable as an invariant, not a feature request — and it is
the honest form of what A-2-i was reaching for.

## What ships

In `classify_closure`'s `FunctionBody::Native` arm: when the resolved `Function` has a `name`,
**route that name through the same door ladder a head takes** rather than consulting
`intrinsic_meta` alone. The one place that knows all four doors is `head_ok` itself, so delegating
to it keeps this as ONE mechanism that cannot drift, instead of a second ladder to maintain in
parallel.

⚠ **The recursion guards must carry across the delegation** — both the FQDN `seen` and the
`closure_seen` pointer set — or a named native reachable from its own body re-enters. A-2-i already
threads both; this stone must not drop them at the hand-off.

An **anonymous** native (`name: None`) keeps A-2-i's behaviour exactly: default-deny, because
nothing names it and nothing can prove it.

## Why this blocks A-2-ii-b, concretely

`wat/query/mem.wat:136` and `:163` — **two live substrate sites** — pass a bare field accessor as
`sort-by`'s key function:

```clojure
sorted (:wat::core::sort-by :wat::query::Row/sk matches)
sorted (:wat::core::sort-by :wat::query::IndexRow/isk matches)
```

Imposing pure ∧ deterministic ∧ total at `sort$native`'s door **today** refuses both — not because
the accessors are impure (they are not; as heads they classify pure) but because the resolved path
cannot see them. **That would be the gate reporting a defect it invented.**
`[[feedback_a_pass_answers_only_the_question_the_instrument_asks]]`

`wat/bracket.wat:783`'s inline-`fn` key function already classifies `true` and is not at risk.

## Out of scope = REJECTED (not deferred)

- **The imposition at `sort$native`'s door, and homing the verb** — **A-2-ii-b**, the next stone.
  This one restores an invariant; nothing consumes it yet.
- **`freeze.rs:803` opting in** — unchanged, still `Static`.

## THE FOUR QUESTIONS — flat YES/NO

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **A-2-ii-a** delegate a resolved NAME to `head_ok`'s ladder | YES | YES | YES | YES | ✅ **ADMITTED** |
| copy the accessor/constructor doors into `classify_closure` | YES | **NO** | **NO** | — | ⛔ **DISQUALIFIED** |
| impose anyway; rune or special-case the two query sites | **NO** | YES | **NO** | — | ⛔ **DISQUALIFIED** |

- **copy-the-doors Simple? NO** — two ladders to keep in step. **Honest? NO** — they will drift, and
  the drift reappears as this same bug at the next door added to one and not the other.
- **impose-anyway Obvious? NO** — a reader meets two live call sites refused for being impure when
  they are not. **Honest? NO** — it reports a defect the instrument invented rather than found.

## Acceptance

| what | command | expected |
|---|---|---|
| the asymmetry is gone | probe: accessor as head vs through a binding | `true` / `true` — **agreeing** |
| still no widening | probe: effectful `keyfn` through a binding | `false` |
| anonymous native still denies | probe: unnamed native through a binding | `false` |
| A-2-i's rows hold | `255-probe-the-classifier-follows-a-capture.wat` | `true` / `false` |
| negative control holds | `255-probe-the-classifier-cannot-see-through-a-closure.wat` | `true` / `false` / `false` |
| additive | `scripts/floor.sh`, exit read UNPIPED | 5109/5109, 0 failed |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |
