# BRIEF — Stone 243.M — the sister-walk (meaningful spans at the parser/eval boundary)

The error-shape class is structurally eliminated (every error type is Pattern A — a location field is mandatory by construction). This stone makes those mandatory locations **meaningful**: replace lazy `Span::unknown()` in error constructions with the real span that is in scope, and broaden the few helper fns that receive a bare slice with no span. Closes the "ArityMismatch-style defensive class at the boundary" (arc 138 deferred it as "cross-file broadening OOS" — that broadening is now in scope).

## The target
~66 `RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::ArityMismatch { .. } }` sites (grep `ArityMismatch` + `unknown` across `src/`), plus any other `Span::unknown()` error constructions in eval/parse code where a real span is available. They cluster in: `io.rs`, `time.rs`, `string_ops.rs`, `thread_io.rs`, `fork.rs`, `assertion.rs`, `spawn.rs`, and similar kernel-verb evaluators.

## The rule (per site)
1. **If a span is already in scope** (`list_span`, `form_span`, `head_span`, `call_span`, etc. — most eval fns receive one), USE IT in the error's `span` field instead of `Span::unknown()`. This is the bulk — a simple in-place substitution.
2. **If the construction is inside a helper that receives a bare slice with no span** (e.g. `fn arity(op, args, n)` at `io.rs:678`, and any sibling arity/validation helpers), broaden the helper: add `list_span: &Span` to its signature and thread it from every caller (the callers are eval fns that have `list_span` in scope). Then use it in the error.
3. **Leave `Span::unknown()` ONLY** where the construction is at a genuinely synthetic site with no originating source node (and say so in the SCORE — affirmative, not lazy).

## Method
Most sites are in-place `Span::unknown()` → `list_span.clone()` (or `*list_span` / the in-scope span) substitutions — do these by hand or with a small Rust tool. The helper-broadening (rule 2) is a handful of signature changes + caller threading; cargo names every caller, iterate to green. Keep shell simple (one vanilla command per line). If you write a tool, put its content-integrity check inside it (compare `original.chars().filter(|c| !c.is_ascii()).count()` to the rewritten count per file; refuse to write on mismatch).

## Discipline
Behavior-preserving except the span value carried (errors gain real locations; no message/semantics change). No new error variants. No `#[allow]`. Do NOT commit; leave the tree dirty.

## Verify — one simple command per line
- `cargo build --release -p wat`
- `cargo build --release --tests`
- `cargo test --release --lib -p wat`  (expect 895 / 0 / 1)
- `cargo clippy --release -p wat`  (result_large_err stays 0)
- `grep -rn "ArityMismatch" src/`  (report how many still construct with `Span::unknown()` and confirm each remaining one is genuinely synthetic)
- `ls tools`  (gone, if you built one)

## Deliverable
Write `docs/arc/2026/05/243-conformare-error-shape/SCORE-STONE-243.M.md`: how many sites threaded a real span, which helpers were broadened (signature + caller count), how many `Span::unknown()` remain and why each is genuinely synthetic, lib parity, content-integrity (if a tool was used). Final message: counts + verify results verbatim + any blocker.
