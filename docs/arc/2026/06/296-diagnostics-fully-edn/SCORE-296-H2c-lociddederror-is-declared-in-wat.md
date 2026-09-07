# SCORE — 296 H-2c: LociDiedError is declared in wat

No commit. Floor and clippy left to the orchestrator. H-2's writer (`variant_tag`) was not moved.

The three sites no longer retype an EDN namespace string. They ask a generated enum whose list lives in wat.

---

## Row 1 — the enum is DECLARED IN WAT

```
wat/kernel/diagnostics.wat:122:(:wat::core::defenum :wat::kernel::LociDiedError :wat::enum::Pure
```

One hit. Variants derived from the Rust builtin registration this form replaces: Panic (message, failure), RuntimeError (message), Disconnected, Stopped, StartupError (error), EntryFormFailure / MainSignature / BadReturn (message). Placed after `Failure` (Panic names it) and before `AssertionFailure` (which names LociDiedError).

## Row 2 — Rust SOURCES from it

```
src/kernel/error.rs
  wat_enum_from!(pub(crate) enum LociDiedError, "wat/kernel/diagnostics.wat", ":wat::kernel::LociDiedError")
src/types.rs
  wat_enum_register_from!(env, "wat/kernel/diagnostics.wat", ":wat::kernel::LociDiedError")
```

`wat_enum_from!` is the intrinsic/mod.rs shape (a generated Rust enum the consumers match). `wat_enum_register_from!` is the enum sibling of `wat_record_from!` — TypeEnv `with_builtins` now sources the EnumDef from the same `defenum`, so there is no second list. Added in `crates/wat-source-derive/src/lib.rs`.

## Row 3 — the hand-typed identity is gone

`grep -rn '"wat.kernel.LociDiedError"' src/ --include=*.rs` = **2 hits, both comments** (types.rs, verbs.rs documenting the retired string). Zero live compares.

Consumers ask `tag_is_variant_of(tag, LociDiedError::WAT_TYPE_PATH)` — the same ns/leaf split `variant_tag` writes. Variant discrimination is `ev.variant_name.parse::<LociDiedError>()` (exhaustive on the generated enum). Constructors use `LociDiedError::WAT_TYPE_PATH` / `.as_str()`.

## Row 4 — the producer goes through the writer

`src/process/verbs.rs::startup_error_chain_edn`: the hand-built `Tag::ns(...) + Vector` is **gone**. It now builds `Value::Enum` of `LociDiedError::StartupError` and calls `value_to_edn_with` — the same writer `emit_chain_envelope` / `thread_crash_*_edn` already use.

## Row 5 — the chain decodes

`cache_probe_startup_error_is_navigable_edn_not_string` **PASSES**. STRICT-decode of the emitted chain yields typed records, not a string-wrapped envelope.

## Row 6 — the message survives as DATA

`deftest_wat_tests_test_test_assert_eq_fail_populates_message` **PASSES**.

## Row 7 — H-2's writer did not move

`probe_arc296_h2` **3/3**. `#[ignore]` count **0**. Added `tag_is_variant_of` beside `variant_tag` (a query, not a write change).

## Row 8 — the floor (ORCHESTRATOR)

Not run.

## Row 9 — clippy (ORCHESTRATOR)

Not run.

## Row 10 — the drift gate is GONE, not passing

No new gate. `wat_enum_from!`'s own docs already deleted `every_rust_enum_matches_its_wat_defenum` as scaffolding; this stone does not reintroduce one. A generated enum cannot drift from its `defenum`.

---

## STOP rows

| STOP | result |
|---|---|
| STOP-1 retype the three literals | **not taken.** Generated enum + `tag_is_variant_of`. |
| STOP-2 stash dance | **not required.** Additive `defenum`; macros read the file at compile time; no checker change that makes the old corpus illegal. |
| STOP-3 declaration ≠ registration | **did not fire.** Stdlib loaded; `cache_probe` decoded. |
| STOP-4 H-2 probe | **3/3 green.** |
| STOP-5 ServiceEvent / sqlite::Cell | **not dragged in.** `builtin_enum_variant_names` still looks LociDiedError up in `with_builtins` (now wat-sourced). |

## Targeted checks (executor)

```
grep -n 'defenum.*LociDiedError' wat/kernel/*.wat     1 hit
grep wat_enum_from! … LociDiedError                   ≥1
grep '"wat.kernel.LociDiedError"' src/ --include=*.rs  comments only
cargo test --release --lib -- cache_probe_startup_error     passed
cargo nextest run --release -E 'test(deftest_wat_tests_test_test_assert_eq_fail_populates_message) | test(probe_arc296_h2)'
  4 passed
```

## Files (this stone)

```
wat/kernel/diagnostics.wat                 defenum :wat::kernel::LociDiedError
crates/wat-source-derive/src/lib.rs        wat_enum_register_from!
src/types.rs                               registration sourced from wat
src/kernel/error.rs                        wat_enum_from! + consumers + constructors
src/edn/render.rs                          tag_is_variant_of (query only)
src/process/verbs.rs                       producer via value_to_edn_with
src/intrinsic/kernel/error.rs              WAT_TYPE_PATH into the two accessors
```
