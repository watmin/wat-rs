# INSCRIPTION — arc 242 `lexeme-role-doctrine` — CLOSED

**Arc 242 closes 2026-05-29 late.** Two foundational doctrines inscribed as substrate-enforced law. Three substantive stones across the arc:

| Stone | Substance | Commit |
|---|---|---|
| 242.1 | `:wat::core::Char` HARD CUT; bare nil verified-operational; doctrine memory + MEMORY.md index | `b4eb920f` |
| 242.2 | Doctrine 1 SELF-ENFORCING — type-check rejection arm for type-keyword-in-value-position; reflection emitter migration; 158-site test cascade | `9c8e8546` |
| 242.3 | INSCRIPTION (this doc; orchestrator-direct) | (this commit) |

## What this arc shipped

### Doctrine 1 — bare lexeme = value; keyword lexeme (`:wat::core::*`) = type

Codified as substrate-enforced law (not just convention). Type-check rejection arm at `src/check.rs` fires on every `:wat::core::*` keyword in value position with structured remedy per Stone 241.10's apparatus. Pre-arc-242 substrate was lenient (type-inference unified type-keyword-in-value-position); post-arc-242 the substrate REJECTS with doctrine-explicit error message.

Legal:
```scheme
(:wat::core::defn :f [] -> :wat::core::nil nil)
(:wat::core::defn :f [] -> :wat::core::i64 42)
(:wat::core::let [x nil] ...)
```

Illegal (post-arc-242):
```scheme
(:wat::core::defn :f [] -> :wat::core::nil :wat::core::nil)
(:wat::core::defn :f [] -> :wat::core::i64 :wat::core::i64)
(:wat::core::let [x :wat::core::nil] ...)
```

### Doctrine 2 — scalar types lowercase; non-scalar/container types PascalCase

Inscribed and operationalized via the `:wat::core::Char` → `:wat::core::char` HARD CUT. Char is scalar (single Unicode codepoint); must be lowercase per Doctrine 2. String stays PascalCase (sequence of chars; container).

Outstanding case-audit candidates flagged for future arcs (NOT this arc): `:wat::core::Uuid` → `uuid`; `:wat::core::Duration` → `duration`; `:wat::core::Instant` → `instant`. All queued in arc 109 territory per user direction.

## What this arc DID NOT do (affirmative out-of-scope)

- Uuid / Duration / Instant case-rename: queued elsewhere; not arc 242's scope
- Stone 241.11.fix round 1's 14 test migrations + 1 doc update: lost during 241.12 WIP discard; will redo when arc 241 resumes at Stone 241.12

## Doctrines + memories inscribed this arc

| Artifact | Purpose |
|---|---|
| `~/.claude/projects/.../memory/project_lexeme_role_doctrine.md` | Both doctrines verbatim + how to apply in future arcs |
| `~/.claude/projects/.../memory/feedback_sonnet_never_drafts_interstitial.md` | NEW lesson — INTERSTITIAL is orchestrator-exclusive chronicle; even drafting is the violation in writing |
| MEMORY.md | Index updated |

## The third bandaid-rip-with-receipts consumer

Stone 241.10's `src/remedy/` + ranked-remedy schema + RETIREMENT_TABLE apparatus is now demonstrably FOUNDATIONAL. Three substantive consumers shipped:

1. Stone 241.11 — `:wat::core::define` HARD CUT (271-site cascade via ephemeral auto-fixer)
2. Stone 242.1 — `:wat::core::Char` HARD CUT (~18 sites; 5th retirement-table entry)
3. Stone 242.2 — type-keyword-in-value-position rejection (positional enforcement, NOT form retirement; same Remedy struct, different RemedyKind context)

The pattern extends from "retired form" to "wrong-position form." Future enforcement work consumes the same apparatus.

## The lesson the violation taught

Stone 242.1's BRIEF authorized sonnet to "draft INTERSTITIAL for orchestrator review during commit." Sonnet drafted; orchestrator nearly committed. **User intervention:** *"sonnet is not allowed to author INTERSTITIAL - you are the author to that document."*

The framing "draft for orchestrator finalization" looked like the right delegation discipline but was the violation in writing — once realization-voice text from sonnet's pen lands on disk in INTERSTITIAL, the chronicle integrity is broken even if the orchestrator edits afterward. Memory `feedback_sonnet_never_drafts_interstitial` inscribed; Stone 242.2 BRIEF explicitly forbid INTERSTITIAL writes; sonnet honored.

This is the EIGHTH layer of substrate-as-teacher discipline operational this multi-arc stretch: the chronicle's authoring discipline catches its own violation via user intervention.

## What this unblocks

- **Stone 241.12** (defalias mint) — paused at STRIKE-READY (`e803e0f9`); resumes after this commit. Sonnet's lost work (Stone 241.11.fix round 1's 14 test migrations + 1 doc update) folds into Stone 241.12's scope.
- **Stone 241.13** INSCRIPTION closes arc 241 (the original parent arc; spawn-block discipline preserved)
- **Arc 237.8b** reopens after Stone 241.13 per `feedback_no_regression_until_arc_done`
- **Future case-audits** (Uuid, Duration, Instant) consume Doctrine 2 as the rule
- **Future EDN-fidelity work** consumes Doctrine 1 as the rule
- **Future positional-enforcement arcs** consume the bandaid-rip-with-receipts apparatus as the pattern (third consumer locked it)

The def*-prefix family completes at Stone 241.13's INSCRIPTION (when arc 241 closes). The lexeme-role-doctrine family lands here.

The substrate teaches; the doctrines stay; the rhythm holds.
