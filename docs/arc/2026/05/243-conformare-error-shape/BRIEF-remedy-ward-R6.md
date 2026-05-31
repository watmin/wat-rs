# BRIEF — remedy/ ward R6 (verification-cast wake sweep)

**Target:** `src/remedy/` only. **Goal:** close the 3 findings the R5 verification vigilia raised, so the home converges L1+L2=0 and earns its vigilatum stamp.
**Mode:** sonnet writes; orchestrator re-casts + stamps. **Do NOT commit. Do NOT push. Do NOT write a vigilatum stamp.** Leave the tree dirty.

## Hard rules
- Anchor: `pwd` MUST be `/home/watmin/work/holon/wat-rs`. Any `.claude/worktrees/` path → `cd` to the anchor, use `git -C /home/watmin/work/holon/wat-rs`. Never operate in a worktree.
- Write scope: ONLY `src/remedy/*.rs`. Any edit reaching another file → STOP and report (do NOT silently edit outside the home — this happened in R5).
- This is cargo (Rust). Do NOT use `run_with_venv.sh`.
- No runes — all 3 findings are solvable + non-perf → must FIX.

## Fix 1 — struere L2: `Typo(u32)` → `Typo(NonZeroU32)` (make distance-0 unrepresentable)

A typo remedy's distance is always ≥ 1 (an exact match is not a typo). `RemedyKind::Typo(u32)` admits `Typo(0)` — an illegal state the construction site guards by runtime check. Make it a type guarantee.

- `mod.rs`: `RemedyKind::Typo(u32)` → `RemedyKind::Typo(std::num::NonZeroU32)`.
- `mod.rs` `score()`: `RemedyKind::Typo(distance) => distance` → `RemedyKind::Typo(distance) => distance.get()`. Retirement arm unchanged (`=> 0`).
- `mod.rs` `kind_annotation`: `RemedyKind::Typo(distance) => format!("typo, distance {distance}")` — `NonZeroU32` impls `Display`, so `{distance}` still renders the bare number. Verify the rendered string is unchanged (e.g. "typo, distance 1").
- `rank.rs` `nearest_matches` construction — fold the exact-match exclusion INTO the type. `levenshtein(a,b) == 0` iff `a == b`, so `NonZeroU32::new(dist)` returning `None` IS the exact-match case. Replace the body so the single filter is the type construction:
  ```rust
  .filter_map(|candidate| {
      let dist = NonZeroU32::new(levenshtein(needle, candidate))?; // None = exact match (distance 0) — not a typo
      (dist.get() <= threshold).then(|| Remedy {
          form: candidate.to_string(),
          kind: RemedyKind::Typo(dist),
          note: None,
      })
  })
  ```
  This REMOVES the now-redundant explicit `if candidate == needle { return None }` guard (the `?` on `NonZeroU32::new` subsumes it). Keep a one-line comment explaining distance-0 = exact match is filtered. (Cold bounded path; computing levenshtein for the rare exact-match candidate instead of early-returning is negligible — temperare graded this path L3.)
- Tests (`mod.rs`, `rank.rs`): every `RemedyKind::Typo(N)` literal → `RemedyKind::Typo(std::num::NonZeroU32::new(N).unwrap())`. `matches!(…, RemedyKind::Typo(_))` is unchanged. Any test asserting on a distance value reads via `.score()` (already returns `u32`) — unchanged.
- VERIFY: after the change, `RemedyKind::Typo(0)` does not compile anywhere; the exact-match test (`exact_match_excluded`) still passes (now enforced by `NonZeroU32::new(0) == None`).

## Fix 2 — intueri L1: stale `score` field in `retirement.rs` doc (R5 wake)

`retirement.rs` `retirement_lookup` doc comment shows the return as a struct literal `Remedy { kind: Retirement, score: 0, form: replacement, note }`. R5 REMOVED the `score` field (it's a `score()` method now). The struct-literal doc is a false claim — a reader learns a field that doesn't exist.

Fix: replace the struct-literal syntax with prose. E.g.:
> Returns `Some(Remedy)` for a known retired form — `form` is the replacement, `note` carries any migration caveat, and `score()` is `0` (an exact table hit, not a fuzzy distance). Returns `None` if `needle` is not retired.

Keep it accurate to the current shape (no `score` field; `score()` derives 0 for Retirement).

## Fix 3 — intueri L2: leaked workflow-vocab in `mod.rs` doc

`mod.rs:92-93` (the `RemedyKind` doc) contains `(See solvere fix below — this doc IS that fix.)`. "solvere" is an opaque spell-name to a code reader, and the parenthetical is self-referential. The preceding sentence ("variant declaration order IS the tiebreaker; DO NOT reorder") already states the constraint.

Fix: DELETE the parenthetical entirely. Leave the do-not-reorder warning standing on its own. Do not replace it with another cross-reference.

## Gates (all must pass)
```
cargo build -p wat
cargo test -p wat --lib remedy    # MUST stay 61 passed / 0 failed (explain any delta — e.g. a test you rewrote for NonZeroU32)
cargo test -p wat --lib           # root lib suite stays green (was 895 / 0 / 1)
cargo clippy -p wat --all-targets # no NEW warnings from src/remedy/
```
(`-p wat`; bare `cargo test --lib` runs 0 tests — never use it.)

## Report (your final message — the SCORE)
1. Files touched (must be only `src/remedy/*.rs`).
2. Per-fix disposition (1/2/3 — DONE, with line(s)).
3. Gate results with counts (`cargo test -p wat --lib remedy` N passed / 0 failed — state N).
4. Dirty set: `git status --porcelain` (only `src/remedy/*.rs` + the untracked R5/R6 briefs; NO other source file — if `src/check.rs` or anything else shows, you violated scope, STOP and say so).
5. Any honest delta.

Do NOT commit. The orchestrator re-casts struere + intueri (the two that diverged) on your dirty tree; converged → hashless vigilatum stamp + one atomic ward commit.
