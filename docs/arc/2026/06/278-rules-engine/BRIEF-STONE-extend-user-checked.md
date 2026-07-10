# BRIEF — substrate stone: USER `extend-type` impl bodies must be type-checked

> **Executor: one sonnet SHADOWDANCER.** The orchestrator isolated the root + built the RED gate; it weighs the
> kill against its own re-run. Work ONLY in `/home/watmin/work/holon/wat-rs/` (`pwd` first; anchor every `git` with
> `git -C /home/watmin/work/holon/wat-rs`; any path containing `.claude/worktrees/` is illegal harness state — ignore
> it). Dogfood with `cargo wat <file>`; test with `cargo nextest run --release` (NEVER `cargo test`). **Commit
> NOTHING** — the orchestrator weighs and commits.

## The work, in one paragraph

`extend-type` is the satisfier construct, and it is the one place a user can ship a wrong type green: a **user**
extend-type impl **body** is never type-checked against the surface it satisfies. Make it checked, by mirroring the
BAKED path — which already works. The full grounded design is `DESIGN-STONE-extend-user-checked.md`; read it first.
The RED gate `tests/types/probe_arc278_extend_user_body_checked.rs` (`#[ignore]`'d) is the target: un-ignore it, and
your fix turns it GREEN (freeze returns a `ReturnTypeMismatch` for a user impl `emit … "a string"` against surface
`emit … -> :i64`).

## Read the rooms, in order (why you're sent to each)

1. **`src/runtime.rs:676–766`** (`register_stdlib_runtime_defs`, the `":wat::core::extend-type"` surface arm) — the
   REFERENCE. It registers each impl clause as a `:<T>/<method>` `Function` in `sym.functions`, with the sig
   INHERITED from the surface's `SurfaceMember::Method { args, ret }` (`self` = fixed_params[0] typed as the concrete
   `ed.type_name`; other args + ret from the surface member). This is exactly the logic user impls need. It is
   carrier-free (needs only `sym.types`, populated by build_env step 5, and the pure `parse_extend_type_form`).
2. **`src/runtime.rs:1936–1971`** (`register_runtime_defs_form`, the user `extend-type` arm) — the BUG. Surface path
   builds `Function`s from `clause.args.fixed_params` / `clause.return_type` (NIL placeholders from
   `parse_extend_type_form`), inserts into `sym.functions`, errors `DuplicateDefine` (1947) on a second insert. This
   runs at freeze step 9 (see room 4) — after the check sweep — so its impls are never checked, AND they carry nil
   sigs. The protocol path (1966–1970, `runtime_def_values`) is UNCHANGED by this strike.
3. **`src/check.rs:826–830`** (`for (path, func) in &sym.functions { … check_function_body(…) }`) — the sweep that
   must reach user impls. It checks each `sym.functions` body against its scheme (`env.get(path)` from
   `CheckEnv::from_symbols`). Whatever lands in `sym.functions` BEFORE `check_program` gets swept. You do NOT touch
   this — you make user impls present in `sym.functions` in time for it.
4. **`src/freeze.rs:806`→`:816`→`:465`** — the pipeline order: `build_env` (line 806) → `check_program(&bundle.residue,
   …)` (step 8, line 816) → `FrozenWorld::freeze` → `register_runtime_defs` (step 9, line 465). Note the `freeze.rs:460`
   comment: `register_runtime_defs` "must run AFTER all capability carriers are installed" — so you do NOT move step 9
   wholesale; only the surface-impl *registration* (carrier-free) moves earlier.
5. **`src/freeze/env.rs:170–191`** (stdlib runtime-def filter) + **`:266`** (the 7.6 call) + **`:192`** (`residue` =
   the user forms) — where you add the new user-side pre-check registration (step ~7.7), before `build_env` returns.

## Implementation sketch (fill it; do not invent the shape)

```rust
// runtime.rs — extract the surface-inheriting registration (from the 676–766 arm):
pub fn register_extend_type_surface_impls(form: &WatAST, sym: &mut SymbolTable)
    -> Result<(), EvalBreak /* match the arm's existing error type */> {
    let (_canonical_key, ed) = parse_extend_type_form(form)?;
    // look up ed.protocol_name in sym.types; if TypeDef::Surface(s) → for each impl clause,
    //   inherit (param_types, ret) from the matching SurfaceMember::Method (self = ed.type_name),
    //   build the Function, insert under "<ed.type_name>/<method>".
    //   IDEMPOTENT: if the key is already present, SKIP (do not error) — user + step-9 both reach it.
    // if NOT a surface → do nothing here (protocol path stays in register_runtime_defs_form).
    Ok(())
}
// register_stdlib_runtime_defs' extend-type arm → call register_extend_type_surface_impls.
// register_runtime_defs_form's extend-type SURFACE arm (1936–1965) → call the SAME routine
//   (now idempotent, so no DuplicateDefine when env.rs pre-registered it). Protocol arm unchanged.
```

```rust
// env.rs — new step 7.7, AFTER 7.6 (register_stdlib_runtime_defs at :266), before `Ok(EnvBundle…)`:
for form in &residue {
    // filter user extend-type forms (mirror the 170–191 matches! on ":wat::core::extend-type")
    register_extend_type_surface_impls(form, &mut symbols)
        .map_err(/* → StartupError::Runtime, mirror 7.6 */)?;
}
```

Then un-ignore `tests/types/probe_arc278_extend_user_body_checked.rs` (delete the `#[ignore …]` + the
`⛔ IGNORE-LEDGER` comment).

## The cascade is the progress meter (expected — do NOT panic or revert)

Run `cargo nextest run --release` after the fix. ~20 existing USER extend-type fixtures
(`tests/types/probe_arc293_*`, `probe_arc267_parametric_extend_type`, `tests/rete/probe_arc278_query_contract`, …)
have impl bodies that were NEVER checked; the sweep now reaches them. Each red is the flaw being caught. **Ground
each one:** read the impl body against its surface. If the body is genuinely wrong (a latent lie) → fix the
fixture's body to satisfy the surface. If the check is a false positive (a self-type / generic-surface edge the
inheritance mis-handles) → fix the routine. Report every red you touched, with which disposition and why.

## STOP triggers (rejection criteria — halt + report, do not improvise)

1. **STOP-CHECK-WEAKEN:** never make a genuinely-wrong impl pass — no defaulting impl returns to `Infer`/`Any`, no
   skipping the body check. The impl must inherit the REAL surface sig and be checked against it. If you cannot make
   a fixture green without weakening the check, STOP and report it (it may be a genuine lie to fix, not a check to
   loosen).
2. **STOP-CASCADE:** if the fix wants to change a function signature threaded through many call sites, STOP — the
   registration rides `&mut symbols`, already in scope; a new threaded param is the wrong direction (this exact
   mistake was paused on the reserved-privilege stone).
3. **STOP-PROTOCOL-REGRESSION:** the PROTOCOL path of extend-type (non-surface targets → `runtime_def_values`) must
   behave identically. If your change alters protocol-path behavior, STOP.
4. **STOP-DISPATCH:** runtime surface-method dispatch (the `:<T>/<method>` keys, 293.4b dispatcher) must still work —
   the S0 `query_contract` test both type-checks AND dispatches. If dispatch breaks, STOP.

## The gate (EXPECTATIONS)

| what | command | expected |
|---|---|---|
| RED gate flips GREEN | `cargo nextest run --release -p wat --test types -E 'test(user_extend_type_wrong_return_rejected)'` | passed (un-ignored) |
| the wrong-typed impl is a compile error | `cargo wat tests/types/probe_arc278_extend_user_body_checked.wat.bad` | `ReturnTypeMismatch` at check |
| baked satisfiers still green | `cargo nextest run --release -E 'test(query_contract)'` | passed |
| the inheritance regression guard | `cargo nextest run --release -p wat --test types` | 0 failed (modulo grounded cascade fixes) |
| whole floor | `cargo nextest run --release` | 0 failed (report the Summary line verbatim; ground every red) |

## Prior comparable to copy for shape

`BRIEF-STONE-extend-baked-inheritance.md` + its landed fix `b441c6bf` (the surface-inheritance logic you're
extracting) and `f60cd639` (the surgical, no-cascade registry fix — the shape of "ride the reference already
threaded, don't add a param").
