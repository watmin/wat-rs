# DESIGN — Stone 242.1 — bare `nil` lexer + value-position migration + `:wat::core::Char` HARD CUT + doctrine inscription

**Status:** READY (sub-DESIGN). Stone 242.1 is the only substantive stone in arc 242 (Stone 242.2 is INSCRIPTION paperwork). Vigilia NOT required (D7 default; legacy flat substrate).

## Scope

Per arc 242 DESIGN.md: codify the two doctrines + apply them as concrete cleanups for nil and Char specifically.

## What this stone delivers

### S1 — Add bare `nil` lexer support

Currently bare `nil` parses as a SYMBOL (per `src/edn_shim.rs:1802` comment: `is_nil() = as_symbol() == Some("nil")`). Stone 242.1 makes bare `nil` a PRIMITIVE NIL LITERAL at the lexer/parser layer — same shape as bare `true`, `false`, numeric literals.

Implementation site: `src/parser.rs` (or `src/edn_shim.rs` if EDN-handling layer). Sonnet identifies the right entry point.

Post-stone: `(:wat::core::defn :f [] -> :wat::core::nil nil)` parses with `nil` as the primitive nil VALUE (not a symbol named "nil").

### S2 — Migrate `:wat::core::nil` VALUE-position uses → bare `nil`

Audit `:wat::core::nil` references (255 hits per scope crawl). Classify each:
- **Value position** (expression returns, assignments, dispatch results): migrate to bare `nil`
- **Type position** (function signature return types, type annotations, type declarations): PRESERVE `:wat::core::nil` (it's the type per Doctrine 1)

Type-position contexts to PRESERVE:
- `-> :wat::core::nil` (function return type)
- `[x <- :wat::core::nil]` (parameter type)
- `(:wat::core::define X <- :wat::core::nil ...)` (variable type annotation)
- Any other type-position contexts sonnet identifies

Value-position contexts to MIGRATE:
- `:wat::core::nil` appearing as a return expression value
- `:wat::core::nil` as argument to a function expecting a nil value
- Any expression-position use

Sonnet's per-site audit + migration.

### S3 — HARD CUT `:wat::core::Char` → `:wat::core::char`

Mirror Stone 241.8/9/11 HARD CUT pattern:

1. **Append RETIREMENT_TABLE entry**:
```rust
(":wat::core::Char", ":wat::core::char"),
```

2. **Mint check.rs HARD-CUT-rejection arm**:
```rust
":wat::core::Char" => {
    return CheckResult::errs(vec![CheckError::MalformedForm {
        head: k.to_string(),
        reason: format!("'{}' is retired (Stone 242.1); use ':wat::core::char' instead (scalar types lowercase per arc 242 doctrine)", k),
        remedies: crate::remedy::remedies_for(k, std::iter::empty()),
        span: head_span.clone(),
    }]);
}
```

3. **Mint `:wat::core::char` as the live form** at the appropriate substrate dispatch site (parser/types/check)

4. **Cascade migrate** all `:wat::core::Char` references to `:wat::core::char` (estimate ~30-100 sites)

### S4 — Reflection emitters

Audit reflection emitters in `src/runtime.rs` producing `:wat::core::nil` AST (as value) or `:wat::core::Char` AST. Migrate:
- Value-position emissions of nil: produce bare `nil` keyword (or whatever the new lexer expects)
- Type-position emissions of `:wat::core::nil`: preserve (type stays)
- All `:wat::core::Char` emissions: migrate to `:wat::core::char`

### S5 — Doctrine inscription

Inscribe both doctrines in:
- `INTERSTITIAL-REALIZATIONS.md` (arc 170 — substantive realization entry naming the arc's contribution)
- New memory: `project_lexeme_role_doctrine.md` (`feedback`-type? `project`-type — load-bearing across arcs)

The inscription:
- Doctrine 1 verbatim with examples
- Doctrine 2 verbatim with examples
- Outstanding case-audit candidates flagged
- The arc's dividend (Stone 241.10 apparatus consumed for Char retirement)

### S6 — Probe verification

`tests/probe_arc242_stone1_lexeme_role.rs` (NEW). FM 2-bis disconfirming. Contracts:
1. Bare `nil` parses as primitive value (post-stone)
2. `:wat::core::nil` still works as type in signature positions (preserved)
3. `:wat::core::Char` HARD-CUT-rejected with structured retirement remedy
4. `:wat::core::char` (lowercase) works as type

## Locked decisions

### D1 — Doctrine 1 governs lexical role

Bare lexeme = value; keyword lexeme `:wat::core::*` = type. The lexer reads this rule structurally.

### D2 — Doctrine 2 governs case convention

Scalar types lowercase; non-scalar/container types PascalCase. The substrate enforces by audit; future case-audits consume this rule.

### D3 — Bare `nil` is a PRIMITIVE VALUE (not a symbol)

Lexer treats `nil` as a primitive literal, same shape as `true`/`false`/numbers. Not a symbol that happens to be named "nil".

### D4 — `:wat::core::nil` STAYS as the type

Per Doctrine 2 (scalar types lowercase), the type stays `:wat::core::nil` (lowercase). Type-position uses are PRESERVED.

### D5 — `:wat::core::Char` HARD CUT to `:wat::core::char`

Per Doctrine 2 (scalar types lowercase). Char is scalar (single Unicode codepoint). Must be lowercase. HARD CUT.

### D6 — No Vigilia (D7 default per `feedback_namespaced_home_vigilia_gate`)

No new namespaced home; substrate edits live in legacy flat substrate. SCORE-green commit. The vigilia-must-fire-independently lesson from Song #44 does not apply to non-namespaced substrate work.

### D7 — Per `feedback_hard_cut_admits_no_bypasses`

The doctrine that Stone 241.11.fix round 2 was killed for violating. Pre-authorized in the BRIEF. No privileged paths for `:wat::core::Char`; no substrate-internal bypasses. Char dies everywhere; only HARD-CUT-rejection text + retirement entry + historical comments are acceptable references.

### D8 — Probe + lib + prior probes preserved

After Stone 242.1:
- Stone 242.1 probe ≥ N/N PASS
- Stone 241.1-241.11 probes preserved
- Arc 237/238 probes preserved
- Lib baseline preserved at 890+

## Trap-door audit

### T1 — Cascade size

`:wat::core::nil` has 255 hits; estimate ~50-150 are value-position uses (the rest stay as type-position). `:wat::core::Char` cascade ~30-100. Net cascade: ~80-250 sites. Auto-fixer pattern from Stone 241.10/241.11 is APPLICABLE.

### T2 — Type-vs-value disambiguation

Sonnet must judge each `:wat::core::nil` site by context. The audit is per-site. Heuristics:
- After `->` in a function signature → type → PRESERVE
- After `<-` in argspec or `define` → type → PRESERVE
- In an expression body or return value → value → MIGRATE
- In macro quasiquote → judge by quoted context

When ambiguous: PREFER PRESERVE (don't break working code; type-position uses are safer to keep as-is than to migrate incorrectly).

### T3 — Bare `nil` lexer surprise

Adding bare `nil` as a primitive may surprise the parser elsewhere. If `nil` appears as a binding name anywhere (`:my::ns::nil`?), the lexer must distinguish. Likely a non-issue (no one names a binding `nil`) but sonnet audits.

### T4 — Reflection emitter conditional logic

Emitters producing `:wat::core::nil` AST may need to know whether the emission is value-position or type-position. The emitter site has context; sonnet adds the disambiguation.

### T5 — Char cascade includes WAT source files

`:wat::core::Char` references appear in `.wat` source files too. Auto-fixer or per-file migration. The cascade discipline from Stone 241.10/241.11 applies.

### T6 — Auto-fixer ephemeral discipline

If sonnet mints `crates/fix-lexeme-role/` or similar, it MUST be deleted before commit per Stone 241.10/241.11 precedent.

### T7 — `nil` as identifier collision

If `nil` is a primitive literal, can a user name a function `nil`? `(:wat::core::defn :nil [] -> :wat::core::nil nil)` — does this work or does the parser refuse `:nil` as a name? Sonnet investigates; document the verdict.

## STOP triggers — REJECTION

1. Compile errors not traced to lexer addition or cascade migration
2. Lib < 890
3. **180 min elapsed** (cascade + Char HARD CUT + doctrine inscription)
4. holon-rs touched
5. `:wat::core::nil` TYPE-position uses migrated incorrectly (broken signature types) — D4 violation
6. `:wat::core::Char` survives as ACTIVE substrate use post-stone (D5 + D7 violation)
7. Auto-fixer crate survives commit
8. Stone 242.1 probe < N/N PASS
9. Stone 241.x or arc 237/238 probes regress
10. Clippy > 902
11. Sonnet classifies a `:wat::core::Char` use as "privileged path" or "intentional bypass" — `feedback_hard_cut_admits_no_bypasses` violation
12. Doctrine not inscribed in INTERSTITIAL + memory (S5 incomplete)

## FM 2-bis evidence

`tests/probe_arc242_stone1_lexeme_role.rs` (NEW). At HEAD:
- Bare `nil` parses as symbol (not primitive value) → C01 expects post-stone primitive-value semantics → FAILS at HEAD
- `:wat::core::Char` works as type at HEAD → C03 expects HARD CUT → FAILS at HEAD
- C02 (preserve `:wat::core::nil` type) and C04 (`:wat::core::char` works) — post-stone semantic contracts

## Calibration

**Target band: 60-150 min Mode A.** Cascade dominant; lexer change minor; HARD CUT mechanical.

Per `feedback_stone_briefs_cite_prior_score`: BRIEF cites SCORE-STONE-241.11.md for cascade pattern + ephemeral auto-fixer discipline; SCORE-STONE-241.10.md for retirement-table append discipline.

## What this unblocks

**Stone 242.2** — INSCRIPTION closes arc 242. Per `feedback_spawn_block_winding`: arc 241 resumes at Stone 241.12 after Stone 242.2 closes.

The doctrine inscribed in this stone GOVERNS all future type names. Future case-audits (Uuid, Time family — queued elsewhere) consume the rule.
