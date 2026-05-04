# Arc 148 Slice 2 — SCORE

**Sweep:** sonnet, agent `a5519c7ae97b4e269`
**Wall clock:** ~45.7 min (2741s) — at the predicted upper bound
of the 30-45 min Mode A band; UNDER the 60-min time-box.
**Output verified:** orchestrator independently re-ran FM 9
baselines + spot-checked rename sites + identified the lone test
failure (pre-existing CacheService.wat noise).

**Verdict:** **MODE A CLEAN SHIP.** 10/10 hard rows pass; 4/4
soft rows pass. 3 honest deltas surfaced; all justified within
the rename's scope.

## Hard scorecard (10/10 PASS)

| # | Criterion | Result |
|---|---|---|
| 1 | File diff scope | ✅ EDITS to `src/runtime.rs` + `src/check.rs` + `src/freeze.rs` + `src/macros.rs` + `src/resolve.rs` + `src/string_ops.rs` + 18 `tests/wat_*.rs` + 4 `wat/` and `wat-tests/` files + 3 `crates/{wat-lru,wat-holon-lru}/wat.../` files. NO new files. NO eval_* body changes. |
| 2 | 8 renames performed | ✅ All 8 verified: `:wat::core::{i64,f64}::{+,-,*,/}` → add `,2` suffix. `:wat::core::i64::+,2` confirmed at `src/runtime.rs:2514`; matching arms for `-`/`*`/`/` and the f64 family. |
| 3 | TypeScheme registrations updated | ✅ Per sonnet's report; relocated to use new names. Spot-checked via grep — old per-Type bare-name registrations absent. |
| 4 | Freeze pipeline pure-redex list updated | ✅ Line 15618 has `:wat::core::i64::+,2`; remaining 7 names follow the same pattern. |
| 5 | Call-site sweep complete | ✅ Independent grep: 100 occurrences of `:wat::core::i64::+,2` in `src/`; bare `:wat::core::i64::+` returns empty from `src/runtime.rs` registration sites. |
| 6 | All baseline tests still green | ✅ `wat_arc146_dispatch_mechanism` 7/7; `wat_arc144_lookup_form` 9/9; `wat_arc144_special_forms` 9/9; `wat_arc144_hardcoded_primitives` 17/17; `wat_arc143_define_alias` 3/3 (45/45 total). |
| 7 | Full workspace `cargo test` passes | ✅ Single-threaded canonical view: only `deftest_wat_lru_test_lru_raw_send_no_recv` (CacheService.wat noise — pre-existing arc 130 issue per arc 146 SCORE-SLICE-4 row 9) fails. Same failure profile as pre-slice baseline. |
| 8 | No new clippy warnings | ✅ Per sonnet: 33 warnings, all pre-existing (function-arg-count, doc-comment-spacing, etc.); none related to slice. |
| 9 | Workspace failure profile unchanged | ✅ Pre-slice + post-slice both: only documented CacheService.wat noise. |
| 10 | Honest report | ✅ Sonnet's report covers all required sections; 3 honest deltas explicitly surfaced. |

## Soft scorecard (4/4 PASS)

| # | Criterion | Result |
|---|---|---|
| 11 | LOC budget (50-300) | ⚠️ 245 insertions (high end of band; reflects per-file pervasiveness — most of the lift was test/wat call sites which the brief anticipated). UNDER the 500 LOC re-evaluate threshold. |
| 12 | Style consistency | ✅ Exact char-substitutions; no incidental refactoring per sonnet's report. |
| 13 | No phantom citations | ✅ Independent verification of registration line numbers + freeze pipeline line; sonnet's claims hold. |
| 14 | Audit-first discipline | ✅ 3 honest deltas surfaced (see below) — none of them invented workarounds; each disambiguated cleanly within scope. |

## The 3 honest deltas (sonnet)

### Delta 1 — Embedded crypto signatures regenerated

Two source-string-as-data tests (`src/runtime.rs:18762-18791` and
`wat-tests/holon/eval-coincident.wat:115-117,168-169`) embed
SHA-256 + Ed25519 constants computed over the literal source
string `"(:wat::core::i64::+ 2 2)"`. After the rename, sonnet
recomputed:

- SHA-256 of `(:wat::core::i64::+,2 2 2)` → `d4e368d75d1972482ae02398a37cef9fed68d2cb572f2354e31930b07ebb37cc`
- SHA-256 of `(:wat::core::i64::*,2 1 4)` → `03e5d2e5386ae6a04a279ad2c3bef2d2c6b6bca0bac25e3f902b68764a5a0484`
- Ed25519 sig (src-A): `3bQjvWistCp2jyK0AU6+9ZQZp/yMk2gB/ycbjIOGpFd3FBIwGaa/TqsHV4Elb4P0HxBo6eSr0q3qwZ8xaKOgBw==`
- Ed25519 sig (src-B): `OrYNwvRnWgytoHL77zLAB8EQItkav/KnUTpmacu9AuxS8LKu4Fjda9dvgc5ruNq5Fc8GB52v+/BGew7rxxiXCw==`

This is **the substrate-as-teacher discipline working as designed**:
the test suite caught a non-obvious source-string dependency that
purely-mechanical rename would miss. Sonnet handled it within the
slice rather than punting.

### Delta 2 — 5 prose-glob doc comments rewritten

Doc comments using `:wat::core::f64::*` to mean "the f64 family
namespace" were ambiguous post-rename (the `*` glob collides with
the new multiply-arith leaf name `:wat::core::f64::*,2`). Sonnet
rewrote 5 prose comments to use unambiguous phrasing
(`:wat::core::f64 namespace` instead of `:wat::core::f64::*`).

Conscious disambiguation; no semantic change.

### Delta 3 — Edit tool whitespace-strip workaround (workflow note)

Sonnet's tool stripped trailing whitespace from `old_string`/
`new_string`, so a naive replace_all of `:wat::core::i64::+ ` (with
trailing space) → `:wat::core::i64::+,2 ` (with trailing space)
silently dropped the trailing space, fusing operator and following
arg. Workaround was a two-pass per file:
1. First pass: `:wat::core::OP` → `:wat::core::OP,2` (drop the space)
2. Second pass: surgical `:wat::core::OP,2X` → `:wat::core::OP,2 X`
   for each follow-character variant

Not a defect; just a footnote on tool behavior. **Future-self note:**
if a future sweep involves names with trailing structural
characters, prefer multi-line context in `old_string` over relying
on trailing whitespace.

## Calibration record

- **Predicted Mode A (~75%)**: ACTUAL Mode A. Calibration matched.
- **Predicted runtime (30-45 min)**: ACTUAL ~45.7 min. AT the upper
  bound; not over the time-box but on the edge. The crypto-signature
  regeneration in Delta 1 + the prose disambiguation in Delta 2 +
  the two-pass Edit-tool workaround in Delta 3 all consumed time
  the brief didn't anticipate. For future rename slices: budget
  ~10 min for unanticipated source-string dependencies.
- **Time-box (60 min)**: NOT triggered. Used 76% of cap.
- **Predicted LOC (50-300)**: ACTUAL 245 (high end). Honest scope
  given the per-file pervasiveness; arc 146's slice 2 had similar
  scope distribution.
- **Honest deltas (predicted 0-1; actual 3)**: surfaced more than
  predicted. All within scope; all reflective of substrate-as-teacher
  catches. Healthy outcome.

## Workspace failure profile (pre/post slice)

- **Pre-slice baseline:** `deftest_wat_lru_test_lru_raw_send_no_recv`
  fails (CacheService.wat — pre-existing arc 130 noise documented
  in arc 146 SCORE-SLICE-4 row 9). ALL OTHER tests green.
- **Post-slice (single-threaded):** SAME — only the CacheService.wat
  noise. Identical failure profile.
- **Post-slice (multi-threaded):** additional concurrency flakes
  vary run-to-run per sonnet's report; pre-existing test-interleaving
  issues with LRU/telemetry tests; NOT introduced by this slice.
  Single-threaded is the deterministic canonical view.

## What this slice closes

- Per-Type arithmetic leaves at bare names (`:wat::core::i64::+`
  etc.) → renamed to `,2`-suffixed form
- Bare names FREED for slice 4 to populate with variadic wat fn
  wrappers
- Naming convention uniform: `,2` is the binary form across
  arithmetic per-Type + polymorphic dispatch (slice 4 ships the
  polymorphic Dispatch entity at `:wat::core::+,2`, etc.)

## What this slice unlocks

- **Slice 4** — numeric arithmetic migration (the bare names are
  now available for variadic wat function wrappers; `(:wat::core::i64::+
  1 2 3 4 5) => :i64 15` UX ships there)
- **Slice 3** — values_compare ord buildout — INDEPENDENT of
  slice 2; can spawn in parallel or sequential (your call)

## Pivot signal analysis

NO PIVOT. The 3 honest deltas are all within-scope substrate-as-
teacher catches, not architectural surprises. Slice 4's brief can
proceed against the locked DESIGN without revision.

The crypto-signature regeneration (Delta 1) is a meta-signal worth
preserving for future rename slices: ALWAYS verify what tests
embed source strings as data BEFORE shipping a rename. Sonnet
caught it via test failure; orchestrator should anticipate it in
brief writing.

The methodology IS the proof. The rhythm held.
