# BRIEF — Arc 249 Stone 249.2b-i — `macro_eval` + reroute the computed-unquote eval (closes F5)

**Mission.** Build the fenced macro evaluator `macro_eval` (a **default-deny validator + the existing
`runtime::eval`**) in `src/macros/eval.rs`, and reroute the computed-unquote eval through it. This
**closes F5** (the unsandboxed expand-time eval): an impure `,(expr)` now errors instead of running.
The stepping stone that proves the restricted evaluator + its purity gate, with minimal integration.

Turns probe gates **A (green→stays green) + B (RED→green)**. Leave C/D ignored (stone 249.2b-ii).

## What `macro_eval` is

> `macro_eval(form, env, sym) = validate_blessed(form)?; runtime::eval(form, env, sym)`

A recursive **default-deny validator** that walks the form and refuses any keyword head **not** on
the blessed allow-list, then runs the *already-validated* form through the existing `runtime::eval`.
The fence is the pre-walk — not threaded through `eval` — so there is no env-leak path and it reuses
all of `eval`. Lives in `src/macros/eval.rs` (the slot intueri reserved). Surface names (the fn, the
error variant) owed an **intueri cast** — propose them, cite the cast.

### The validator (`validate_blessed`)

Walk the `WatAST` form recursively. At each `WatAST::List` whose head is a `WatAST::Keyword`:
- **head ON the blessed allow-list** → recurse into the args.
- **head NOT on the list** → return `MacroError { kind: MacroErrorKind::ImpureInMacro { head } }`
  (name the variant via the intueri cast; this is the default-deny refusal).
- **SKIP quasiquote** (`:wat::core::quasiquote` / `:wat::core::quote` / `:wat::holon::Atom`
  template contents) — those are *data the runtime form will hold*, not expand-time code. (Mirror
  how `expand_form` already skips quote/quasiquote, expand.rs:~85.)

### The blessed allow-list — DEFAULT-DENY

The allow-list is the **pure-total subset of `dispatch_keyword_head_value`'s arms** (runtime.rs:5318).
Enumerate from that match. **Bless:** arithmetic / comparison / boolean (`:wat::core::i64::*`,
`f64::*`, `=`, `not=`, `and`/`or`/`not`, the polymorphic `+`/`-`/`*`), collection ops (`first`,
`rest`, `cons`, `conj`, `get`, `length`, `empty?`, `count`, `map`, `filter`, `foldl`, `foldr`,
`Vector`, `Tuple`, …), control (`if`, `cond`, `match`, `let`), keyword/symbol construction, and the
pure form ops. **DENY (and default-deny everything else):**
- the entire `:wat::kernel::*` namespace (IO, spawn, channels, signals),
- any mutation / `set-*!` / config-mutation head,
- `:wat::core::apply` + `:wat::core::eval-ast!` (dynamic-escape — they could route around the gate),
- any non-`:wat::core::` / non-`:wat::holon::` head (named user-`defn` calls — the totality vector),
- anything not explicitly blessed (the default).

**The suite teaches completeness.** Default-deny's only failure mode is *too strict* — a pure head
you forgot to bless makes a real computed-unquote fail (a RED test), never a silently-admitted
effect. So: build the list, run the full suite, and any false-refusal (a stdlib macro / probe gate A
that expands a pure `,(expr)`) names a head to add. Iterate to green. (Missing an *effectful* head
is harmless — it stays denied.)

## The reroute (F5 closure)

`unquote_argument` (expand.rs:~866) and `splice_argument` (expand.rs:~940) currently call
`crate::runtime::eval(&substituted, env, sym)`. **Swap both to `macro_eval(&substituted, env, sym)`.**
Update the two `BEWARE (arc 249 finding F5 …)` breadcrumbs to note F5 is now CLOSED here (gated by
`macro_eval`).

## Probe

Un-ignore **`mint_impure_computed_unquote_rejected`** (gate B) in
`tests/probe_arc249_macro_engine.rs` — it must pass (the impure `~(:wat::kernel::stopped?)` is now
refused). **Gate A** (`regression_pure_computed_unquote_preserved`) must stay green (the pure
`~(:wat::core::i64::+ …)` still expands through `macro_eval`). **Leave C/D `#[ignore]`d** (program
bodies = 249.2b-ii).

## Constraints (hard)
- Edits: `src/macros/eval.rs` (new), `src/macros/mod.rs` (declare + re-export the new module/fn),
  `src/macros/error.rs` (the `ImpureInMacro` variant + Display), `src/macros/expand.rs` (the 2-line
  reroute + breadcrumb update), `tests/probe_arc249_macro_engine.rs` (un-ignore B). No other file.
- Do NOT delete `expand_template`'s "must be quasiquote" gate yet (that's 2b-ii's body-model).
- Behavior-preserving for every *pure* computed-unquote — the existing suite + gate A prove it.
- No new deps. No `holon-rs`.

## Verify (plain single commands; vanilla cargo — no `./scripts/*` wrapper)
- `cargo build --release --tests`
- `cargo test --release --lib -p wat`  → **898 passed; 0 failed; 1 ignored** (unchanged)
- `cargo test --release --test probe_arc249_macro_engine`  → **2 passed; 0 failed; 2 ignored**
  (A + B green; C/D still ignored)

Do NOT commit, push, or run git — the orchestrator owns commits + the gate. Report the new
`src/macros/eval.rs`, the blessed allow-list you settled (+ any heads the suite forced you to add),
the diff stat, the command outputs, and any STOP.

## Refs
- DESIGN-STONE-249.2b.md § "The reachability boundary — DEFAULT-DENY" + § "Representation".
- `dispatch_keyword_head_value` (runtime.rs:5318); kernel arms (:6033+); `value_to_watast` (:10458);
  the restricted-eval precedent `eval-ast!` (refuses mutation, runtime.rs:~25718).
- The F5 sites + breadcrumbs: expand.rs:866/940. The probe: `tests/probe_arc249_macro_engine.rs`.
