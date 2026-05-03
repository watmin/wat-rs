# Arc 146 Slice 3 — SCORE

**Sweep:** sonnet, agent `ad5e08ad8ec157d9a`
**Wall clock:** ~23.6 minutes (1414s) — well under the 80-min
time-box; UNDER the 25-40 min Mode A predicted band.
**Output verified:** orchestrator independently re-ran all
baselines + workspace + clippy.

**Verdict:** **MODE A WITH HONEST SUBSTRATE COMPLETION.** 10/10
hard rows pass; 4/4 soft rows pass. Sonnet shipped 4 dispatch
migrations end-to-end + surfaced + fixed one substrate-completion
delta (stdlib_loaded test fixture missing dispatch registration —
slice 1/2 oversight).

The cascade closes 4 more chain links. After slice 3:
- 5/10 originally-violating primitives migrated (length + empty?
  + contains? + get + conj)
- All polymorphic primitives in the substrate now use Dispatch
- Slice 4 (5 alias migrations) closes the remaining 5 single-impl
  ops via arc 143's define-alias

## Hard scorecard (10/10 PASS)

| # | Criterion | Result |
|---|---|---|
| 1 | File diff scope | ✅ EDITS to `src/runtime.rs` (+731/-605) + `src/check.rs` (+531/-) + `wat/core.wat` (+33). NO new test files. NO arc 144 hardcoded_primitives test edits required (Q2 inheritance). |
| 2 | 10 per-Type impls | ✅ All 10: vector/hashmap/hashset for empty?; vector/hashmap-contains-key/hashset for contains?; vector/hashmap for get; vector/hashset for conj. Each with inner helper for substrate-impl fallback (slice 2 Delta 1 pattern). |
| 3 | 10 dispatch arms | ✅ All 10 added to dispatch_keyword_head. dispatch_substrate_impl extended with 10 new entries for vals.first()/vals.get(1) routing. |
| 4 | 10 TypeScheme registrations | ✅ All 10 in register_builtins, adjacent to slice 2's length block. Existing helpers reused (vec_of, hashmap_of, opt, etc.). |
| 5 | 4 dispatch declarations in wat/core.wat | ✅ All 4 with correct multi-arg pattern shapes. Notational gotcha: type-vars use `:T`/`:K` keyword form (not bare `T`); the BRIEF's bare-T was a notational gap that sonnet correctly corrected. |
| 6 | 4 sets of old machinery RETIRED | ✅ 16 retirement targets confirmed: 4 eval_* fns + 4 eval dispatch arms + 4 infer_* fns + 4 infer_list dispatch arms + 4 arc 144 fingerprints. |
| 7 | arc 144 hardcoded_primitives tests | ✅ 17/17 PASS without modification. Slice 2 Delta 4's `dispatch_to_signature_ast` synthesis path inheritance: signature-of for these primitives now returns the Dispatch's polymorphic-function shape, which the existing test assertions accept. |
| 8 | All baseline tests pass | ✅ wat_arc146_dispatch_mechanism 7/7; wat_arc144_lookup_form 9/9; special_forms 9/9; hardcoded_primitives 17/17; wat_arc143_lookup 11/11; manipulation 8/8; define_alias 3/3 (length canary stays green). |
| 9 | Workspace failure profile | ✅ UNCHANGED from post-slice-2 — only `deftest_wat_lru_test_lru_raw_send_no_recv` (CacheService.wat noise). |
| 10 | Honest report | ✅ ~500-word report with all required sections; Q1-Q3 decisions explicit; 3 honest deltas surfaced. |

## Soft scorecard (4/4 PASS)

| # | Criterion | Result |
|---|---|---|
| 11 | LOC budget (400-700) | ✅ Net +85 LOC after retirements (1421 inserts - 1336 deletes effectively across runtime+check). Excellent — retirements offset the new code. |
| 12 | Style consistency | ✅ Each migration mirrors slice 2's pattern; helpers reused; comments match arc 146 convention. |
| 13 | clippy clean | ✅ 40 → 40 warnings. |
| 14 | Audit-first discipline | ✅ Q1-Q3 decisions named with rationales; 3 honest deltas surfaced including the slice 1/2 substrate-completion gap (Delta 3). |

## The 3 honest deltas

### Delta 1 — wat/core.wat colon-correction (Q1 surface)

The BRIEF's example arm `((:Container<X> X) :impl)` used bare
`X` for the second-arg type position. Substrate's `parse_arm`
parses each pattern element as a TypeExpr; bare `X` parses as a
Symbol (runtime variable), not a TypeVar. Sonnet correctly
adapted to `((:Container<X> :X) :impl)` — type position requires
a type literal.

Foundation-honest: the parser's strictness here is correct.
Bare identifiers are runtime symbols; type positions need
keyword-prefix. Documenting in the SCORE so future briefs use
the right surface.

### Delta 2 — In-runtime.rs unit-test semantics update

5 in-source unit tests in `src/runtime.rs::tests` asserted
pre-arc-146 semantics that the dispatch corrects:
- `vec_contains_*` triplet (was: valid-index check) → now
  element-membership tests
- `keys_contents_match_map` → uses `(:contains? ks "alpha")`
  not `(:contains? ks 0)`
- `hashset_get_*` pair → restructured to use `:contains?` (per
  DESIGN audit table: HashSet's "get-by-equality" IS just
  contains?)
- `hashmap_get_requires_hashmap_arg` → expects `MalformedForm`
  (no-arm-match) instead of `TypeMismatch` (single-handler shape)

These updates align the unit tests with the post-migration
semantics. Honest scope correction.

### Delta 3 — Slice 1/2 substrate-completion: stdlib_loaded test fixture missing dispatch registration

The check-tests fixture `stdlib_loaded` didn't register stdlib
dispatches. Slice 2 didn't surface this because length isn't
called from any stdlib `.wat` file. Slice 3's `:wat::core::get`
IS called from `wat/console.wat:165`; without dispatch_registry
attached to the test fixture, console.wat fails to type-check
during stdlib bootstrap, breaking 12 check-tests.

Fix mirrors slice 2's freeze.rs step 4a (Delta 3): parse +
register `define-dispatch` forms before macro expansion;
attach the registry to both the macro_sym AND the returned
symbols. ~25 LOC.

Same shape as slice 2 Delta 3 (load-order distinction between
stdlib + user dispatches). Slice 1/2 oversight surfaced by
slice 3's first-call-from-stdlib usage. Now FIXED.

## Calibration record

- **Predicted Mode A (~60%)**: ACTUAL Mode A. Calibration
  matched.
- **Predicted runtime (25-40 min)**: ACTUAL ~23.6 min. UNDER
  band! The mechanism + slice 2's substrate completions made
  this slice MORE efficient than predicted.
- **Time-box (80 min)**: NOT triggered.
- **Predicted LOC (400-700)**: ACTUAL +85 net (offset by
  retirements). Under net but appropriately large gross
  (1421 inserts, 1336 deletes).
- **Predicted Mode B branches (Q1/Q2/Q3)**: Q1 SURFACED
  cleanly (parser strictness on type-vars); Q2+Q3 NOT HIT
  (worked first try).
- **Predicted Mode B-arc144-test (~10%)**: NOT HIT — test
  inheritance via slice 2 Delta 4 worked.

## Q1-Q3 decisions

- **Q1 (multi-arg pattern grammar)**: ACCEPTED — parser handles
  any number of pattern elements per arm. Notational gotcha: type-
  vars require keyword prefix (`:T`).
- **Q2 (get's per-arm return-type variance)**: WORKS unchanged.
  `infer_dispatch_call` correctly returns the matched arm's
  specific instantiated return type via `apply_subst`.
- **Q3 (mixed-verb impls)**: WORKS unchanged. `parse_arm`
  accepts any keyword as `impl_name`; no naming-uniformity
  enforcement.

All 3 questions resolved with the recommended defaults. No
slice 1 substrate gaps surfaced (Delta 3 was a TEST FIXTURE gap,
not a substrate-machinery gap).

## Discipline notes

- The cascade rhythm holds: slice 2 closed length; slice 3
  closes 4 more in one bundled sweep. Slice 4 closes the last 5
  via aliases. Slice 5 closes the arc.
- Each substrate-completion delta caught + fixed in same sweep
  prevents Mode B-cascade-into-multi-respawn (arc 143 slice 5b
  + arc 146 slice 2 + this slice — same pattern).
- The stdlib_loaded fixture gap (Delta 3) confirms the
  substrate-as-teacher pattern: stdlib usage of new dispatches
  (here, `:wat::core::get` from console.wat) surfaces gaps that
  user-only tests miss.

## Pivot signal analysis (arc 147)

Re-examining arc 147 conditional pivot signals:

- **"Check/runtime inconsistency arc 147 would have prevented":**
  NO. Delta 3 is a TEST FIXTURE gap, not registration drift.
- **"Migration slice hits half-completion bug class":** NO.
  Slice 3's migrations didn't hit the registration-drift class.
- **"Aggregate cost exceeds arc 147 investment":** NO.
  Slice 3 ran at 23.6 min (under predicted band). Subsequent
  slices benefit from slice 1/2/3's machinery completions; slice
  4 (alias-based) doesn't even use the same registration pattern.

**Verdict: NO PIVOT.** Arc 147 stays in planned slot (after
arc 146 closes).

## What this slice unblocks

- **Slice 4** — alias migrations for the 5 single-impl ops
  (assoc/dissoc/keys/values/concat). Different mechanism (arc
  143's define-alias); smaller scope. ~10-15 min predicted.
- **Slice 5** — closure paperwork.
- **Arc 144 slice 4** — verification simpler (all polymorphic
  primitives are dispatches; reflection uniform).
- **Arc 130 RELAND v2** — accessible after arc 146 closes.

User's finish line: **every defined symbol can be queried at
runtime.** Slice 3 closes 4 more of the 9 remaining. Slice 4
closes the last 5. Then arc 146 ships.

The mechanism is the proof; this slice is the rhythm at scale.
