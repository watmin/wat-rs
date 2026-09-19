# SCORE — a fence-local binder may not shadow a rete variable

Written after the strike, against `EXPECTATIONS.md`'s rows in order.

## EXPECTATIONS rows

| what | expected | REAL RESULT |
|---|---|---|
| ⭐ the one real corpus site still compiles | `where-inline-computed.wat:166`'s `(let [x ?k] …)`, as a fixture, compiles and fires | **MET, and widened.** `tests/rete/probe_arc278_fence_binder_shadow.wat` row 1 reproduces the exact shape and passes under `plain_binders_still_compile`. The re-derived census (see "Deltas" below) found this is not the corpus's ONLY plain-binder fence — rows 2-3 of the same fixture reproduce two more real shapes from `where-control.wat` (a `let` binder feeding a boolean composition; a `match` arm's variant-payload binder). All three compile. |
| the hazard is refused | `(where (let [?k "str"] …))` refused at rule-compile, naming the binder | **MET.** `tests/rete/probe_arc278_fence_binder_shadow_let.wat.bad` is refused with `FenceBinderShadowsReteVar { rule: "fbsl::shadow-in-let", form: "let", binder: "?k" }`. |
| `match` — driven, then decided | either refused with a driven demonstration it binds, or left alone with evidence | **DRIVEN, THEN REFUSED.** See "The `match` question" below — a scratch probe (since deleted; see "Deltas") proved a bare `?k` match-arm pattern binds and shadows, beside a condition binding `?k` to `i64`. `tests/rete/probe_arc278_fence_binder_shadow_match.wat.bad` refuses the identical shape with `FenceBinderShadowsReteVar { rule: "fbsm::shadow-in-match", form: "match", binder: "?k" }`. |
| the variant carries no `fact_type` | read `error.rs` | **MET.** `FenceBinderShadowsReteVar { rule, form, binder }` — no `fact_type` field, matching `FenceConstraintTypeMismatch`'s precedent and its stated reason (a `where` fence sits at `:when` top level, bound to no fact). |
| the retired claim is recorded | `check_fence_interior`'s doc rewritten, not deleted | **MET.** The doc now states the shadowing-risk claim is RETIRED, cites the DESIGN.md path, and restates the honest remaining reason the walk still skips both bodies (not-knowable operand types, not shadowing risk). See `src/rete/validate/typing.rs` around the `check_fence_interior` doc comment. |
| ⛔ MUTATION — the refusal is load-bearing | delete the refusal in live source, re-run; hazard fixture REDs, restore ⇒ green | **MET, both forms independently.** Deleted the `let` push-error block and the `match` per-arm scan call; rebuilt; `let_shadow_is_refused` AND `match_shadow_is_refused` both went RED (the fixtures loaded clean — no error at all), `plain_binders_still_compile` stayed green. Restored; rebuilt; all three green again (confirmed via `diff` against a pre-mutation backup — byte-identical). |
| ⛔ MUTATION — the guard is load-bearing | make the refusal fire on ANY binder, not just `?`-prefixed; plain-binder fixture REDs | **MET.** Changed both `.starts_with('?')` guards (the `let` pair-name check and the `match` pattern-scan's `Symbol` arm, plus the hash-destructure key check) to fire unconditionally. Rebuilt; `plain_binders_still_compile` went RED, naming all three legitimate binders (`x`, `s`, `v`) as violations; `let_shadow_is_refused` and `match_shadow_is_refused` stayed green (they were already refused for the right reason, so a wider guard still refuses them). Restored; rebuilt; green again. |
| nothing else moved | the 27-cell matrix in the SCORE of this stone | **N/A for THIS stone** — that matrix belongs to a different, larger strike (fence-interior-types' own SCORE, if one exists); this stone's blast radius is `typing.rs` + `error.rs` + new fixtures only. The one existing sibling test file that shares the function under test, `probe_arc278_fence_interior_types.rs`'s three tests, was re-run and stayed green (`fence_interior_type_error_is_refused`, `inline_twin_is_refused`, `legal_fences_still_compile`). |
| floor | 0 failed, passed ≥ 5523 + new tests | **MET on the SECOND run** (see "Deltas" — the first run correctly caught a defect in my own process, not the corpus). Final: `Summary [ 465.352s] 5526 tests run: 5526 passed (1 slow), 19 skipped`. `grep -c FAIL .floor/latest/clean.log` = **0**. |
| clippy | `cargo clippy --all-targets --release -- -D warnings`, rc=0 | **MET.** Ran once against the finished (unmutated) source; clean, rc=0. |

## The `match` question — driven, then decided

The brief's open question: does a `?`-prefixed symbol in a `match` arm PATTERN inside a fence
actually bind (shadow), or is it refused/inert some other way? **Driven, not inferred.**

A scratch probe (`wat-scripts/scratch-pad/arc278-fence-binder-shadow/probe-match-pattern-binds.wat`,
run against the **pre-cure** binary) bound `?k` outer-scope to `i64` via a fact pattern, then put a
fence `(match "shadow" (?k (string::= ?k "shadow")))` beside it — a subject with **no legitimate
i64 reading**, so the rule can only fire if the arm's bare `?k` pattern actually rebinds to the
subject. It printed:

```
"match-arm 1, control 1"
```

The match-arm rule FIRES (1), identical to its where-fence-free control (1) — proving the pattern
binds and shadows, exactly like `let`. This matches the static reading (`lower_pat`'s bare-`Symbol`
arm, `src/rete/expr_ir/mod.rs:862`, mints a `Pat::Bind` unconditionally — no `?`-prefix special
case) but was not assumed from it. **Verdict: refused, in `match` exactly as in `let`.**

### One widening beyond the brief's literal sketch, and why

The brief's sketch checks only the arm's top-level pattern. `lower_pat` is **recursive** —
`(:Type::Variant payload)` lowers its `payload` through `lower_pat` again, and a hash-destructure
`{var :field}` lowers each `var` through `cx.slot` the same way `let`'s binder does. So a
`?`-prefixed name nested inside a variant payload (`(:wat::core::Some ?k)`) or a hash-destructure
key (`{?k :field}`) is **the identical hazard**, one level down, and a top-level-only check would
miss it. `check_match_pattern_for_shadow` (`src/rete/validate/typing.rs`) recurses to mirror
`lower_pat`'s own shape rather than the brief's shallower sketch. This is not scope creep — it is
the same contract decision (refuse a `?`-prefixed BINDER in pattern position) applied everywhere
`lower_pat` can introduce one, rather than only where the sketch happened to look. The
over-rejection guard (`plain_binders_still_compile` row 3, `(:wat::core::Some v)`) proves the
recursion does not over-refuse a plain nested binder.

## Corpus counts — re-derived, and the DESIGN's number was WRONG

Per the brief's explicit instruction ("RE-RUN THE CORPUS COUNT BEFORE LEANING ON IT"), the count was
re-derived with a **structural** walker
(`wat-scripts/scratch-pad/arc278-fence-binder-shadow/census-fence-binders.wat` — `read-string` +
`ast->children`, never a grep), anchored on a known positive (the scratch probe above: while it
still existed, the census correctly printed `MATCH-FENCE …probe-match-pattern-binds.wat @L36
binder=?k`, proving the instrument is not inert), over **every** `.wat` under `wat-scripts/` and
`tests/` (1539 files, `.wat.bad` excluded since those are supposed to fail).

| | DESIGN.md's claim | RE-DERIVED |
|---|---|---|
| `?`-prefixed binder in a fence-local `let`/`match`, anywhere in the corpus | 0 | **0 — confirmed.** |
| fences using a `let` at all | **"exactly 1"** | **7 — DESIGN.md's number was FALSE.** 3 in `where-inline-computed.wat` (lines 128, 140, 166 — all the same plain binder `x` holding `?k`'s value, in three different surrounding expressions) + 4 in `where-control.wat` (lines 176, 190, 233, 253 — binders `s`, `c`, `s`, `s`, one feeding a pure fn call, the rest feeding boolean compositions). |
| fences using a `match` with a bare-symbol top-level pattern | not stated | **0** (the corpus's only fence `match` sites — `where-control.wat` rows 6-7 — use literal patterns `0`/`1`/`2`/`3` or a nested variant pattern `(:wat::core::Some v)` whose TOP-LEVEL pattern is a `List`, not a bare `Symbol`). |

**Why the DESIGN's "1" was wrong, diagnosed:** the original count was very likely a grep for the
exact substring `where (:wat::rete::core::let` on a single line, which the doc's own anchor
(`where-inline-computed.wat:166`) is shaped like — but `where-control.wat`'s six fences put the
`where` and the `let`/`match` on different lines (a multi-line `:when` vector), so a same-line or
narrow-window grep undercounts silently. This strike's walker recurses the actual AST instead, so
it is not vulnerable to line-wrapping. The corrected number does not change the CONTRACT decision
(all 7 real `let` sites are plain binders, refused nowhere), but "exactly 1" was a specific,
falsifiable claim and it was false — recorded here rather than quietly carried forward, per this
session's own five prior instrument errors on this exact question (EXPECTATIONS.md's "Believing
this brief" trap door, hit and caught, not avoided).

## Deltas from the brief — the floor's first run, and what it caught

The first `./scripts/floor.sh` run went **RED**:

```
Summary [ 466.190s] 5526 tests run: 5524 passed (2 slow), 2 failed, 19 skipped
FAIL [   4.875s] ( 227/5526) wat::lint rete_compile_gate::every_wat_scripts_rete_rule_compiles_shard_01
FAIL [ 249.025s] (2101/5526) wat::lint wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime
```

**Both named exactly one file**, verbatim from `ARM.txt`:

```
wat-scripts/scratch-pad/arc278-fence-binder-shadow/probe-match-pattern-binds.wat
    startup: #wat.rete/ReteCheckErrors {… FenceBinderShadowsReteVar {… :rule "mps::r" :form "match"
    :binder "?k" …} …}
```

**Not a corpus regression.** This was my own scratch probe — written and driven (per the brief's
own instruction) BEFORE the cure existed, to prove the `match` hazard was real. It succeeded at
that job; then the cure landed and, correctly, started refusing the exact shape the probe was built
to demonstrate. Two of `wat-rs`'s own gates sweep **every** `.wat` under `wat-scripts/` with **no
exemption category** (`rete_compile_gate`'s own panic message: *"THIS GATE HAS NO EXEMPTION
CATEGORY, BY DESIGN"*) — so a scratch file that can no longer compile cannot go on living there. Per
the wat-rs floor doctrine (do NOT re-run on red; capture, name the arm, THEN act), I:

1. Captured the whole arm (both blocks, above and in `.floor/2026-09-10T23-40-22Z/ARM.txt`).
2. Named the exact mechanism: my own probe file, not a defect in the cure or the pre-existing corpus.
3. Deleted the probe (its finding is preserved permanently as `probe_arc278_fence_binder_shadow_match.wat.bad`, a fixture whose whole *job* is to fail) and removed its now-dangling path from the census script's file list.
4. Re-ran the two previously-red gates directly — both green.
5. Re-ran the **whole** floor once, clean: `Summary [ 465.352s] 5526 tests run: 5526 passed (1 slow), 19 skipped`.

This is the STOP-2 mechanism working as designed, just triggered by my own tooling instead of a
pre-existing corpus file — the same discipline applies either way: capture whole, name the arm,
fix the root cause, re-run once.

## What the brief got wrong

- **DESIGN.md's "exactly 1 fence uses a `let` at all" was false** — corrected above to 7, all
  plain-binder and unaffected by the cure. The "0 `?`-prefixed binders" contract-relevant number
  held.
- **Nothing else in the brief/DESIGN was wrong.** STOP-1 (a defensible reading for a `?`-prefixed
  binder) did not trigger — no such reading was found. STOP-3 (needing a scope stack or a local
  type env) did not trigger — the fix is exactly the shape sketched, plus the pattern-recursion
  widening above.

## What this stone does NOT do (restated precisely, per EXPECTATIONS' trap door)

This does **not** make `let`/`match` bodies type-checked. `check_fence_interior` still returns
without descending into either body. `(:wat::rete::core::i64::> x 100)` inside
`where-inline-computed.wat:166`'s `let` still type-checks nothing about `x`, because `x` is a
fence-local binder — not in the rule-wide `binds` map, not a field, not a literal —
`resolve_operand_type` still answers `ComputedNotDerivableHere` and the operand is skipped. This
stone retires the JUSTIFICATION the old doc gave for that skip (shadowing risk) and replaces it
with the true remaining reason (no local type environment exists to check against); it does not
build that environment. Descending into the bodies is unchanged, separate, and larger work.

## Files touched

- `src/rete/validate/error.rs` — new `ReteCheckErrorKind::FenceBinderShadowsReteVar { rule, form, binder }` + `Display` arm.
- `src/rete/validate/typing.rs` — `check_fence_interior`'s `let`/`match` arms refuse a `?`-prefixed binder before walking outer-scope values/subjects as before; new `check_match_pattern_for_shadow` recursive scanner; doc comment rewritten (shadowing-risk claim retired, honest remaining reason stated).
- `tests/rete/probe_arc278_fence_binder_shadow.rs` — 3 tests (`let_shadow_is_refused`, `match_shadow_is_refused`, `plain_binders_still_compile`).
- `tests/rete/probe_arc278_fence_binder_shadow_let.wat.bad`, `_match.wat.bad` — the two hazard fixtures.
- `tests/rete/probe_arc278_fence_binder_shadow.wat` — the over-rejection control (3 rows: the real corpus `let` site, a `let`-feeding-boolean site, a `match`-variant-payload site).
- `wat-scripts/scratch-pad/arc278-fence-binder-shadow/census-fence-binders.wat` — the re-derivable corpus census instrument (kept, per "a metric without its instrument cannot be rechecked"); the probe that drove the `match` question was deleted after use (see Deltas).

## Floor — verbatim

```
Summary [ 465.352s] 5526 tests run: 5526 passed (1 slow), 19 skipped
```

`FAIL` lines in `.floor/latest/clean.log`: **0**.
