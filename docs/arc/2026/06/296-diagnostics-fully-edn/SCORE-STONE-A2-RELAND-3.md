# SCORE — STONE A-2 RELAND-3: the class, not the sites

## Start-here restoration

`git log --oneline -12` found the two WIP save-points: `60813552a` ("the join landed — PRESERVED")
and `12985e9f8` ("three mechanisms closed, ② fenced — PRESERVED"). `12985e9f8` is the later, full
one (confirmed via `git show --stat`: 6 files, 903 insertions — a superset of `60813552a`'s join).
Restored its `src/` state onto the current tree:

```
git checkout 12985e9f8 -- src/types.rs src/declare/register.rs src/check.rs src/freeze/env.rs \
  tests/types/probe_arc296_a2_a_variant_is_a_type.rs
```

`cargo build --release` — clean, before touching anything further.
`cargo nextest run --release -E 'test(a2_a_variant)'` — 11/11 PASS at the restored baseline (already
green; RELAND-2 had already un-ignored all 11 rows).

## ① The census — the stone's real deliverable

Full table: `docs/arc/2026/06/296-diagnostics-fully-edn/TABLE-STONE-A2-RELAND-3-intrinsic-parameter-checks.md`.

**72 `fn infer_*` functions censused** (grep count; the 73rd "`infer_*`" match in the brief's
estimate is a `#[test] fn infer_rete_form_names_...` sharing the prefix, excluded). Of those, **3
carry the bare-`unify`-instead-of-`assignable` defect class**:

1. `infer_ordering` (`<`/`>`/`<=`/`>=`) — **fixed this stone**, the brief's named ②.
2. `infer_hashset_constructor` — **found, confirmed by probe, NOT fixed** (out of this stone's
   prescribed scope).
3. `infer_hashmap_constructor` — **found, confirmed by probe, NOT fixed** (ditto).

**STOP-1 does not fire** — 3 is well under the ~15 threshold, so this stays a stone. Per the brief's
own instruction ("closes the comparison family's TWO sub-defects" — a named, scoped fix, not "fix
everything the census turns up"), items 2-3 are reported as findings for the orchestrator to shape
into their own stone, not fixed here. They are a genuine, probe-confirmed instance of the identical
class RELAND-2 and this stone both named — `infer_persistentmap_constructor` /
`infer_persistentvector_constructor` already encode the correct fix as a live precedent
(`if declared.is_some() { assignable(...) } else { unify(...) }`), so the follow-up stone has a
template, not a design question.

Also censused but explicitly NOT counted toward the 3 (see the table's Note column for why each is
out of frame): `infer_equality`'s own latent Parametric-vs-Parametric gap in its subtype fallback
(operand-vs-operand, not arg-vs-declared-param — same shape-family as ②, un-named by any floor red);
`infer_form_matches`'s `check_comparison` helper (a RETE clause's `(= field lit)`, same
operand-vs-operand shape, not itself one of the 72 `infer_*` functions); `infer_defclause`'s five
`unify` calls (all body-vs-own-declared-return-type or ensure-fn-annotation-vs-declared-return-type
— declaration-vs-declaration, not call-site argument-vs-param).

## ② The comparison family — both sub-defects, `src/check.rs` (now ~13897, `infer_ordering`)

**Sub-defect 1** — the operand comparison called `unify(&a_resolved, &b_resolved, …)` directly,
which is exact on the head and rejects both a bare enum against its own variant (`Option` vs
`Option::Some`) and two sibling variants of the same enum (`Result::Ok` vs `Result::Err`). Fixed by
widening EACH side independently to its own enclosing enum first
(`widen_to_enclosing_enum` — already established by mechanism ③, RELAND-2) before the compatibility
`unify`. Widening only ever changes the head, never the argument `Vec`, so it binds the same type
variables `a_resolved`/`b_resolved` already carry; a non-enum type (`i64`, `f64`) widens to itself
(no-op), so the `both_numeric` cross-type exception is byte-for-byte unaffected.

**Sub-defect 2** — `is_type_orderable` listed `(Option :- [T])` and `(Result :- [T E])` but not a
variant of one. Fixed by asking `TypeEnv::enclosing_enum` (the SAME primitive `widen_to_enclosing_
enum`/`join_types` already consult — never a hand-listed set of variant names, satisfying STOP-5):
when the resolved type's head names a registered variant whose parent differs from itself, widen to
the parent and recurse. A no-op for anything that isn't a variant.

After both fixes, the orderable gate checks BOTH operands independently (mirroring `infer_equality`'s
own existing two-sided `is_type_equatable(&a,...) || is_type_equatable(&b,...)` check, its sibling in
this same file) rather than a single merged "unified" value — a sibling-variant pair keeps its own
head; args are never merged across operands (an `Ok`'s own unbound `E`-var must never be forced to
agree with `Err`'s declared payload type; only the compatibility `unify` above binds across sides).

## ③ Golden recapture — `probe_arc278_journal_surface.rs`

`tests/services/probe_arc278_journal_surface.wat.bad` still fails, correctly — three genuine
`TypeMismatch`s. The assertion in `tests/services/probe_arc278_journal_surface.rs`
(`wrong_response_type_at_reply_site_is_compile_error`) pinned the bare-enum spelling
`:wat::query::Store::PutResponse`; the restored join now reports the more precise variant
`:wat::query::Store::PutResponse::Success`. RECAPTURED, kept pinning (never normalised) per the
seam's standing ruling.

## ⛔ Mechanism ② (nested widening) — confirmed STILL FENCED, STOP-3 did not fire

`tests/wat_lang/wat_core_try.wat`'s `:t::app-describe` (line 38) is the concrete carrier of the
brief's fenced shape — a user `defn` whose declared param is
`(Option :- [(Result :- [i64 String])])`, called at line 47 with an argument whose inferred type is
`(Option::Some :- [(Result::Err :- [:?N String])])`. This routes through `assignable` (a user defn's
own door, untouched by this stone), whose per-argument check is deliberately invariant `unify` one
level down — exactly the fenced question. Re-ran after both fixes landed:

```
cargo nextest run --release -E 'binary_id(=wat::wat_lang) and test(wat_core_try)'
```
```
FAIL [0.356s] (13/13) wat::wat_lang wat_core_try::try_inside_match_arm_propagates
  thread '...' panicked at src/freeze.rs:1162:9:
  call_beside_value: fixture beside ".../tests/wat_lang/wat_core_try.rs" failed to freeze:
  #wat.check/CheckErrors {:message "1 type-check error" ... :errors [#wat.check/TypeMismatch
  {:message ":t::app-describe: parameter #1 expects (:wat::core::Option :- [(:wat::core::Result
  :- [:wat::core::i64 :wat::core::String])]); got (:wat::core::Option::Some :- [(:wat::core::Result::Err
  :- [:?2730 :wat::core::String])])" ... :callee ":t::app-describe" :param "#1" ...}]}
Summary [0.357s] 13 tests run: 6 passed, 7 failed, 241 skipped
```

Byte-for-byte the same shape as before this stone's edits, still 7 failed (the whole
`wat_core_try.wat` fixture fails to freeze, taking down all 7 tests that call `run_expr`, which is
why the WIP commit classified this cluster as "7"). Neither `infer_ordering`'s widen-then-unify fix
nor `is_type_orderable`'s variant-widening fix touches `assignable` or argument-position invariance
at all — confirmed by inspection (no edits outside `infer_ordering`/`is_type_orderable`) and by this
re-run. **STOP-3 did not fire.**

## Acceptance rows

| Row | Result |
|---|---|
| `cargo nextest run --release -E 'test(a2_a_variant)'` | **11 passed, 0 skipped** |
| `test(p1_annotation) or test(p1b_a_parametric) or test(p2prereq) or test(p3_one_question) or test(a1_one_rule)` | **38 passed** (10+4+4+5+4+11, all controls in one sweep) |
| `binary_id(=wat::types) and test(wat_arc148_ord_buildout)` | **46 passed, 0 failed** (was 34/46 — the 12 named "comparison family" failures now pass) |
| `test(probe_arc278_journal_surface)` (`--test-threads=1`) | **2 passed** (golden recapture holds) |
| `binary_id(=wat::services)` | **133 passed, 2 skipped** |
| `binary_id(=wat::types) or binary_id(=wat::comms) or binary_id(=wat::process) or binary_id(=wat::channel) or binary_id(=wat::function) or binary_id(=wat::program)` | **1009 passed, 2 failed** (down from RELAND-2's 997/1011 — the 12 ord failures are gone; the remaining 2 are RELAND-2's own Clusters B/C, confirmed pre-existing, unrelated to this stone) |
| `test(wat_scripts_fixes_load)` | **PASS** (159s, whole `wat-scripts/` corpus) |
| `test(every_ungated_wat_checks)` | **PASS** |
| `test(hibernate)` (`--test-threads=1`) | **3 passed** |
| `binary_id(=wat::wat_lang) and test(wat_core_try)` | **6 passed, 7 failed** — the 7 are mechanism ②, confirmed unchanged/still-fenced (verbatim above) |

## Classification of everything else touched, by reason

**Cluster B (RELAND-2's own, unchanged)** — `wat::function recursive_patterns::nested_options_three_levels`:
a nested-Option `match` scrutinee stays narrowed before an inner `match`'s pattern-shape checker,
which refuses `Some`/`None` patterns against the narrow variant. Confirmed still failing, same shape,
after this stone's edits — a `match`-side gap outside this stone's four/two named mechanisms.

**Cluster C (RELAND-2's own, unchanged)** — `wat::types
probe_arc296_p2a_a_monomorphic_variant_is_a_type::a_generic_enums_variant_stays_refused`: a
superseded P-2a fixture asserting `code == 1` (refused) where RELAND-1's join now correctly accepts
(`code == 0`). Stale relative to RELAND-1's supersession, not a regression from this stone.

Both confirmed via the same broad sweep above — 2 failures, byte-identical to RELAND-2's own final
report, neither introduced nor touched by `infer_ordering`/`is_type_orderable`'s edits.

## STOP triggers — dispositions

- **STOP-1** (census > ~15 needing change): did not fire — 3 found, well under threshold.
- **STOP-2** (join controls red): did not fire — all 38 controls green (see acceptance table).
- **STOP-3** (mechanism ② starts passing): did not fire — `wat_core_try`'s 7 failures are
  byte-for-byte unchanged, verbatim block above.
- **STOP-4** (scoping by namespace/prefix): not invoked — no scope cut anywhere in this stone's fix.
- **STOP-5** (hand-listing variant names to fix `is_type_orderable`): not invoked — the fix asks
  `TypeEnv::enclosing_enum`, the same structural primitive `widen_to_enclosing_enum`/`join_types`
  already consult; zero variant names appear in the new code.

## Files changed (working tree, uncommitted per tier)

- `src/check.rs` — `is_type_orderable` (new `types: &TypeEnv` param, variant-widening arm before the
  existing match) and `infer_ordering` (widen-then-unify compatibility gate; two-sided orderable
  check on both operands). Both changes scoped to these two functions/one helper signature; no other
  function touched.
- `tests/services/probe_arc278_journal_surface.rs` — golden recapture (`got` spelling only; `expected`
  and the test's intent unchanged).
- `docs/arc/2026/06/296-diagnostics-fully-edn/TABLE-STONE-A2-RELAND-3-intrinsic-parameter-checks.md`
  — new, the full census.
- `docs/arc/2026/06/296-diagnostics-fully-edn/SCORE-STONE-A2-RELAND-3.md` — this file.
- `src/types.rs`, `src/declare/register.rs`, `src/freeze/env.rs`,
  `tests/types/probe_arc296_a2_a_variant_is_a_type.rs` — restored verbatim from `12985e9f8` (RELAND-1
  join + RELAND-2 mechanisms ③/④), untouched beyond the restoration.

Per tier: `scripts/floor.sh`, an unfiltered `cargo nextest run`, and `clippy` were **not** run — the
orchestrator runs those centrally.
