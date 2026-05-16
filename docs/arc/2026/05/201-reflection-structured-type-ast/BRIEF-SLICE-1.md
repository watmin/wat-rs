# Arc 201 Slice 1 BRIEF — Structured type-AST emission in `signature-of`

**Phase:** First slice of arc 201 (reflection layer carries structured types instead of flattened keyword strings). See `DESIGN.md` for full arc scope + iteration history.

**Originating signal:** Arc 170 Stone D2 (commit `64cc793`) blocked on flat type strings; arc 201 opened per `feedback_any_defect_catastrophic`.

## Goal

Replace `type_expr_to_kw` (`src/runtime.rs:8895`) with a recursive `type_expr_to_holon` (sonnet picks final name — `/gaze` if needed) that preserves `TypeExpr` structure when synthesizing signature HolonASTs.

**Emission rules:**
- `TypeExpr::Path(p)` → `HolonAST::Atom(symbol p)` — atomic, unchanged from today's behavior for monomorphic paths
- `TypeExpr::Parametric { head, args }` → `HolonAST::Bundle [Atom(":"+head), ...recurse(args)]`
- `TypeExpr::Tuple(args)` → `HolonAST::Bundle [Atom(":Tuple"), ...recurse(args)]` (or similar — keep consistent with existing parser representation)
- `TypeExpr::Fn(args, ret)` → `HolonAST::Bundle [Atom(":Fn"), ...recurse(args), Atom("->"), recurse(ret)]`
- `TypeExpr::Var(v)` → `HolonAST::Atom(symbol v)` — type variables stay atomic

Apply uniformly across all signature-AST builders:
- `function_to_signature_ast` (`src/runtime.rs:8906`)
- `type_scheme_to_signature_ast` (`src/runtime.rs:8980`)
- `typedef_to_signature_ast` (`src/runtime.rs:9125`)
- `macrodef_to_signature_ast` (grep for it; mirror)
- `dispatch_to_signature_ast` (grep for it; mirror)

## Consumer verification

The reflection consumers must still work post-emission-change:

- `:wat::runtime::extract-arg-names` (`src/runtime.rs:9894`) — reads pair[0] (the name Symbol). Unchanged behavior; the type slot (pair[1]) becomes a Bundle for parametric types but extract-arg-names doesn't touch it. Should pass through.
- `:wat::runtime::rename-callable-name` (`src/runtime.rs:9774`) — audit. If it does anything with the type slot, it needs update. Likely fine (it operates on the head/name).
- `:wat::runtime::define-alias` macro (`wat/runtime.wat:22-29`) — uses signature-of + extract-arg-names + rename-callable-name. Run the existing define-alias tests to confirm no regression.
- Any other consumer found via grep.

If a consumer DOES break, fix it inline (this is the atomic slice 1 + consumer sweep — DESIGN settled atomic).

## Test

Add a unit test that calls `signature-of` on a known parametric-typed fn and asserts the structured emission:

```rust
#[test]
fn signature_of_emits_structured_parametric_types() {
    // Freeze a wat program with a parametric-typed fn:
    let src = r#"
        (:wat::core::defn :my::needs-peer
          [peer <- :wat::kernel::ThreadPeer<wat::core::String,wat::core::i64>]
          -> :wat::core::nil
          :wat::core::nil)
    "#;
    let world = freeze_ok(src);
    // signature-of returns Option<HolonAST>; unwrap; walk to find the
    // peer arg's type slot; assert it's a Bundle with head :ThreadPeer
    // and args [:String, :i64].
    ...
}
```

Plus monomorphic fn + Tuple + Fn test cases as sanity.

## STOP triggers (true emergencies — surface, do not paper over)

1. **A consumer breaks subtly** (e.g., define-alias's rename-callable-name does type-string equality somewhere) — surface; don't paper over with a back-compat keyword shim
2. **Test infrastructure surprises** (e.g., format_type is called from MORE than the signature builders — used in error messages, diagnostics) — those callers KEEP using format_type's keyword output (this slice ONLY touches signature emission, not type display). Surface if the boundary is unclear.
3. **TypeExpr has variants this BRIEF didn't enumerate** (e.g., `TypeExpr::Concrete` or other shapes not in DESIGN list) — surface what you found; ask before extending the recursion. Don't silently add.
4. **Workspace baseline regresses** — STOP, surface the new failure
5. **Any urge to mint a new substrate type / verb / struct** — STOP. This slice is a single-fn rewrite + consumer sweep, no new substrate

## HARD constraints

- DO NOT commit. Orchestrator commits atomically after independent verification.
- **cwd discipline:** FIRST action: `cd /home/watmin/work/holon/wat-rs/`; verify with `pwd`. Harness may launch into `.claude/worktrees/agent-<id>/` — ignore it; operate on the real repo per `docs/COMPACTION-AMNESIA-RECOVERY.md` § 7-bis.
- DO NOT mint any new substrate verb, type, struct, or special form (`feedback_no_new_types`). This slice replaces an EXISTING helper.
- DO NOT change `format_type`'s behavior (it's used for diagnostics + error messages — that's its job). Only change SIGNATURE emission paths.
- DO NOT touch `signature-of` (the eval handler) directly — only its support helpers. Slice 4 renames it; slice 3 adds a sibling.
- DO NOT touch INSCRIPTIONs / past SCOREs / DEFERRAL-VIOLATIONS / SUPERSEDED BRIEFs / AUDIT / recovery doc / past STONE BRIEFs/EXPECTATIONS/SCOREs.
- DO NOT modify INTERSTITIAL files or arc 201 DESIGN.md (orchestrator owns).
- DO NOT use `--no-verify` / `--no-gpg-sign` / skip hooks.

## SCORE methodology

5 rows YES/NO + evidence:

| Row | What | Evidence |
|-----|------|----------|
| A | `type_expr_to_holon` (or chosen name) replaces `type_expr_to_kw`; emits Bundle for Parametric/Tuple/Fn, Atom for Path/Var | grep + impl inspection |
| B | All 5 signature-AST builders use the new emission | grep `type_expr_to_kw` → 0 hits (or only legacy fallback if absolutely needed); each builder updated |
| C | Existing consumers (extract-arg-names, rename-callable-name, define-alias) still work | their existing tests pass; new test does not regress |
| D | New unit test asserts structured emission for parametric + Tuple + Fn shapes | `cargo test --release -p wat -- signature_of_emits` (or test name) → all pass |
| E | Workspace test failure count ≤ baseline | full workspace cargo test failures ≤ baseline (4 stable + lifeline flake) |

## Honest deltas to capture in SCORE

- Final name picked for the new function (`type_expr_to_holon` vs alternative — note if `/gaze`-style picked something different)
- Which `TypeExpr` variants existed beyond DESIGN's enumeration (if any)
- Did `define-alias` need any consumer-side change? If yes, what + why
- Any places where format_type still flattens (intentionally — diagnostics) vs places where the new emission applies
- Any naming-related ambiguity (e.g., `HolonAST::Atom` constructor vs `Atom` variant — keep consistent with existing nomenclature)

## Time-box

60-90 min predicted. Hard stop 120 min.

## Workspace baseline (commit `90bb496`)

- `cargo build --release --workspace --tests`: clean
- `cargo test --release --workspace --no-fail-fast`: 4 stable failures (lifeline flake + 3 pre-existing)

Post-slice-1 target:
- ≥ baseline + 1-3 new passes (structured-emission tests)
- ≤ baseline failures (purely additive)

## On completion

1. Write `docs/arc/2026/05/201-reflection-structured-type-ast/SCORE-SLICE-1.md` per § SCORE methodology + § Honest deltas.
2. Return final summary to orchestrator: rows passed/failed + workspace delta + path to SCORE + any surprises observed + naming decisions + consumer-sweep findings.

You are launching now. T-minus 0.
