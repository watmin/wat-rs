# BRIEF — Arc 249 Stone 249.2a-R2 — drive `src/macros/` ward findings to convergence

**Mission.** Apply the vigilia R1 findings on `src/macros/` (7-spell guard: intueri/struere/solvere/
purgare/sequi/temperare + circumspicere). Fix every *fixable* finding; add 3 runes for genuinely
transitional code; add 3 missing tests. **Two findings are deliberately OUT OF SCOPE — leave them
untouched** (they are annihilated by the macro-eval engine, stone 249.2b; the stamp waits for that).
Behavior-preserving throughout — the lib suite must stay 895/0/1.

## DO NOT TOUCH (held for 249.2b — leave exactly as-is)

- **The computed-unquote eval path** `expand.rs` `unquote_argument` / `splice_argument` (the
  `crate::runtime::eval(...)` calls). circumspicere flagged it as an unsandboxed expand-time eval
  (an impurity/determinism hole). It is closed by the 249.2b engine's purity gate, not here. Do not
  add a gate, a rune, or a comment claiming it's bounded.
- **The `env`/`sym` threading** through the expand chain (struere's wrong-level finding). The
  eval-context is redesigned by 249.2b. Do not extract an `ExpandCtx` now.

If you find yourself editing either, STOP — they are the next stone's work.

## FIX (drive to zero)

### A. Visibility tightenings (purgare — 8 over-exports from the lift)
All are consumed only *inside* `src/macros/`; the lift bumped them too wide. Tighten to the minimum:
- `expand.rs`: `expand_form`, `expand_macro_call`, `substitute_bindings`, `unquote_argument` —
  `pub(crate)` → **`pub(super)`** (callers: expand/parse/tests, all children of `macros`).
- `parse.rs`: `is_defmacro_form`, `parse_defmacro_form` — `pub(crate)` → **`pub(super)`**.
- `registry.rs`: `macro_byte_equivalent` — `pub(crate)` → **private `fn`** (callers: same file only).
- `mod.rs`: `pub mod error` → **`pub(crate) mod error`** (parity with the other submodules;
  `MacroError`/`MacroErrorKind` stay `pub` — genuine public API).
- **Leave `pub` exactly as-is** on: `expand_all`, `expand_once`, `register_defmacros`,
  `register_stdlib_defmacros`, `MacroDef`, `MacroRegistry`, `MacroError`, `MacroErrorKind`,
  `EXPANSION_DEPTH_LIMIT` — these are re-exported and used crate-wide (verify with
  `grep -rn "crate::macros::<name>" src/`).
- After tightening, `cargo build` confirms `pub(super)` reaches `macros::tests`.

### B. Stale comments (intueri + purgare — Honest-lens)
- `expand.rs:~98`: the comment cites `expand_macro_call line 531` — a line number from the OLD flat
  file. Drop the number: `(fixpoint loop in expand_macro_call)`.
- `parse.rs:~18`: the doc-link `[`expand_form`]` no longer resolves (it moved to `expand.rs`). Fix to
  `[`expand::expand_form`]`.
- `error.rs:~15`: the `// NOTE: Must be pub...` comment is wrong twice (names `MacroError` while
  above `MacroErrorKind`; claims a lib.rs re-export that doesn't exist). Rewrite to the true reason:
  `// MacroErrorKind is pub because it's the type of MacroError's pub `kind` field (no private-in-public).`

### C. Doc-honesty (circumspicere F1+F2 — claim-vs-code, HIGHEST severity)
Rewrite the `mod.rs` module doctrine so the home's contract tells the truth about what it implements:
- The "What this slice supports" enumeration currently lists only quasiquote bodies, fixpoint, and
  hygiene. **Add the four implemented dispatch forms**: threading macros `->`/`->>`, `keyword/of`,
  the `for`-comprehension in splice position, and **computed-unquote** (`,(expr)` evaluated at expand
  time via `runtime::eval`, arc 143). For threading + `keyword/of`, note they are *transitional
  Rust desugars* (rehomed to wat code in arc 249's later stones).
- The "What's deferred" section currently says computed/conditional templates are deferred — but
  computed-unquote SHIPS. **Move computed-unquote out of "deferred"** into "supported"; leave only
  genuinely-absent capabilities (arbitrary recursion / conditionals in macro bodies) as deferred.
- Add a one-line **file-map** to the doc (intueri's gap): `registry` = storage · `parse` = form →
  MacroDef · `expand` = call-site → AST · `error` = the error type.

### D. Craft fixes
- **`parse.rs` `parse_defmacro_form`** (struere): the `.next().expect("len=6")`/`("len=7")` chain
  (6–7 calls) after a `match items.len()` — replace with a slice-pattern destructure
  (`if let [_, name, argvec, arrow, rettype, body] = items.as_slice()` for the len-6 arm, analogous
  for len-7) so the arity is enforced by the pattern, not a panic string. Behavior identical.
- **Move `expand_once`** from `parse.rs` to `expand.rs` (solvere): it's `macroexpand-1`, the sibling
  of `expand_all`, and it calls `expand::expand_macro_call`. Move the fn; update its `use`; update
  the `mod.rs` re-export from `pub use parse::expand_once` → `pub use expand::expand_once`. Keep it
  `pub`.
- **De-dup `walk_template`** (solvere L2): the List arm and Vector arm contain near-identical
  ~50-line `for`-comprehension + splice blocks differing only in the final container constructor
  (`WatAST::List` vs `WatAST::Vector`). Extract one helper —
  `fn splice_children(items, bindings, macro_scope, macro_name, call_site_span, depth, env, sym) ->
  Result<Vec<WatAST>, MacroError>` — that both arms call, each then wrapping the result in its own
  container. Behavior identical (the existing macro tests + `probe_arc248` prove it).

### E. Tests (circumspicere F3+F4 — the untested invariant + negative space)
Add to `src/macros/tests.rs` (in-crate, `#[cfg(test)]`):
- **Depth-limit guard** (F3 — load-bearing, currently asserted by NOBODY): a self-recursive macro
  whose expansion re-emits a call to itself, expanded via `expand_all`, `unwrap_err()` matching
  `MacroErrorKind::ExpansionDepthExceeded`. This must go red if the `depth > EXPANSION_DEPTH_LIMIT`
  check is ever removed.
- **`expand_once`** (F4): a direct unit test — register a macro, call `expand_once` on a call site,
  assert it expands ONE step (not to fixpoint).
- **`register_stdlib` bypass** (F4): assert the privileged reserved-prefix path registers a
  `:wat::*` macro that the non-privileged `register` would reject — the security-relevant gate, now
  directly exercised.
- (Threading + `for` already have integration coverage — `tests/probe_arc249_threading.rs` 6/0 and
  `probe_arc248`; no new home-unit-tests for those transitional desugars.)

## RUNE (genuinely transitional — about to be deleted/rehomed by a named imminent stone)

- On `expand_form`'s built-in dispatch block (the `keyword/of` + `->`/`->>` arms):
  `// rune:solvere(historical-shape) — keyword/of + threading are transitional in-pass Rust desugars; HARD-CUT when reborn as wat code in arc 249.3/249.4. Splitting to a module now would mint a home destined for deletion.`
- On `match_unquote` / `match_for_comprehension` (the recognizers in expand.rs):
  `// rune:solvere(historical-shape) — template-local recognizers kept beside their sole caller walk_template; match_for_comprehension rehomes with the for-comprehension in arc 249.4.`

## Constraints (hard)
- Edit **only** `src/macros/*.rs`. No other file (verify `git status` shows only `src/macros/`).
- **Behavior-preserving.** The moves/extractions/destructures must not change semantics. No logic
  changes beyond the explicit fixes above.
- Do NOT touch the two held findings (computed-unquote eval; env/sym threading).
- No new dependencies. No `holon-rs`.

## Verify (plain single commands; vanilla cargo — no `./scripts/*` wrapper)
- `cargo build --release --tests` — clean.
- `cargo test --release --lib -p wat` — **898 passed; 0 failed; 1 ignored** (895 baseline + 3 new
  tests). Confirm the 3 new tests are present and green.
- `cargo test --release --test probe_arc249_threading` — **6 passed** (threading unaffected).

Do NOT commit, push, or run git — the orchestrator owns commits + the gate. Report: `git diff --stat`,
the new test names + outputs, and any STOP.
