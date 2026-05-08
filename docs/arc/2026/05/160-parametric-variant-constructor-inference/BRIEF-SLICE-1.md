# Arc 160 — Slice 1 BRIEF (diagnostic)

**Drafted 2026-05-07.** Slice 1 of arc 160 — find the substrate step
that loses parametric type info for variant-constructor-typed let
bindings.

## Working directory

`/home/watmin/work/holon/wat-rs` on branch
`arc-159-wip-substrate-and-sweep`.

## Workspace state pre-spawn

- HEAD: `80ebca6` (arc 160 DESIGN shipped on WIP branch)
- Working tree clean
- 9 currently-failing tests, ALL share the same root cause —
  variant-constructor-typed let bindings produce `?fresh` payload
  positions where they should produce concrete types

Sample failing test (run to reproduce):
```bash
cargo test --release --test wat_recursive_patterns literal_fallback_to_general_arm 2>&1 | tail -10
```

Expected error:
```
thread '...' panicked at '...': startup: Check(CheckErrors([MalformedForm {
    head: ":wat::core::match",
    reason: "int literal pattern in :?29 position",
    span: ...
}]))
```

## Your task — DIAGNOSTIC ONLY (no fix in this slice)

Find the EXACT substrate step that loses parametric type info between:

**Step A — Constructor inference** (`src/check.rs` line 4426-4469):

```rust
if head_is_ok_fqdn {
    let t_ty = infer(&args[0], env, locals, fresh, subst, errors)
        .unwrap_or_else(|| fresh.fresh());
    let e_var = fresh.fresh();
    return Some(TypeExpr::Parametric {
        head: "Result".into(),
        args: vec![t_ty, e_var],
    });
}
```

**Step D — Match arm IntLit pattern check** (`src/check.rs` line 5384-5396):

```rust
WatAST::IntLit(_, _) => match expected_ty {
    TypeExpr::Path(p) if p == ":i64" => Some(false),
    other => {
        errors.push(CheckError::MalformedForm {
            head: ":wat::core::match".into(),
            reason: format!(
                "int literal pattern in {} position",
                format_type(other)
            ),
            ...
        });
        None
    }
},
```

The question: between A returning `Parametric { head: "Result", args: [Path(":i64"), Var(?_e)] }` and D's `expected_ty` being `Var(?_29)` (a fresh var, NOT `:i64`), WHICH intermediate step is responsible?

## Diagnostic walk (the chain)

Walk the type-info flow in order:

### B — `process_let_binding` (`src/check.rs` line 7570)

For new shape `(name rhs)`:
- Calls `infer(rhs, ...)` — receives the Result type from step A
- Inserts into `out_scope`

**Diagnostic question for B:** is the type inserted as
`Parametric { head: "Result", args: [Path(":i64"), Var(?)] }`,
or has it been generalized / stripped / converted to a fresh var?

Add temporary `eprintln!("B: rhs_ty = {:?}", rhs_ty)` to confirm
what gets inserted.

### C — Match scrutinee lookup

Match scrutinee inference reads the let-bound name from locals.
Find where match-keyword's inference path looks up the scrutinee
(grep `:wat::core::match` in `infer` body or similar).

**Diagnostic question for C:** when match looks up `resp`'s type,
does it apply current substitution? Does it preserve the
parametric structure? Or does it strip / re-fresh / abstract?

### D — Pattern arm position resolution

Find where match arm pattern position type is computed for Ok in
Result<T, E> (likely the function around line 5258 with
`("Ok", MatchShape::Result(t, _))`).

**Diagnostic question for D:** what is the source of `t` here?
Is it the scrutinee's args[0]? Or a freshly-allocated var? Does
unification with the scrutinee happen before pattern check?

## Constraints

- **DIAGNOSTIC ONLY.** Do NOT fix anything in this slice. Slice 2
  applies the fix once you've identified the lossy step.
- **Read-and-trace.** Add `eprintln!` debug prints if needed; remove
  them before reporting (or note that they're temporary in your report).
- **NO scope creep.** Don't widen the question beyond "which step is
  lossy." Don't propose alternative architectures yet.
- **Time-box: 30 min wall-clock.**

## Pre-flight crawl

1. `docs/arc/2026/05/160-parametric-variant-constructor-inference/DESIGN.md` — full read
2. `docs/arc/2026/05/159-untyped-let-bindings/DESIGN.md` — context on what arc 159 ships
3. `src/check.rs::infer` (find the function; main inference dispatcher)
4. `src/check.rs::process_let_binding` (line 7570)
5. `src/check.rs::check_subpattern` (line 5368)
6. The match-arm position-resolution function (search for `MatchShape::Result`)

## Reporting (~150-200 words)

State precisely:

1. **The lossy step** — which function in src/check.rs is
   responsible, with line number
2. **What the type SHOULD be at that step** — `Result<:i64, ?>` or
   similar
3. **What the type ACTUALLY is at that step** — `?fresh` or
   `Result<?, ?>` or whatever you observed
4. **Why** — what mechanism in the lossy function caused the loss
   (e.g., `apply_subst` not called; `instantiate` introduces fresh
   vars without unifying; whatever)
5. **Proposed fix** — one-paragraph sketch (slice 2 will implement)

DO NOT commit. DO NOT write a SCORE doc. Orchestrator scores after
your diagnostic + slice 2 fix.

If the chain doesn't have a lossy step in A-D (the gap is
elsewhere), report what you found — that's still useful.

## Time-box

30 minutes wall-clock. Wakeup scheduled.

## Why slice 1 standalone

Diagnostic before fix per the proactive stepping-stones discipline.
If slice 1 reveals the gap is shallow (one-line fix at a specific
site), slice 2 is trivial. If it reveals deeper inference work
(Hindley-Milner-grade unification), the orchestrator may escalate
to fresh-opus per user direction 2026-05-07: *"if we need to solve
Hindley-Milner-grade work it may require opus to do it as its
cheaper to have opus solve it instead of sonnet making many
attempts."*
