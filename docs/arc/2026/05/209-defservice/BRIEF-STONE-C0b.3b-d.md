# BRIEF — Stone C0b.3b-d (foundation): the `user.program` injection seam

**Executor:** Shadowdancer (sonnet). **Anchor:** `/home/watmin/work/holon/wat-rs/` (verify `pwd`;
operate ONLY here; `git -C /home/watmin/work/holon/wat-rs`). Design (read fully):
`DESIGN-STONE-C0b.3b-d-user-program-foundation.md`. The RED probe is on disk + verified RED at
HEAD (`tests/probe_arc209_c0b3bd_user_program_foundation.rs` — `E0432: invoke_user_main_with_program`
absent). Do NOT commit — the Inquisitor weighs.

## The work in one paragraph

`invoke_user_main` (src/freeze.rs) hardcodes the 7th field of `:wat::program::Env` to
`(:wat::program::EmptyEnv)` at line 1095, with no way to inject a `user.program`. Add an ADDITIVE
sibling `invoke_user_main_with_program(frozen, args, user_program: Value)` that installs the given
Record as `user.program` instead of EmptyEnv. Keep `invoke_user_main(frozen, args)` EXACTLY as is
(it delegates with the EmptyEnv default) — its ~30 callers must stay untouched. The env build at
1095 binds the value as a local and references it by name, exactly like the thread-tier init-fn
pattern at `spawn.rs:441–457`.

## Read in order (the rooms)

1. `src/freeze.rs:1056` — `pub fn invoke_user_main(frozen, args)` (the public entry; keep its sig).
2. `src/freeze.rs:~1066` — `fn invoke_user_main_orchestrated(frozen, args)` (holds the 1095 env
   build). This gains the injected `user.program` (as `Option<Value>` internally is fine).
3. `src/freeze.rs:1090–1101` — the env build: `env_src` with the literal
   `(:wat::program::EmptyEnv)` as the 7th arg → `eval_in_frozen(env_ast, frozen, Environment::new())`
   → `install_program_env`. This is the ONE site to change.
4. `src/kernel/spawn.rs:439–457` — the EXACT pattern to mirror: bind `user-program` as a local in
   a `ctor_env`, reference it by name in the constructor source, eval against that env.

## Implementation sketch

```rust
// NEW pub sibling — the injecting seam.
pub fn invoke_user_main_with_program(
    frozen: &FrozenWorld, args: Vec<Value>, user_program: Value,
) -> Result<Value, RuntimeError> {
    invoke_user_main_orchestrated(frozen, args, Some(user_program))
}

// UNCHANGED public sig — delegates with the default.
pub fn invoke_user_main(frozen: &FrozenWorld, args: Vec<Value>) -> Result<Value, RuntimeError> {
    invoke_user_main_orchestrated(frozen, args, None)
}

fn invoke_user_main_orchestrated(
    frozen: &FrozenWorld, args: Vec<Value>, user_program: Option<Value>,
) -> Result<Value, RuntimeError> {
    // ... existing bootstrap, pid/tid/boot_nanos/cpu_count unchanged ...
    let user_program_val = match user_program {
        Some(v) => v,
        None => eval_in_frozen(
            &crate::parse_one!("(:wat::program::EmptyEnv)").expect("EmptyEnv ctor parses"),
            frozen, &crate::runtime::Environment::new(),
        )?.value_owned(),
    };
    let ctor_env = crate::runtime::Environment::new().child()
        .bind_unknown_span("user-program",
            crate::value::TrackedValue::from(user_program_val))
        .build();
    let env_src = format!(
        "(:wat::program::Env (:wat::time::at-nanos {boot_nanos}) (:wat::time::now) \
         {pid} {tid} :wat::program::PeerKind::process {cpu_count} user-program)"
    );
    let env_ast = crate::parse_one!(&env_src).expect("...");
    let program_env = eval_in_frozen(&env_ast, frozen, &ctor_env).map(|tv| tv.value_owned())?;
    let _program_env_guard = crate::services::install_program_env(program_env);
    // ... existing run-main block unchanged ...
}
```

Confirm the exact `Environment` builder API + `TrackedValue::from` against `spawn.rs:441–447`
(copy it verbatim). The current `invoke_user_main_orchestrated` body moves into the new 3-arg
private fn; only the env-build paragraph changes.

## Blast radius

`src/freeze.rs` ONLY. NO `check.rs`, NO `spawn.wat`, NO changes to any of the ~30
`invoke_user_main(frozen, args)` callers (the seam is additive). The probe is already on disk.

## STOP triggers (rejection — ship nothing, report)

1. **STOP-1:** the bind-local + reference-by-name env build (spawn.rs:441 pattern) doesn't work in
   freeze's context — STOP, report.
2. **STOP-2:** the default path (`invoke_user_main` / None) stops producing `EmptyEnv` — STOP; it
   must be byte-identical to today.
3. **STOP-3:** any existing `invoke_user_main` caller needs editing to compile — STOP; the old
   signature is preserved.

## The gate (report each exact `test result:` line; do NOT commit)

```
cargo test --release -p wat --test probe_arc209_c0b3bd_user_program_foundation -- --test-threads=1   # 2 passed
cargo test --release -p wat --lib -- --test-threads=1                                                # 915 passed / 36 failed (PRE-EXISTING baseline — ZERO new; see note)
cargo test --release -p wat --test nursery -- --test-threads=1                                       # 895 passed / 4 failed (baseline; ZERO new)
cargo test --release --workspace --no-run                                                            # full surface compiles (all ~30 callers unchanged)
```
NOTE: the lib unit suite has 36 PRE-EXISTING failures (`check::tests::*` / `runtime::tests::*`, a
legacy module) — they are NOT yours; confirm the count stays 36 (zero new). Run `cargo test`
PLAINLY (no setsid/timeout). The harness may show stale rust-analyzer diagnostics that contradict
a clean `cargo build` — trust your own build.

## Prior comparable (copy the shape)

`spawn.rs:439–457` (the thread-tier init-fn record-build — the exact bind-local + eval pattern) and
`BRIEF-STONE-C0b.3b-c.md` (the just-shipped sibling, same freeze/spawn neighborhood).
