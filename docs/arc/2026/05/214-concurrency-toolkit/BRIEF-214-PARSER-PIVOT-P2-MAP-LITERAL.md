# BRIEF — Arc 214 Parser-Pivot P2 — `{...}` map literal in expression position

**Stone:** mint `{...}` as canonical map literal in expression position; parser-level expansion to `:wat::core::HashMap` verb form shipped by P1.
**Type:** Sonnet Mode A (single agent, single session).
**Time budget:** 30-50 min target; 75 min STOP.
**Depends on:** P1 (#403, commit 564d5e6 — `:wat::core::HashMap :K :V k v ...` Vector-symmetric form).
**Unblocks:** Slice 4 (#385) — ProgramEnv shape uses `HashMap<Keyword, HolonAST>` and benefits from literal sugar.

## What this stone does

Mint `{...}` brace-form as a **map literal** in expression position, parsed into the canonical `(:wat::core::HashMap :wat::core::Keyword :wat::holon::HolonAST :k0 (:wat::holon::Atom v0) :k1 (:wat::holon::Atom v1) ...)` verb-call form at parse time. Pinned type `HashMap<Keyword, HolonAST>` — non-keyword keys and non-leaf values are rejected at parse with diagnostics naming the position. Existing arc 169 `{outcome residue}` struct-destructure shape (bare symbols only) preserved.

Dispatch is **content-shape**: the parser examines the first child after parsing `parse_brace_body` and routes:

| First child | Empty | Routes to | Result |
|---|---|---|---|
| (none — empty `{}`) | yes | empty map literal | `(:wat::core::HashMap :wat::core::Keyword :wat::holon::HolonAST)` |
| Keyword | — | map literal | desugared HashMap verb-call with auto-wrap |
| bare Symbol | — | struct pattern (arc 169) | `WatAST::StructPattern` |
| anything else | — | parse error | `MalformedBraceLiteral` naming the offending position |

The discriminator IS the content; the rename `parse_map_literal_body` + `parse_struct_destructure_body` (per task #404) is the dispatch surface clarification.

## Why this shape (four-questions verdict, ran inline)

`{:k v :k v ...}` desugaring to the P1 HashMap verb-call directly:

| | |
|---|---|
| Obvious? | YES — `{:k v}` reads as a map at first glance; expansion target is the only HashMap constructor; one form, one meaning |
| Simple? | YES — no macro layer, no new AST variant; parser synthesizes the canonical List form; type-check + runtime inherit P1 |
| Honest? | YES — pinned `HashMap<Keyword, HolonAST>` is explicit; non-keyword keys + non-leaf values rejected at parse with position-named diagnostics; verb form remains for non-pinned shapes |
| Good UX? | YES — ProgramEnv + most map use cases hit the pinned shape; verbose path stays for everything else |

YES YES YES YES. Ship.

## In scope

1. **`src/parser.rs` LBrace dispatch refactor** (parser.rs:200-230)
   - Read brace body content-agnostic (existing `parse_brace_body` unchanged or renamed to neutral)
   - Branch on first child shape:
     - Empty body → synthesize `WatAST::List([Keyword(":wat::core::HashMap"), Keyword(":wat::core::Keyword"), Keyword(":wat::holon::HolonAST")], span)`
     - First child Keyword → enter `parse_map_literal_body` semantic path (existing `parse_brace_body` body OK; just the dispatch + post-validation differs)
     - First child Symbol → preserve arc 169 StructPattern path (rename helper to `parse_struct_destructure_body` for symmetry per task #404)
     - First child anything else → `MalformedBraceLiteral` with reason naming the actual child kind
   - Auto-wrap rule: every odd-indexed child (the value position) becomes `WatAST::List([Keyword(":wat::holon::Atom"), v], v.span())`
   - Validation rules for map-literal branch:
     - All even-indexed children (key positions) must be Keyword; first non-keyword key emits `MalformedBraceLiteral` with span on offender + reason `"map-literal key must be a keyword (got {kind}); pinned key type is :wat::core::Keyword"`
     - Body length must be even; odd length emits `MalformedBraceLiteral` with reason `"map-literal body must alternate keyword-key + value pairs; got {n} forms"`
   - **arc 169 retirement adjustment:** the "empty `{}` is degenerate" check (parser.rs:211-218) is **moved** to the struct-pattern branch only; empty `{}` is now empty map literal at parser level. The struct-pattern branch retains its "at least one bare symbol" rule when dispatched (which it won't be for empty body — empty body is map-literal-empty).

2. **`src/parser.rs` error variant**
   - New: `ParseError::MalformedBraceLiteral { span, reason: String }` — mirrors `MalformedStructPattern`'s shape
   - Display impl follows the existing pattern (`"malformed brace-literal at {span}: {reason}"`)
   - Keep `MalformedStructPattern` for the struct-pattern arc 169 path; do not collapse them — they describe genuinely different malformed shapes

3. **`src/parser.rs` `ast_variant_label` extension**
   - Already covers leaf-literal kinds (line 372-384); confirm no new variant needed (we're not adding a new `WatAST` variant — desugaring lands in `WatAST::List`)

4. **Probe matrix** — `tests/probe_brace_map_literal.rs`
   - **Probe 1:** Empty `{}` → empty HashMap (length 0); proves the arc 169 degeneracy retirement
   - **Probe 2:** Single pair `{:foo 42}` → length 1, get :foo returns Some(Atom 42); proves auto-wrap
   - **Probe 3:** Multi pair `{:a 1 :b 2 :c 3}` → length 3, get :b returns Some(Atom 2); proves alternation
   - **Probe 4:** Nested in expression `(:wat::core::length {:a 1 :b 2})` → 2; proves expression-position composability
   - **Probe 5:** Map-literal-of-map-literal `{:outer {:inner 42}}` — the inner `{...}` evaluates to a HashMap value; auto-wrap of an inner HashMap value via `(:wat::holon::Atom <HashMap>)` either type-checks (if Atom is polymorphic enough) or fails honestly with a diagnostic. Probe the actual behavior; LIMITATION-comment whatever the truth is.
   - **Probe 6:** Non-keyword key `{42 :v}` → `MalformedBraceLiteral` at parse; error message names "key must be a keyword (got integer literal)"
   - **Probe 7:** Odd count `{:foo}` → `MalformedBraceLiteral` at parse; error names "body must alternate keyword-key + value pairs; got 1 forms"
   - **Probe 8:** Bare symbols still parse as struct pattern in let-binding position; reuse an existing arc 169 test pattern to prove this stone didn't break that
   - **Probe 9:** Keyword in binding position rejected. Build a `(:wat::core::let [{:foo bar} ...] ...)` form; should produce a downstream type-check or lower-time error (NOT parser error — parser produces a List form which let-binding then rejects). Probe captures the error message at whatever layer rejects it; LIMITATION-comment the exact layer.

5. **Documentation sweep**
   - **`docs/WAT-CHEATSHEET.md` § 8** — extend Collection constructors with the `{...}` literal row:
     ```
     {:k0 v0 :k1 v1 ...}    ;; desugars to (:wat::core::HashMap :wat::core::Keyword :wat::holon::HolonAST :k0 (:wat::holon::Atom v0) :k1 (:wat::holon::Atom v1) ...)
     ```
     Add a "Position discipline" sub-paragraph: expression position → map literal; binding position (`let` LHS) → struct destructure (bare symbols only). Cite arc 214 P2 + arc 169.
   - **`docs/058-...` row** (find the live arc-058 spec file; the most recently-touched row is the model — match shape)
   - **`docs/arc/2026/05/214-concurrency-toolkit/DESIGN.md`** — any examples that build ProgramEnv-shaped HashMaps switch from verb form to `{...}` literal form

6. **SCORE doc** — `docs/arc/2026/05/214-concurrency-toolkit/SCORE-214-PARSER-PIVOT-P2-MAP-LITERAL.md`
   - 20-row scorecard (analogous to P1's 22-row); per-deliverable PASS/FAIL with line-citation
   - Mode declaration (A)
   - Honest deltas section (if any in-session pre-scope finds need calling out)

## Out of scope (DO NOT TOUCH)

- **Match-arm `{...}` pattern matching** (task #402) — separate stone; this stone targets EXPRESSION position only
- **Generic `HashMap<K,V>` literal syntax** — pinned `HashMap<Keyword, HolonAST>` only
- **Macro layer** — no wat-side `defmacro` for `{...}`; substrate primitive serves both verb-call and literal-expansion paths
- **Auto-wrap value-type smartness** — do NOT detect "already-an-Atom" and skip the wrap; uniformly wrap; verbose form is the escape hatch
- **WARD-PASS** — out-of-zone per `feedback_ward_zone_comms_only` (this stone is in `{src,tests}/parser*/check*/runtime*}` — all out-of-zone). Orchestrator decides post-ship whether to spawn anyway; default: no ward pass.
- **INTERSTITIAL entry** — orchestrator-direct post-ship per `feedback_sonnet_no_realization_voice`; not in sonnet's scope

## STOP triggers

- **STOP-1** — empty `{}` semantics surprise: if existing arc 169 tests EXPLICITLY assert `{}` rejects at parser level (rather than at struct-destructure validation), the retirement adjustment ripples broader than the BRIEF anticipated. STOP, surface the ripple, ask for direction.
- **STOP-2** — auto-wrap-of-HashMap-value (probe 5) reveals a Atom polymorphism gap that isn't a clean "rejected with diagnostic" outcome. STOP, surface the actual behavior, ask whether to defer or fix in-stone.
- **STOP-3** — any pre-existing test ASSERTS that `parse_form()` consumes `{...}` to a `StructPattern` regardless of content; this stone's content-dispatch changes that assertion. STOP, surface the test, ask whether to revise the test (intent preserved) or revise the dispatch.
- **STOP-4** — time budget hits 75 min with any deliverable incomplete. STOP, report what shipped, defer remainder.

## Verification command

```
cargo build --release
cargo test --release --test probe_brace_map_literal -p wat
cargo test --release --test probe_brace_struct_pattern -p wat   # if exists; smoke-check arc 169 path
cargo clippy --release -- -D warnings
```

## Closure shape

After PASS:
- Stone ships in sonnet's commit (BRIEF + SCORE + WAT-CHEATSHEET + arc 058 row + probe file + parser changes + DESIGN.md examples)
- Orchestrator drafts INTERSTITIAL post-commit (next-session work)
- Task #404 marked complete; unblocks Slice 4 (#385)

*Reads as a map. Becomes the verb form. Stays honest about the pinned shape.*
