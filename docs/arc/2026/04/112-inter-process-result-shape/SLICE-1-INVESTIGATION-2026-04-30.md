# Arc 112 slice 1 — investigation note (2026-04-30)

Investigation of the substrate's phantom-type-param support before
implementing slice 1. **Findings recorded; implementation reverted to
clean baseline (`fe84bc5` wat-rs HEAD).**

## What we confirmed

1. **Phantom type params are doctrinally supported.** The arc-071
   comment at `src/runtime.rs:1213-1220` explicitly anticipates them:
   *"No parametric built-in structs exist today, but user-declared
   parametrics get synthesized through the same machinery."*

2. **`parametric_decl_type(name, type_params)` at
   `src/runtime.rs:1186`** builds the canonical
   `TypeExpr::Parametric { head, args }` for a struct with
   type_params. Phantom case (params not in fields) handled by the
   same code path.

3. **`StructDef::type_params` is just `Vec<String>`** — no
   constraint that params must appear in fields.

4. **`register_struct_methods`** auto-synthesizes
   `Type/new` and per-field accessors with
   `type_params: struct_def.type_params.clone()` and the parametric
   `struct_type`. Phantom case works mechanically.

5. **An empty probe with `(:my::go (p :wat::kernel::Process<i64,i64>)
   -> :())` parsed and type-checked clean** — the lexer and parser
   handle nested-angle parametric types fine.

## What didn't work in the spike

I edited:
- `src/types.rs` — `Process` and `ForkedChild` StructDefs gained
  `type_params: vec!["I".into(), "O".into()]`
- `src/check.rs` — `fork-program`/`fork-program-ast`/
  `spawn-program`/`spawn-program-ast` schemes: `type_params: vec!["I",
  "O"]`, return type `Parametric { head: "wat::kernel::Process",
  args: [Path(":I"), Path(":O")] }` wrapped in `Result<...,
  StartupError>`. Plus four new schemes for `process-send` /
  `process-recv` / `fork-send` / `fork-recv`.
- `src/check.rs::validate_comm_positions` — extended to permit
  the four new comm verbs.

The substrate built clean. Probe wat file:

```scheme
((sr :Result<wat::kernel::Process<i64,i64>,wat::kernel::StartupError>)
  (:wat::kernel::spawn-program "()" :None))
```

failed with:

```
:wat::core::let*: parameter binding 'sr' expects
  :Result<wat::kernel::Process<i64,i64>,wat::kernel::StartupError>;
got
  :Result<wat::kernel::Process,wat::kernel::StartupError>
```

The "got" type is missing the inner `<i64,i64>` — `Process` appears
as `Path` rather than `Parametric` somewhere in the inferred return
chain. **I traced through `instantiate`, `rename`, `unify`,
`format_type`, `apply_subst`, `expand_alias`, `reduce` — every one
should preserve `Parametric` structure recursively.** I could not
locate the path that strips the args.

A control probe with the existing parametric scheme `eval-ast!`
(returns `Result<T, EvalError>`) worked fine. So existing parametric
returns work; my new shape doesn't. The difference may be in the
NESTED parametric (`Process<I, O>` inside `Result<...>`) — but
`eval-ast!`'s ret is already nested (`Result<T, ...>`), so that
doesn't fully explain it.

## Hypothesis

The most likely failure modes:

1. **Subtle scheme registration ordering** — maybe the auto-generated
   `Process/new` constructor (registered AFTER the manual scheme via
   `register_struct_methods`) is binding a different shape that
   overrides at lookup time. Worth checking — `register_struct_methods`
   at `runtime.rs:1200` runs at freeze time after manual registration.

2. **A canonicalization pass** between scheme lookup and unification
   I haven't found, possibly in `freeze.rs` or in the dispatch chain
   for keyword-headed calls in `infer_list`.

3. **A `from_symbols`-vs-`with_builtins` ordering issue** —
   `CheckEnv::from_symbols` at `check.rs:309` overlays user-define
   schemes; if Process struct's auto-generated `Process/new` derives
   a scheme that gets registered for the BARE `:wat::kernel::Process`
   path (instead of the qualified one), it could collide.

## What slice 1 needs to land

A fresh-context session should:

1. **Add temporary `eprintln!` instrumentation** in
   `instantiate` (after rename) and at the spawn-program call's
   `infer_call` return site, dumping the actual TypeExpr structure.
2. **Re-run the probe** — read the actual TypeExpr that's
   produced. The exact divergence will name the bug.
3. **Fix at the named layer.**
4. Implement the runtime fns (`eval_kernel_process_send`,
   `eval_kernel_process_recv`, plus fork-* siblings).
5. Sweep dispatcher demo + ping-pong proof + relevant docs.
6. Move `/tmp/arc112-probe*.wat` to `wat-tests/kernel/process-comm.wat`
   as a deftest.
7. Commit + push.

Estimated effort: 2-3 hours focused, assuming the diagnostic
instrumentation finds the bug quickly. Could be 1 hour if it's an
obvious scheme-collision issue; could be longer if it requires
substrate machinery changes.

## What stayed clean

- DESIGN.md (committed at `5159f92`) is correct.
- arc-110's existing grammar walk untouched.
- arc-111's structural shipment unaffected.
- Both repos green at the pre-investigation HEAD.

## Next session entry point

Read this file. Re-create the substrate edits from this doc's "What
didn't work" section. Add the eprintln instrumentation. Run the
probes at `/tmp/arc112-probe*.wat`. The diagnostic will name the
real fix path.
