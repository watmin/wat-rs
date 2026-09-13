# SCORE 2a1 REFUTE — the door reports a program's macro-generated types

Branch: `replay/grok-rete`. **Committed, not pushed.** Main untouched.
Parent: `175b49ea9` SCORE 2a1.

```
floor  scripts/floor.sh   .floor/2026-09-13T08-09-43Z
       Summary [ 214.230s] 5425 tests run: 5425 passed, 22 skipped   exit=0
clippy cargo clippy --release --all-targets -- -D warnings           CLIPPY_RC=0
```

No STOP. D1 still holds: sift_rules **13.8 ms**, stale bodies still register.

## R

| # | result |
|---|---|
| R1 | **PASS.** `tests/diagnostics/probe_diagnostic_c3_macro_emits_record_def.wat`: `:demo::Req` `[n]` and `:demo::Op` `Go [req]`. User defmacros register first; a top-level call of one is kept. `:wat::core::defn` is a stdlib macro and is not expanded. |
| R2 | **PASS, and went RED once.** Type-decl heads are [`classify_type_decl`](src/types.rs) (the freeze door), not a second list. Pre-expansion type-minting heads (`defrecord`, `defservice`, …) cannot be derived from that classifier; `declared_types_filter_admits_pre_expansion_type_macros` names any that the door drops. Dropping `:wat::core::defrecord` from `PRE_EXPANSION_TYPE_FORMS` → RED (verbatim below). Restored. |
| R3 | **PASS.** Two-form program: well-formed `defenum` then `(:wat::core::defenum)`. `Refused.form` is the second (arity &lt; 3), not the first. Failures are mapped through the error span onto the covering top-level form. |
| R4 | **PASS for every case that loads.** Colour (defenum), Point (defrecord), Ping `::Op` (defsurface), `:my::aws::Waf::Op` / CreateWebACL (acronym file). sift_rules and W2f do not load (stale bodies) — they are R1's D1 pin, not the oracle. |

## R2 — the red, verbatim

```
thread 'check::tests::declared_types_filter_admits_pre_expansion_type_macros' (1209740) panicked at src/check.rs:23658:13:
:wat::core::defrecord mints types at expansion but the door drops it
```

## Blast radius

`src/freeze/env.rs` (keep user-macro calls; span-faithful refusal) · `src/types.rs` (`classify_type_decl` pub(crate)) · `src/macros/registry.rs` (`names`) · `src/check.rs` (c3, filter, two-form refusal, oracle 1–4). Not pushed.
