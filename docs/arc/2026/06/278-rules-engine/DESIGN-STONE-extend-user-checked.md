# DESIGN — substrate stone: USER `extend-type` impl bodies must be type-checked

> **Mandate (the builder):** *"extend-type allows lies — we have made core truthful — users must be forced into honesty."*

## Why

`extend-type` is THE satisfier construct (arc 232/293): every `Store`/`ReadStore` impl, every surface a user
implements. Core is truthful by construction — `defn`, records, surface *declarations*, and surface-method *call
sites* are all type-checked. The ONE hole: a **user** `extend-type` impl **body** is never checked against the
surface it claims to satisfy. A satisfier can produce the wrong type, call an unknown function, or misuse its args,
and it compiles + runs. That is a magic-free-floor (R3) doctrine violation at a load-bearing construct.

**Proven** (`tests/types/probe_arc278_extend_user_body_checked.wat.bad`, RED gate `83a8d145`): a user impl
`(emit [self x] "i am a string")` against surface `emit … -> :i64` freezes clean at HEAD. Also confirmed by probe:
a body calling a non-existent fn reaches **runtime** `UnknownFunction`, not a compile error — the whole body is
invisible to the checker, not just the return type.

## The mechanism (grounded, file:line)

- `check_program` sweeps function bodies via `check_function_body` at **check.rs:826** (`for (path, func) in
  &sym.functions { … }`) — it checks every body in `sym.functions` against its scheme.
- **Baked** extend-type impls register into `sym.functions` at build_env **step 7.6**
  (`register_stdlib_runtime_defs`, runtime.rs:650) — BEFORE check_program (freeze.rs:816). So they ARE swept.
  (This is why mem.wat's nil impls failed `ReturnTypeMismatch` and needed the `b441c6bf` surface-inheritance fix.)
- **User** extend-type impls register into `sym.functions` only at **step 9**
  (`register_runtime_defs_form` surface arm, runtime.rs:1936) — AFTER check_program (freeze.rs:465). The baked-fix
  author documented this exactly (runtime.rs:698–701): *"the user-source equivalent … whose Functions only reach
  `sym.functions` at freeze time, AFTER `check_program` already ran."* Never swept.
- The user arm (runtime.rs:1936) also builds from `clause.return_type` / `clause.args.fixed_params` — the **nil
  placeholders** from the pure `parse_extend_type_form`. So even hoisted, it would register nil-typed sigs and
  check nothing meaningful. The baked arm (runtime.rs:676–766) already fixed this: it inherits the real per-method
  sig from the surface's `SurfaceMember::Method { args, ret }`, with `self` (fixed_params[0]) typed as the concrete
  satisfier.

## Scope (grounded — one class, one fix)

- extend-type impls are **always bare-param**: annotating an impl arg errors *"method impl arg must be a bare
  Symbol (no type annotation in extend-type)."* So a user cannot *declare* a mismatched sig — the sig is always
  inherited from the surface. There is exactly ONE lie to police: **the impl body vs the inherited sig.**
- Closing it closes all three faces (wrong return, unknown call in body, arg misuse) — they all fall through to
  runtime today and all become compile errors once the body is swept.

## The one contract decision

**Extract the surface-inheriting extend-type registration into ONE shared routine; run it for USER surface
impls BEFORE `check_program`, mirroring the baked path — do NOT keep a second copy of the inheritance logic.**

A second copy of "inherit sigs from `SurfaceMember::Method`" is exactly the dual-impl-drift class we keep killing
(the baked/user arms already drifted — that drift IS this bug). One routine, two call sites (stdlib 7.6, user
7.7), the step-9 surface arm yields.

Rejected: (a) a naive freeze-order hoist of all of step 9 — **wrong**, `register_runtime_defs` "must run AFTER all
capability carriers are installed" (freeze.rs:460); only the surface-impl *registration* (carrier-free) moves, not
the `def` value-eval. (b) A second parallel copy of the inheritance logic — re-opens the drift. (c) Weakening the
check to `Infer`/`Any` for impl returns — that ships the lie.

## Files touched (blast radius)

- `src/runtime.rs` — extract `register_extend_type_surface_impls(form, &mut sym)` from the stdlib arm
  (676–766); call it from `register_stdlib_runtime_defs`; make the step-9 `register_runtime_defs_form` **surface**
  arm (1936–1965) yield when the impl key is already registered (protocol arm unchanged).
- `src/freeze/env.rs` — new step ~7.7: filter `residue` for user `extend-type` forms and register their surface
  impls via the shared routine, before `build_env` returns (⇒ before check_program).
- `tests/types/probe_arc278_extend_user_body_checked.rs` — un-ignore (the RED gate → GREEN).

No new params threaded through call sites (STOP-CASCADE). No change to `parse_extend_type_form`'s arity.

## Landed

The strike surfaced a **THIRD** drifting copy of "inherit sigs from the surface" — worse than the two mapped
above. `collect_splice_defs_ctx` (check.rs:8912) re-registered each `:<T>/<method>` as a **nil**-typed
`TypeScheme` in the sequential form loop, which runs *after* `CheckEnv::from_symbols` already populated the same
key with the correct surface-inherited scheme — so it **overwrote correct-with-nil**. Inert before this stone (user
impls weren't in `sym.functions` yet, so nothing to clobber); step 7.7 made `sym.functions` correct for the first
time, exposing the clobber (`query_contract` went red: *"signature declares nil"*). Fixed: the check.rs surface
branch is now a no-op (defers to the single source `sym.functions`); the protocol edge is unchanged. So the fix
collapses THREE copies to one routine — the doctrine landing exactly: one source of truth, the rest yield.

Verdict (weighed by the orchestrator's own re-run): the RED gate is GREEN (`bad.wat` → `ReturnTypeMismatch`
i64/String at check); `query_contract` green; whole floor `4115 run: 4114 passed, 1 failed` — the one failure the
pre-existing `no_inlined_wat_in_tests` reminder (baseline 4113→4114; the +1 is exactly the un-ignored gate; zero
new failures). Files: `src/runtime.rs` (the shared routine + two arms), `src/freeze/env.rs` (step 7.7),
`src/check.rs` (the third copy → no-op), the un-ignored gate.

## The cascade is expected (FM 15)

~20 existing **user** extend-type fixtures (`tests/types/probe_arc293_*`, `probe_arc267_*`,
`tests/rete/probe_arc278_query_contract.wat`, …) have impl bodies that were NEVER checked. The fix sweeps them.
Any that go red are the flaw being caught, not a regression: **ground each** — a genuine wrong impl → fix the
fixture's body; a false positive (self-type / generic-surface edge) → fix the routine. NEVER loosen the check to
pass a genuine lie (STOP-CHECK-WEAKEN). `probe_arc293_4e_pre_iii_extend_impl_inherits_types.wat` (inheritance) and
the S0 `query_contract` (user ReadStore extend-type) are the load-bearing regression guards.
