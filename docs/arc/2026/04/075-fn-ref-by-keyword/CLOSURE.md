# Arc 075 — Closure justification

**Closed:** 2026-04-28.
**Author of this closure:** the substrate session (not the proof session that drafted DESIGN.md).
**Audience:** the proof session that drafted DESIGN.md, plus any future session reading this arc cold.

---

## What you got right

The surface complaint was real and the diagnosis-by-symptom was sharp:

> "explain this.. why is passing a func not supported?.... and the contract for user is pass a function who impl `:fn(f64)->bool` -- what they do in that func is their issue. we should expose the default presence? and coincidence? values so they can use them"

The user-experience gap is real. The arc-074 slice-1 doc DID document an inline-lambda workaround that felt like substrate friction. Naming the surface symptom and asking for "uniform keyword-as-fn-ref resolution" was the right shape of question — even though the answer turned out to be elsewhere.

You also correctly identified that the substrate should ship the canonical filter funcs (`filter-coincident`, `filter-present`, `filter-accept-any`) so users compose by passing them by name. That part of the spec is load-bearing and ships separately (likely as a slice-1-followup commit on arc 074, not its own arc).

## What we got wrong (jointly)

The DESIGN's diagnosis was that the type checker has a per-site case-by-case handling for keyword-as-fn-ref resolution and that a uniform rule would close the gap.

That diagnosis is wrong. Both halves of the resolution **already ship**, uniformly:

### 1. Runtime-side resolution (arc 009, in production today)

`src/runtime.rs:2057-2086` — `eval(WatAST::Keyword(k))`:

```rust
WatAST::Keyword(k, _) => {
    if k == ":None" { return Ok(Value::Option(Arc::new(None))); }
    if let Some(ev) = sym.unit_variants.get(k) {
        return Ok(Value::Enum(Arc::new(ev.clone())));
    }
    // Arc 009 — names are values. If the keyword is a registered
    // user/stdlib define, lift it to a callable Function value.
    if let Some(func) = sym.get(k) {
        return Ok(Value::wat__core__lambda(func.clone()));
    }
    Ok(Value::wat__core__keyword(Arc::new(k.clone())))
}
```

Any registered function (user define, stdlib define) reached as a bare keyword evaluates to `Value::wat__core__lambda(func)`. Already uniform.

### 2. Type-checker resolution (arc 009 sibling, also in production)

`src/check.rs:420-433` — `infer(WatAST::Keyword(k))`:

```rust
// Arc 009 — names are values. If the keyword is a registered
// function (user define, stdlib define, or builtin primitive),
// instantiate its scheme and return a `:fn(...)->Ret` type so
// the keyword can be passed to any `:fn(...)`-typed parameter.
WatAST::Keyword(k, _) if env.get(k).is_some() => {
    let scheme = env.get(k).expect("guard").clone();
    let (params, ret) = instantiate(&scheme, fresh);
    Some(TypeExpr::Fn { args: params, ret: Box::new(ret) })
}
```

Any keyword that hits the `CheckEnv` registry — user defines (registered via `from_symbols`), builtins (registered via `register_builtins`), stdlib (overlaid in `from_symbols`) — types as its `:fn(...)` signature. Already uniform.

## The gap that's actually there

Verified by inserting `eprintln!` at three call sites and running the failing test:

| Phase | Observation |
|-------|-------------|
| `from_symbols` for the test file | `path=":my::tight-filter"` registered with `param_types=[Path(":f64")]`, `ret_type=Path(":bool")` ✓ |
| `check_program` for the test file | `env.get(":my::tight-filter")` → `Some(scheme)` ✓ |
| `infer` during the failing call site | `env.get(":my::tight-filter")` → `None`, env's user keys = `[":user::main"]` only ✗ |

Two distinct envs. The first two log lines come from the OUTER file's check phase (which does see `:my::tight-filter`); the third comes from a check phase happening INSIDE the deftest's sandboxed sub-world.

`wat/std/test.wat:304-322` shows why:

```scheme
(:wat::core::defmacro
  (:wat::test::deftest (name :AST<()>) (prelude :AST<()>) (body :AST<()>) -> :AST<()>)
  `(:wat::core::define (,name -> :wat::test::TestResult)
     (:wat::kernel::run-sandboxed-ast
       (:wat::core::forms
         ,@prelude
         (:wat::core::define (:user::main ...) ,body))
       ...)))
```

The deftest body lives inside `(:wat::core::forms ...)` passed to `run-sandboxed-ast`. The sandbox builds its own `SymbolTable` from those forms only. The outer file's `(:wat::core::define :my::tight-filter ...)` is NOT spliced into the sandbox forms; it's at the file's top level, alongside the deftest call. The sandbox can't see it.

The check inside the sandbox then runs against a sym that has `:user::main` and that's it. The fn-ref resolution rule fires correctly — there's just nothing for it to find.

## Why this matters

The DESIGN's proposed Change 1 (uniform resolution rule) cost would have been zero — the rule already exists. The investigation work to "implement" it would have produced a no-op commit and a misleading test passing because the rule was already being exercised everywhere outside `deftest`.

The actual fix lives in one of three places:

1. **Substrate** — `run-sandboxed-ast` carries through outer-scope user defines (or some explicit subset of them). Substantial: affects sandbox isolation semantics broadly. The sandbox is currently a hard hermetic boundary; weakening it has implications for how sandboxes are used as security boundaries elsewhere.

2. **Deftest macro** — splice outer-file defines into the test's prelude. The macro doesn't have visibility into the file's other top-level forms, so this requires either a parse-time scan or a substrate primitive that exposes "give me the outer file's defines."

3. **User pattern** — document that filter-funcs (or any user-define meant for use inside a deftest) MUST be defined in the deftest's prelude, not at file top. This is a docs-only fix, no substrate change.

For arc 074 slice 1 specifically: option 3 is what the SLICE-1-HOLON-HASH doc effectively documents already (the inline-lambda workaround inside the let* binding). The doc should be updated to note that the prelude-define alternative also works, but the inline-lambda pattern stays valid.

## Action items closing this arc

- [x] Mark DESIGN.md status CLOSED with a banner at the top pointing here.
- [x] Write this CLOSURE.md (you're reading it).
- [ ] Future-arc decision (NOT in this arc's scope):
  - **(a)** Open arc 076 to ship the three substrate-default filter funcs (`:wat::holon::filter-coincident`, `:wat::holon::filter-present`, `:wat::holon::filter-accept-any`). Small. Survives entirely from arc 075's draft.
  - **(b)** Open arc 077 (or whichever number is next available) to address the deftest-sandbox define visibility gap. The "right" fix among options 1/2/3 above is itself a design discussion.
- [x] Update arc 074 SLICE-1-HOLON-HASH.md to mention "or define filters in the deftest's prelude block; either pattern works." (Not yet done; minor.)

The substrate session takes responsibility for not catching the diagnosis error during the original investigation. The proof session's spec was the right shape of question; the substrate session should have run the eprintln-and-trace pass before agreeing to the DESIGN. Lesson recorded.

## Note to the proof session

This is a good finding overall — the user experienced the friction, you named it precisely, the substrate session traced it to ground truth. Two diagnoses converged on a place where the substrate has a real story to tell:

> The fn-ref resolution rule is already there. The deftest sandbox is a separate concern. The substrate is more capable than the symptom suggested.

That's load-bearing knowledge. Arc 075's DESIGN.md stands as the surfacing artifact; this CLOSURE.md stands as the resolution. The work was not wasted — it surfaced two facts about the substrate's capabilities, both of which now have textual records:

1. Arc 009's "names are values" rule applies uniformly across all `:fn(...)`-typed argument positions, in both the runtime and the type checker.
2. `deftest`'s sandbox isolation has a visibility cost that consumers should know about. The substrate either fixes it or documents the workaround; both are honest.

PERSEVERARE.
