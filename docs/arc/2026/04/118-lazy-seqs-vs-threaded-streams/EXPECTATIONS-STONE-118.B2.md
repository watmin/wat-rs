# EXPECTATIONS — 118.B2 · collapse to `Seqable<T>`

**Written before the strike, 2026-08-18, against `eab12e05`** (floor 4714/4714, clippy 0,
ignores 13). Fixed here so the result cannot move the goalposts.

## The scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 0 | ★ baseline is green before touching anything | `scripts/floor.sh` Summary | **4714 passed, 0 failed** |
| 1 | the proven shape still runs | `./target/release/wat wat-scripts/scratch-pad/probe-118B2-one-clause-lazy-producer.wat` | `2,4 \| 2,4 \| 2,4 \| 2,4` / `0,1,2,3,4` / `0,2,4` |
| 2 | ★★ **the seven twins are GONE** | `grep -c 'defn :[^ ]*-stream' wat/seq.wat` | **0** (is 7) |
| 3 | ★★ **the identical arms are GONE** — a COUNT, not a claim | rider reports the per-verb arm census **before and after** | 6 verbs × 1 definition each |
| 4 | every verb still behaves identically | floor + `wat-tests/` | **0 FAIL** |
| 5 | `Seqable` has real consumers now | `grep -c 'Seqable<T>' wat/seq.wat` | **≥ 6** beyond the surface decl |
| 6 | laziness preserved on every migrated verb | probe row 3 + `seq-of-infinite-stream-stays-lazy` | terminates |
| 7 | `seqable->stream` still present (it is `Seqable/seq`'s impl) | `grep -c 'seqable->stream' wat/seq.wat` | **> 0** |
| 8 | `reduce`'s eager arms untouched | diff review | `foldl` arms byte-identical |
| 9 | floor | `scripts/floor.sh` → Summary | **≥ 4714 passed, 0 failed, 0 timed out** |
| 10 | clippy | `cargo clippy --release --all-targets` | **0** |
| 11 | ignores | the `#[ignore]` grep | **13** |

**Rows 2 and 3 are the stone.** Row 4 is what makes it safe. A green floor with the twins still
present is not this stone — it is a no-op wearing a green badge.

⚠ **Row 3 must be a measured census, not a sentence.** My own pre-brief grep for per-verb arm counts
was WRONG — it counted the twins' recursive self-calls together with the delegating arms. The rider
produces the number with a validated pattern and shows the before/after.
`[[feedback_validate_a_search_pattern_before_trusting_its_count]]`

## Independent prediction

**Runtime: 50–80 minutes.** Six bodies, each genuinely different (two carry state, one carries an
index, one carries a seen-set, one splits on the first element). The shape is proven so there is no
discovery cost, but `dedupe`/`distinct`/`interpose` each need their state threaded through the new
`match` arms correctly, and `stream->pvec` is tail-recursive and must stay so.

**Time-box: 160 minutes** (2× the upper bound).

## Trap doors — named before the strike

1. **★ TCO on `stream->pvec`.** It is tail-recursive today via `if`, and the drain for the entire
   language. `match` **does** carry a tail position — proven, with a control that SIGSEGVs at the
   same depth (`probe-118B-match-{tco-drain,no-tco-control}.wat`). But the migrated body must keep
   the recursive call in **tail position**; nesting it inside a `cons`/`+` silently makes the
   language's materializer O(n)-stack, and per tasks #58/#86 that death is a **silent SIGSEGV**.
2. **The `-stream` twins' recursion is NOT tail** and must not be made so — they are lazy producers
   returning `(stream/cons … (recur …))` inside `stream/lazy`. Laziness bounds the depth. Row 6.
3. **`interpose` is not uniform** — its first element is emitted bare and only subsequent ones are
   preceded by `sep`. The twin exists partly to carry that split. Read its comment before rewriting.
4. **`dedupe` threads `prev <- Option<T>`; `distinct` threads `seen <- HashSet<T>`.** Their state
   must move into the collapsed definition's parameters, which changes their public arity — ⚠ if a
   verb's PUBLIC arity would change, that is a STOP, not a design choice (see STOP-3).
5. **Deleting `seqable->stream`.** It is `Seqable/seq`'s implementation now. Row 7.

## What would make me call this Mode B

- The twins deleted but the arms merely *reduced* rather than collapsed to one (rows 2+3 disagree).
- Any verb's public name or arity moving.
- A `#[ignore]` added, or a test deleted to make the floor green.
- `stream->pvec` migrated out of tail position (trap 1) — that is a silent production defect that a
  green floor will not catch.

## What I will re-run myself before committing

Rows 1, 2, 3, 4, 9, 10, 11 — independently, on my own invocation, per FM 9. **Row 3 especially**: it
is the number that says whether the stone happened at all, and it is the one I already got wrong
once.

## What this stone is NOT allowed to claim when it lands

That the memory defect is fixed — **both memos are still in place**; B3 owns that, and its
acceptance is the separate pair (`f` runs exactly N times AND the per-element slope reaches the flat
column measured in `MEASURED-118.B-memo-off-is-flat.md`). Nor that the three doors are closed (B4),
nor that `extract_lazyable_elem` is gone (a later Rust stone).
