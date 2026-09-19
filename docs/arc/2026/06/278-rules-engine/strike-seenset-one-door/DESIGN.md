# DESIGN — A4: the fixpoint's dedup set is two halves held by convention

Board row A4, re-grounded at HEAD 2026-09-06. **Sixth instance of the shape this session has cured
five times** — `JoinLeftIndex`, `BetaStore`, `ClassPlan`, `SlotZip`, `FactBag`.

## ⚠ Severity, measured and stated first

**The hazard is NOT live.** I checked the thing that would make it live and it does not hold:

`AggregateValue::from_parts` (`value.rs:1064-1083`) is the **sole** stamping site, and all four
constructors — `struct_`, `record`, `record_arc`, `holon_record` — funnel through it. The stamp is a
pure function of `(nature, class, fields)`, so **equal aggregates always agree on stamping** and can
never land in different halves. There is even an `if id == 0 { 1 }` sentinel guard so a legitimate
zero hash cannot read as "unstamped".

So this is **structural hygiene, not a correctness fix**, and the SCORE must say so. What it buys is
that rete stops *depending* on a `src/value/` invariant it does not own and cannot enforce.

## The finding

`src/rete/kernel/fire/delta.rs:188-196` — one logical set, two containers:

```rust
pub(crate) fn seen_insert(ids: &mut FxHashSet<u64>, rest: &mut FxHashSet<Value>, v: &Value) -> bool {
    match v {
        Value::Aggregate(a) if a.identity() != 0 => ids.insert(a.identity()),
        _                                        => rest.insert(v.clone()),
    }
}
```

`seen_insert` is the only **insert** door, but the halves are not behind it. They are threaded as
**two independent `&mut` handles** through ~20 sites: `pass/mod.rs:155-156` (ctx fields),
`alpha.rs:133-134`, `production.rs:31-32`, eight sites in `delta.rs:491-657`, `round_census.rs:33-34`.

Two consequences visible today:

- `production.rs:88` — `seen_ids.reserve(…)` reserves one half alone.
- `round_census.rs:133` — `seen_ids.len() + seen_rest.len()`. **The union's cardinality, computed by
  hand at the call site**, because no type owns it.

If the halves ever disagreed about one logical value, both inserts would return "novel" and the
fixpoint's dedup — its termination mechanism — would silently stop deduping.

## THE ONE CONTRACT DECISION

**`SeenSet` owns both halves; `insert` and `len` are its only doors.**

```rust
pub(crate) struct SeenSet { ids: FxHashSet<u64>, rest: FxHashSet<Value> }
// insert(&mut self, v: &Value) -> bool     — today's seen_insert body
// len(&self) -> usize                      — today's hand-written sum
// reserve(&mut self, n: usize)             — production.rs:88's call, named
```

The ~20 sites carrying two `&mut` handles collapse to one. `round_census.rs`'s hand-summed
cardinality becomes `seen.len()`.

⛔ **This is encapsulation, and it does NOT cure the cross-path hazard.** An unstamped-but-shallow
aggregate would still take the `rest` arm inside the door. Say that plainly rather than letting the
type read as a fix.

## ★ And a guard, which is the part that survives the note

The invariant is *"no logical value in both halves."* Express it where it lives:

**In `insert`, under `debug_assert`, when an `Aggregate` takes the `rest` arm** — that is the only
suspicious case — check that the value is not also present as a stamp in `ids`. A `debug_assert`
costs nothing in release, and it converts the silent dedup failure into a loud panic if the
`src/value/` invariant recorded in
`docs/arc/2026/04/109-kill-std/NOTE-value-hash-has-two-functions-selected-by-a-flag.md`
is ever broken.

**STOP-2 if that cannot be done without re-deriving a hash on the hot path.** Report the cost rather
than shipping a `debug_assert` that is expensive even in debug; a cheaper expression of the same
invariant, or none with the reason stated, both beat a slow one.

## Out of scope = REJECTED

- **Touching `src/value/`.** Not rete. The `Hash`-split and `value_is_shallow` findings are filed in
  arc 109 and are not fixed by this strike.
- Changing what is deduped, or the `identity != 0` split itself — it is the fast path and it is
  correct today.
- Merging the two halves into one `FxHashSet<Value>`. That discards the u64 fast path measured by
  `accum_alpha_cost.rs`'s S-arm; nothing here has measured that it is affordable.

## Mutation proof

Available and required, on the encapsulation: after the change, reaching a half directly must not
compile. Show the `error[E0616]`/`E0609` from **outside** the owning module — not from inside it,
where a private field is legitimately reachable (that mistake was made earlier this session on the
`SlotZip` bypass probe).

## STOP triggers

1. Any measured value changes → STOP. This is a refactor; dedup behaviour is identical.
2. The `debug_assert` needs a hot-path hash → STOP and report the cost.
3. Collapsing the handles needs a signature change outside `src/rete/` → STOP.
4. The floor's fixpoint counts (`prod:derivations`, `seen_facts`) move → STOP; dedup changed.
