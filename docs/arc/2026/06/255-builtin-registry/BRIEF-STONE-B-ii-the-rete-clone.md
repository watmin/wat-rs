# STONE B-ii — the rete DSL clone: `:wat::rete::core::{i64,f64}::` → `:wat::rete::{i64,f64}::`

DRAWN + BRIEFED 2026-08-26 against `ae2330bc1`.
DESIGN: `DESIGN-STONE-the-numerics-get-their-homes.md`. Prior art: **B-i** (`ae2330bc1`) moved the
core half — read its commit message before you start; its six-consumer finding is why this brief
lists consumers instead of claiming there are none.

## ★ THE INVARIANT THAT SHAPES THIS STONE — the rete name is DERIVED, not chosen

`src/rete/vocabulary.rs:69` states it, and **a gate enforces it** (`:1583`,
`rete_name_is_core_name_with_rete_inserted_after_wat`):

```rust
let expected = op.core_name.replacen(":wat::", ":wat::rete::", 1);
assert_eq!(op.rete_name, expected);   // modulo NAMING_RULE_EXCEPTIONS
```

> *"`rete_name` = `core_name` with `rete::` inserted immediately after `wat::`. One rule, no exceptions."*

**So you cannot rename the rete spelling on its own.** `:wat::rete::i64::+` is only reachable by
setting that row's **`core_name` to `:wat::i64::+`** and letting the rule derive the rest:

```
core_name ":wat::core::i64::+"  ->  rete_name ":wat::rete::core::i64::+"   (today)
core_name ":wat::i64::+"        ->  rete_name ":wat::rete::i64::+"         (target)
```

The gate is your safety net: get the pairing wrong and it goes red immediately. **Do not add a
`NAMING_RULE_EXCEPTIONS` entry to make it pass** — that is silencing the instrument.

## The consumers — measured, not assumed

B-i's brief said "zero Rust changes" and the rename found **six** consumers of one name, each a
hand-maintained table keyed by a literal string. This one has four, already located:

| # | site | what it holds |
|---|---|---|
| 1 | `src/rete/vocabulary.rs` | **30** numeric occurrences — the `core_name`/`rete_name` rows |
| 2 | `src/rete/expr_ir.rs` | **25** match arms keyed on the **core** spelling (`:wat::core::i64::+ => I64Add`) |
| 3 | `src/check.rs:2365` | `match core_name { … }` — the rete-form check path |
| 4 | the corpus | **402** i64 + **39** f64 `.wat` sites |

Plus test drift: 20 `tests/rete/**` and `tests/types/**` files name these spellings in fixtures or
assertions. Expect a cascade; it is the progress meter, not a crisis.

## ★ THE TOOL YOU ALREADY HAVE — do not write a second one

`fold_numeric_home` (`src/runtime.rs`, above `dispatch_substrate_impl`) already folds a new-spelling
per-type numeric onto its `:wat::core::` twin, and already excludes `max-of`/`min-of` (the one place
"same operation, two spellings" is FALSE — builder ruling 2026-08-26, *"keep variadic - clojure is
the destination"*).

**Consumers #2 and #3 should FOLD, not grow a second copy of their arms.** Make `fold_numeric_home`
`pub(crate)` and call it before the match. One function, one place, one exclusion list.

⚠ **And read its doc comment before you move it.** It lives OUTSIDE `dispatch_substrate_impl` on
purpose: the rete purity completeness gate censuses that function's *body* for `":wat::…"` literals
to find every dispatched verb, and it read an inline prefix and a `format!` template as three
unclassified VERBS. Wherever you place code with verb-shaped literals, check it is not inside a
scanned body.

## The corpus half — a wat-fix RULES codemod

Copy `wat-scripts/fixes/rename-core-numerics-to-their-homes.wat` (B-i's, committed and proven) —
two rules become two rules, only the prefixes and the `subs` offsets change (`:wat::rete::core::i64::`
is 23 chars, not 17). Its two traps still apply and are not optional: `rename-keyword-prefix` is a
**silent no-op** on `::`-terminated prefixes, and the **KEYWORD-ONLY** guard is mandatory.

**Get the population from wat-grep, not from text.** B-i's brief carried a text census of 1613 where
the structural count was 1495 — 76 of the difference being comments and string literals the codemod
must not touch. `wat-scripts/grep/core-numerics-ops.wat` is the shape; adapt it for the rete prefixes
and let its output be the bar.

## Your role

cwd `/home/john/work/holon/wat-rs`; run `pwd` first. **Ending your turn ENDS you** — every command
FOREGROUND, blocking. **You may not spawn sub-agents.** Do not commit, push, stash, revert, or
`git checkout`; `git stash@{0}` must never be touched.

You may run `cargo build --release`, `./target/release/wat --grep|--check <f>`,
`./target/release/wat <f>`, and single named tests. **Not** the floor, **not** clippy.

## STOP triggers — each rejects

1. **STOP-1 — the pairing gate can only be satisfied by a `NAMING_RULE_EXCEPTIONS` entry.** Report
   the row; do not add the exception. The gate is the invariant, not an obstacle.
2. **STOP-2 — a consumer needs a second copy of its match arms** rather than a fold. Report which
   and why the fold cannot reach it.
3. **STOP-3 — a `.wat` site needs a SHAPE change.** `max-of`/`min-of` diverge; the corpus had zero
   such sites in B-i. If the rete corpus has one, the census was wrong and I want it before any edit.
4. **STOP-4 — a room's line number does not hold.** Written against `ae2330bc1`.

## Acceptance — every row derives its bar

```bash
# 1. the old rete spelling is gone from .wat. BAR: 0.
git grep -oE ':wat::rete::core::(i64|f64)::' -- '*.wat' ':!docs' | wc -l

# 2. the pairing gate — the invariant that makes the rename correct.
cargo test --release --lib rete::vocabulary::naming_rule_tests

# 3. the rete corpus still loads and type-checks.
cargo test --release --test lint every_wat_scripts_file_loads_on_the_current_runtime

# 4. codemod idempotence — second run, zero matches.
./target/release/wat --grep wat-scripts/fixes/<your-codemod>.wat   # after applying

# 5. the builds.
cargo build --release && cargo build --release --all-targets
```

## Report back with

- Each command's actual output, naming the command that produced each number.
- **The wat-grep population vs any text count**, and the difference explained.
- **The /tmp dry-run diff**: files, hunks, and confirmation every hunk is a prefix rewrite.
- Which consumers you folded and which needed real edits, with `file:line`.
- The cascade's waterfall, and for each number which command produced it.
- Anything the brief got wrong. What you did NOT do, and why.
