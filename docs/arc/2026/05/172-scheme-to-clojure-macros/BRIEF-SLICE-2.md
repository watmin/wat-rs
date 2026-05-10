# Arc 172 slice 2 — BRIEF

**Consumer sweep; sonnet.** Migrate every Scheme-style
quasiquote use (`,name` / `,@list`) to Clojure-style
(`~name` / `~@list`) across all wat sources. Closes the
intentional RED state slice 1 left.

Slice 1's lexer change made commas pure whitespace at the
main lex loop; `~name` and `~@list` are the new Unquote /
UnquoteSplicing tokens. Existing macro bodies in wat sources
broke as predicted; this slice sweeps them.

**Atomic-commit pair with slice 1.** Per recovery doc §
"Atomic commit across coordinated sweeps" — slice 1 ships
its lexer change uncommitted in the working tree; slice 2
sweeps the consumers; both commit TOGETHER when workspace
returns to green.

## Mission

Transformation rules (mechanical):
- `,<identifier>` → `~<identifier>` (unquote)
- `,@<identifier>` → `~@<identifier>` (unquote-splicing)
- `,,<identifier>` → `~~<identifier>` (nested unquote per arc 029; double-comma → double-tilde)
- `,@,@<identifier>` → `~@~@<identifier>` (similar nesting)

Comma stays valid INSIDE:
- `(...)` tuple keyword bodies (arc 171 carve-out)
- `<...>` parametric keyword bodies (slice 1f-W rule)
- String literals (always; comma in string is just text)

Apostrophe-in-keyword-body convention from arc 171 is
unchanged — `:wat::core::op'2` etc. parse the same way.

## Scope

Pre-grep 2026-05-10:
- **`wat/` substrate stdlib**: 7 files use macro quasiquote
  (`wat/test.wat`, `wat/core.wat`, `wat/holon/Amplify.wat`,
  `wat/holon/Trigram.wat`, `wat/holon/Project.wat`,
  `wat/holon/Subtract.wat`, `wat/holon/Bigram.wat`)
- **`wat-tests/`**: verify scope at sweep time (existing test
  macros may use unquote)
- **`examples/`**: verify scope at sweep time
- **`crates/*/wat/`** + **`crates/*/wat-tests/`**: verify
  at sweep time
- **`wat-scripts/`**: verify at sweep time
- **`tests/wat_*.rs` embedded wat strings**: verify if any
  embedded macro bodies use comma-unquote

## What to NOT do

- **No lexer / parser changes.** Slice 1 shipped those; this
  slice is consumer-only.
- **No commit.** Atomic-commit pair with slice 1; orchestrator
  commits both together when workspace is green.
- **No archived arc docs.** `docs/arc/*/complected-*/` stays
  unchanged per "what is inscribed is inscribed."
- **No commas inside `(...)` tuple bodies or `<...>` parametric
  bodies touched.** Those are arc 171's domain (still valid
  separators).
- **No INSCRIPTION authoring.** Orchestrator handles closure
  paperwork.

## Substrate-grep citations

- Pattern: `` ` `` followed by anything containing `,name` or
  `,@list` at quasiquote depth — find via
  `grep -rEln '\`\([^"]*,@?[a-zA-Z]'` across all `.wat` +
  `tests/*.rs`
- Arc 029 (nested-quasiquote) — `,,foo` semantics; nested
  unquote depth tracking
- Memory `feedback_apostrophe_dispatch_separator.md` — apostrophe
  convention from arc 171; unchanged in this slice

## Test approach

Phase the work:
1. Sweep `wat/test.wat` first (the deftest macros) — biggest
   concentration
2. Sweep other `wat/*.wat` files including `wat/holon/*`
3. Sweep `wat-tests/*` if any
4. Sweep `examples/*` if any
5. Sweep `crates/*/wat*/*` if any
6. Sweep `wat-scripts/*` if any
7. Sweep embedded wat strings in `tests/wat_*.rs` if any

After each phase: `cargo test --release --workspace --no-fail-fast`
should trend toward green. Final target: 1334 passed / 854
failed (arc 171 baseline) ± 5.

## Ship criteria

| Row | What | Pass criterion |
|-----|------|----------------|
| A — All `wat/*.wat` macro bodies use `~` | grep finds 0 `,name` / `,@list` patterns in quasiquote bodies in wat/ | ✓ |
| B — `wat-tests/`, `examples/`, `crates/*/wat*/` swept | similar grep returns 0 hits | ✓ |
| C — Tuple commas preserved | `:(A,B,C)` patterns unchanged; grep finds them intact | ✓ |
| D — Parametric commas preserved | `<K,V>` patterns unchanged | ✓ |
| E — Workspace at 1334/854 ±5 | `cargo test --release --workspace --no-fail-fast` count | ✓ |
| F — `cargo check --release` green | no compile errors | ✓ |
| G — Slice 1f-α tests green | `cargo test --release --test wat_arc170_slice_1f_alpha_helpers` → 10/10 | ✓ |
| H — Zero new dependencies | Cargo.toml unchanged | ✓ |
| I — Honest deltas surfaced | per FM 5 | ✓ |

## Honest delta categories

- **Nested unquote (`,,`) sites** — arc 029 semantics. If
  any `,,foo` exists (double-unquote for macro-generating-macro
  shapes), surface for explicit `~~foo` migration.
- **String-literal commas** — `"`...,..."` is unchanged
  (commas in strings are just text). Sonnet must distinguish.
- **Comment-containing commas** — `;; commas in comments`
  unchanged. Sonnet distinguishes via line-comment context.
- **`tests/wat_*.rs` raw strings with embedded macro
  bodies** — surface any sites where comma-unquote lives in
  embedded raw strings.
- **Doc-comments containing example wat syntax** — Rust
  doc-comments may show example wat code with comma-unquote
  for illustration. Update those examples too OR leave (they're
  examples of historical syntax; could go either way; surface
  for orchestrator).

## Predicted runtime

60-90 min sonnet. ~7 wat files in substrate stdlib confirmed;
additional file count surfaces during sweep. Mechanical
migration with cargo-test validation per phase.

**Hard cap:** 180 min.

## Reference

- DESIGN.md (this arc; broader scope)
- BRIEF-SLICE-1.md + slice-1 in-working-tree state (lexer
  shipped `~`/`~@` tokens; comma is whitespace)
- Arc 029 DESIGN.md (nested-quasiquote semantics)
- Memory: `feedback_apostrophe_dispatch_separator.md`
