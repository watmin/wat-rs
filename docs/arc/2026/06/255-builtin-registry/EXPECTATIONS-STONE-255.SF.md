# EXPECTATIONS-STONE-255.SF — scorecard (run every row before claiming green)

| # | what | command | expected |
|---|---|---|---|
| 1 | north-star probe GREEN | `cargo test --test test render_doc_of` | `render_doc_of_if`, `render_doc_of_let`, `render_doc_of_bytes_to_hex` all **ok** (3 passed) |
| 2 | full wat-tests suite green | `cargo test --test test` | no regressions vs HEAD (pre-existing `test-run-string-entry-direct` may stay failing — note it) |
| 3 | lib tests green | `cargo test --lib` | no NEW failures vs the 36-fail floor; the doc cross-check tests (`pure_declared_matches_is_effectful_op`, `purity_mandated_examples`, `doc_arg_ret_types_match_checker_scheme`) pass |
| 4 | wat-doc crate green | `cargo test -p wat-doc` | all pass (existing 25 + any new special-form-parse tests) |
| 5 | nursery green | `cargo test --test nursery` | 8/8 (or more) pass |
| 6 | bytes UNCHANGED | `cargo test --test test render_doc_of_bytes_to_hex` | still ok — the value exemplar is not regressed |
| 7 | clippy clean | `cargo clippy` | no new warnings in touched files |
| 8 | metadata-of reports SpecialForm | a probe or manual: `(metadata-of :wat::core::if)` Kind | `Kind::SpecialForm` |

**Runtime prediction:** multi-round (R2/R3 likely) — cross-crate (wat-doc + wat-macros + lib), bool→enum cascade.

**Trap-doors named:**
- The `pure: bool → purity: Purity` field change ripples through cross-check tests + `derive_pure_deterministic`; if it reaches the checker type table, that's STOP-2.
- `lookup` returning `None` for `handler: None` entries is load-bearing — if a special form accidentally dispatches via the registry instead of inline, eval breaks. Verify `if`/`let` still EVALUATE correctly (run a few `(if …)` / `(let …)` tests), not just reflect.
- A new `.wat` test file is auto-discovered by `wat::test! {}` — no registration needed.

**Weigh discipline (orchestrator, NOT the agent's report):** re-run rows 1–7 myself against the main-repo disk; read the diff; confirm `if`/`let` still EVAL (not just reflect); confirm no git worktree was used. Credit nothing the disk does not show.
