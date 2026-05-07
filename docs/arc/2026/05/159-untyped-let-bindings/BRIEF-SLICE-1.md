# Arc 159 — Substrate BRIEF (slice 1)

**Drafted 2026-05-07.** Slice 1 of arc 159.

## Workspace state pre-spawn

- HEAD: `42a7803` (arc 158a INSCRIPTION shipped)
- Working tree: clean
- Pre-baseline: **2036 / 0 / 0 warnings**

## Goal

Substrate-only edits that enable the new untyped binding shape
end-to-end:

- Inference (`process_let_binding`): accept `(name rhs)` shape;
  populate `out_scope` with name → inferred rhs type
- Runtime (`parse_let_binding`): accept new shape
- Walker (`LegacyTypedLetBinding` CheckError + walker): fires on
  legacy `((name :T) rhs)` shape per substrate-as-teacher Pattern 3

After slice 1 ships, both shapes work; legacy shape fires walker;
consumer sweep (slices 2-3) clears legacy sites.

## CRITICAL — destructure preservation

v1 sonnet's wrong sweep: `(((a b) p))` → `((a p))` (mangling
destructure as if it were legacy typed binding `((name :T) rhs)`
with type=`b`).

Arc 159 substrate MUST correctly distinguish:

| Binding shape | Substrate path |
|---|---|
| `(name rhs)` — `name` is `WatAST::Symbol` | NEW: untyped binding |
| `((name :T) rhs)` — binder is List, len==2, [0] Symbol, [1] Keyword | Legacy typed (walker fires) |
| `((a b ...) rhs)` — binder is List, all elements Symbols, NO Keyword | Destructure (existing path; unchanged) |

The `is_typed_single` check at line 7580 already does this
correctly:

```rust
let is_typed_single = binder.len() == 2
    && matches!(&binder[0], WatAST::Symbol(_, _))
    && matches!(&binder[1], WatAST::Keyword(_, _));
```

Slice 1 ADDS a fourth case (Symbol-at-[0]) BEFORE the binder-as-
List logic. Existing destructure path stays.

**Test #9 explicitly verifies destructure preservation. v1's bug
must NOT recur.**

## Substrate edits

### `src/check.rs`

#### 1. `process_let_binding` extension (line 7570)

Current shape: returns early when `kv[0]` isn't a List. Add a new
branch BEFORE the binder-as-List logic:

```rust
fn process_let_binding(
    pair: &WatAST,
    env: &CheckEnv,
    rhs_scope: &HashMap<String, TypeExpr>,
    out_scope: &mut HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
    form: &str,
) {
    let kv = match pair {
        WatAST::List(items, _) if items.len() == 2 => items,
        _ => return,
    };

    // Arc 159 — canonical new shape: `(name rhs)` where `name`
    // is a bare Symbol. Type is inferred from `rhs`; no
    // annotation needed (per arc 145 lesson). Walker dependency
    // settled in arc 158a (pair-deadlock walker pattern-matches
    // RHS for channel-related shapes).
    if let WatAST::Symbol(ident, _) = &kv[0] {
        let name = ident.name.clone();
        let rhs = &kv[1];
        let rhs_ty = infer(rhs, env, rhs_scope, fresh, subst, errors);
        if let Some(ty) = rhs_ty {
            out_scope.insert(name, ty);
        }
        return;
    }

    let binder = match &kv[0] {
        WatAST::List(inner, _) => inner,
        _ => return,
    };
    // ... existing typed_single + destructure paths unchanged ...
}
```

#### 2. `LegacyTypedLetBinding` CheckError variant + Display + diagnostic

Mirror `BareLegacyLetStar` (arc 154) exactly:

```rust
LegacyTypedLetBinding {
    binding_name: String,
    span: Span,
}
```

Display: "let binding `((<name> :T) expr)` is legacy form
post-arc-159; use `(<name> expr)` — type is inferred from the
expression". Reference arc 159 + canonical fix.

`diagnostic()` arm: `Diagnostic::new("LegacyTypedLetBinding")` with
`binding`, `canonical`, `location` fields.

#### 3. `walk_for_legacy_typed_let_binding` walker

Mirror `validate_legacy_let_star` (arc 154) shape:
- Walk full AST
- For each `:wat::core::let` form, inspect each binding
- If binding shape is `((<keyword> <type-expr>) <expr>)` (List
  binder, len==2, [0] Symbol, [1] Keyword) → emit
  `LegacyTypedLetBinding`
- Other shapes (new symbol-at-[0], destructure) → no-op

Wired into `check_program` after the def-position walker (arc 157
precedent).

### `src/runtime.rs`

#### 4. `parse_let_binding` extension

Current shape: accepts `((name :T) rhs)` legacy + destructure.
Add new branch BEFORE the binder-as-List logic (mirror sonnet 1a
v1's approach which was reverted):

```rust
// Arc 159 — canonical new shape: `(name rhs)` where `name` is
// a bare Symbol.
if let WatAST::Symbol(ident, _) = &kv[0] {
    return Ok(LetBinding::Single {
        name: ident.name.clone(),
        rhs: &kv[1],
    });
}
```

#### 5. `step_let` (verify; extend if needed)

Read existing `step_let` to confirm whether new-shape needs
explicit handling. Sonnet 1a v1 extended this; verify and
mirror if necessary.

### NEW `tests/wat_arc159_let_bindings.rs`

10-13 tests per DESIGN § Slice plan. Specifically:

**End-to-end (4-5 tests):**
1. `(let ((x 2)) (i64::+,2 x 1))` → 3 (the user goal)
2. Multi-binding sequential → final value
3. Closure capture through new shape
4. Type-mismatch via inferred-from-rhs check
5. Sequential binding semantics

**Walker on legacy (3 tests):**
6. Single legacy binding fires walker
7. Multi-binding all-legacy fires walker per binding
8. Mixed (legacy + new) — walker fires only on legacy

**Destructure preservation (CRITICAL — 2 tests):**
9. 2-element destructure `(let (((a b) p)) (+ a b))` works as
   destructure, NOT misread as new-shape
10. 3-element destructure works

**Regression (1 test):**
11. Arc 128 outer-scope deadlock pattern in NEW shape — `ScopeDeadlock`
    fires (post-inference walker now sees new-shape bindings
    because `process_let_binding` populates `extended`)

## Constraints

- **Substrate-only edits.** EXACTLY 3 files: `src/check.rs`,
  `src/runtime.rs`, NEW `tests/wat_arc159_let_bindings.rs`. NO
  other crate. NO consumer wat edits. **NO embedded-wat edits in
  src/ test modules (this is what scope-crept v1; explicit STOP.)**
- **DO NOT COMMIT.** Working tree dirty for atomic commit with
  slice 2 per recovery doc § 7.
- **The workspace WILL break post-substrate-change** — every
  legacy `((name :T) expr)` site fires `LegacyTypedLetBinding`.
  EXPECTED. Slice 2 sweep clears them.
- **STOP at unexpected red.** Distinguish:
  - **Expected:** `LegacyTypedLetBinding` on legacy sites
    workspace-wide; new tests in `wat_arc159_let_bindings.rs`
  - **Unexpected:** anything else (substrate panic, parse error,
    DESTRUCTURE TESTS BREAKING — that's the v1 bug; if any
    pre-existing destructure test fails post-slice-1, STOP and
    report)
- **No grinding.** No bracket form. No restructuring beyond the
  named substrate edits.
- **Time-box: 60 min wall-clock**.

## Pre-flight crawl (mandatory)

1. `docs/arc/2026/05/159-untyped-let-bindings/DESIGN.md` — full read
2. `docs/arc/2026/05/158-untyped-let-bindings/REALIZATIONS.md` —
   v1 lessons (especially category 2: destructure-mangling bug)
3. `docs/arc/2026/05/158a-let-binding-walker-migration/INSCRIPTION.md`
   — walker dependency settled; understand the foundation
4. `src/check.rs::process_let_binding` (line 7570) — function you're extending
5. `src/check.rs::parse_binding_for_pair_check` (line 3215, post-158a) —
   confirm new-shape walker handling already lands cleanly
6. `src/check.rs::BareLegacyLetStar` variant + Display +
   `validate_legacy_let_star` walker (arc 154) — closest precedent
7. `src/runtime.rs::parse_let_binding` — function you're extending
8. `src/runtime.rs::step_let` — verify
9. `tests/wat_arc154_kill_let_star.rs` — test harness shape
10. `tests/wat_arc157_def.rs` — recent walker+sweep arc precedent
11. `src/check.rs::tests::let_star_destructures_a_pair` — destructure
    test that v1 broke; ensure arc 159 doesn't break it

## Pre-flight verification

```bash
cd /home/watmin/work/holon/wat-rs
cargo test --release --workspace 2>&1 | grep -E "FAILED|^test result" | tail -5
```

Confirms 2036 / 0 baseline.

## Verification (after edits)

```bash
cargo test --release --test wat_arc159_let_bindings 2>&1 | tail -10
cargo test --release --workspace 2>&1 | grep -E "test result|FAILED" | head -10
cargo test --release --lib let_star_destructures_a_pair 2>&1 | tail -3
cargo test --release --lib let_destructure_requires_tuple 2>&1 | tail -3
```

Expect:
- 10-13 of 10-13 new tests pass
- Workspace failures = pre-existing destructure tests STILL PASS
  + many `LegacyTypedLetBinding` firings on legacy sites (expected)
- Specifically: `let_star_destructures_a_pair` MUST pass
  (the v1 regression check)

## Reporting (~250 words)

- Pre-flight crawl confirmation
- Edit summary per file with LOC delta
- New test pass count
- Workspace failure shape (count of LegacyTypedLetBinding;
  destructure tests still pass)
- Path classification (Mode A / B / C)
- Honest deltas, especially:
  - Did `process_let_binding`'s new branch fire correctly for
    new-shape bindings without affecting destructure path?
  - Did runtime `step_let` need extension or was it untouched?
  - Did `walk_for_legacy_typed_let_binding` interact cleanly
    with the new `validate_def_position_with_wrapper` (arc 157)?
  - Any surprise around the `process_let_binding` extension
    accidentally affecting the typed_single or destructure paths?

DO NOT commit. DO NOT write a SCORE doc. Orchestrator commits
1 + 2 atomically + scores after.

If genuinely impossible, STOP and report.

## Time-box

60 minutes wall-clock.

## Why this matters

User direction 2026-05-07: *"this is the first time we've split
over two arcs... we usually have arcs self-contained... what's
done and what's pending for removed typed bindings?"* Arc 158a
shipped substrate prep; arc 159 is the user-visible deliverable.
After arc 159 ships, `(:wat::core::let ((x 2)) (:wat::core::+ x 2))`
evaluates to 4 — the originally stated end goal.

The proactive stepping-stones discipline (recovery doc § 5)
predicted this slice would be tractable once arc 158a's walker
dependency closed. This BRIEF reflects that closed dependency
plus explicit destructure-preservation tests to prevent v1's
sonnet-scope-creep bug from recurring.
