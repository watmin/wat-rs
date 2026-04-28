# wat-rs arc 071 — Parametric built-in enum constructors must carry their type — INSCRIPTION

**Status:** shipped 2026-04-27. One commit, ~30 minutes — substrate
fix surfaced by proof 018's first attempt to consume arc 070's
`:wat::eval::walk` from the lab `wat::test! {}` harness.

Builder direction (2026-04-27, after the lab probe failed where
the substrate unit tests succeeded):

> "if you find a true flaw — we'll construct an arc for it and
>  we'll get infra to fix it"

> "we haven't shipped a bug /in a long time/ — this is atypical
>  for us"

The flaw is real. The diagnosis in the DESIGN was partially
wrong; the real mechanism + the test-discipline fix that
prevents recurrence are below.

---

## The actual mechanism (vs. the DESIGN's diagnosis)

The DESIGN named "register_enum_methods isn't called on the lab
harness path" as the bug. Reading the substrate confirmed:
`freeze.rs` (which the lab harness reaches via
`startup_from_source`) already calls `register_enum_methods`. The
unit-test `stdlib_loaded()` calls it too. Both paths register.

The actual bug was inside `register_enum_methods` (and
`register_struct_methods`, by symmetry):

```rust
let enum_type = crate::types::TypeExpr::Path(enum_def.name.clone());
//                                ^^^^^^^^^^
// Bare path. For monomorphic enums this is correct.
// For parametric enums (`WalkStep<A>`), this drops <A>.

let func = Function {
    type_params: enum_def.type_params.clone(),  // ["A"]
    ret_type: enum_type.clone(),                // :wat::eval::WalkStep — WRONG
    ...
};
```

The synthesized constructor's `type_params` field correctly
declared `["A"]`, but the `ret_type` was a bare `Path` — so the
type checker saw the body produce `:wat::eval::WalkStep` and
rejected against the declared signature `:wat::eval::WalkStep<i64>`.

The fix is one helper:

```rust
fn parametric_decl_type(name: &str, type_params: &[String]) -> TypeExpr {
    if type_params.is_empty() {
        TypeExpr::Path(name.into())
    } else {
        TypeExpr::Parametric {
            head: name.trim_start_matches(':').into(),
            args: type_params.iter()
                .map(|p| TypeExpr::Path(p.clone()))
                .collect(),
        }
    }
}
```

Used at both `register_struct_methods` and `register_enum_methods`
where they previously built bare `TypeExpr::Path` for the decl's
own type. Now monomorphic decls (every existing built-in struct
and enum) get the same `Path` they had before; parametric decls
get the correct `Parametric` head + args.

`WalkStep<A>` is the first parametric built-in enum the substrate
has shipped — that's why the bug surfaced now and not earlier.

---

## Why the substrate's own tests passed

Arc 070's `walk_w1` / `walk_w2` / `walk_w3` / `walk_w4` tests live
in `runtime.rs::mod tests` and run through the local `run` helper:

```rust
fn run(src: &str) -> Result<Value, RuntimeError> {
    let (stdlib_sym, stdlib_macros) = stdlib_loaded();
    // ... parse + macro-expand + register_defines ...
    eval(form, &env, &sym)?
}
```

**`run` does not call `check_program`.** It parses, expands
macros, registers user defines, and evaluates. The type checker
is bypassed entirely.

So arc 070's walk tests verified runtime behavior — `walk` correctly
iterates, the visitor is called per coordinate, `Skip` short-
circuits, etc. — without ever triggering the type-check that
would have caught the bare-path return type.

The lab harness goes through the full `startup_from_source` →
`startup_from_forms_post_config` pipeline, which DOES call
`check_program` (line 580 in freeze.rs). The lab caught what the
substrate's own tests couldn't.

---

## The discipline that eliminates the failure mode

This is why the user said "we haven't shipped a bug in a long time
— this is atypical." The defense that's been working:

**Bug arc 064 → 069 → 070's pattern was: ship a substrate
addition with substrate tests + a wat-tests-integ probe. Both
tiers must pass before commit.** Arc 070 had wat-tests probes
(none for `walk` parametric typing), but the lab integration
test was deferred. That deferral is the gap.

**Arc 071 closes the gap with an explicit substrate-tier discipline:**

> Any new substrate surface that touches **type-checking** —
> parametric built-ins, new variant constructors, new type schemes,
> Lab-visible polymorphism — needs at least one regression test
> that goes through `startup_from_source` (not just `run`). The
> pipeline checker MUST be exercised on a use-site. Same shape the
> existing `tests/wat_user_enums.rs`, `tests/wat_typed_if_match.rs`,
> etc. use.

The new file `tests/wat_parametric_enum_typecheck.rs` is the model:
three tests, each exercising `WalkStep::Continue`, `WalkStep::Skip`,
and the full walker pattern at use sites that go through
`check_program`. Pre-arc-071 these tests fail with the same error
the lab probe shows; post-fix all three pass.

**Going forward, the discipline applies to every new parametric
type:** when adding a parametric struct, parametric enum, or new
type scheme with `type_params`, write at least one
`startup_from_source`-driven test that exercises the type checker.
The substrate's local `run` helper is fine for runtime semantics;
but anything visible through the type system needs the full
pipeline.

---

## What shipped

| Slice | Module | LOC | Tests | Status |
|-------|--------|-----|-------|--------|
| 1 | `src/runtime.rs` — new private helper `parametric_decl_type(name, type_params) -> TypeExpr` that returns `Path` for monomorphic decls and `Parametric { head, args }` for parametric ones. `register_struct_methods` and `register_enum_methods` both updated to use it where they previously built bare `TypeExpr::Path`. | ~25 Rust | 3 new (`tests/wat_parametric_enum_typecheck.rs`: `walkstep_continue_parametric_inference_at_use_site`, `walkstep_skip_parametric_inference_at_use_site`, `walk_visitor_signature_matches_at_use_site`) | shipped |

**wat-rs unit-test count: 730 → 730. Workspace: 1093 → 1096
(+3 regression tests). All passing.**

Build: `cargo build --lib` clean. `cargo clippy --lib`: zero
warnings.

---

## What this unblocks

- **Proof 018's walker rewrite** — the parametric-typing block on
  `walk` is gone. The remaining `:wat::core::second: parameter #1
  expects tuple or Vec<T>; got :?N` diagnostic the proof sees is a
  separate type-inference issue specific to how the proof
  destructures the `Result<(HolonAST, A), EvalError>` return —
  proof-side fix, not substrate.
- **Lab umbrella 059 slice 1's L1+L2 cache** — can now consume
  `walk` directly per arc 070's USER-GUIDE example.
- **Future parametric built-ins** — anything that adds a
  parametric struct or enum to the substrate now produces a
  type-checkable constructor without per-arc plumbing.

---

## What this arc deliberately did NOT do

- **Factor the SymbolTable build path into one canonical fn** —
  DESIGN's open question 3. Reading the code clarified there isn't
  meaningful divergence: `freeze.rs` is the canonical pipeline and
  the lab harness reaches it via `startup_from_source`. The
  unit-test path (`runtime.rs::run`) deliberately bypasses
  type-checking for runtime-only tests; that's a feature, not a
  drift to factor away. The discipline change above is what
  eliminates the parity failure mode.
- **Add type-check enforcement to `runtime.rs::run`** — would
  break existing dynamic-eval tests that intentionally use forms
  that don't type-check. Keep `run` as the runtime-semantics test
  helper; new type-system tests use `startup_from_source` per the
  discipline.
- **Audit other parametric-friendly code paths** — `register_
  newtype_methods` may have the same shape but newtype's parametric
  story isn't exercised; defer until a parametric newtype actually
  surfaces.

---

## The thread

- **Arc 070** — shipped `:wat::eval::WalkStep<A>` (first parametric
  built-in enum). Substrate unit tests passed; lab harness was
  not exercised.
- **Proof 018 walker rewrite (2026-04-27)** — first lab consumer
  of arc 070. Surfaced the bug.
- **DESIGN drafted (2026-04-27)** — diagnosis named the wrong
  mechanism (missing call site) but framed the fix correctly
  (lab harness needs working parametric variant constructors).
- **Arc 071 (this)** — the actual mechanism: bare-path return
  type in synthesized variant constructors, fixed via
  `parametric_decl_type`. Plus the test-coverage discipline that
  catches this class of bug going forward.
- **Next** — proof 018 finishes its walker rewrite (proof-side
  fix on the remaining destructure inference). Lab umbrella
  059 slice 1 starts using `walk`.

PERSEVERARE.
