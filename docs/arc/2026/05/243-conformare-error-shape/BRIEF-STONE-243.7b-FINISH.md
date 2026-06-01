# BRIEF — Stone 243.7b FINISH (recover an interrupted strike)

The original 243.7b Shadowdancer's connection dropped mid-cascade (conf wifi). It left the tree with the types MINTED (`EvalSignal`/`EvalBreak`/`From` in runtime.rs) and most of the subgraph cascaded, but **the build is red with a bounded set of remaining errors + housekeeping undone.** This brief finishes it. The base contract is `DESIGN-STONE-243.7b.md` + `BRIEF-STONE-243.7b.md` (read them); below are the orchestrator-decided fixes for the specific remaining problems.

**State at handoff:** HEAD `90a5ab6f` (STRIKE-READY; NOT committed). Working tree: `src/runtime.rs` + `src/runtime_error_edn.rs` modified; `tools/transform-evalbreak/` present (the ephemeral tool — must be DELETED); `src/check.rs` NOT yet touched; no SCORE yet.

## The two contract decisions (orchestrator-made — apply them, don't re-derive)

### A. The 4+ syntax errors = the transform tool appended `.into()` inside MATCH PATTERNS (illegal)

Sites cargo named: `src/runtime.rs:32861, 32891, 32894, 32916` — e.g. `Err(RuntimeError::UnknownFunction(name, _).into()) =>`. `.into()` is illegal in a pattern (LHS of `=>`).

**Fix:** in a match PATTERN, wrap — do NOT `.into()`:
- WRONG (what the tool produced): `Err(RuntimeError::X { .. }.into()) =>`
- RIGHT: `Err(EvalBreak::Diagnostic(RuntimeError::X { .. })) =>`

**This is broader than the 4 cargo surfaced.** `grep -nE "\.into\(\)\)\s*=>" src/runtime.rs` (and any `RuntimeError::.*\.into\(\)` in match-arm position) to find EVERY pattern-site the tool corrupted; fix each by wrapping in `EvalBreak::Diagnostic(...)`. **Construction sites (RHS / `return Err(...)`) where `.into()` lifts `RuntimeError → EvalBreak` are CORRECT — leave those.** The discriminator: pattern (before `=>`) → wrap; value (after `=>` / in `return`/`Err(...)` expr position) → `.into()` is fine.

### B. Contain `EvalBreak` to the eval subgraph — do NOT leak it into `freeze.rs`

The 6× `error[E0277]: ? couldn't convert EvalBreak → StartupError` mean `EvalBreak` was propagated INTO the startup layer. **freeze.rs is NOT eval subgraph.** The fix is NOT to add `From<EvalBreak> for StartupError`.

**Fix:** the top-level eval entry that `freeze.rs` calls must return `Result<_, RuntimeError>` (collapse `EvalBreak → RuntimeError` AT that boundary). A stray `EvalBreak::Signal` at top level is the interpreter-bug path the propagation handler / trampoline already covers (see runtime.rs ~25036/25064 + the signal Display messages) — collapse `EvalBreak::Diagnostic(re) => re` and handle `EvalBreak::Signal(_)` per the existing top-level stray-signal logic. Then `freeze.rs` keeps its `RuntimeError`-typed signatures and the existing `From<RuntimeError> for StartupError` (freeze.rs:598) works unchanged.

- Find where freeze calls into eval; ensure that entry returns `RuntimeError`, not `EvalBreak`. Revert any freeze.rs signature the original agent flipped to `EvalBreak`.
- **STOP trigger:** if the eval entry freeze calls GENUINELY cannot collapse cleanly (it legitimately must expose EvalBreak), STOP and surface — do NOT invent a new StartupError variant and do NOT add a bare `panic!`/`unreachable!` for the Signal arm unless that is the codebase's ESTABLISHED idiom for proven-impossible internal states.

## Then finish + housekeeping

1. Resolve the remaining `E0308` mismatches per the base contract (signal-path fn → `EvalBreak`; leaf → `RuntimeError`, `?` lifts; match arms use `EvalBreak::Diagnostic`/`Signal`).
2. Fix the `unused import: EvalSignal` (runtime_error_edn.rs:29) and any other warnings the split introduced.
3. `src/check.rs` doc-comment prose (8371, 8488, 14487): `RuntimeError::TryPropagate`/`OptionPropagate` → `EvalSignal::…`. No code.
4. **DELETE `tools/transform-evalbreak/` entirely** (`rm -rf tools/transform-evalbreak`; if `tools/` is then empty, remove it too). The ephemeral tool must NOT land.
5. Build any FURTHER corrective script as a **Rust Cargo binary — NEVER Python or shell (both are blocked in this environment).** Most of the remaining work is hand-fixes from the cargo error stream; you likely need no tool at all.

## Verify (run these; report verbatim)
- `cargo build --release -p wat` → clean (0 errors).
- `cargo test --release --test probe_arc243_stone7b_signal_split` → 4 / 0.
- `cargo test --release --lib -p wat` → **895 / 0 / 1** (parity — behavior-preserving).
- `cargo build --release --tests` → clean.
- `cargo clippy --release -p wat 2>&1 | grep -c result_large_err` → 0 (if it fires, box `EvalBreak::Diagnostic(Box<RuntimeError>)`; no `#[allow]`).
- `grep -nE "RuntimeError::(TailCall|TryPropagate|OptionPropagate)" src/` → 0.
- `git status --porcelain` → only `src/runtime.rs`, `src/runtime_error_edn.rs`, `src/check.rs` (+ the pre-existing `docs/DUNGEON-CRAWL.md` which is the orchestrator's, leave it); **NO `tools/`**.

## Deliverable
Write `docs/arc/2026/05/243-conformare-error-shape/SCORE-STONE-243.7b.md` (mirror `SCORE-STONE-243.6a.md`): the cascade size, the recovery (4 pattern-fixes + the freeze containment), probe 4/0, lib parity, clippy result, behavior-identical confirmation, tool deleted. Do NOT commit. Report back concisely: what you fixed, verify results verbatim, any STOP hit.
