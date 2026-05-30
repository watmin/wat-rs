# BRIEF — Stone 243.3 R3-β sweep — 11 FIX clusters + 4 attested-defer runes

You are sonnet. Stone 243.3 R3-β sweep — the post-debate resolution of the R2 vigilia round. Every finding was triaged one-by-one with the orchestrator via four-questions. This brief carries the FIX verdicts + the attested-defer runes. The LEAVE-DISPUTED finding (⑬) gets NO code change.

**Anchor cwd:** `/home/watmin/work/holon/wat-rs/`. Verify with `pwd`. Reject `.claude/worktrees/`.

## Pre-spawn state

Working tree is CLEAN at checkpoint `7f3a9c2e` (R3 main + addendum + R3-α + DESIGN merge all committed). Your R3-β changes are fresh modifications; orchestrator commits Stone 243.3 atomic after vigilia R3 verification + SCORE Phase B.

**Gates baseline:** lib 890/0 · tests/function 8/0 · probe_arc243_stone3 3/0 · workspace test-build clean · clippy 897. All must hold.

## CRITICAL discipline — the F9 lesson (read first)

In the R2 round, struere's F9 finding ("infer fall-through returns fresh Var — silent failure") was a **FALSE POSITIVE**: its premise was factually wrong about the code structure (the primary `infer` match is exhaustive; the real `_ => {}` is a no-op transition that's intentionally permissive, already hardened by arc 234). A blind fix would have broken `:wat::core::defn`.

**Therefore: for every structural-migration fix below (⑦, ⑩, ⑪), VERIFY THE PREMISE against the actual code BEFORE editing.** If the premise doesn't hold (the signature isn't what's described, the cascade is bigger than stated, the "redundant" thing is load-bearing), STOP and surface verbatim — do NOT force the fix.

## The 11 FIX clusters

### ① walk_for_legacy_stream — manual prefix strip → strip_prefix

`src/check.rs` ~3446. `&s[LEGACY_STREAM_PREFIX.len()..]` after `s.starts_with(LEGACY_STREAM_PREFIX)`. Rewrite to `if let Some(rest) = s.strip_prefix(LEGACY_STREAM_PREFIX) { ... }`. Clippy-clean.

### ② walk_for_arc170_legacy — drop 3 unneeded `return;`

`src/check.rs` ~3053, 3075, 3081 (line numbers shifted by R3-α; grep `fn walk_for_arc170_legacy` and find the tail-position `return;` statements clippy flags). Drop the redundant trailing `return;` (tail position only — clippy flags exactly these). Control flow unchanged.

### ③ ast_variant_name — eliminate 3-way copy + label DIVERGENCE

Three copies exist: `src/types.rs` (~3151, function `ast_variant_name`), `src/check.rs` (~8936, `ast_variant_name_check`), `src/runtime.rs` (`ast_variant_name`). They DIVERGE: types.rs uses `"string literal"`/`"int literal"`/`"float literal"`/`"bool literal"`; check.rs uses `"string"`/`"int"`/`"float"`/`"bool"`. Same AST node → different user-facing label depending on path. This is a correctness defect.

Action:
1. Read `src/ast.rs` (the `WatAST` home; leaf module — no import cycle, which is WHY the copies exist).
2. Add a single authoritative free fn (or method) in `src/ast.rs`: `pub fn ast_variant_name(ast: &WatAST) -> &'static str` (or `impl WatAST { pub fn variant_name(&self) -> &'static str }`).
3. **Label set: BARE** — `"string"`, `"int"`, `"float"`, `"bool"` (not `"… literal"`). The value IS a string; "literal" is parser-internal vocabulary. Use bare for all variants.
4. Delete all 3 copies (types.rs + check.rs `ast_variant_name_check` + runtime.rs).
5. Update all call sites to the `ast.rs` function. All three files already `use crate::ast::WatAST`.

**Premise-verify:** confirm the 3 copies are functionally identical EXCEPT the literal-suffix labels before merging. If any copy handles a variant the others don't, surface it — don't silently drop coverage.

### ④ DELETE arc-117 deadlock-walker cluster (4 dead fns)

`src/check.rs`: `validate_scope_deadlock` (~2987), `walk_for_deadlock` (~3818), `check_let_for_scope_deadlock` (~3899), `parse_binding_for_typed_check` (~3959). All `#[allow(dead_code)]`, zero live callers (purgare confirmed). They form one dependency chain (delete together).

Action: delete all 4 functions + their `#[allow(dead_code)]` attributes + their doc comments. git history preserves the arc-117 pattern. The "reintroduction recipe" / walker-pattern teaching is ALREADY preserved as present-tense comments on the live retirement sites (R3.15/R3.16). **Verify** those live-site comments exist before deleting (so the teaching isn't lost); if they don't, surface before deleting.

### ⑤ DELETE LegacyTypedLetBinding dead variant (3 sites)

`src/check.rs`: variant def (~356), Display arm (~867), Diagnostic arm (~1515). Never constructed (emitter retired arc 159 slice 4). Delete all 3 sites. (Line numbers shifted by R3-α; grep `LegacyTypedLetBinding`.)

### ⑥ check_alias_reaches + check_union_member_reaches — invariant WHY comment

`src/types.rs` ~3280 (`check_alias_reaches`) + ~3408 (`check_union_member_reaches`). Both use a `visiting: &mut HashSet<String>` with manual insert/remove pairing. The invariant (every insert paired with a remove before any `?`-return) is currently upheld but only by convention. Add a WHY comment at each site (SAFETY-style, documenting the load-bearing invariant — NOT deferral):

```rust
// INVARIANT: every `visiting.insert(name)` is paired with a `visiting.remove(name)`
// before any `?`-propagation can early-return — the cycle-detection set must not
// leak names across recursive calls. New `?`-paths must preserve this pairing.
```

No code change beyond the comment.

### ⑦ CheckEnv — 5 pub fields → pub(crate) + setter [STOP-CAP]

`src/check.rs` ~1978-2007. `pub defined_values`, `pub defined_value_spans`, `pub binding_metadata`, `pub redef_allowed`, `pub defclause_registrations`. Accessors already exist (`get_defined_value_type` etc.).

**Premise-verify FIRST:** grep all read/write sites of these 5 fields across the workspace (`src/`, `tests/`, and check whether any lab crate reads them). 

Action:
1. Downgrade all 5 to `pub(crate)`.
2. Add `pub(crate) fn set_redef_allowed(&mut self, flag: bool)`; update `check_program`'s direct `env.redef_allowed = ...` mutation (~2456) to the setter.
3. Update any in-crate raw-field access to route through accessors where one exists (or keep direct `pub(crate)` access if no accessor — `pub(crate)` is the goal, not full privacy).

**STOP-CAP:** if the cascade exceeds ~10 sites OR any access is from an untouched lab crate / external consumer, STOP and surface the cascade list — do not force it. We'll reassess (possibly it earns its own R-round).

### ⑧ infer_list `_ => {}` — improved WHY comment (LEAVE behavior)

`src/check.rs` ~7068 (grep the `_ => {}` arm inside `fn infer_list`). The permissive fall-through is CORRECT (probe-confirmed: it's a no-op transition into the defclause→scheme→fallback cascade; converting to a hard error would break `:wat::core::defn` and `struct-new`; the genuine silent-pass was already hardened by arc 234 Stone 234.3c). Do NOT change the behavior. Replace/augment the comment at the `_ => {}` arm:

```rust
// Any keyword head not handled by an explicit arm above falls through to
// the defclause dispatch → env.get scheme lookup → unregistered-scheme
// fallback below. Reaches here legitimately: :wat::core::defn and other
// declaration forms (checked by separate pre-pass walkers, not infer_list),
// :wat::core::struct-new (intentional runtime-only dispatch, no scheme),
// and user functions called before scheme registration.
// Do NOT convert to MalformedForm — see arc 160 (constructors hoisted out)
// and arc 234 Stone 234.3c (the one real silent-pass was narrowed to
// UnknownCallee there). The remaining permissive paths are silent-by-intent.
_ => {}
```

### ⑩ dispatch_rust_scheme → CheckResult<TypeExpr> [PREMISE-VERIFY + STOP-CAP]

`src/check.rs` ~15102. Currently `-> Option<TypeExpr>` + takes `errors: &mut Vec<CheckError>`. This is the arc-236 dual-channel anti-pattern (CheckResult was made to kill it).

**Premise-verify FIRST:** confirm the signature is actually `Option<TypeExpr>` + `&mut Vec<CheckError>`, and grep its caller(s). struere claimed "one remaining old-pattern call site" — VERIFY that (the F9 lesson).

Action: migrate to `-> CheckResult<TypeExpr>`; remove the `errors` param; the `None`+push sites become `CheckResult::errs(...)`; the `Some(t)` sites become `CheckResult::ok(t)`; update caller(s) to thread the result (drain errors through the combinator chain instead of inspecting `errors`).

**STOP-CAP:** if the caller-cascade exceeds ~8 sites or ripples into untouched files, STOP and surface. This may earn its own R-round rather than riding R3-β.

### ⑪ process_let_binding → CheckResult<HashMap<String, TypeExpr>> [PREMISE-VERIFY + STOP-CAP]

`src/check.rs` ~12814. Currently 8 params incl. `out_scope: &mut HashMap` + `errors: &mut Vec<CheckError>`, returns `()`. Caller `infer_let` calls it in a loop, detecting failure via `errors.len()` diffing.

**Premise-verify FIRST:** confirm the signature + the `infer_let` call-loop shape.

Action: return `CheckResult<HashMap<String, TypeExpr>>` (the new bindings to merge); drop `out_scope` + `errors` params; `infer_let` merges the returned map into its extended scope and drains errors through the combinator chain.

**STOP-CAP:** if the loop-merge rework ripples beyond `process_let_binding` + `infer_let`, STOP and surface.

### ⑫ unify_union_union → HashSet<String> intersection (O(n+m))

`src/check.rs` ~15043. Currently nested-loops members of u1 × u2 (O(n×m)). 

**Premise-verify FIRST:** confirm `format_type` produces stable canonical keys suitable for set membership (it's used elsewhere as a type key — verify it's deterministic for the member types here).

Action: build `HashSet<String>` from u1's `format_type`-keyed members; probe u2's members against it. O(n+m). No new `Hash` impl on `TypeExpr` (use the format_type string keys). Clarity-neutral-to-positive.

## The 4 attested-defer runes

These findings are DEFERRED to named within-reach stones of the OPEN arc 243 (per the within-reach deferral doctrine). Place a rune at each site citing its owner stone. These are DISTINCT from stalled-arc runes — they cite NEXT-in-chain stones that exist as PLANNED entries in `docs/arc/2026/05/243-conformare-error-shape/DESIGN.md` (verify they're listed there before placing the rune).

### ⑨ parse_defstruct → Stone 243.5

`src/types.rs` ~1891. Place immediately above `fn parse_defstruct`:
```rust
// rune:solvere(deferred-stone-243.5) — parse_defstruct braids 7 concerns
// (arity, slot discrimination, metadata-map, field-vector, restrictions).
// Decomposition (parse_metadata_map + parse_field_vector) lands with the
// src/types/ home carve at Stone 243.5, where types.rs splits into a home;
// deliberate decomposition belongs to that restructure, not a hygiene sweep.
```

### ⑭ check_program walker chain → Stone 243.6

`src/check.rs` ~2195 (above the first of the 10 walker-pass loops in `check_program`). 
```rust
// rune:temperare(deferred-stone-243.6) — these independent walker passes
// each traverse all function bodies (10× total). sequi confirmed they are
// independent accumulator-drains (state-safe to fuse). Fusion into one
// per-body pass lands with the src/check/ home carve at Stone 243.6.
```

### ⑮ collect_hints → Stone 243.6

`src/check.rs` ~ the `collect_hints` fn definition. 
```rust
// rune:temperare(deferred-stone-243.6) — collect_hints runs 9 hint fns and
// is invoked from both Display and diagnostic() for the same error. Caching
// (a computed-hints field on the CheckError outer struct) folds into the
// CheckError Pattern A retrofit at Stone 243.6.
```

### ⑯ CheckError not Pattern A → Stone 243.6 (ONE enum-level rune)

`src/check.rs` ~89 (immediately above `pub enum CheckError`). ONE rune for the whole enum (NOT per-variant):
```rust
// rune:conformare(deferred-stone-243.6) — CheckError is a flat 34-variant
// enum (some variants multi-span, no canonical primary; diagnostic() does
// N-path span extraction). Pattern A retrofit (outer struct + kind enum;
// multi-span per CONFORMARE.md § Multi-span) lands at Stone 243.6 — the
// peer retrofit to TypeError (Stone 243.3). conformare attested this scope
// in the 243.3 R2 cast.
```

## ⑬ LEAVE-DISPUTED — NO code change

`check_program`'s `Arc::new(types.clone())` (temperare T-L1-3) gets **NO change** — no fix, no rune, no comment. It is left to reveal through real work (per `feedback_let_need_reveal_through_work`). The SCORE Phase B (orchestrator-authored) records the dispute. Do not touch it.

## Cadence

1. Baseline gates (890/0 · 8/0 · 3/0).
2. ①② (clippy strips) → cargo clippy check.
3. ③ (ast_variant_name → ast.rs, premise-verify identical-except-labels) → lib test.
4. ④⑤ (deletes, verify teaching preserved / zero callers) → lib test + build.
5. ⑥ (invariant comments) → build.
6. ⑦ (CheckEnv, premise-verify cascade + STOP-CAP) → lib test.
7. ⑧ (infer_list comment) → build.
8. ⑩ (dispatch_rust_scheme, premise-verify + STOP-CAP) → lib test.
9. ⑪ (process_let_binding, premise-verify + STOP-CAP) → lib test + function test.
10. ⑫ (unify_union_union, premise-verify format_type keys) → lib test.
11. ⑨⑭⑮⑯ (place 4 attested-defer runes; verify owner stones listed in DESIGN.md).
12. Final gates: lib ≥ 890 · tests/function 8/0 · probe_arc243_stone3 3/0 · workspace test-build clean · clippy ≤ 897.
13. DO NOT COMMIT. Return paragraph.

## STOP triggers (REJECTION)

1. Lib < 890 · 2. tests/function < 8 · 3. probe < 3 · 4. workspace build fails · 5. clippy > 897
6. 90 min elapsed
7. holon-rs touched (STOP-5)
8. ⑬ touched (must stay untouched)
9. Any premise-verify FAILS (signature not as described / cascade bigger than stated / "redundant" thing load-bearing) — STOP, surface verbatim, do NOT force
10. STOP-CAP exceeded on ⑦/⑩/⑪ — surface the cascade list
11. New deferral language (the 4 runes are attested-stone, not deferral-prose — they cite EXISTING planned stones; if a cited stone is NOT in DESIGN.md, STOP)
12. INTERSTITIAL touched · 13. vigilia/conformare cast by sonnet · 14. commit attempted

## Critical doctrine

- Premise-verify before every structural migration (the F9 lesson)
- Sonnet writes substrate (`feedback_sonnet_writes_substrate`)
- HARD CUT deletes (④⑤) — no commented-out corpses, no compat shims
- Attested-defer runes cite EXISTING DESIGN.md stones only
- DO NOT commit — orchestrator commits Stone 243.3 atomic after vigilia R3 + SCORE Phase B

## Read in order

1. `docs/arc/2026/05/243-conformare-error-shape/DESIGN.md` (verify 243.5/243.6 are listed as PLANNED — the rune targets)
2. `docs/arc/2026/05/243-conformare-error-shape/BRIEF-STONE-243.3-R3-beta.md` (this brief)
3. The fix sites (src/check.rs + src/types.rs + src/ast.rs per each cluster)

## Predicted band

**60-90 min Mode A.** 8 mechanical fixes (①②③④⑤⑥⑧ + 4 runes) + 3 premise-verified structural migrations with STOP-caps (⑦⑩⑪) + 1 algorithmic (⑫). The structural migrations carry the real uncertainty; the STOP-caps bound it.

## Return paragraph (≤ 250 words)

Which clusters landed (①-⑫ + 4 runes); for ⑦⑩⑪ the premise-verify result + cascade size; ⑬ confirmed untouched; final gates; any STOP-CAP hits or premise failures surfaced; any ADDITIONAL findings (honest, per `feedback_pre_existing_is_not_exemption`).
