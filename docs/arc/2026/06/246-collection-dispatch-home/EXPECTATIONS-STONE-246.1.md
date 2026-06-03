# EXPECTATIONS — Stone 246.1 — collection dispatch lift

Verified against an independent orchestrator re-run, not the agent's self-report. This is a **behavior-preserving lift** — the regression guard is the existing suite staying green; the "disconfirmer" is structural (the fns moved).

## Gates (raw commands)

1. `cargo test --release --lib -p wat` → **895 passed / 0 failed / 1 ignored** (UNCHANGED — any delta means the move changed behavior, which it must not).
2. `cargo build --release --tests --workspace` → clean (no new warnings beyond pre-existing).
3. **The move is complete** — `grep -rnE "^fn (infer_(contains|conj|get|assoc)|eval_(vector|hashmap|hashset|list|vec)_)" src/check.rs src/runtime.rs` → **zero** (the lifted fns no longer live in the flat files; no duplication).
4. **The home exists + is registered** — `src/collection/{mod,infer,eval,transform}.rs` present; `pub mod collection;` in `src/lib.rs`.
5. **The arms redirect** — `grep -rn "collection::" src/runtime.rs src/check.rs` shows the central `dispatch_keyword_head_value` arms + the 4 `infer_list` collection arms calling `collection::eval::*` / `collection::transform::*` / `collection::infer::*`.

## Structural verification (the HARD READ)

- **Pure move:** the lifted fn bodies are byte-identical (path/visibility changes only — `pub(crate)`, `use crate::…`). No logic edits. Confirm via diff that `collection/*.rs` fn bodies match the originals.
- **mod.rs doctrine:** states the clause-vs-intrinsic partition (the projective flavor, `get` as the worked proof) citing `docs/DISPATCH.md`; the word `intrinsic` appears in prose, NOT as a filename. Mirrors `function/mod.rs` doc shape. **No vigilatum stamp** (246.2 earns it).
- **Module split honest:** `infer.rs` imports only check-side types (CheckEnv/InferCtx/Subst/TypeExpr); `eval.rs`/`transform.rs` only runtime types (Value/Environment/SymbolTable/RuntimeError). If a shared helper emerged, it lives in `mod.rs` (not a 4th file).

## Scope guard (do NOT touch)

- `dispatch_keyword_head` / `dispatch_keyword_head_value` (central match — stays, redirects), `infer_list` (check-side keyword dispatch — stays, redirects, **PARTITION marker comment intact**).
- `dispatch_rust_scheme` (rust-deps), `eval_eq` / `eval_not_eq` / `infer_equality` (equality — relational intrinsic, separate home concern), `values_equal`.
- No op's runtime/check behavior changes. No `holon-rs`.

## Hand-off

Leave all changes uncommitted. Do not commit/tag/push — the orchestrator scores against an independent re-run and commits. **NEXT:** 246.2 (vigilia ward → L1+L2=0, earn the vigilatum stamp) → 246.3 (inscribe + INSCRIPTION).
