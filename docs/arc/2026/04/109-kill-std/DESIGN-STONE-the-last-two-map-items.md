# DESIGN — the map's last two items: `option`/`result` and the purity classifier

> Items 6 and 7 of `[[NOTE-partire-RECAST-on-the-current-runtime]]` — *"option / result: 7 items,
> edges exist"* and *"purity classifier: 2 items -> src/rete/purity.rs. Level 1: actively misleading
> where it sits."* After this, the recast's map is closed and the residue is the eval spine, the
> in-file tests, and 4d.

## The measurement — 11 items, not 9

| destination | items | lines | why there |
|---|---:|---:|---|
| `src/option/mod.rs` (new) | 3 | 132 | edge `src/intrinsic/option.rs` |
| `src/result/mod.rs` (new) | 4 | 196 | edge `src/intrinsic/result.rs` |
| **`src/assertion.rs`** (exists) | 2 | 101 | ⬅ the shared `expect` machinery — see below |
| `src/rete/purity.rs` (exists) | 2 | 26 | the map's home, verified below |
| | **11** | **455** | `runtime.rs` 19,293 → **18,838** |

★ **The map said 7; the closure adds `expect_panic` and `extract_panics`.** Their only callers are
`eval_option_expect` and `eval_result_expect` — both movers. Left behind, both new homes would reach
into the megafile for their own verb machinery: the numeric disease, twice.

★★ **And their home is neither `option` nor `result`.** `expect_panic` takes the verb name as a
parameter and builds a `crate::assertion::AssertionPayload`; `src/assertion.rs` **owns that type**
and already holds `eval_kernel_assertion_failed`, a verb impl of the identical shape (evaluate, build
payload, panic). Placement by the concept and the constructed type — not by inventing a third module
for two functions, and not by arbitrarily giving them to one of the two homes that share them.

## ⛔ `eval_try` in a test is a NAME COLLISION, not a consumer

`tests/value/probe_rational_C4_mixed_float.rs:16` defines its own `fn eval_try(src: &str)` — a local
harness helper. My first consumer scan counted it as an external consumer of the intrinsic.
**Confinement measured correctly: every real consumer of the seven verbs is its own edge file.**
`[[feedback_validate_a_search_pattern_before_trusting_its_count]]`

## Item 7's home — the map was right, and here is the reason it never gave

`is_effectful_op` calls `crate::intrinsic::registry()` first and falls back to `effectful_by_prefix`;
they are one two-tier classifier and move as a pair. The map sent both to `src/rete/purity.rs`
without stating why that is not a cycle. Measured:

- `src/rete/purity.rs` **already** calls `crate::intrinsic::registry()` (lines 461, 628). The move
  adds **no new edge**.
- The reverse edge — `src/intrinsic/mod.rs` → `crate::rete::purity::effectful_by_prefix` — exists
  only inside `#[test] declared_purity_vs_effectful_by_prefix_census` (line 1423). At a crate
  boundary that is a **dev-dependency, not a cycle.**

## ⛔ A doc comment on disk asserts the opposite of the code it describes

`src/intrinsic/rete.rs:15`:

> *"★ `:wat::rete::` is deliberately **ABSENT** from `effectful_by_prefix` (`src/runtime.rs`) — this
> wave's whole premise is that these nine are read-only, so nothing here widens that list."*

`effectful_by_prefix`'s body contains `head.starts_with(":wat::rete::")`. Both sides dated:

```
e01428497  W5a  wrote the comment       — TRUE when written
2bc1135aa  W5b  added the prefix        — commit title: "six rete mutators homed —
                                          THE WIDENING W5a FORBADE, and my framing was wrong on two"
```

★★★ **W5b knew it was overturning the claim, said so in its own commit message, and left the comment
standing.** The claim has been false on disk ever since. It is in this stone's blast radius because
this stone moves the function the comment names, so it is corrected here — not as tidying, but
because a stone that relocates a function and leaves a false claim about it is shipping the same
defect one file over. `[[feedback_a_patch_fixes_one_copy_of_a_claim]]` ·
`[[feedback_a_blocker_note_is_a_claim_with_a_date_on_it]]`

## THE ONE CONTRACT DECISION — pinned

**`option` and `result` are DIRECTORIES (`src/option/mod.rs`), not top-level files.**

`src/option.rs` would be cheaper for three items. But the builder's stated sequence is *"once those
partition lines are drawn in `src` … we begin the move to crates"* and *"long term `src/*.rs` is
likely to only hold a `lib.rs`."* **A directory IS the partition line.** A new top-level `.rs` adds to
the pile this campaign exists to empty, and would have to be converted again before the crate move.
Three items in a `mod.rs` is a home that grows; a file at `src/option.rs` is a step backwards taken
for tidiness.

## THE FOUR QUESTIONS — flat YES/NO

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **11 items, 4 destinations, one stone** | YES | YES | YES | YES | ✅ **ADMITTED** |
| the map's 9 items (leave the `expect` pair) | YES | YES | **NO** | — | ⛔ DISQUALIFIED |
| `expect_panic`/`extract_panics` into `src/result/` | **NO** | YES | YES | — | ⛔ DISQUALIFIED |
| a new `src/expect/` for the pair | YES | YES | YES | **NO** | ⛔ DISQUALIFIED |
| `src/option.rs` / `src/result.rs` as files | YES | YES | YES | **NO** | ⛔ DISQUALIFIED |
| split into two stones (6 and 7 separately) | YES | YES | YES | **NO** | ⛔ DISQUALIFIED |

- **map's-9 Honest? NO** — measured: both new homes would reach into `runtime.rs` for the `expect`
  machinery their own verbs need. That is the defect an independent cast found in `src/numeric/`.
- **pair-into-result Obvious? NO** — `eval_option_expect` calls it too; a reader asking why Option's
  verb machinery lives under Result gets no answer that is true.
- **new-`src/expect/` Good UX? NO** — two functions, and `src/assertion.rs` already owns the type
  they construct and a verb of the same shape.
- **files-not-directories Good UX? NO** — see the contract decision; it is a step the crate move
  would immediately undo.
- **two-stones Good UX? NO** — 455 lines total across four destinations with no interaction; two
  floor cycles buy nothing. (Obvious/Simple/Honest all hold — this is a real UX cut, not a dodge.)

## Out of scope = REJECTED (not deferred)

- **4d, the shared `Fault`/`Failure` residue.** Still deliberately unassigned; the crate boundary is
  its forcing function.
- **`src/assertion.rs:34`'s facade import** (`use crate::runtime::{eval, Environment, …}`) — a real
  instance of the artifact, and the re-point sweep's. Touching it here makes a red unattributable.
- **Whether `effectful_by_prefix` should still exist at all.** Arc 255 made the registry the sole
  authority and this is its named fallback. Questioning it is a registry ruling, not a relocation.

## Acceptance — rows chosen to be unfakeable

| what | command | expected |
|---|---|---|
| the two new homes exist as directories | `ls src/option/mod.rs src/result/mod.rs` | both |
| the edges stop naming the megafile | `grep -c "crate::runtime::eval_" src/intrinsic/option.rs src/intrinsic/result.rs` | **0**, **0** |
| the `expect` pair landed with its type | `grep -c "fn expect_panic\|fn extract_panics" src/assertion.rs` | **2** |
| the classifier pair moved together | `grep -c "fn effectful_by_prefix\|fn is_effectful_op" src/rete/purity.rs` | **2** |
| ⛔ the false claim is corrected | `grep -c "deliberately ABSENT from .effectful_by_prefix" src/intrinsic/rete.rs` | **0** |
| ⛔ the spine | `grep -c "^pub(crate) fn eval_tail\|^pub(crate) fn eval_inner\|^pub fn eval\b" src/runtime.rs` | **3** |
| ⛔ 4d residue untouched | `grep -c "fn fault_value\|fn fault_with_cause\|fn check_failed_cause\|fn failure_names" src/runtime.rs` | **4** |
| bodies verbatim | diff each moved item vs `git show HEAD:src/runtime.rs` | byte-identical |
| runtime.rs | `wc -l` | 19,293 → **~18,838** |
| floor | `scripts/floor.sh`, exit read UNPIPED | 5114/5114, 0 failed |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |

★ **Half 1 and Half 2 are both EMPTY, derived.** Every one-hop dependency is already `pub(crate)` and
no import is orphaned — the third consecutive stone where the class costs nothing.
