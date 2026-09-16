# EXPECTATIONS — what a door allocates belongs to the session it opens

> Written **before** the strike. **Every row's command was run against HEAD and its pre-value is
> recorded**, because three scorecard rows this session could not do their job.

## ⛔ NO PINNED TEST COUNT

**The floor must be ≥ 5,213 plus every arm you drive. Exceeding it is a PASS.** Report the number.

## The scorecard, with pre-values measured at HEAD `8a3ec39a1`

| # | what | command | pre-value AT HEAD | expected after |
|---|---|---|---|---|
| 1 | neither call at the door | `grep -c 'check_session_ceiling\|mark_session_origin' src/rete/export.rs` | **0** (measured) | ≥ 1 — both calls present |
| 2 | an unmarked origin reads zero | a probe: mark one key, allocate, read both | **marked 2097268 / unmarked 0** (measured) | the imported session reads the build it paid for |
| 3 | `from_pairs` is quadratic | a probe over 500/1k/2k/4k, six samples, minimum | **1.05 / 1.95 / 2.57 / 4.87 µs per pair** (measured) | **unchanged — `pmap.rs` is cut**. This row is the BEFORE-curve for a later speed stone |
| 4 | the cap refuses | a probe importing past the cap | — | refused with `malformed`, naming the cap and the count |
| 5 | the cap was measured | read the constant | — | states the corpus maximum AND the worst-case cost on DESIGN's curve. A bare round number fails |
| 6 | non-clobber preserved | read the sibling | — | `or_insert`, never `insert` |
| 7 | the header | `grep -c 'Five independent walls' src/rete/export.rs` | **1** (measured) | says six, and the new wall is described |
| 8 | blast radius | `git diff --stat` | — | `export.rs` + `alloc_counter.rs` + probes. **`pmap.rs` absent** |
| 9 | lints | `cargo nextest run --release -E 'binary_id(wat::lint)'` | **116/116** (measured) | green — the rider runs this |
| 10 | floor | `./scripts/floor.sh` | **5213/5213** (measured) | ≥ 5,213 + every new arm, zero FAIL rows |
| 11 | clippy | `cargo clippy --release --workspace --all-targets -- -D warnings` | **rc=0** (measured) | silent |

## The mutation proofs — one per arm, and the arms are named

Three arms, three mutations, because they are three mechanisms:

1. **The origin capture** — move `mark_session_origin_at` to *after* the build using `thread_bytes()`
   at that moment (i.e. re-create the defect). The row-2 probe must go RED. If it stays green, the
   probe is not measuring the build.
2. **The cap** — set it to `usize::MAX`. The row-4 probe must go RED.
3. **The non-clobber rule** — change `or_insert` to `insert`. A probe filing an origin twice must go
   RED, or trap 2 is untested.

Per arm: **proven** / **reachable but not driven** / **not reachable, and why**. An unreached arm
named as unreached is a pass; one not mentioned is a fail.

## Runtime prediction

50–70 minutes. Two calls and a constant are small; the measurement (trap 3) and the three mutations
are where the time goes.

## What would make this strike a failure even if every test passes

**Marking the origin after the build.** It is the natural place to put it — the key exists there —
and it reproduces the exact defect while every probe that only checks *"is an origin filed"* goes
green. Row 2 and mutation 1 exist for this and nothing else.

The second: **a cap with no arithmetic.** The finding is that N was unbounded on a quadratic curve.
A constant that does not say what it costs at the limit has moved the unstated criterion rather than
removed it.

The third: **touching `pmap.rs`.** It is cut, with a reason, and row 8 checks it.
