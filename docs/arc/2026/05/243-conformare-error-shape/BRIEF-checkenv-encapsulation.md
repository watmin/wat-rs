# BRIEF — ⑦ CheckEnv encapsulation (struere F6, Stone 243.3)

You are sonnet. Close struere F6 (an R2 vigilia L1): `CheckEnv` exposes 5 `pub` fields that bypass its own accessor surface. The R3-β STOP-cap that deferred this was a FALSE PREMISE — verified below.

**Anchor cwd:** `/home/watmin/work/holon/wat-rs/`. Verify with `pwd`. Reject `.claude/worktrees/`.

## The false-premise correction (already verified by orchestrator)

R3-β's STOP-cap claimed "3 tests read binding_metadata directly." Those tests read `frozen.symbols.binding_metadata` — that is **`SymbolTable.binding_metadata`** (runtime.rs:1766, legitimately `pub`), a DIFFERENT struct from **`CheckEnv.binding_metadata`** (check.rs:1972). Verified: all 5 `CheckEnv` fields have ZERO external reads outside `src/check.rs`, and ZERO tests touch the `CheckEnv` struct directly. So this fix has NO external cascade — it is fully contained in check.rs.

**Do NOT touch `SymbolTable.binding_metadata` in runtime.rs — that field stays `pub` (tests + freeze.rs legitimately use it).**

## What to do — in src/check.rs ONLY

The 5 `CheckEnv` fields (around lines 1965-1975):
- `pub defined_values`
- `pub defined_value_spans`
- `pub binding_metadata`
- `pub redef_allowed`
- `pub defclause_registrations`

### Step 1 — downgrade all 5 to `pub(crate)`
Change each `pub <field>` → `pub(crate) <field>`. (Goal: no external write/read surface; in-crate access stays fine.)

### Step 2 — add a setter for redef_allowed
`check_program` mutates `env.redef_allowed` directly (search for `redef_allowed =` or `.redef_allowed`). Add:
```rust
pub(crate) fn set_redef_allowed(&mut self, flag: bool) {
    self.redef_allowed = flag;
}
```
near the other CheckEnv methods (around the `get_binding_metadata` accessor ~line 2111). Update the direct-mutation site(s) to call `self.set_redef_allowed(...)` / `env.set_redef_allowed(...)`.

### Step 3 — verify no breakage
`cargo build --release --tests --workspace` should compile clean (no external consumer exists). If ANY compile error names an external reader of these CheckEnv fields, STOP and surface verbatim — that would contradict the verified premise and needs orchestrator review.

## Gates (must hold)
```
cargo test --release --lib -p wat 2>&1 | tail -1      # 890/0
cargo test --release --test function 2>&1 | tail -1   # 8/0
cargo test --release --test probe_arc243_stone3_typeerror_pattern_a 2>&1 | tail -1  # 3/0
cargo clippy --release 2>&1 | grep -cE "^warning:"    # <= 894
```

## STOP triggers
1. Any compile error naming an EXTERNAL reader of a CheckEnv field (contradicts premise — surface)
2. Lib < 890 / function < 8 / probe < 3 / clippy > 894
3. SymbolTable.binding_metadata touched (off-limits — stays pub)
4. holon-rs touched (STOP-5)
5. Any file other than src/check.rs modified
6. Commit attempted (orchestrator commits)
7. 15 min elapsed

## Discipline
- Sonnet writes; orchestrator commits.
- DO NOT commit. DO NOT touch INTERSTITIAL. Scope is src/check.rs ONLY.

## Return paragraph (≤ 100 words)
- The 5 fields downgraded (confirm); setter added + call site(s) updated
- Build clean (yes/no); if any external-reader compile error, the verbatim text
- Gates (lib/function/probe/clippy)
