# Arc 160 — Slice 2 BRIEF (the fix)

**Drafted 2026-05-07.** Slice 2 of arc 160 — apply the fix sonnet's
slice 1 diagnostic identified.

## Working directory

`/home/watmin/work/holon/wat-rs` on branch
`arc-159-wip-substrate-and-sweep`.

## Workspace state pre-spawn

- HEAD: `d4b5131` (arc 160 slice 1 BRIEF shipped; slice 1 ran and
  reported)
- Working tree clean
- 9 currently-failing tests; sonnet's diagnostic identified the
  exact lossy step

## Slice 1 diagnostic summary (recap)

`infer_list` in `src/check.rs` has two structurally-separate regions:

- **Region A** (lines 3661–4374): `if let WatAST::Keyword(k, _) = head { match k.as_str() { ... } }` — handles all keyword-headed lists; always exits via match arm or fall-through.
- **Region B** (lines 4376–4530+): bare-Symbol constructor checks (`head_is_ok_bare`, `head_is_ok_fqdn`, `head_is_some_fqdn`, `head_is_err_fqdn`).

**The bug:** `head_is_ok_fqdn` etc. check `WatAST::Keyword(k, _) if k == ":wat::core::Ok"`. But Region A already pattern-matches `WatAST::Keyword`. If head IS a Keyword, Region A intercepts; Region B never runs. So `head_is_*_fqdn` checks are DEAD CODE.

For a Keyword-headed FQDN constructor call like `(:wat::core::Ok 418)`:
1. Region A enters
2. `match k.as_str()` has no arm for `:wat::core::Ok`
3. Falls to `_ => {}` (line 4243)
4. `env.get(":wat::core::Ok")` returns None (Ok isn't a registered scheme)
5. `infer_list` returns None
6. `process_let_binding` fails to populate `out_scope`
7. Match scrutinee `resp` lookup in locals returns None
8. `scrutinee_ty = None` → MatchShape gets fresh vars
9. IntLit pattern checks against `Var(?296)` → "int literal pattern in :?296 position"

## Goal

Make FQDN keyword constructor calls (`(:wat::core::Ok value)` etc.)
hit Region A's match arms and return the parametric type the
existing logic in Region B already computes correctly.

## Fix shape (per sonnet's slice 1 proposal)

### Step 1 — Hoist constructor inference into helpers

Extract Region B's `head_is_ok_*` / `head_is_err_*` / `head_is_some_*`
blocks into three helper functions:

```rust
fn infer_ok_constructor(
    items: &[WatAST],
    head_span: &Span,
    is_bare: bool,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if is_bare {
        errors.push(CheckError::TypeMismatch {
            callee: "Ok".into(),
            param: "(retired bare-symbol exception)".into(),
            expected: ":wat::core::Ok".into(),
            got: "Ok".into(),
            span: head_span.clone(),
        });
    }
    let args = &items[1..];
    if args.len() != 1 {
        errors.push(CheckError::ArityMismatch {
            callee: if is_bare { "Ok".into() } else { ":wat::core::Ok".into() },
            expected: 1,
            got: args.len(),
            span: head_span.clone(),
        });
        for arg in args {
            let _ = infer(arg, env, locals, fresh, subst, errors);
        }
        return Some(TypeExpr::Parametric {
            head: "Result".into(),
            args: vec![fresh.fresh(), fresh.fresh()],
        });
    }
    let t_ty = infer(&args[0], env, locals, fresh, subst, errors)
        .unwrap_or_else(|| fresh.fresh());
    let e_var = fresh.fresh();
    Some(TypeExpr::Parametric {
        head: "Result".into(),
        args: vec![t_ty, e_var],
    })
}
```

Same shape for `infer_err_constructor` and `infer_some_constructor`.

### Step 2 — Call from Region A (FQDN keyword path)

Inside `infer_list`'s Region A, add three match arms BEFORE the
`_ => {}` fall-through:

```rust
":wat::core::Ok" => return infer_ok_constructor(
    items, head_span, /*is_bare=*/ false, env, locals, fresh, subst, errors,
),
":wat::core::Err" => return infer_err_constructor(
    items, head_span, /*is_bare=*/ false, env, locals, fresh, subst, errors,
),
":wat::core::Some" => return infer_some_constructor(
    items, head_span, /*is_bare=*/ false, env, locals, fresh, subst, errors,
),
```

(Some has no payload-arity-1 invariant; check sonnet's slice 1
diagnostic + the existing Region B code for Some's exact shape.)

### Step 3 — Replace Region B's blocks with helper calls

Region B's bare-Symbol path stays for backward compat:

```rust
if head_is_ok_bare {
    return infer_ok_constructor(
        items, head_span, /*is_bare=*/ true, env, locals, fresh, subst, errors,
    );
}
```

Same for `head_is_err_bare` / `head_is_some_bare` if those exist.

The `head_is_*_fqdn` checks in Region B should be RETIRED (they
were dead code; now the FQDN path runs in Region A). Per arc 113
"orphaned scaffolding" precedent, leave the variant + Display
references intact if they exist; just retire the Region B firing
arms.

## Constraints

- **Substrate-only edits.** EXACTLY 1 file: `src/check.rs`. No
  other crate. No consumer wat edits. No new test files (the 9
  currently-failing tests verify the fix end-to-end).
- **DO NOT COMMIT.** Working tree dirty for orchestrator review.
- **Workspace MUST go from 9 failed → 0 failed.** All 9 failures
  share root cause; fix should clear all of them.
- **STOP at unexpected red.** Distinguish:
  - **Expected:** the 9 currently-failing tests pass
  - **Unexpected:** any pre-existing test breaks. The substrate
    change is intended to be additive (fixing dead code paths);
    no regressions expected.
- **Time-box: 45 min wall-clock.**

## Pre-flight crawl

1. `docs/arc/2026/05/160-parametric-variant-constructor-inference/DESIGN.md`
2. `docs/arc/2026/05/160-parametric-variant-constructor-inference/BRIEF-SLICE-1.md` — context
3. Sonnet's slice 1 diagnostic findings (in this BRIEF's recap)
4. `src/check.rs::infer_list` Region A (line 3661 onward; find the
   keyword-headed branch and `match k.as_str() {` block)
5. `src/check.rs::infer_list` Region B (lines 4376–4530; the
   bare-Symbol constructor blocks)
6. The arc 159 substrate's `process_let_binding` (line 7570) — to
   confirm the inferred type now propagates correctly post-fix

## Pre-flight verification

```bash
cargo test --release --workspace 2>&1 | grep -E "test result|FAILED" | tail -5
```

Confirms 9 failures pre-fix. Note the specific test names; verify
the same 9 pass post-fix.

## Verification (after edits)

```bash
cargo test --release --test wat_recursive_patterns 2>&1 | tail -5
cargo test --release --workspace 2>&1 | grep -E "test result|FAILED" | tail -5
```

Expect: workspace = 0 failed; ~1413+ passed; 0 LegacyTypedLetBinding.

## Reporting (~200 words)

- Pre-flight crawl confirmation
- Edit summary (LOC delta; functions added/modified)
- New test pass count (9 previously-failing now pass; no regressions)
- Path classification (Mode A / B / C)
- Honest deltas:
  - Did `Some` constructor inference share the same bug? Was it
    similarly fixed? (Some has no separate Err type, so its
    parametric is `Option<T>` with one arg.)
  - Did the helper extraction surface any other shared patterns
    that could be unified later?
  - Did any tests previously passing accidentally cover the bug
    by having an explicit type annotation that's now redundant?

DO NOT commit. Orchestrator commits + scores after.

## Time-box

45 minutes wall-clock.

## Why slice 2 matters

This unblocks arc 159 closure (the 9 cleanup tests will pass
post-fix; arc 159's binding-shape change ships cleanly). Per the
mass-refactor discipline: "this work is meant to catch exactly
these kinds of problems" — arc 160 IS the catch.
