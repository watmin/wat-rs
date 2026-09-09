# TABLE — STONE purgare: the cascade

Six sites touched across three files to take `cargo clippy --release --all-targets -- -D warnings`
from 5 `dead_code` errors to 0. Five were the named diagnosis; the sixth was the one cascade the
brief predicted ("removing `pattern_coverage` may orphan `Coverage` itself, its helpers, or their
imports") — surfaced not as a new clippy warning but as a hard `E0599` compile error the moment the
`Wildcard` variant was deleted, because the enum's only other consumer matched on it exhaustively.

| # | Round | Item | File | What orphaned it |
|---|-------|------|------|-------------------|
| 1 | 1 (named in brief) | `fn pattern_coverage` (+ its doc comment) | `src/check.rs:6990` | Superseded by `cover_variant_arm` (live, `src/check.rs:6407`/6686) at `480f38d05`. Confirmed: every construction site of `Coverage::Wildcard` lived inside this function's body only. |
| 2 | 1 (named in brief) | `enum Coverage` variant `Wildcard` | `src/check.rs:6580` (enum decl) | Never constructed in reachable code — its only constructors were inside `pattern_coverage` (item 1), itself unreachable. |
| 3 | 1 (named in brief) | `fn try_match_pattern_ast` | `src/runtime.rs:13610` | Arc 068 β-reduction step rule; no external callers (only self-recursive within its own dead body). Superseded by the current evaluator's dispatch. |
| 4 | 1 (named in brief) | `fn substitute_many` | `src/runtime.rs` (immediately after `substitute`) | Same arc 068 lineage as item 3; sibling `substitute` (singular) stays live — it is still called from three sites (`src/runtime.rs:13362`, `:13366`, `:13865`, line numbers pre-edit). `substitute_many` itself had zero callers anywhere in the tree. |
| 5 | 1 (named in brief) | field `ident_span` on `MatchArm::Binding` | `src/match_arm.rs:37` | Populated at construction (`src/match_arm.rs:85-88`) but read at none of its three consumption sites (`src/closure_extract.rs:1353`, `src/check.rs:6336`, `src/runtime.rs:8653` — all destructure `{ ident, .. }`). |
| 6 | 2 (compiler-forced cascade of #2) | match arm `Some(Coverage::Wildcard) => { wildcard_seen = true; covers_option_none = true; covers_option_some = true; covers_result_ok = true; covers_result_err = true; }` | `src/check.rs:6453` (in `infer_match`, the caller of `cover_variant_arm`) | Deleting the `Wildcard` variant (item 2) made this exhaustive-match arm a hard compile error (`E0599: no variant … Wildcard`) — not a new `dead_code` warning, since Rust's `match` requires every arm name a real variant. The arm was already dead in practice: `cover_variant_arm` (the ONLY function whose `Option<Coverage>` return feeds this match) never constructs `Wildcard` — confirmed by grepping every `Coverage::Wildcard` construction site and finding all three (`src/check.rs` original lines 7025/7115/7119, inside `pattern_coverage`) inside the already-dead function from item 1. Its sibling local `wildcard_seen` is set from three OTHER live arms (list-arm, hash-destructure arm, generic-symbol arm) — those stay untouched; only the one unreachable branch was removed. |

## Rounds

- **Round 1** — the five named items deleted verbatim per the brief's diagnosis. `cargo build --release`
  after round 1 produced exactly one error: `E0599` at `src/check.rs:6453` (item 6 above).
- **Round 2** — deleted the orphaned match arm (item 6). `cargo build --release` then succeeded clean,
  and `cargo clippy --release --all-targets -- -D warnings` exited 0 on the first try — no further
  `dead_code` warnings surfaced.

Cascade size: **6 sites, 2 rounds** — under the ~20-item STOP-2 threshold; not a campaign.

## What was left alone (checked, not touched)

- `fn substitute` (singular) — confirmed live (3 call sites), not deleted.
- Two prose comments referencing `pattern_coverage` by name in surrounding doc comments
  (`src/check.rs:7466`, `:7779`) and one referencing `try_match_pattern_ast`
  (`src/runtime.rs:13577`) are now stale historical references to a deleted function name. They are
  comments only — no compile or clippy effect — and editing them was out of scope for a
  delete-only/follow-the-compiler stone (STOP-1 territory: touching them is not forced by the
  compiler and changes no behaviour, but rewriting prose beyond the mechanical deletion is the kind
  of "while I'm here" the brief rules out). Flagging here for the orchestrator's awareness rather than
  silently leaving them.
- An unrelated local variable also named `ident_span` in `src/macros/expand.rs:584` (a different
  function, different struct, no relation to `MatchArm::Binding`) — left untouched.
