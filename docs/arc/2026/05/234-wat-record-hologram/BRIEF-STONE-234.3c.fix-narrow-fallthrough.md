# BRIEF — Stone 234.3c.fix-narrow-fallthrough

**Status:** READY TO SPAWN.

## What to do

Narrow the check.rs fall-through at `src/check.rs` line 5906-5908 (added by Stone 234.3c). Currently returns polymorphic T for ANY unknown-verb 1-arg keyword call. Should ONLY fire when receiver type is record/struct/HashMap (or unresolved type var).

ONE file: `src/check.rs`.

## Read in order

1. `docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.3c.fix-narrow-fallthrough.md` — 8 locked decisions + 8 trap-doors
2. `docs/arc/2026/05/234-wat-record-hologram/EXPECTATIONS-STONE-234.3c.fix-narrow-fallthrough.md` — scorecard
3. `tests/probe_arc234_stone3c_fix_narrow_fallthrough.rs` — load-bearing test
4. `src/check.rs` line 5896-5910 — the fall-through site (Stone 234.3c marker comment + over-permissive arm)

## Implementation

Replace lines 5906-5908 with type-discriminated fall-through:

```rust
// Arc 234 Stone 234.3c.fix-narrow-fallthrough — narrow the polymorphic-T
// return to only fire when receiver type is record/struct/HashMap (or
// unresolved). Concrete types like :i64 fall through to UnknownFunction
// at check time, preventing cascaded runtime type-confusion.
if args.len() == 1 {
    let receiver_ty = infer(&args[0], env, locals, fresh, subst, errors);
    let resolved = receiver_ty.map(|t| apply_subst(&t, subst));
    let acceptable = match &resolved {
        None => true,                                            // inference failed; let it through
        Some(TypeExpr::Var(_)) => true,                          // unresolved polymorphic; runtime decides
        Some(TypeExpr::Path(p)) if p == ":wat::Record" => true,
        Some(TypeExpr::Path(p)) if env.types.is_struct(p) => true,
        Some(TypeExpr::Parametric { head, .. }) if head == "wat::core::HashMap" => true,
        Some(_) => false,                                        // concrete non-accessor type; UnknownFunction
    };
    if acceptable {
        return Some(fresh.fresh());
    }
}
return None;  // falls through to existing UnknownFunction
```

NOTE: this is a SKETCH. Sonnet investigates:
- Exact `apply_subst` helper name + signature
- Exact predicate for "is registered struct" (`env.types.is_struct` is a guess; locate via grep)
- Whether the existing `for arg in args { infer... }` loop at line 5893-5895 should be folded with this (don't double-infer args[0])

The args inference loop at 5893-5895 already does `for arg in args { let _ = infer(arg, ...); }`. Capture args[0]'s result instead of discarding it, OR call infer ONCE on args[0] separately if cleaner.

## Discipline

- src/check.rs ONLY (STOP-5)
- DO NOT touch: probes (except writing the new probe; that's orchestrator-side, already on disk), wat sources, prior SCORE docs, runtime.rs, parser.rs, holon-rs (STOP-4)
- DO NOT commit (orchestrator atomic-commits)
- DO NOT change runtime dispatch (the runtime side is correct; only check.rs is over-permissive)
- DO NOT add new error variants (UnknownFunction already exists)

## STOP triggers (REJECTION)

1. unexpected compile errors
2. lib baseline < 827
3. 60 min elapsed
4. holon-rs touched
5. Rust changes outside check.rs
6. scope creep
7. new probe doesn't flip 4/4 PASS (or 3/4 with NAMED deferral for probe 4 if hard to construct)
8. 234.3c regression
9. 234.4 regression
10. any prior arc 234 regression
11. clippy > 54

If lib tests reveal consumer reliance on the over-permissiveness: REPORT IT. Don't auto-fix consumer tests. Orchestrator decides whether to update tests or revise the narrowing.

## SCORE doc

`docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.3c.fix-narrow-fallthrough.md` (NEW). Capture: implementation pattern; whether lib tests surfaced consumer reliance; cascade depth.
