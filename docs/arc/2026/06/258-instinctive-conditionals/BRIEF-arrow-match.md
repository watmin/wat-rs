# BRIEF — `-> :T` annihilation, sub-strike 2: `match` (the hard one)

**North-star probe (RED at HEAD, verified):** `wat-tests/core/match-no-ascription.wat`
— bare `(match (Some 5) ((Some v) v) (:None 0))` fails *"match now requires `-> :T`"*.
GREEN when `infer_match` infers the result by **unifying the arm bodies** (the mechanism
`infer_if` already uses for its bare 3-arg form).

## The work — a CHECKER capability, not a syntax removal

`match` is the mirror-opposite of `if`: `infer_if` already has a bare-unify path;
`infer_match` *mandates* `-> :T` and ascribes to the declared `:T`. Give it the bare path.

1. **`infer_match` (check.rs:6295) — the heart.** Remove the `args[1] == "->"` mandate
   (the early-return ~6307) and the declared-`:T` parse (~6337). Re-index arms from
   `args[3..]` → `args[1..]` (scrutinee stays `args[0]`). **Replace "check each arm body
   against `declared_ty`" with "infer each arm body, UNIFY them into an accumulator"** —
   the result type is that unified type (mirror `infer_if`'s `unify(then, else)`,
   generalized to N arms). PRESERVE all the existing machinery untouched: shape detection
   (`detect_match_shape`), coverage tracking (Option/Result/Enum/Open), pattern binding,
   hash-destructure arms. Only the body-typing axis changes.
2. **Runtime — `eval` match + `step_match` (runtime.rs).** Re-index to the bare layout
   (arms at `[1..]`, no `->`/`:T` to skip). Behavior identical otherwise.
3. **Codemod — 143 sites** `(match scrut -> :T arm…)` → `(match scrut arm…)` via the
   **GENERIC** `:wat::fix::strip-arrow-ascription` (head-set `{:wat::core::match}`) — the
   tool from sub-strike 1, already proven. A thin entry-point
   `wat-scripts/fixes/strip-match-ascription.wat` (mirror the expect one).
4. **`.rs`-embedded fixtures** (e.g. `runtime.rs:28383` recv-match) — codemod can't reach
   `.rs` strings; hand-fix in the cascade (same as sub-strike 1's `28380`).

## ⚠ This sub-strike is ORCHESTRATOR-OWNED (the bootstrap forbids delegation here)

Unlike a normal sonnet build, **nothing can `cargo build` OR `cargo test` until the corpus
is codemodded**: the new `infer_match` rejects the 143 old-form sites, and the stdlib
freeze (build-time) chokes on the stdlib's own `match -> :T`. So the loop is inseparable:
write `infer_match` → stash it → build old binary → codemod corpus → unstash → rebuild new
→ read the cascade → fix `infer_match`/arms → repeat. That tight author-run-iterate loop
is the orchestrator's (see the BOOTSTRAP header in `wat/fix.wat`). A sonnet would write
blind. So: **author the Rust here, drive the bootstrap+cascade myself.**

## The cascade (the real risk — non-unifying arms)

After the codemod + new checker, most match sites' arms unify cleanly (the `-> :T` was
redundant). Some won't — the genuine cases where `-> :T` ascribed a *supertype* the arms
don't unify to (e.g. arms returning different concrete types joined under a common
declared type). Each such failure is a STOP-to-weigh: either the arms truly unify (fine)
or the site needs a real fix (ascribe an arm via `ann-form`, use `:wat::core::Value`, or
restructure). The fail-count is the progress meter; do NOT force a unify that isn't there.

## Gate
- `cargo test --test test match_no_ascription` → GREEN (bare match, arms unified).
- `grep -rnE 'match [^]]+ -> :' wat/ wat-tests/` (code, not comments) → zero.
- `cargo test --test test` at floor; `cargo test --lib` at 962/36 floor.
- `if`'s `-> :T` UNTOUCHED (sub-strike 3 fold-in); `readln`/`apply` untouched.
