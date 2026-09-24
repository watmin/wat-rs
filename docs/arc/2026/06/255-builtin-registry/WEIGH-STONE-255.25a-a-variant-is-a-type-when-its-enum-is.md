# WEIGH — STONE 255.25a: a variant is a type the moment its enum is — ACCEPTED

**Executor commit `bd0fd8be3`.** Weighed by the orchestrator against disk on 2026-09-24.

## Re-measured

| row | measured | result |
|---|---|---|
| tree · floor | `git status`; `.floor/2026-09-24T21-21-00Z/clean.log` | clean; `6063 tests run: 6063 passed (9 slow), 22 skipped` |
| a variant in an edge target | `wat …_variant.wat` | rc=0; pre-stone rc=3 `EdgeFreeTypeName` (the executor's pre binary; the orchestrator reproduced the same refusal on `main` before drawing) |
| an undeclared variant | `wat …_undeclared_variant.wat.bad` | rc=3 |
| `register_variant_types` | grep over `src/` | one mention left, in a doc comment; the pass is gone |

Taken from the report without re-running:

- clippy 0;
- census `no STOP-8`, with byte-identical path/rc lists;
- delta NEW 2 / RECOVERY 0;
- ledger 215;
- the new test file against the pre-stone `src/` gives 4 passed and 1 failed, the variant row.

## What landed

- **One helper, `TypeEnv::insert_enum_with_variants`,** sits at the only two doors that insert an enum:
  `register_validated` (every user, stdlib, surface-synthesised and door-replace enum) and
  `register_builtin` (`Option`, `Result`, …). It registers the variant singletons and their
  `Variant <: Enum` edges in the same act.
- **The separate whole-env pass is retired, and no caller needed it.** `pass_order.rs` is unchanged:
  the pass never called `record(...)`, so nothing left the traced order.
- Door-replace already retracted variants with their enum (2a4b); only its doc changed.
- **One verdict kept unchanged:** `validate_aggregate_containment` skips variant singletons. They copy
  their parent's fields, and a HashMap walk could otherwise report an impure field under `E.V` instead
  of `E`.
- **Declaration order:** a struct declared after an edge that names it is refused today (255.22's wall
  asks at the edge's own registration). A variant now behaves the same way.

## ⛔ Finding — the debug build is RED on `main`, and the release floor cannot see it

Reproduced by the orchestrator on HEAD:

```
$ cargo test --lib freeze::pass_order::tests::the_startup_passes_run_in_the_declared_order
thread '…the_startup_passes_run_in_the_declared_order' panicked at src/types.rs:1070:9:
builtin leaf :wat::core::Option already registered as a structured TypeDef
test result: FAILED. 0 passed; 1 failed
```

The arm is the first `debug_assert!` in `register_builtin_leaf`: `!self.types.contains_key(&name)`.

- The executor measured it on `7b0cbcc20`, **before this stone**, with none of its edits.
  `check::tests::declared_stdlib_types_retracts_old_variant_singletons` panics the same way.
- **Every debug-mode lib test that builds builtins is red.**
- The floor is `cargo nextest run --release`, which compiles `debug_assert!` out, so **no floor since this
  began has been able to see it.**

CLAUDE.md: *"A `debug_assert!` panic is a real failure: debug surfaces conditions release compiles out."*
**It needs its own stone:**

- find when it began (the commit that registers `Option` structurally before its leaf);
- cure it at its cause;
- ask whether the floor should also run a debug pass, so this class cannot hide again.

**Not a disposition.** "Only in debug" is the dismissal CLAUDE.md names.

## Brief errors, recorded

- There is no `defenum` arm in `splice_type_decls`. Enums register through `register_validated`, and the
  builtin door was not named.
- Item 3 (door-replace retracts variants) was already true on disk.

## Next

255.25 (C-b4) resumes from `scratchpad/s25/255.25-stopped.patch` (29 files) and its codemod in
`s25/untracked/`.
