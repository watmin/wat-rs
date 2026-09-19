# SCORE — STONE 251.8c: close the check hole (#95)

Branch: `main`. **Committed, not pushed.** Stone `bc93125aa` (folded AMEND of `4e9fb9e17`;
loose `contains` asserts replaced with exact TypeMismatch-field pairing — see
`SCORE-AMEND-251.8c-the-loose-assert-gate.md`).
Parent brief: `BRIEF-STONE-251.8c-close-the-check-hole.md`. Floor/clippy: orchestrator's row (not run).

## The brief was wrong, and the code won

The brief asked me to confirm or kill this hypothesis: *Keyword heads force the callee's signature eagerly; Symbol heads compute the call type lazily, so unconstrained result positions skip arg/arity.*

**Killed.** Measured:

1. **Slash heads ARE normalized on residue.** `build_env` dump: parse `Symbol(user/f)` → residue `Keyword(:user::f)`. `user/nope` is still `UnresolvedReference :path ":user::nope"`.
2. **`check_program` does not type-check that residue body.** It walks `sym.functions_iter()` → `FunctionBody::Wat` snapshots taken at `register_defines` (step 6), which is **before** `normalize_symbol_refs` (step 7). Colon-quoted source is already a Keyword when snapshotted. Slash source is still a Symbol. `infer_list`'s Keyword path never sees it.
3. **The class is every namespaced-symbol call inside a function body**, not `format` `:v`. Statement, let-bound, assertion-failed kwarg, `str`, `println`, and `format` `:v` all leaked (`--check` rc=0). Nested `(:user::g (user/f "boom"))` as `:user::main`'s result was rc=1 for **ReturnTypeMismatch** (main declared `nil`, g returns i64) — the String arg was **not** rejected. The brief's four "already closed" slash rows were that class of probe error.

`format` is a macro (unconstrained `str` of the slot). That is a real unconstrained *result* position, but it was not a distinct mechanism.

## The fix (no-form, one path)

`normalize_stored_function_bodies` runs the existing Symbol→Keyword rewrite over stored `FunctionBody::Wat` after residue normalize. `check_program` then sees Keyword heads. Did **not** patch `infer_list` to accept Symbols. Did **not** invert 8b. Did **not** need 255 (colon-quoted builtins already check; user fns already have schemes).

## Matrix (all-same after the fix)

| shape | colon | slash | reason |
|---|---|---|---|
| statement `do` | TypeMismatch f String/i64 | **same** | |
| let-bound | same | **same** | |
| nested arg, wrapped in `do`+`nil` | same | **same** | unwrapped slash was ReturnTypeMismatch (probe artifact) |
| `assertion-failed! :actual` | same | **same** | |
| `format "{v}" :v` | same | **same** | |
| `(user/nope 1)` | — | UnresolvedReference `:user::nope` | normalizer still fires |

Test: `check::tests::stone_251_8c_slash_and_colon_heads_share_the_call_path`. `cargo nextest list --release -E 'test(stone_251_8c)'` → 1 test (N>0). Predicted delta +1 `#[test]`.

## Walls I ran (not the floor)

- `--check` matrix against `./target/release/wat` after the commit's binary: all slash rows rc=1 TypeMismatch, matching colon.
- `cargo nextest run --release -E 'kind(lib)'` Summary \[19.218s\] **1525 passed**, 2 skipped.
- `scripts/replay/census.sh` `.census/2026-09-19T23-04-04Z.txt` files=2189; `--diff` vs `.census/2026-09-19T12-44-16Z.txt` → `census-diff: no STOP-8`.

Floor + clippy: **not run** (brief: orchestrator, uncontended).

## STOP

None. Do not push. 8d (corpus flip) must not start until the orchestrator greens the floor.
