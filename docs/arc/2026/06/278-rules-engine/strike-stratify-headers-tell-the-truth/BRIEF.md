# BRIEF — replace two false stratify headers; add one pointer

## The work

Three prose edits, no logic. The replacement text is given VERBATIM below because it must not
introduce a new false claim — that is the defect being fixed. Apply exactly; if any line does not
match the tree, STOP (trigger 1).

## Read in order

1. `docs/arc/.../strike-stratify-headers-tell-the-truth/DESIGN.md` — the audit table. Six of the
   eight `mirror`/`lockstep` claims were verified to HOLD and must be left alone.
2. `src/rete/kernel/stratify.rs:221-227` — the divergent term itself.
3. `src/rete/kernel/tests/stratify_numbers.rs` — the gate the new prose cites.

## Edit 1 — `src/rete/kernel/stratify.rs`, the file header

REPLACE this line:

```rust
// impl that moves in lockstep with it (the dual-impl doctrine — no `native?` flag anywhere).
```

WITH:

```rust
// impl (the dual-impl doctrine — no `native?` flag anywhere).
//
// ⛔ IT IS NOT IN LOCKSTEP, AND THIS HEADER CLAIMED IT WAS FOR THE WHOLE ARC — while listing
// `stratify-sweep` among the faithfully ported functions. `native_stratify_sweep` carries a
// term `stratify-sweep` has no counterpart for: a type bagged by `:exists` / accumulate
// `:from` that THIS rule set also derives gets +1 (`:221-227`). The oracle folds those into
// `rule-consumes`, where `req-pos` is explicitly NOT +1 (`stratify.wat:233-234`).
//
// DRIVEN 2026-09-07, both engines in one process, one binary: for `tally :- Seed, [?n <-
// (acc::count) :from Ok]` with `Ok` derived, native gives `Tally` stratum 1 and the oracle
// gives 0. Gate: `kernel/tests/stratify_numbers.rs`. Every OTHER function in the list above
// was re-read against its oracle twin the same day and DOES mirror.
//
// THE FACTS AGREE — `probe_arc278_derived_exists_acc`, and the `accum-over-derived` grid axis
// at depth 9 with Clara live. The two engines reach that agreement by DIFFERENT routes: native
// stratifies, so the bag is closed before it is counted; the oracle counts early and supersedes
// (`fire-support-fixpoint`, `wat/rete/oracle/fire.wat`). Do not "fix" either side into the
// other — the divergence is in the numbers, and each side's correctness argument is its own.
```

## Edit 2 — `src/rete/kernel/stratify.rs`, above `native_stratify_sweep`

REPLACE these three lines:

```rust
/// One sweep over all rules' (produced, negated, consumed) triples, raising `type_strata` entries.
/// For each rule: `required = max(stratum[n]+1 for n in negated, default 0)`; for each produced
/// type `p`: `stratum[p] = max(stratum[p], required)`. Returns `true` iff any stratum rose.
/// Mirrors `stratify-sweep` (`wat/rete/oracle/stratify.wat`).
```

WITH:

```rust
/// One sweep over all rules' (produced, negated, consumed, bagged) views, raising `type_strata`.
/// For each rule, `required` is the max of THREE terms, not one:
///   * `stratum[n] + 1` for every negated `n`            (the oracle has this)
///   * `stratum[b] + 1` for every bagged `b` THIS SET DERIVES, else `+ 0`  (the oracle does NOT)
///   * `stratum[c]`, NO `+1`, for every positively consumed `c`  (the oracle has this)
/// then for each produced `p`: `stratum[p] = max(stratum[p], required)`. Only RAISED strata are
/// recorded, so an all-zero map comes back EMPTY. Returns `true` iff any stratum rose.
///
/// ⛔ DOES NOT MIRROR `stratify-sweep` — the second term is the whole divergence, and this
/// docstring used to state the first term ALONE as the formula while the code folded all three.
/// See the ⛔ block at the head of this file; driven, both engines, one process, native 1 /
/// oracle 0 (`kernel/tests/stratify_numbers.rs`).
```

## Edit 3 — `wat/rete/oracle/stratify.wat`, the `rule-consumes` header

This claim is **TRUE** and is NOT being corrected — native's `rule_consumes` does include both,
via `consume_types` (`stratify.rs:178-179`). It gets a pointer only. REPLACE:

```
;; accumulate :from ARE — lockstep with native `rule_consumes`. A `?n`
;; accumulate head is not a type.
```

WITH:

```
;; accumulate :from ARE — lockstep with native `rule_consumes`. A `?n`
;; accumulate head is not a type.
;;
;; ⛔ THAT SENTENCE IS TRUE AND IT IS NOT THE WHOLE STORY — it is why the divergence below
;; went unnoticed. Native ALSO routes those same `:exists`-inner / acc-`:from` types into a
;; SECOND list, `rule_bag_consumes`, and its sweep gives that list a +1 when the bagged type
;; is one THIS rule set derives (`src/rete/kernel/stratify.rs:221-227`). Here they are only
;; ever positive reads, and `req-pos` below is NOT +1. So the two engines number such a rule
;; differently — driven 2026-09-07, native 1 / oracle 0, gate
;; `src/rete/kernel/tests/stratify_numbers.rs`. The facts agree; the strata do not.
```

## Blast radius

Comments only, in two files. **No executable line changes.** `wat/` is `include_str!`'d, so this
still needs a rebuild and a full floor.

## STOP triggers

1. **If any REPLACE block above does not match the tree byte-for-byte** — STOP and report the
   actual text. Do not paraphrase a near-match into place.
2. **If you believe any replacement text is itself inaccurate** — STOP and say which clause and
   why. Shipping a corrected header that is also wrong is the worst outcome available here, and
   it is exactly what happened last time this prose was written.
3. Do not touch `stratify.rs:35` / `produced_type` — the DESIGN rows it as unproven, and it needs
   a drive, not an edit. Do not touch any ✅ row in the DESIGN's audit table.
4. No logic. If a diff line is not inside a comment, you have gone too far.
