# SCORE — Arc 248 Stone 248.1 — the `for` template comprehension

Scored against an **independent orchestrator re-run on disk**, not the agent's
self-report. The agent reported clean; this score is what the disk says.

## Gates (independent re-run)

| Gate | Expected | Observed | ✓ |
|---|---|---|---|
| `cargo test --release --test probe_arc248_macro_for_comprehension` | 3 / 0 / 0 | **3 passed / 0 failed / 0 ignored** | ✓ |
| `cargo test --release --lib -p wat` | 895 / 0 / 1 | **895 passed / 0 failed / 1 ignored** | ✓ |
| `cargo build --release --tests --workspace` | clean | clean (pre-existing warnings only) | ✓ |
| git-state | no agent commit, no strays | HEAD `504decf4` (mine); dirty = `src/macros.rs` + probe; zero `??` | ✓ |

The 2 mints un-ignored: `mint_for_yields_elements` (`(for [x items] ~x)` ≡ `,@items`,
→ first `10`) and `mint_for_transforms_per_element` (`(for [x items] (i64::+ ~x 1))`,
→ first `11`). The regression (`,@rest` splice) stays green.

## The HARD READ — hygiene + boundedness (the load-bearing axis)

Read from `git diff -- src/macros.rs` this session, not the agent's prose.

- **No leak / no persist** ✓ — each iteration is `let mut iter_bindings = bindings.clone();
  iter_bindings.insert(binder_name, element.clone());` — cloned *from the original* every
  pass. The binder never mutates the shared `bindings`, so it cannot persist across
  iterations or leak to the call site. Clone-per-iteration (not clone-once-mutate) is the
  correct shape.
- **Reuses sets-of-scopes, no parallel hygiene** ✓ — the recursive `walk_template` is handed
  the *same* `macro_scope`; template-origin symbols hit the existing `Symbol` → `add_scope`
  arm. The binder is reached via *explicit unquote* (`~x`) — the same path as a macro param,
  explicit substitution rather than a free symbol, so there is no capture surface.
- **Bounded — map, not eval** ✓ — `match_for_comprehension` is a pure pattern-match (exactly
  3 items; head `:wat::core::for`; second a `Vector` of exactly 2 `Symbol`s). The loop is
  iterate → clone → insert → walk → push. No recursion beyond the existing `walk_template`
  descent, no conditionals, no expansion-time computation. The quasiquote-only virtue holds;
  `for` is the one sanctioned extension.
- **`depth` passed unchanged (= 1)** ✓ — the element template is walked at splice depth, so
  `~x` fires; `transforms_per_element` (→ 11) proves the unquote fired per element.
- **Honest errors** ✓ — wrong-type list binding → `MalformedTemplate`; missing binding →
  `UnboundMacroParam`. No silent mis-expansion.

## One note (candidate, not a defect)

The `for` recognition block is **duplicated across the `List` and `Vector` arms** of
`walk_template` (~50 lines, character-identical). It is NOT a new asymmetry: it mirrors the
pre-existing `splice_argument` duplication already present in those same two arms, so it
follows the function's established idiom. A future cleanup could extract
`expand_for_comprehension(splice_arg, bindings, macro_scope, …) -> Result<Vec<WatAST>>` and
call it from both arms — but that touches the pre-existing splice duplication too, so it is
its own small stone, not 248.1's debt. Flagged here so it is not silently inherited.

## Scope guard (held)

`for` is the only new template form — no `if`/`when`/`cond`, no nested-list flattening beyond
the single splice, no expansion-time function calls. Existing quasiquote/unquote/splice
behavior unchanged (regression gate + lib 895/0/1 guard it). No `holon-rs`.

## Verdict

**248.1 PASSES.** The bounded `for`-comprehension is minted, hygiene reuses the existing
sets-of-scopes machinery (verified by read, not trusted by report), boundedness is structural,
gates are green on an independent re-run. The tool the chain descended to build now exists.

**NEXT:** 248.2 — use `for` to make `=`/`not=` a macro-generated `defclause` (the equality
consolidation 237.8c's Shape B deferred) → 237.8d (collections STAY intrinsic) → 237.9
(INSCRIPTION) → 237 dies.
