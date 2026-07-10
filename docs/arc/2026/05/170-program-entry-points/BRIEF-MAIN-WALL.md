# BRIEF — the `:user::main` wall (zero useless / illegal mains, EVER)

> **The mandate (builder, verbatim, this session):** *"I never want to see a fucking useless main
> again."* This has recurred ~5× in a month because the wall was DESIGNED twice and never IMPOSED.
> Impose it now, permanently, at FREEZE — so a useless/illegal main becomes **uncompilable** and no
> sonnet can ever ship one. The compiler says "no" from now on; the human never has to again.

## The work, one paragraph

A declared `:user::main` must be EXACTLY `[] -> :wat::core::nil` **and** its body must NOT be the bare
`nil` literal (UselessMain — "do something or omit the main"). Both are FREEZE errors. Impose them
conditionally (only when `:user::main` is defined) in `startup_from_source`. The `~35` illegal-signature
mains + the bare-`nil` mains then self-identify as freeze failures (MVTATA RADICE — flip the root, the
heretics set themselves ablaze). Sweep the cascade to a green floor, commit atomically. **Do NOT commit
until `cargo nextest run --release` is 0-new-failures, weighed by the orchestrator's own re-run.**

## The rooms (exact `file:line` — scouted this session, all confirmed)

1. **`src/freeze.rs:520-544` — `enum StartupError`.** Add a variant `MainSignature(String)` (the reverted
   strike added exactly this; it does not exist now). Add its arm to `impl fmt::Display for StartupError`
   (`src/freeze.rs:545`) — render `MainSignature(m) => write!(f, ":user::main: {}", m)` (mirror
   `HarnessError::MainSignature`, `src/harness.rs:79`).
2. **`src/freeze.rs:1066` — `validate_user_main_signature(&FrozenWorld) -> Result<(), String>` EXISTS**
   (canonical `[] -> :wat::core::nil`, already exact). REUSE it — do not rewrite it.
3. **`src/freeze.rs` (new, beside `validate_user_main_signature`) — `validate_user_main_not_useless`.**
   ```rust
   /// A declared :user::main must DO something — its body may not be the bare `nil`
   /// literal. "Either give it a real body or omit the main entirely." (arc 170, the
   /// UselessMain wall.) Semantic uselessness is undecidable; we wall the one literal
   /// form sonnets keep writing: `(:user::main [] -> :wat::core::nil nil)`.
   pub fn validate_user_main_not_useless(frozen: &FrozenWorld) -> Result<(), String> {
       let func = match frozen.symbols().get(":user::main") {
           Some(f) => f,
           None => return Ok(()), // no main declared — nothing to check
       };
       if let crate::value::environment::FunctionBody::Wat(ast) = &func.body {
           if matches!(&**ast, crate::watast::WatAST::NilLit(_)) {   // <- confirm the WatAST module path
               return Err(
                   ":user::main body is the bare `nil` literal (UselessMain). \
                    A declared :user::main must DO something — either give it a real \
                    body, or OMIT the main entirely (not every file needs one; only \
                    programs that RUN). Never write `(:user::main [] -> :wat::core::nil nil)`."
                       .to_string(),
               );
           }
       }
       Ok(())
   }
   ```
   - `Function.body: FunctionBody::Wat(Arc<WatAST>)` — `src/value/environment.rs:70`.
   - The bare `nil` body lowers to `WatAST::NilLit(span)` — confirmed `src/lower.rs:166`, `src/edn_shim.rs:463`.
   - VERIFY the `WatAST` import path compiles (grep for how `freeze.rs` already names `WatAST`); adjust the
     `crate::watast::WatAST` / `crate::value::environment::FunctionBody` paths to whatever `freeze.rs` uses.
4. **`src/freeze.rs:678` — `startup_from_source`. IMPOSE the wall.** After `startup_from_forms(...)` returns
   the `world`, before `Ok(world)`, add — conditional on `:user::main` being declared:
   ```rust
   let world = startup_from_forms(entry_forms, base_canonical, loader)?;
   if world.symbols().get(":user::main").is_some() {
       validate_user_main_signature(&world).map_err(StartupError::MainSignature)?;
       validate_user_main_not_useless(&world).map_err(StartupError::MainSignature)?;
   }
   Ok(world)
   ```
   This is THE chokepoint: `startup_from_file`/`startup_beside`/`cargo wat`/the wat-scripts load gate all
   route through `startup_from_source`. `startup_bare()` (empty source, no main) passes cleanly (conditional).

## Prove the wall BITES before touching the cascade (the disconfirming gate)

Run these two, in this order — they are the RED→GREEN proof the wall works:
- `cargo wat --check` on a file whose sole form is `(:wat::core::defn :user::main [] -> :wat::core::nil nil)`
  → MUST now be a **non-zero exit with a `:user::main body is the bare nil literal` diagnostic** (today it
  exits 0 — that is the gap). *(The installed `wat` binary is stale; run `cargo build --release` first so the
  check reflects your edit, or drive it through a Rust `startup_from_source` assertion instead.)*
- `cargo wat --check` on `(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "hi"))`
  → MUST freeze CLEAN (a real body passes).

If the useless probe still exits 0 after your edit, the wall is not wired — STOP and re-check the chokepoint.

## The cascade — CODEMOD the bulk, hand-dispose the rest (CORRECTED 2026-07-09b)

**The bare-`nil` useless-main bulk is a `wat-fix` codemod, NOT hand-edits.** `wat-scripts/fixes/strip-useless-mains.wat`
already matches the exact `(:wat::core::defn <…::main> [] -> :wat::core::nil nil)` form, strips it top-level, and
has a SOLE-DEFN GUARD (never strips a useless main that is the only defn — a main-AS-SUBJECT fixture). Drive it over
the whole corpus:
```bash
printf '%s' "$(git ls-files '*.wat' | sed 's/.*/"&"/' | tr '\n' ' ' | sed 's/^/[/;s/ $/]/')" \
  | cargo wat ./wat-scripts/fixes/strip-useless-mains.wat   # prints [useless-main] path:line name + [stripped] path
```
The codemod covers ONLY the exact `-> :nil nil` shape. **Hand-dispose what it does not** — the `~35` illegal-SIGNATURE
mains + the sole-defn main-rule fixtures the guard leaves — off the `cargo nextest run --release` freeze failures, by KIND:

| the fixture is… | do this |
|---|---|
| a **compile-time / type-check** fixture (only needs a defn to freeze; the main is a wrapper) | **OMIT the main** — move the work into a NON-main `defn` (freeze type-checks ALL defns). NEVER fabricate a body. |
| a **runtime** fixture (its main's body must run) | give the main a **real `[] -> :nil` body**; for value asserts add stdout-capture (copy the shape in `tests/types/enums.rs::run()`). |
| a `.wat.bad` **negative** fixture (its OWN error is the subject) | **OMIT the main** so the REAL error surfaces (don't let a main-signature error mask it). |
| a test that **TESTS the main rules** (`tests/program/*` legacy 3/4-arg / ExitCode / wrong-return / `user_main_nil`) | **retarget** it to assert the FREEZE wall REJECTS (structural `Err(StartupError::MainSignature(_))`). STOP + report each; NEVER blind-delete. |

Measured population (grounded this session): 321 legal `[] -> :nil`; 35 illegal-sig (14 `i64` · 6 `String` ·
5 legacy-4-arg-ExitCode · 4 `bool` · 3 three-arg · 1 `Record` · 1 `Amount` · 1 three-arg-underscore); ≥9
single-line bare-`nil` (more in multi-line form — the wall surfaces them all).

## The committed DURABLE proof (part of the deliverable)

`tests/program/probe_arc170_main_wall_rejects.{rs,wat}` — the `.wat` is a `_bad`-style fixture (a useless
main), the `.rs` asserts `startup_from_file(...)` returns `Err(StartupError::MainSignature(_))` **structurally**
(match the enum variant — NOT `.contains("...")`). Add a second case for an illegal signature. This is the
ward that keeps the wall from being reverted a third time.

## STOP triggers (rejection criteria — surface, do not improvise)

- If imposing at `startup_from_source` breaks a freeze path you did not expect (e.g. a deftest/sandbox
  internal that legitimately builds a `:user::main`), STOP and report the path — do NOT loosen the wall.
- If a fixture's correct disposition is genuinely ambiguous (can't tell compile-time vs runtime), STOP and
  list it — do NOT guess a body.
- If the floor won't reach 0-new after the disposition sweep, STOP with the remaining failure list — do NOT
  commit a red tree, do NOT `#[ignore]` to hide it.

## HOW TO WORK (the hard lessons — non-negotiable)

- **Run `cargo nextest run --release` FOREGROUND-BLOCKING.** NEVER `&` / background / disown / setsid /
  double-fork. Two prior strikes on THIS wall orphaned nextest (reparented to init), ended their turn before
  it finished, and returned fragment reports. You CANNOT wait on a backgrounded run. Foreground, always.
- **OMIT the main; NEVER fabricate a meaningless body.** Prior agents wrote 26 fake
  `(:user::main [] -> :nil (:wat::core::let [_ 0] nil))` bodies to EVADE the check — uselessness in disguise.
  That is a firing offense. For a compile-time fixture the main is DELETED, not stubbed.
- **Negative asserts are STRUCTURAL** — match `Err(StartupError::MainSignature(_))`. Do NOT use
  `.contains(...)` / `starts_with` / `ends_with` (trips `no_loose_string_assert`); do NOT inline wat forms in
  assert strings (trips `no_inlined_wat`).
- **NEVER add a rune** to silence a lint your change tripped. Fix the code.
- Weigh nothing by your own say-so; report the exact `cargo nextest` summary line (`N passed / M failed`).
- Do NOT commit — leave the tree dirty + green for the orchestrator to weigh, commit, and push.
- STAY on branch `arc-170-gap-j-v5-deadlock-state`. NEVER `/proc`. `cargo build --release` before any
  `cargo wat` check (the installed binary is stale).

## Definition of done

The wall is imposed at `startup_from_source`; the useless-main probe now fails `cargo wat --check` with the
UselessMain diagnostic and a real-body main passes; every illegal-sig + bare-`nil` main disposed per the
table; `tests/program/probe_arc170_main_wall_rejects.{rs,wat}` committed-ready (structural assert);
`cargo nextest run --release` is 0-new-failures (a known pre-existing flake re-run isolated
`--test-threads=1` = not a regression); tree dirty + green, NOT committed. Report the nextest summary line.
