# BRIEF — 118.B2 · collapse six sequence verbs to ONE `Seqable<T>` clause each

You are a rider, not the orchestrator. **Ending your turn ENDS you** — it does not suspend you,
nothing wakes you, no notification is coming. Run every verification in the **FOREGROUND** and block
on it: your turn ends when the numbers are in your hands, not when a command is launched.

Work in `/home/watmin/work/holon/wat-rs/`. **Do not commit, push, stash, or revert.**

## Read first, in this order

1. `docs/arc/2026/04/118-lazy-seqs-vs-threaded-streams/DESIGN-STONE-118.B2-collapse-to-seqable.md`
2. `…/EXPECTATIONS-STONE-118.B2.md` — the scorecard you will be graded against.
3. **`wat-scripts/scratch-pad/probe-118B2-one-clause-lazy-producer.wat`** — ★ **THE WORKED
   PATTERN.** It runs green. Copy its shape; do not invent one.

## The work in one paragraph

In **`wat/seq.wat` only**, six lazy verbs — `interpose`, `keep`, `keep-indexed`, `map-indexed`,
`dedupe`, `distinct` — are each a `defclause` with one arm per container, every arm's body
byte-identical, delegating to a `<verb>-stream` twin. Replace each family with **ONE** definition
whose collection parameter is `:wat::core::Seqable<T>` and whose body walks with
`:wat::stream::next`, then delete the twin. Do the same for `reduce`'s Stream arms and for
`stream->pvec`. Every verb keeps its public name and arity, so **no call site anywhere moves.**

## The shape, proven and runnable

```wat
(:wat::core::defn :wat::core::keep<T,U>
  [f <- :wat::core::Fn(T)->wat::core::Option<U>
   coll <- :wat::core::Seqable<T>] -> :wat::stream::Stream<U>
  (:wat::stream::lazy
    (:wat::core::match (:wat::stream::next (:wat::core::Seqable/seq coll))
      ((:wat::stream::NextOutcome::Item value rest)
        (:wat::core::match (f value)
          ((:wat::core::Some v) (:wat::stream::cons v (:wat::core::keep f rest)))
          (:wat::core::None (:wat::core::keep f rest))))
      (:wat::stream::NextOutcome::Exhausted (:wat::stream::empty)))))
```

`rest` is a `Stream<T>`; handing it back into a `Seqable<T>` parameter is correct and proven.

## Rooms — `wat/seq.wat`

| line | what |
|---|---|
| **148** | `stream->pvec` — ★ the drain for the whole language; **tail-recursive, must stay so** |
| **226 / 232** | `reduce-stream` / `reduce` (only the Stream arms change; the `foldl` arms are untouched) |
| **458 / 466** | `interpose-stream` / `interpose` |
| **500 / 509** | `keep-stream` / `keep` |
| **527 / 540** | `keep-indexed-stream` / `keep-indexed` |
| **558 / 568** | `map-indexed-stream` / `map-indexed` |
| **586 / 600** | `dedupe-stream` / `dedupe` |
| **617 / 627** | `distinct-stream` / `distinct` |

The surface and its four `extend-type` impls are at the top of the same file. `Stream<` appears in
no other file under `wat/`.

## The gate

| # | assertion |
|---|---|
| 0 | run `scripts/floor.sh` FIRST and record the baseline: **4714 passed, 0 failed** |
| 1 | `./target/release/wat wat-scripts/scratch-pad/probe-118B2-one-clause-lazy-producer.wat` prints `2,4 \| 2,4 \| 2,4 \| 2,4` / `0,1,2,3,4` / `0,2,4` |
| 2 | ★★ `grep -c 'defn :[^ ]*-stream' wat/seq.wat` → **0** (it is 7 now) |
| 3 | ★★ report the per-verb clause-arm census **before and after**, with the pattern you used. ⚠ Validate the pattern against a known case first — an obvious one counts the twins' own recursive calls as arms and inflates the number |
| 4 | floor GREEN — read the **Summary line**, never a piped exit code |
| 5 | laziness holds: probe row 3 terminates, and `deftest …seq-of-infinite-stream-stays-lazy` passes |
| 6 | `grep -c 'seqable->stream' wat/seq.wat` → **> 0** (it is `Seqable/seq`'s implementation; it is NOT yours to delete) |
| 7 | `cargo clippy --release --all-targets` → **0** |
| 8 | `#[ignore]` count → **13** |

## STOP triggers — ship nothing on that axis; report and stop

- **STOP-1 — `stream->pvec`'s recursion would leave TAIL POSITION.** It is the materializer every
  eager drain funnels through. `match` carries a tail position (proven), but nesting the recursive
  call inside a `cons`/`+`/argument makes it O(n)-stack, and that death is a **silent SIGSEGV** a
  green floor will not catch. If you cannot keep it in tail position, STOP and report.
- **STOP-2 — a verb's public NAME or ARITY would change.** `dedupe` and `distinct` thread state
  (`prev`, `seen`) that currently lives on the twin. If collapsing forces that state into the public
  signature, STOP — that is a design question, not yours to settle.
- **STOP-3 — the floor goes red for any reason other than a line-number shift in a golden.** Do NOT
  re-run first: `scripts/floor.sh` has already kept the untruncated log at `.floor/latest/`. Copy
  the failing test's **entire** stdout and stderr **verbatim** — never a summary, never a
  `| head`/`| tail` window — and name the exact assertion or match arm that fired. **There is no
  such thing as a known flake.**
- **STOP-4 — the `#[ignore]` count moves off 13.**

⚠ **Goldens:** an `.edn` golden under `tests/diagnostics/` failing because a **line number in a
source file shifted** is yours to update — that IS the work. Say which moved and by how much.
Anything else red is STOP-3.

## Out of scope — affirmative cuts, each with its owner

- **The two memos** (`stream/mod.rs:66`, `:124`) — B3. Measured: with any three-call walker still
  alive, deleting a memo makes user code run **3×**.
- **`first` / `rest` / `empty?` on Stream** — B4, and a dialect ruling the builder owns.
- **`seqable->stream`'s public name, `stream->vec`, `extract_lazyable_elem`** — B5 / a Rust stone.
- **`dorun` building a Vector and binning it** — a leaf of B3/B5.
- **`keep-stream`'s deep recursion on a long run of `None`s** — pre-existing, tracked with #58/#86.
  Do not fix it; do not make it worse.

## Report

The scorecard row by row, the before/after census with the pattern you used, the honest deltas
(anything that surprised you), and line counts. If a STOP fired, the verbatim evidence and which
one.
