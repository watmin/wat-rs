# INTUERI — Cast Report (TARGET 4 — the test corpus) — **CLEAN**

> Written verbatim as returned, before any synthesis. Evidence only — this file carries NO status.

### What I swept

**Re-derived every handed-down number independently, from disk, this session:**

```
find tests/rete -name '*.rs' | wc -l      → 100
find tests/rete -name '*.wat' | wc -l     → 144
find tests/rete -name '*.edn' | wc -l     → 23
find tests/rete -name '*.wat.bad' | wc -l → 19
find src/rete/kernel/tests -name '*.rs'   → 20
                                    total → 306 files ✓
find ... -type f | xargs wc -l            → 38058 lines ✓
```

**The 613 test count**, independently re-derived (not trusted from the brief): `grep -rn '#\[test\]'` over both trees gives **615**. Subtracting lines whose `#[test]` sits inside a `//` comment — exactly two, both in prose (`tests/rete/probe_arc278_P4c_native_retraction.rs:18` and `probe_arc278_P4a_native_fire_rules.rs:16`, both reading "every scenario a `#[test]` needs...") — gives **613**. This matches the two commented-out mentions the brief said two of the four prior derivations found.

I then wrote a Python extractor to pull every real test-fn *name* attached to each of the 613 `#[test]` attributes (handling same-line `#[test] fn x() { ... }` forms in `probe_arc278_seq1b_list_hofs.rs`, and `//`-comment lines sitting between `#[test]` and `fn` in `probe_arc278_export.rs` — **both bugs I hit and fixed in the extractor itself**, re-verifying the total stayed at 613 after each fix).

**Sampling rule** (breadth over depth, reproducible): a mechanical grep over the 613 names for conjunction/quantifier/claim vocabulary — `_and_`, `every`, `only`, `exactly`, `never`, `always`, `all_`/`_all`, `proves`, `confirms`, `guarantees`, `verifies`, `_each_`, `_both_`, `complete`, `entire`, `fully`, `_per_`, `names_the`, `because`, `blames` — the shapes the assignment named as the highest-risk pattern for the 4L1 defect. This yielded **99 candidates**. I read every one of those 99 function bodies in full (brace-matched extraction, not truncated), plus a separate random sample of 40 names drawn independently (`shuf`), for **128 unique test bodies read against their names** — **20.9% of the 613**, weighted toward the riskiest shape.

### The name-vs-body answer: NOT a class — it stayed isolated

Of the 128 bodies read, **exactly one** delivers less than its name promises: the already-rowed `render_phase_table_proves_missing_phase_and_zero_total` at `src/rete/kernel/tests/rank_and_instrument.rs:261` (`4L1`, not re-reported per instruction).

Every other conjunction/quantifier name I checked delivers on **both** halves of its promise. This corpus turns out to write `_and_`/`per`/`every` with real discipline — a sample of the pattern, confirmed by re-reading:

- `arm_lease.rs:11 fire_rules_reuses_arm_across_fire_and_insert_overlay` — asserts no-rebuild across a second `fire` (`assert_eq!(after_second, builds_after_first, ...)`) **and** across an insert overlay (`assert_eq!(after_overlay, builds_after_first, ...)`). Both delivered.
- `fanout_cost.rs:191 fanout_rhs_key_alloc_census` and `:250 fanout_per_call_alpha_census` — both carry explicit non-vacuity guards (`prod:derivations == 40_000`) precisely so a dead-fire zero cannot masquerade as the zero the test claims.
- The five `probe_fence_names_the_head.rs` / `then_user_forms.rs` tests named `*_names_the_offending_head_and_axis` — each asserts a full-string `assert_eq!` on the compile error, e.g. `"...expr is not pure — ':wat::io::IOReader/open-file' is not pure"` — which literally contains both the offending head *and* the axis word (`pure`/`total`/`deterministic`) in one check. Both conjuncts delivered by one assertion, honestly.
- The four `probe_arc278_field_span.rs` tests named `*_names_the_field_keyword_not_the_whole_X` — each uses `assert_edn_matches_file!` against a pinned golden carrying the exact `:col`/`:end` span, which is a stronger proof of "names the keyword, not the whole form" than a substring check would be.
- `support_index_has_an_entry_per_derived_fact` (`probe_arc278_P12a_explain_substrate.rs:32`) — asserts the index length equals the derived-fact count exactly.
- `leading_exists_passes_its_token_once_per_fire_not_once_per_round` / `leading_not_...` (`probe_arc278_leading_filter_multiplicity.rs:57,74`) — each checks both round-lengths (2-round and 6-round chains) equal 1, which is exactly "once per fire" vs "once per round".
- `the_banked_d5_repro_pair_both_load` (`probe_arc278_match_arm_is_not_a_call.rs:195`) — asserts `then_ok` and separately `when_ok` — both halves of "both load" checked independently, not inferred from one.

One false-positive worth naming as a caught trap (matches prior-art trap #1, "prose about the thing matched a grep for the thing" — mine, on my own gate): `all_folds` (`tests/rete/probe_arc278_8i_accumulator_folds.rs:61`) matched my quantifier grep for `all`, but reading the file showed `all` there is the literal name of the `:wat::rete::acc::all` accumulator being tested — a domain noun, not an "every-test" quantifier. Correctly ruled out by reading, not reported.

**This is a real negative result, not an absence of looking**: 128/613 names (20.9%) checked against their bodies, weighted at essentially 100% coverage of the conjunction/quantifier vocabulary (99/99 matches read), turned up zero new instances of the name-promises-more-than-body-delivers shape. `4L1` looks like an isolated lapse in an otherwise disciplined corpus, not a class.

### Other things I looked for and did not find

- **`rune:intueri`**: 0 hits, confirms the brief's claim. Nothing needed a rune I found missing.
- **Mumbling structure**: no `utils.rs`/`helpers.rs`/`common.rs`/`misc.rs` anywhere in either tree.
- **Bare 1–2 letter function names** outside tight loops: found exactly three — `ev` (`pass_semantics.rs:11`), `kw` (`probe_arc278_export.rs:709`), `el` (`right_index_counter_invariant.rs:590`) — all private, file-local s-expression/value constructors (eval/keyword/element), called dozens of times a few lines from their definition. Judged acceptable under the spell's own scope-discipline rule and not flagged.
- **Stale/WHAT-only comments**: none encountered across the ~130 bodies read — the corpus's comments are almost entirely WHY (design rationale, "measured on 2026-08-30", explicit non-vacuity reasoning). Not exhaustively swept (6,430 comment lines is out of reach for one pass), but zero hits in a substantial, representative sample.

### Findings

None. No `file:line` in this target carries a name that lies (Level 1) or mumbles (Level 2) beyond the already-rowed `4L1`.

### Boundary respected

I did not flag anything belonging to `solvere` (braiding), `purgare` (dead code), `perspicere` (deep types), or `vocare` (test vantage) — several candidates I read already carry runes or prior findings from those wards; I left them to their owners.

**CLEAN**
