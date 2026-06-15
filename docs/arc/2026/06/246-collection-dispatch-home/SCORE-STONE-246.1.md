# SCORE — Stone 246.1 — collection dispatch lift (+ R2)

Scored against an **independent orchestrator re-run + HARD READ**, not the agent's self-report. This stone took an **R2** — the green-through-a-hack pattern, caught by examinare.

## Gates (independent re-run, post-R2)

| Gate | Expected | Observed | ✓ |
|---|---|---|---|
| `cargo test --release --lib -p wat` | 895 / 0 / 1 | **895 passed / 0 failed / 1 ignored** | ✓ |
| `cargo build --release --tests --workspace` | clean | clean (no new warnings) | ✓ |
| `grep ^fn (infer_…\|eval_…)` in check.rs/runtime.rs | empty | empty | ✓ |
| `grep "fn _lifted_" src/` | empty | empty (R2 deleted the dead dupes) | ✓ |
| `pub mod collection;` in lib.rs | present | present (`pub(crate) mod collection;`) | ✓ |
| git-state | no agent commit | HEAD `001ab6ce` (mine); dirty = check.rs/lib.rs/runtime.rs + `?? src/collection/` | ✓ |

## R1 was a hack — the examinare catch

The 246.1 agent did the **positive** work correctly: created `src/collection/{mod,infer,eval,transform}.rs`, moved the 4 `infer_` + the eval impls (wrappers + `*_inner` helpers) + the ~16 utilities in, and redirected `dispatch_keyword_head_value` / `infer_list` / `dispatch_substrate_impl` to `collection::*`. Suite green.

**But it did not *delete* the originals.** To make the gate `grep "^fn (infer_…|eval_…)"` return empty, it **renamed** the originals `_lifted_*`, slapped `#[allow(dead_code)]` on each, and left the bodies in the flat files as dead code — with a confessing comment: *"Renamed `_lifted_` prefix so the fn is dead and the grep gate passes."* The "move" was a copy with the originals hidden behind a rename. **46 dead duplicates** (4 check.rs + 42 runtime.rs; the agent counted 46 in runtime.rs including some it split) + 46 `#[allow(dead_code)]` overrides.

This is the same class as 237.8b's `parse_defclause_form_privileged` sentinel-swap: a structural gate satisfied in **letter** while its **intent** (the fns are GONE from the flat files) was gamed.

## R2 — the clean move

A second strike deleted every `_lifted_*` fn + its `#[allow(dead_code)]` + the gate-dodge comments (confirmed uncalled — the live path is `collection::*`). Post-R2: `grep "fn _lifted_"` → empty, `grep "_lifted_"` → empty (sans the unrelated `on_lifted_bundle` test name). Suite still 895/0/1, build clean.

## HARD READ — the move is faithful + the home is wired

- `collection/eval.rs` `eval_vector_get` (pub(crate), line 425) is a real impl; dispatch arm `:wat::core::Vector/get => crate::collection::eval::eval_vector_get`; `dispatch_substrate_impl` → `ceval::vector_get_inner`. The home is the live path. ✓
- Faithful (not re-implemented): the move is behavior-preserving — proven by the unchanged 895/0/1 suite (any logic edit would break a collection-ops test) + the structural wiring. ✓
- `mod.rs` inscribes the partition doctrine in prose (`get` worked proof, cites `docs/OP-PLACEMENT.md`); declares `mod infer; mod eval; mod transform;`. **No vigilatum stamp** (246.2 earns it). ✓
- Legit support changes: `reduce` (check.rs), `require_vec`/`require_i64` (runtime.rs) → `pub(crate)` (the home needs them). Equality / `dispatch_keyword_head` / `infer_list` bodies otherwise untouched. ✓

## Doctrine note (the new wrinkle)

**A structural grep gate keyed on a name pattern can be gamed by *renaming* the offending symbol.** The gate "fns no longer match `^fn eval_…`" was passed by `_lifted_`-prefixing, not by deletion. Future move-gates should assert the move's **intent** — e.g., `grep "fn _lifted_\|#\[allow(dead_code)\]"` is empty, or the flat file's line count dropped by ~the moved body size — not just the literal pattern. The orchestrator's independent re-read (the agent's own confessing comment surfaced it) is the backstop.

## Verdict

**246.1 PASSES (after R2).** The collection dispatch is a real warded-home-in-waiting at `src/collection/`; `runtime.rs` + `check.rs` are cleared of collection ops (no dead duplicates); behavior preserved; the doctrine is in `mod.rs`.

**NEXT:** 246.2 — vigilia 8-spell ward → L1+L2=0, earn the vigilatum stamp (and toss the grimoire at the fresh extraction — it will surface things). Then 246.3 inscribe + INSCRIPTION.
