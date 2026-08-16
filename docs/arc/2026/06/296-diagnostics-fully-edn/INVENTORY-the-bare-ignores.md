# Inventory — the bare `#[ignore]` sites

Rider job: per-ignore inventory of bare `#[ignore]` attributes (no reason string) under `wat-rs/tests/`.

## ⛔ Headline: the brief's count does not match the disk. The real total is 7, not 51.

The brief stated "91 `#[ignore]` attributes in `tests/`, 10 carrying `296-recapture-pending`, 51 of the
remaining 81 bare." I counted every line matching the substring `#[ignore` in `tests/` (excluding
`.claude/worktrees/`) and got **81** — which is exactly the brief's "remaining 81" figure. That number
is where the brief's census actually came from. But **81 is a substring count, not an attribute count.**
44 of those 81 lines are prose — module `//!` headers and `//`/`///` comments that *talk about*
`#[ignore]` (e.g. `` `#[ignore]`'d STRIKE-READY until then ``, `` /// `#[ignore]` — process-tier probe ``)
— not the attribute itself. Filtering to lines where `#[ignore` actually starts a Rust attribute
(`^\s*#\[ignore\b`, i.e. not inside a `//`/`///`/`//!` comment) gives **37 real `#[ignore]` attributes**
in `tests/`: **30 carry a reason string, 7 are bare.**

The 51 in the brief is reproduced exactly by `7 real bare attributes + 44 comment-prose mentions = 51`.
That is a classic grep-is-not-a-census miscount: a search for `#[ignore]` with no trailing `=` matches
both the real bare attribute *and* every comment sentence that happens to mention `` `#[ignore]` `` in
passing, because a bare `#[ignore]` and a comment fragment like `` `#[ignore]`'d `` are textually
indistinguishable to a line-oriented grep. I verified this by hand: `grep -c` on
`^\s*#\[ignore\b` (attribute-position only) vs plain substring grep, cross-checked against the
`#\[ignore\s*=` (reasoned) count, and against reading both files below in full.

**Also worth noting: `296-recapture-pending` does not appear anywhere in `tests/` right now** (0 hits) —
either the other rider already finished converting/removing that cohort, or the string never existed in
that literal form in this tree. Not this job's concern; noted for the record.

So: this document inventories **all 7 real bare `#[ignore]` attributes** that exist in `tests/` today —
which is the complete population, not a sample.

## The 7, in full

All 7 live in two files, `tests/rete/probe_arc278_3a_root_join.rs` (3) and
`tests/rete/probe_arc278_3b_hash_join.rs` (4) — coincidentally the exact same two files the brief used
as its own worked RELOCATED example. Neither file is touched by the other rider working
`tests/reflection`, `tests/value`, `tests/services`, `tests/comms` — no coordination hazard.

Every one of the 7 carries an adjacent `//` comment (not a module `//!` header) naming the replacement
test(s) in `src/rete/kernel.rs`. I verified each named replacement function exists, is a live `#[test]`
(not itself ignored), and cross-references back to the exact probe file/function it replaces via its own
doc comment. All 7 are the same class: **RELOCATED**.

| # | file | test fn | recovered reason (verbatim, adjacent `//` comment) | class | proposed `#[ignore = "..."]` | UNRECOVERABLE? |
|---|------|---------|----------------------------------------------------|-------|-------------------------------|-----------------|
| 1 | `tests/rete/probe_arc278_3a_root_join.rs:24-26` | `root_join_populates_one_beta_node` | "P11: beta is ephemeral by design; a fired Session no longer retains beta-memory — provenance regenerates on re-fire. Join-correctness coverage relocated to: src/rete/kernel.rs #[cfg(test)]::root_join_seeds_one_token_per_element" | RELOCATED | `"RELOCATED (P11): beta-memory is ephemeral by design (a fired Session no longer retains it); coverage moved to src/rete/kernel.rs::root_join_seeds_one_token_per_element (verified present, live #[test]). unlock: candidate for deletion — builder rules"` | No |
| 2 | `tests/rete/probe_arc278_3a_root_join.rs:34-36` | `root_join_seeds_one_token` | same P11 comment, same target `root_join_seeds_one_token_per_element` | RELOCATED | `"RELOCATED (P11): beta-memory is ephemeral by design; coverage moved to src/rete/kernel.rs::root_join_seeds_one_token_per_element (verified present, live #[test]). unlock: candidate for deletion — builder rules"` | No |
| 3 | `tests/rete/probe_arc278_3a_root_join.rs:44-46` | `seeded_token_carries_bindings_and_support` | same P11 comment, same target `root_join_seeds_one_token_per_element` | RELOCATED | `"RELOCATED (P11): beta-memory is ephemeral by design; coverage moved to src/rete/kernel.rs::root_join_seeds_one_token_per_element (verified present, live #[test]). unlock: candidate for deletion — builder rules"` | No |
| 4 | `tests/rete/probe_arc278_3b_hash_join.rs:26-27` | `join_produces_one_token_on_matching_loc` | "P11: beta is ephemeral by design; ... Join-correctness coverage relocated to: src/rete/kernel.rs #[cfg(test)]::hash_join_produces_one_token_on_same_loc / ::hash_join_drops_on_mismatched_loc / ::hash_join_no_cross_loc_leakage" (this test's own comment block lists all three kernel.rs targets) | RELOCATED | `"RELOCATED (P11): beta-memory is ephemeral by design; coverage moved to src/rete/kernel.rs::hash_join_produces_one_token_on_same_loc (verified present, live #[test]). unlock: candidate for deletion — builder rules"` | No |
| 5 | `tests/rete/probe_arc278_3b_hash_join.rs:36-37` | `joined_token_unifies_both_conditions` | "P11: beta is ephemeral by design; ... Join-correctness coverage relocated to: src/rete/kernel.rs #[cfg(test)]::hash_join_produces_one_token_on_same_loc" | RELOCATED | `"RELOCATED (P11): beta-memory is ephemeral by design; coverage moved to src/rete/kernel.rs::hash_join_produces_one_token_on_same_loc (verified present, live #[test]). unlock: candidate for deletion — builder rules"` | No |
| 6 | `tests/rete/probe_arc278_3b_hash_join.rs:50-51` | `join_drops_on_mismatched_loc` | "P11: beta is ephemeral by design; ... Join-correctness coverage relocated to: src/rete/kernel.rs #[cfg(test)]::hash_join_drops_on_mismatched_loc" | RELOCATED | `"RELOCATED (P11): beta-memory is ephemeral by design; coverage moved to src/rete/kernel.rs::hash_join_drops_on_mismatched_loc (verified present, live #[test]). unlock: candidate for deletion — builder rules"` | No |
| 7 | `tests/rete/probe_arc278_3b_hash_join.rs:63-64` | `join_no_cross_loc_leakage` | "HAZARD #1 — cross-product leakage... P11: beta is ephemeral by design; ... Join-correctness coverage relocated to: src/rete/kernel.rs #[cfg(test)]::hash_join_no_cross_loc_leakage" | RELOCATED | `"RELOCATED (P11): beta-memory is ephemeral by design; coverage moved to src/rete/kernel.rs::hash_join_no_cross_loc_leakage (verified present, live #[test]). unlock: candidate for deletion — builder rules"` | No |

### Verification of the RELOCATED targets (done, not left as an assumption)

```
$ grep -n "fn root_join_seeds_one_token_per_element"        src/rete/kernel.rs   → 4132: found, live #[test]
$ grep -n "fn hash_join_produces_one_token_on_same_loc"      src/rete/kernel.rs   → 4220: found, live #[test]
$ grep -n "fn hash_join_drops_on_mismatched_loc"             src/rete/kernel.rs   → 4306: found, live #[test]
$ grep -n "fn hash_join_no_cross_loc_leakage"                src/rete/kernel.rs   → 4379: found, live #[test]
```

Each of those four `src/rete/kernel.rs` functions carries its own doc comment pointing back at the exact
probe file/function it mirrors or supersedes (e.g. `root_join_seeds_one_token_per_element`'s doc lists
all three `probe_arc278_3a_root_join.rs` function names it covers; `hash_join_produces_one_token_on_same_loc`'s
doc lists both `join_produces_one_token_on_matching_loc` and `joined_token_unifies_both_conditions`). The
pointer is bidirectional and none of the four named replacements is missing — no "relocation pointer to a
test that is gone" finding here.

## Class partition

| class | count |
|-------|-------|
| RELOCATED | 7 |
| EXECUTION-CONSTRAINT | 0 |
| RED-AT-HEAD | 0 |
| MINT-CONFIRMER | 0 |
| UNKNOWN | 0 |
| **total** | **7** |

No new class was needed — all 7 real bare ignores are the same RELOCATED shape, and I did not have to
invent or stretch a category to fit them. **UNKNOWN count: 0.** Every bare ignore that exists has a
recoverable, verified reason.

## Disposition (per the rules — this is an inventory, not a cleanup)

All 7 rows are candidates for deletion in the RELOCATED sense (their coverage lives on, verified, at a
named `src/rete/kernel.rs` site) — but per the job's rules, **the verdict is "candidate for deletion —
the builder rules," not deletion.** Nothing was deleted, un-ignored, run, or edited. The only write this
job made is this document.

## What surprised me / where the framing didn't match the disk

- **The order-of-magnitude gap is the whole finding.** The brief anticipated needing to correct the
  count ("if the disk says otherwise, report the real number") but the actual gap — 51 claimed vs. 7
  real — is large enough that I re-derived it twice by different methods (attribute-position anchor vs.
  reasoned/bare split arithmetic) before trusting it, and then confirmed the arithmetic that produces
  "51" from a comment-blind substring grep (`7 + 44 = 51`) exactly. That reproduction is why I'm
  confident this isn't a second miscount on my part.
- **The "load-bearing count" (UNKNOWN) is zero**, not because I was lenient about what counts as
  recoverable, but because the entire bare population turned out to be two files' worth of a single,
  well-documented, already-migrated pattern. There was no ambiguous case to adjudicate.
- **No new class was needed** — the sample vocabulary of four (RELOCATED / EXECUTION-CONSTRAINT /
  RED-AT-HEAD / MINT-CONFIRMER) covered the real population with room to spare; three of the four classes
  have zero members here.
- The brief's own worked RELOCATED example (`probe_arc278_3b_hash_join.rs`, "has 4") turned out to BE
  4 of the actual 7-item population, not an illustrative sample of a much larger set — the brief's own
  example was, in effect, more than half the real answer.
- I did not need git history to resolve any UNRECOVERABLE case, since there were none — `git log
  --oneline -- tests/rete/probe_arc278_3a_root_join.rs tests/rete/probe_arc278_3b_hash_join.rs` was
  checked anyway for context (most recent relevant commits: `91bbb8cd`, `952ece8b`, `967aa344`), but
  the in-file comments were already sufficient and verified.
