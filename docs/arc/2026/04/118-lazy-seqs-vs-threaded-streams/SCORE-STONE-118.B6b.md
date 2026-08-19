# SCORE — STONE 118.B6b · `foldr` is retired

Written after the orchestrator's own independent re-run. Rider: sonnet, one flight, no commit.

## The scorecard

| # | what | expected | RESULT |
|---|---|---|---|
| 1 | `(foldr …)` is refused, message names the replacement | refused | **PASS** — but see ★ below; the refusal had to be BUILT |
| 2 | ★ `(reduce f init (reverse coll))` gives `foldr`'s answer | **2**, not −6 | **PASS**, re-run by the orchestrator |
| 3 | the 4 test call sites rewritten, not deleted | present, same assertions | **PASS**, read the diff |
| 4 | all three ledgers clean | build + rete gates | **PASS**, nothing fired |
| 5 | ★ both capability headers name their REAL consumers | `mappable()` drops `foldr`; `ordered()` = `reverse`/`concat` | **PASS** |
| 6 | floor | ≥4772 run, 0 FAIL, 19 skipped | **PASS** — `4772 tests run: 4772 passed, 19 skipped` (own invocation, second run; first was RED — see below) |
| 7 | clippy | 0 | **PASS** — 0 |
| 8 | ignores | 13 | **PASS** — 13 (line-anchored `#[ignore` grep) |
| 9 | `wat/seq.wat` untouched | `reduce`/`foldl` unchanged | **PASS** |
| 10 | the retirement TABLE names the replacement | row present, diagnostic carries it | **PASS** |
| 11 | `wat-scripts/` still LOADS | `every_wat_scripts_file_loads` green | **PASS** |
| 12 | the negative control survives | renamed, not deleted | **PASS** |

**Net −144 lines** (152 insertions, 296 deletions) across 20 files, plus 7 golden fixtures.

## ★ THE FLOOR WENT RED, AND IT WAS THE GOLDENS CARVE-OUT — ONE FILENAME OVER

Five tests failed on the first floor: `probe_diagnostic_value_snapshot_in_errors` probes 1/2/6/7/8.
Every one pinned `:file "src/runtime.rs" :line 25339`; the site had moved to `25336`.

**Ratified before touching them, the documented way:**

```
only :line differs — :col 17 IDENTICAL in all five, every other field byte-identical
one hunk in runtime.rs before old line 25339, net −3 — the deleted `foldr` dispatch arm
the moved site is `crate::rust_caller_span!()` in both HEAD and the working tree
```

The rider had already found this class in `check.rs` (2 fixtures, +5, same ratification) and stopped
on it as STOP-4 — correctly. But its census was **`grep -rl '"src/check.rs"' tests/`**: it found the
class, then searched for it under the one filename it had just been burned by. The complete census —
`grep -rl ':file "src/' tests/` — returns **8** fixtures pinning a real Rust line: `runtime.rs` ×5,
`check.rs` ×2, `freeze.rs` ×1 (untouched, passed). The other six pin synthetic `.wat` paths.

★ **The rider's instrument was scoped to the evidence that produced it.** Same shape as yesterday's
census, one level down. `[[feedback_state_what_the_instrument_can_see_before_quoting_it]]`

## ★★ THE STONE'S REAL FINDING — the refusal had to be BUILT, and that is a root

EXPECTATIONS row 10 predicted that deleting the dispatch arm alone would yield *"a generic
unknown-form error"*. **It yields nothing.** The rider found it; the orchestrator measured it
independently, with a positive control:

```
:wat::core::i64::+ 1 2                 rc=0   CONTROL — a real verb passes
:wat::core::totally-invented-verb 1 2  rc=0   ← INVENTED. ACCEPTED.
:wat::core::totally-invented-verb 1    rc=0   ← INVENTED. ACCEPTED.
:user::totally-invented-verb 1 2       rc=1   UnresolvedReference — the check EXISTS
```

`check.rs:5568` — `// HARVEST (236.2): silent-by-intent — no scheme found for multi-arg form; accept
and pass.` The narrowing that *can* emit `UnknownCallee` (`:5489`) is gated on `!k.starts_with(":wat::")`.

So B6b had to add an explicit `MalformedForm` arm in `infer_list` — mirroring the `:wat::core::try`
zombie arm, which exists for exactly the same reason. **Every retirement this substrate has shipped
has paid a per-verb patch to work around one permissive fallback**, and a plain typo in a `:wat::`
verb still type-checks green. Filed as **task #110** with the probe and the mechanism. It is the
root; the HARD CUT arms are the stem.

⚠ **My first probe was malformed** and returned `rc=1` on all three arms — which reads as "the
fallback rejects." It was my `let` syntax; the check never reached the call. Caught only because the
*control* failed identically. An instrument that fails the control is not measuring the question.

## Honest deltas

- **The rider disclosed a re-run it should not have made**, and named why: its first capture used
  `| tail -30` and truncated the arm past recovery, so it re-ran to retrieve it. That is FM's
  truncating-pager failure producing the very re-run the red protocol forbids — disclosed rather
  than hidden, which is the right handling of a wrong move.
- **Three prose-only `foldr` mentions** outside the brief's file list (`probe_arc278_fence_hof.rs`,
  `probe_arc247_hof_fn_first.rs`, `probe-rete-predicate-termination-routes.wat`) were swept. No
  semantic change; flagged by the rider rather than smuggled.
- **Runtime overran the 45–70 minute prediction**, and the overage is fully accounted for: the
  checker-fallback investigation and the STOP-4 root-cause work were both unplanned.

## Out of scope — affirmative cut, tracked

**The rete sub-language loses the right fold.** The vocabulary has `foldl`/`map`/`filter`/`reduce`
and **no `reverse`**, so `(reduce f init (reverse coll))` has no rete spelling. Removing `foldr`'s
row was forced (a `Redispatch` alias whose `core_name` is deleted is a dangling declaration).
Minting `:wat::rete::core::reverse` is **not** one table row: `:wat::core::reverse` sits in
`purity.rs`'s `KNOWN_UNREVIEWED` — the frozen names of dispatched verbs with no purity ruling — and
`every_rete_row_is_total` demands `total: true`. Every verb that HAS a rete row is ruled; the three
seq verbs that are not (`reverse`/`take`/`drop`) have none. **The wall is already holding.**
Tracked as **task #109**; the builder rules it.
