# BRIEF — STONE 118.B8 · `dorun` stops retaining, and a stale deletion order comes out

Read `DESIGN-STONE-118.B8-the-arcs-tail.md` first. It carries the ruling, the three-way finding,
and four traps.

**This brief covers PARTS 1 and 2 only.** Part 3 (the class census) is an instrument whose entire
output is a claim, so the orchestrator runs it — you are not being asked for it, and nothing here
depends on it.

## ⛔ YOU DO NOT RUN THE FLOOR

**You MAY**, in the FOREGROUND: `cargo build --release`, `./target/release/wat --check <file>`, a
`.wat` probe, and a SCOPED `cargo nextest run --release -E 'test(<pattern>)'`.
**You may NOT** run `scripts/floor.sh` or an unscoped `cargo nextest`. The orchestrator measures
centrally, once, on a quiescent tree.

Cap anything long: `systemd-run --user --scope -q -p MemoryMax=4G -p MemorySwapMax=0 timeout 900 …`
Read exit codes directly, never through a pipe.

---

## PART 1 — `dorun` walks with `next` and retains nothing

### Read in order

1. **`wat/seq.wat:209-210`** — `dorun` as it stands. Two lines; the whole subject.
2. **`wat/seq.wat:206-207`** — `doall`, its neighbour. ★ **Do not touch it.** It RETURNS the Vector,
   so `(into [] coll)` is correct for it. The two verbs differ in exactly this.
3. **`wat/seq.wat:168-174`** — `stream->pvec-spec`, the drain. **This is the shape to copy**: a
   `match` on `(:wat::stream::next s)`, `Item` arm recursing in TAIL POSITION, `Exhausted` arm
   returning the base case.
4. **`wat/seq.wat:155-167`** — the drain's header, which states the tail-position rule and names the
   probe that proved it.

### The sketch

```wat
(:wat::core::defn :wat::core::dorun<T> [coll <- :wat::stream::Stream<T>] -> :wat::core::nil
  (:wat::core::match (:wat::stream::next coll)
    ((:wat::stream::NextOutcome::Item _value rest) (:wat::core::dorun rest))
    (:wat::stream::NextOutcome::Exhausted nil)))
```

The `Item` arm's recursive call **must sit in tail position** — `probe-118B-match-tco-drain.wat`
carries a non-tail sibling control that SIGSEGVs at the same depth. Nesting the recursion inside any
argument silently makes `dorun` O(n)-stack.

Update `dorun`'s doc comment (the block at `wat/seq.wat:200-205` covers both verbs) so it says what
each verb now does and why they differ.

### Prove it, two ways

- **Retention.** `wat-scripts/scratch-pad/probe-118B-dorun-retention-slope.wat` already carries the
  methodology — peak RSS at increasing n, with an unbounded source so the source itself cannot
  pollute the number, and an O(1) accumulator so the scaffolding cannot swamp the signal. **Read its
  header before adapting it**; both of those choices exist because an earlier draft lied. Point the
  method at `dorun` at 100k/200k/400k/800k. Report the four numbers, before and after.
- **Effects still run.** A probe that calls `dorun` over a stream of n elements whose producer
  records each force, asserting **exactly n**. `probe-118B-memo-state-detector.wat` is the shape for
  counting forces.

---

## PART 2 — the deletion order comes out of `extract_lazyable_elem`'s doc

### Read in order

1. **`src/collection/infer.rs:656-658`** — the standing order: *"this function's hand-rolled
   four-head match is exactly what that stone would delete."*
2. **`src/collection/infer.rs:665-701`** — the function. Count its heads: Vector, List,
   PersistentVector, Stream, **and `Seqable` itself**.
3. **`src/collection/infer.rs:673-687`** — B7's comment on the `Seqable` arm, which explains why that
   arm exists and what it prevents.

### The work

⛔ **The function stays. All six call sites stay.** (`infer.rs:734, 810, 887, 1016, 1079, 1142`.)

Rewrite the doc block so it records what the function BECAME rather than what a superseded stone
planned for it:

- It is **the one door** that knows the `Seqable` set — four concrete heads plus the surface.
- `Seqable` did not replace it; **B7 made `Seqable` its fifth head**, and that arm is what keeps the
  eager-container tax B6 removed from returning through the front door.
- **Delete the "would delete" instruction.** An order whose reason has expired is worse than no note,
  because its whole job is to stop the next person looking.

Keep the refuted-blockers history (`:640-664`) — that is a record of something learned, and it is
framed as history. Only the standing instruction goes.

---

## Blast radius

`wat/seq.wat` (one `defn` body + its doc block) · `src/collection/infer.rs` (one doc block) · one or
two `wat-scripts/scratch-pad/` probes. **No `src/` logic changes at all** — Part 2 is comment text,
Part 1 is wat.

## STOP triggers — ship nothing further, report the gap, stop

**STOP-1** — ★ **the expand-time trap.** `:wat::core::dorun` is listed in `is_pure_total`
(`src/macros/eval.rs:565`), deliberately, so it is legal at MACRO-EXPANSION time. Its body becomes
**self-recursive**, and task #107 records that a macro body's reach into wat-defined functions is
restricted. If a macro-expansion-time `dorun` now fails where it previously worked, **STOP and report
the payload verbatim.** Do not route around it, and do not revert Part 1 to make it go away — the
finding is worth more than the stone.

**STOP-2** — the retention numbers do NOT go flat. Report all eight (four before, four after) and
stop. A `dorun` that is merely faster is B5's result, not this stone's; if the slope survives, the
mechanism is not what the stone says it is.

**STOP-3** — anything outside `wat/seq.wat`, `src/collection/infer.rs`, and
`wat-scripts/scratch-pad/` needs to change.

**STOP-4** — a scoped `nextest` fails outside what you touched.

## On a RED, in this order

There is no such thing as a known flake. (a) Do **NOT** re-run — a re-run that goes green destroys
the only evidence. (b) Copy the failing test's whole stdout+stderr block **verbatim** — never a
summary, never a `| head`/`| tail` window; that truncation is what forced a re-run last stone.
(c) Name the exact assertion that fired. (d) Surface it.

## Your report

1. `dorun` before and after, quoted.
2. **The eight retention numbers**, in a table, before and after.
3. The effect-count probe: n in, n forces out.
4. `extract_lazyable_elem`'s doc block before and after — and confirmation the function and all six
   call sites are untouched.
5. Everything you ran, with results. State plainly that you did not run the floor.
6. Honest deltas — anything that surprised you, anything this brief got wrong. Line counts.
   Wall-clock against a **35–55 minute** prediction.

Slow is smooth, smooth is fast.
