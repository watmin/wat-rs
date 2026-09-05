# BRIEF — STONE: the first layout rules, driven end to end

Lift comment-aware reading to the wat level, then drive R1 (`defn`) and R11 (sibling all-or-nothing)
as `defrule` files over one real file. Read `[[DESIGN-STONE-the-first-layout-rules]]` first — it pins
the one contract decision (**a rule asserts a `Break`, never text**) and the acceptance.

## READ IN ORDER — the rooms, and why each

1. **`src/intrinsic/ast.rs:69-101`** — `read-string`'s handler and its `#[wat_intrinsic(...)]`
   attribute. **Your new verb is this, mirrored.** The doc rows above it are the shape to copy.
2. **`src/edn/render.rs`, `write_wat_source_with_comments` + `parse_all_with_comments`** — the Rust
   half that landed last stone. Your verb surfaces `parse_all_with_comments` to wat. Note the
   `cfg_attr(not(test), expect(dead_code, …))` on 13 items: **wiring a caller makes those go RED,
   and removing them is part of this stone.** That is the exemption self-retiring, by design.
3. **`wat/grep.wat:383-393`** — `run-one`: read → `facts-of` → overlay → query → print. **This is
   the pipeline your driver copies**, with emit instead of print.
4. **`wat/grep.wat:33-97`** — the fact records (`Node` `Named` `Span` `Written`) and `extent-of`,
   the ONE door that unwraps a span. Use it; do not unwrap a span anywhere else.
5. **`wat-scripts/scratch-pad/277-layout-shape-probe.wat`** — a working two-rule `defrule` set over
   those facts, incl. a DERIVED fact feeding a second rule. Copy its shape.
6. **`wat-scripts/scratch-pad/277-width-as-a-fold.wat`** — a recursive wat walk over `WatAST` with
   `ast->children` / `extent-of`. Your emitter is this shape.
7. **`docs/arc/2026/06/277-wat-lint-fix-fmt/NOTE-wat-fmt-structural-autoformat.md`** — R1 specified
   exactly, in the OLD `:-` spelling. The live spelling is `<-` for binders and `->` for return;
   `:- [T]` is the PARAM-SPEC. `wat/bracket.wat:32` is R1 already practiced, verbatim.

## SKETCH

```wat
;; wat/fmt.wat
(:wat::core::defrecord :wat::fmt::Break [id <- :wat::core::i64  indent <- :wat::core::i64])

;; the emitter: forms + comments + Breaks -> text. DUMB. It holds no style opinion.
;; A node with a Break starts a new line at its indent; otherwise it follows a single space.
;; ★ A COMMENT PINS A NEWLINE AFTER ITSELF — nothing may share a line after a `;;`.
(:wat::core::defn :wat::fmt::emit
  [forms <- … comments <- … breaks <- …] -> :wat::core::String …)
```

```wat
;; wat-scripts/fmt/rules/defn.wat — R1, ONE file, nothing else
(:wat::rete::defrule :fmt::defn-argspec-breaks
  :when [ … head is :wat::core::defn … the arg-spec child … ]
  :then [(:wat::fmt::Break :id ?argspec :indent 2)])
```

## BLAST RADIUS

```
src/intrinsic/ast.rs        ADD one verb beside read-string. read-string itself UNCHANGED.
src/edn/render.rs           REMOVE the 13 cfg_attr(not(test), expect(dead_code)) — now wired.
wat/fmt.wat                 NEW
wat-scripts/fmt/rules/*.wat NEW
```

**No change to `read-string`, `ReadOutcome`, `lex`, `parse_all_with_file`, or `WatAST`. No `--fmt`
CLI flag. No file is rewritten — print to stdout.**

## STOP TRIGGERS

- **STOP-1 — if a rule needs to build or inspect TEXT, STOP.** The contract is that a rule asserts a
  `Break` and nothing else. A rule reaching for a string means the decision record is wrong; surface
  what forced it.
- **STOP-2 — if adding R11 requires touching `fmt.wat` or `defn.wat`, STOP AND SAY SO.** That is the
  acceptance failing, and it is the single most important thing this stone can report. **Do not
  quietly make it work by editing the engine** — a green built that way is worth less than an honest
  red.
- **STOP-3 — if `fmt(fmt(x)) != fmt(x)` on any input, STOP.** Non-idempotence is a design defect, not
  a tuning problem. Report the input and both outputs verbatim.
- **STOP-4 — if you find yourself needing a form's WIDTH, STOP.** R15 is deliberately not in this
  stone and rete cannot derive width (`[[NOTE-width-is-a-fact-not-a-rule]]`). R1 and R11 are purely
  structural. Needing width means you are building the wrong rule.
- **STOP-5 — if any existing test goes red, STOP.** Capture the whole block verbatim; do not re-run.

## PRIOR COMPARABLE

`[[SCORE-STONE-comments-survive-the-round-trip]]` — same arc, immediately prior, and its
ORCHESTRATOR VERDICT names two things worth carrying: put a new `mod tests` at the END of a file, and
**every preservation claim needs a printed non-vacuity count**, because a pass over zero items is
indistinguishable from success.

## THE FLOOR IS MINE

`scripts/floor.sh` and `cargo clippy --all-targets -D warnings` are the orchestrator's. Note that
clippy has caught a real red in **each** of the last two stones — run what proves your change, and
leave those to me rather than reporting them green unrun.
