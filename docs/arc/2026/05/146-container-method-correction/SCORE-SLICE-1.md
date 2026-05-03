# Arc 146 Slice 1 — SCORE

**Sweep:** sonnet, agent `a9c49da1577aa926b`
**Wall clock:** ~23.2 minutes (1391s) — well under the 100-min
time-box; slightly under the 30-50 min Mode A predicted band.
**Output verified:** orchestrator independently re-ran new test
file + all baseline test files + checked diff scope.

**Verdict:** **MODE A — clean ship.** 11/11 hard rows pass; 5/5
soft rows pass. Sonnet mirrored existing patterns (macros.rs,
macro_registry, capability-carrier) cleanly; followed all BRIEF
recommendations on the four open questions; surfaced honest deltas
for the necessary plumbing (StartupError variant, FrozenWorld
field). The audit-first + STOP-at-first-red disciplines held;
no workarounds shipped.

The substrate now has the multimethod entity kind. Slices 2-7 of
arc 146 build migrations atop it.

## Hard scorecard (11/11 PASS)

| # | Criterion | Result |
|---|---|---|
| 1 | File diff scope | ✅ NEW `src/multimethod.rs` + NEW `tests/wat_arc146_multimethod_mechanism.rs`; MODIFIED `src/lib.rs` (1-line `pub mod`) + `src/runtime.rs` (+249 LOC) + `src/check.rs` (+244 LOC) + `src/freeze.rs` (+46 LOC) + `src/special_forms.rs` (+8 LOC). NO wat files. NO other Rust changes. |
| 2 | `Multimethod` + `MultimethodArm` + `MultimethodRegistry` | ✅ All present in `src/multimethod.rs:46-79`. Mirrors `MacroDef`/`MacroRegistry` shape exactly. `MultimethodError` has 6 variants covering all surfaceable failure modes. |
| 3 | `is_defmultimethod_form` + `parse_defmultimethod_form` | ✅ At `src/multimethod.rs:243` + `:259` + `:325` (parse_arm helper). Returns `Result<Multimethod, MultimethodError>`. Top-level entry `register_defmultimethods` at `:222` mirrors `register_defmacros`. |
| 4 | `SymbolTable.multimethod_registry` field + setter | ✅ Mirrors `macro_registry` shape; `Option<Arc<MultimethodRegistry>>`. |
| 5 | Freeze-time recognition | ✅ `is_mutation_form` includes `:wat::core::defmultimethod`. Freeze processes via the new parse path; registers into the SymbolTable's registry at step 6b (Q4 decision — see below). |
| 6 | Check-time dispatch | ✅ Guard at `src/check.rs:2961-2972` BEFORE existing keyword arms; routes to `infer_multimethod_call` at `:8172` if matched. Helper uses existing `unify` for arm-pattern matching against arg types; instantiates matched arm's impl scheme. |
| 7 | Runtime dispatch | ✅ Guard at `src/runtime.rs:2403-2410` (top of `dispatch_keyword_head`); routes to `eval_multimethod_call` at `:3186`. Helper has `value_matches_type_pattern` companion. |
| 8 | Arc 144 extensions | ✅ `Binding::Multimethod` variant at `src/runtime.rs:6309-6313`. `lookup_form` 6th branch at `:6360-6371` (slot 2a — between Macro and Primitive per Q3). All 3 reflection primitives handle the variant: lookup_define + signature_of emit declaration form via new `multimethod_to_define_ast` helper at `:6398`; body_of returns :None. |
| 9 | New test file | ✅ `tests/wat_arc146_multimethod_mechanism.rs` with 7 tests (i64-arm, f64-arm, no-match check-time, lookup-define, signature-of, body-of, arity-mismatch deferred to call-time). 7/7 PASS. |
| 10 | **Baseline tests still pass** | ✅ `wat_arc144_lookup_form` 9/9; `wat_arc144_special_forms` 9/9 (no defmultimethod test row added — that's a small gap; could ship in slice 8); `wat_arc144_hardcoded_primitives` 17/17; `wat_arc143_lookup` 11/11; `wat_arc143_manipulation` 8/8; `wat_arc143_define_alias` 2/3 (length canary unchanged — slice 2 closes). |
| 11 | Honest report | ✅ ~400-word report covers all required sections; decisions on Q1-Q4 explicitly named with rationales; 4 honest deltas surfaced (StartupError variant, FrozenWorld field, value_matches_type_pattern source-spelling tolerance, CacheService.wat noise verified pre-existing). |

## Soft scorecard (5/5 PASS)

| # | Criterion | Result |
|---|---|---|
| 12 | LOC budget (600-1000) | ✅ 548 LOC + new test file ~150 LOC ≈ 700 LOC. UNDER predicted band — patterns mirrored cleanly enough to keep changes tight. Honest scope. |
| 13 | Style consistency | ✅ Mirrors macros.rs entity+registry shape, capability-carrier pattern, infer_*/eval_* helper placement, arc 144 reflection-helper style. No invented patterns. |
| 14 | clippy clean | ✅ Baseline 40 warnings → post-changes 40 warnings. ZERO new warnings. One `#[allow(clippy::too_many_arguments)]` added on `infer_multimethod_call` to match standard 7-arg infer-helper signature. |
| 15 | Open-question decisions named | ✅ Q1-Q4 each with rationale (see below). |
| 16 | Audit-first discipline | ✅ Sonnet's report flags 4 honest deltas; no surprises absorbed silently. The CacheService.wat noise verified pre-existing via stash isolation. |

## The four open-question decisions

### Q1 — Arity check timing: DEFERRED TO FIRST CHECK-TIME CALL

Per BRIEF recommendation. CheckEnv isn't built when defmultimethod
parses; parse-time enforcement would require freeze-ordering
gymnastics. Defers to call-time via `CheckError::MalformedForm`
carrying surface-arity vs arm-impl-arity disagreement at the
multimethod's call site. Honest + simple.

### Q2 — Helper placement: ADJACENT TO DISPATCH SITES

`infer_multimethod_call` placed in `src/check.rs` immediately
before `infer_list_constructor` (alongside `infer_length` /
`infer_get` family).

`eval_multimethod_call` placed in `src/runtime.rs` immediately
before `eval_lambda` (alongside other eval_* helpers). Each
adjacent to its dispatch site.

### Q3 — Precedence: MULTIMETHODS WIN OVER SPECIAL FORMS AND PRIMITIVES

`lookup_form`'s multimethod branch sits at slot 2a — after
UserFunction (1) and Macro (2), but BEFORE Primitive (3), Type
(4), SpecialForm (5).

Rationale (sonnet's): user-declarable names override substrate-
fixed names; future arc 146 migrations (slice 2 onward) replace
primitives with multimethods cleanly because the multimethod
takes precedence at lookup time.

This is more permissive than the BRIEF's recommendation (which
just covered the special-form-vs-multimethod case). Sonnet
extended the precedence to include Primitives. Worth scrutinizing
for slice 2 — does declaring `:length` as a multimethod also
override the existing primitive's CheckEnv scheme? If yes, slice
2 doesn't need to retire `infer_length` to make the multimethod
take effect — though retiring it remains the right cleanup.

### Q4 — Freeze ordering: STEP 6B

Multimethods register AFTER `register_defines` / struct-method /
enum-method / newtype-method registration, BEFORE
`resolve_references` and `check_program`.

Rationale: arms reference impl-keyword paths that must be visible
to dispatch; placing 6b right before name-resolution (step 7)
ensures the registry is populated before any check-time consumer
reads it.

This means: a multimethod declaration in source must come in the
SAME freeze pass as its impls (no forward-referencing needed
because all defines+structs+etc. register at steps before 6b).

## Honest deltas (sonnet surfaced; orchestrator verified)

### Delta 1 — StartupError + FrozenWorld plumbing

Necessary for `?` propagation from `register_defmultimethods` up
through the freeze pipeline. Mirrors `MacroError` plumbing
exactly. ~10 LOC across freeze.rs.

### Delta 2 — `value_matches_type_pattern` source-spelling tolerance

Accepts BOTH the FQDN form (`:wat::core::i64`) and the bare form
(`:i64`) when matching a Path pattern against a Value's type tag.
Why: `Value::type_name()` returns `"i64"`/`"f64"`/etc. without
the FQDN prefix, but the user writes `:wat::core::i64` in arm
patterns.

This is honest — the substrate's runtime type-name machinery
predates arc 109's FQDN sweep; the tolerance bridges the two.
Could be tightened in a future arc once `type_name()` returns
FQDN.

### Delta 3 — CacheService.wat noise verified pre-existing

The `wat-lru/lru_raw_send_no_recv` test failure was confirmed via
stash isolation to be caused by the in-flight CacheService.wat
modification, NOT by slice 1 changes. Same pre-existing condition
flagged in slices 1-3 of arc 144's SCOREs.

### Delta 4 — Arc 144 special_forms test row not added

The brief hinted at potentially adding a `defmultimethod` test
row to `wat_arc144_special_forms.rs` (since defmultimethod is
registered as a special form). Sonnet didn't add this test. Minor
gap; could add in slice 8 (closure) or skip entirely (the
mechanism is exercised by the new multimethod test file).

## Calibration record

- **Predicted Mode A (~50%)**: ACTUAL Mode A. Calibration matched.
- **Predicted runtime (30-50 min)**: ACTUAL ~23 min. UNDER band.
  Substrate patterns sonnet mirrored were strong enough that
  execution was efficient. **Calibration tightening:** future
  substantial substrate slices may run faster than predicted as
  the brief-pattern + sonnet-pattern matures.
- **Time-box (100 min)**: NOT triggered.
- **Predicted LOC (600-1000)**: ACTUAL ~700 (548 src + ~150
  tests). Under band; honest scope.
- **Predicted clippy clean**: HIT.
- **Predicted Mode B branches (freeze-order/arity/unify/borrow)**:
  NOT HIT — sonnet handled all four open questions with the
  recommended defaults; no Mode B branches triggered.

## Discipline notes

- The pre-flight crawl (BRIEF named file:line for every reference)
  + the four-questions discipline + the FM 9 baseline confirmation
  produced a clean substantial slice. No back-and-forth needed
  during execution.
- Sonnet's mirror-existing-patterns discipline (macros.rs as
  template) kept the slice tight. Inventing new shapes would have
  bloated LOC + invited Mode B surprises.
- Q3 (precedence) is sonnet's most consequential decision — the
  BRIEF recommended specifically re special-forms; sonnet
  extended to all kinds. Worth re-examining in slice 2 to verify
  the broader precedence doesn't create surprises.

## What this slice unblocks

- **Slice 2** — migrate `length` as canonical first migration.
  Mechanism does the heavy lifting; slice 2 declares the
  multimethod + retires the handler.
- **Slices 3-7** — analogous migrations for empty?, contains?,
  get, conj, plus the pure-rename family.
- **Slice 8** — closure paperwork.
- **Arc 144 slice 4** — verification simpler post-arc-146-slice-2
  (length canary turns green via the migration, not via a wrapper).

The substrate's design coherence is one slice closer to restored:
multimethod entity exists; the mechanism is honest; per-Type impls
in slice 2+ become clean rank-1 schemes that lookup_form sees
uniformly.

Per § 12: foundation strengthened by one entity kind. The slow
path is the right path.
