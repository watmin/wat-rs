# BRIEF — 259 S1: expose `closure_extract` as the wat verb `:wat::kernel::fn-forms`

**The work (one paragraph).** Add a wat-callable verb `:wat::kernel::fn-forms` that reifies a fn
value into shippable forms — the wat surface of the existing (currently caller-less) Rust
`closure_extract::extract_closure`. It takes a fn and a bind-name; it returns a `Vector<WatAST>` of
forms that, run in a *fresh* universe (a forked not-shared child), `(def <name> <the-fn>)` together
with the fn's transitive deps — with the `ImpureCapture` portability check surfaced as a wat error.
This is the missing piece that lets the not-shared bracket path ship an anonymous/named work-fn
across a fork (proven needed: `scratchpad/probe-child-inherits-defns.wat` — the child is a fresh
universe; `scratchpad/probe-bracket-closure-seam.wat` — `spawn-process` takes forms, not a closure).

**The one contract decision (pinned).**
```
(:wat::kernel::fn-forms [f <- <the fn>  name <- :wat::core::keyword] -> :wat::core::Vector<wat::WatAST>)
```
- Internally: `extract_closure(f_value, /*entry_name*/ None, parent_symbols, parent_types)` →
  `ClosurePackage { prologue, entry_form }`. Use the **inline-lambda path (`None`)** uniformly — it
  reconstructs the fn-form from the value AND walks the body for deps, so it works for an anonymous
  block *and* a named fn passed by reference (both arrive as a resolved `Value::wat__core__fn`).
- Return value: **`prologue ++ [(:wat::core::def <name> <entry_form>)]`** as a wat `Vector<WatAST>`.
  The child universe then resolves `<name>` (the deps + the def are all present).
- On `ExtractionError::ImpureCapture` → a wat runtime error carrying the capture's name + type
  (the structural "can this be EDN?" gate — impure captures cannot cross). `UnresolvedSymbol` /
  `Internal` → honest wat errors too.

**Read in order (rooms):**
- `src/closure_extract.rs:157` — `extract_closure(fn_value, entry_name: Option<&str>, parent_symbols: &SymbolTable, parent_types: &TypeEnv) -> Result<ClosurePackage, ExtractionError>`. The API you front. `ClosurePackage{prologue: Vec<WatAST>, entry_form: WatAST}` at :61; `ExtractionErrorKind::ImpureCapture{name,type_name,path}` at :84.
- `src/runtime.rs:5089` — the `":wat::kernel::spawn-process" => eval_kernel_spawn_process(args, list_span, env, sym)` dispatch arm. **Copy this shape** for the `:wat::kernel::fn-forms` arm (it hands you `env`/`sym` = the SymbolTable + the way to reach the TypeEnv).
- `src/process/verbs.rs:652` — `eval_kernel_spawn_process(...)` — the model for a kernel verb that evals its args + reaches the parent world. Copy how it gets `parent_symbols`/`parent_types` and how it builds a wat value out of `Vec<WatAST>` (a `:wat::core::Vector<wat::WatAST>` value).
- The builtin/type registration site for kernel verbs (grep where `spawn-process` / a fn-taking verb registers its checker signature) — `fn-forms` must type-check: `f` accepts **any** `Fn`, `name` is `:wat::core::keyword`, returns `:wat::core::Vector<wat::WatAST>`. Mirror how an existing fn-taking kernel verb is typed.

**Implementation sketch:**
```
// runtime.rs dispatch:
":wat::kernel::fn-forms" => eval_kernel_fn_forms(args, list_span, env, sym).map_err(Into::into),

// new eval_kernel_fn_forms (closure_extract.rs or process/verbs.rs):
//   1. expect 2 args: eval arg0 -> Value::wat__core__fn (else type error);
//      arg1 is a keyword literal -> the bind name string.
//   2. let pkg = extract_closure(&fn_value, None, sym, &types)?;   // types from env/sym
//   3. let mut forms = pkg.prologue;
//      forms.push(WatAST::list([:wat::core::def, keyword(name), pkg.entry_form]));  // the def-binding
//   4. wrap forms as a Vector<WatAST> wat Value; return it.
//   ImpureCapture/others -> map to a wat RuntimeError.
```

**Blast radius:** `src/runtime.rs` (one dispatch arm) + a new `eval_kernel_fn_forms` fn + the checker
signature registration for `:wat::kernel::fn-forms`. **Do NOT** modify `closure_extract::extract_closure`
itself, or the spawn/bracket code. No new deps.

**STOP triggers (rejection criteria — ship nothing, report the gap):**
- **STOP-1:** if `extract_closure` with `entry_name = None` does NOT reconstruct the fn-form + deps for
  a plain `Value::wat__core__fn` (e.g. it needs a name, or errors on an anon lambda) — STOP, report
  exactly what it needs. Do not invent a workaround.
- **STOP-2:** if the checker cannot type a verb whose first param is "any `Fn`" without a new
  type-system feature — STOP, report the gap (it is a substrate prereq, not this stone's scope).
- **STOP-3:** if reaching the `TypeEnv` from the verb's dispatch context isn't already how
  `eval_kernel_spawn_process` does it — STOP, report (don't thread a new param through the world).

**The gate (RED → GREEN):** `scratchpad/probe-s1-fn-forms.wat`. Today it fails
`UnknownFunction: :wat::kernel::fn-forms` (line 15) with everything else clean. When S1 lands it must
print **`"6 10"`** — the anon block reified by `fn-forms`, shipped to a process worker, streamed. Also
run the full floor (`cargo nextest run --release`) — **0 new failures** (the pre-existing
`no_inlined_wat` lint is the only known red).

**Report:** the diff scope, the probe output (`"6 10"` or the gap), the floor delta, and any STOP hit.
Weigh nothing by claim — the orchestrator re-runs the probe + floor independently.
