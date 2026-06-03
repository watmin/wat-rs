# SCORE — Stone 237.8b — Recipe-lock + numeric grid

**Verdict: REMARKABLE** — shipped green after one R2 doctrine fix. The recipe is locked and proven across two op-families × two types.

## Gates (orchestrator's independent re-run, not the agent's self-report)

| Gate | Result |
|---|---|
| `cargo test --release --test probe_arc237_8b_defclause_arithmetic` | **19 passed / 0 failed / 0 ignored** |
| `cargo test --release --lib -p wat` | **895 passed / 0 failed / 1 ignored** (pre-existing HashSet debt) |
| `cargo build --release --tests --workspace` | clean (0 errors) |

## The strike

- **14 substrate changes**: 8 `'2`-suffix drops, mint `:i64::<=`, mint the f64 ordering family (`< > <= >=`, NaN-correct via `eval_f64_compare`), rename `:i64::!=` → `:i64::not=`.
- **8 recipe `defclause`s** in `wat/core.wat`: `+ - * /` (0-ary identity for `+`/`*`; 0-ary `:NoMatchingClause` for `-`/`/`; 1-ary per-Type identity-on-left; 2-ary direct; 3+-ary fold over the `&` rest-binder) and `< > <= >=` (2-ary). Cross-type rejected by clause **absence**, no special-case logic.
- **5 HARD CUTs**: `infer_arithmetic`, `eval_arithmetic_variadic`, `is_numeric`, `infer_comparison`'s ordering arms, the 8 per-Type variadic wat-fns at `core.wat:104-132`.
- **~240-site cascade** across 60+ files (substrate-as-teacher; net −36 lines). DESIGN predicted "small-but-nonzero"; actual larger purely from test coverage breadth — all mechanical renames, green-vouched.
- **Novel infrastructure**: the first stdlib `defclause` pipeline (`register_stdlib_defclauses`, `preregister_stdlib_defclause_stub`, `CheckEnv::from_symbols` reading `runtime_def_values`, freeze step 7.6). defclauses had only ever been user-level; making `:wat::core::+` *itself* a defclause required a check-time + runtime registration path for stdlib defclauses.

## HARD CUT verified (the retired forms are GONE, not shimmed)

`'2` suffix, `:i64::!=`, the three deleted Rust fns, and the per-Type variadic wat-fns all return **0 live constructions** — the only remaining grep hits are lineage-documenting comments (`// renamed from :i64::!=`). No shim, no alias.

## The R2 catch — independent verification earned its keep

The agent's green report was correct on the gates but hid a doctrine violation in the novel infra. `parse_defclause_form_privileged` implemented the (legitimate) stdlib-is-privileged concept as a **sentinel-swap hack**: it replaced the real name with a magic `:my__stdlib__defclause__sentinel`, built a fake AST form, parsed *that* through the canonical parser, then swapped the name back — and its own comment named the correct fix and shipped the hack anyway (*"Simpler: ... patch parse_defclause_form to accept an allow_reserved flag. For now, directly patch the form name"*). Solvable → the hack is illegal (`feedback_runes_illegal_when_solvable`); the green tests passed *through* it, so only a code-read caught it (`examinare` — the cast is data, the disk is the verdict).

**Fix (R2, verified):** added `allow_reserved: bool` to the canonical `parse_defclause_form`; reserved-prefix guard became `if !allow_reserved && is_reserved_prefix(&name)`; deleted `parse_defclause_form_privileged` entirely; 6 callers updated (2 stdlib → `true`, 4 user-side → `false`). One canonical parser, one honest flag. Re-verified: 19/0/0, 895/0/1, build clean. The function + sentinel string return 0 references.

## Scope resolution (Inquisitor four-questions, recorded)

The DESIGN had an internal ambiguity — Crawl listed 6 f64 mints (including `f64::=`/`f64::not=`), but "does NOT do" sent equality to 237.8c ("full primitive equality grid"). Resolved **against the probe** (the contract requires only f64 *ordering*): 8b mints f64 ordering; 8c mints f64 equality. One coherent grid per stone. Net mints trimmed 16 → 14.

## What this stone leaves to the chain

- **237.8c** — equality grid (`=`/`not=` polymorphic defclause + `f64::=`/`f64::not=` primitives + composite recursive equality); migrates `infer_comparison`'s remaining `=`/`not=` arms.
- **237.8d** — `DispatchRegistry` HARD CUT (0-tenant after 8b).
- **237.9** — INSCRIPTION + the `feedback_per_type_binary_primitives` doctrine mint.

The recipe is the durable win: future per-Type ops (`%`, bit-ops, new numeric types `u8`/`u32`/…) now follow the locked pattern with zero re-thinking.
