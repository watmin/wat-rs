# DESIGN — STONE: the first layout rules, driven end to end

## WHY THIS SHAPE — the gate is one verb, and everything else already exists

`wat-grep` proves the whole pipeline is already wat-side (`wat/grep.wat:383`):

```
read file → facts-of → overlay (fire rules) → query → print
```

A formatter is that pipeline with a different last step. **One thing is missing and only one:**
`read-string` yields forms, never comments. The Rust half landed last stone
(`parse_all_with_comments`); this stone lifts it to wat and spends it.

## ⛔ THE ONE CONTRACT DECISION — a rule asserts a BREAK, never text

A layout rule must never build a string. It asserts **where a line begins and at what indent**:

```
(:wat::core::defrecord :wat::fmt::Break
  [id     <- :wat::core::i64      ;; the node that STARTS a new line
   indent <- :wat::core::i64])    ;; ABSOLUTE column, in spaces
```

`indent` is an **absolute column**, derived from the parent's span in the current source
(`parent.col + 1` for a child line; `parent.col + 2` for a continuation inside `[`). Not a
hardcoded `2`. Nested forms inherit the parent's column; a top-level form at col 1 still
lands children at 2.

A specific rule asserts `Claim {form}` on the form it owns. `ClaimedUnder` is the transitive
closure over `Node.parent` — the claimed node and every descendant. The default rule (R11)
fires only where **no ancestor is claimed**. A rule that lays out a form owns that form's
whole extent.

⚠ **Unanswered, the builder's:** should a rule assert only WHERE a line begins, and the
emitter compute the column from its own descent (`emitted-indent + 2`)? Absolute columns
are computed from the *current* source, which formatting moves. Recorded in
`[[REFUTE-a-claim-must-cover-a-subtree-not-a-form]]`. Not patched here.

Everything follows from this:
- **Rules stay declarative and composable.** Two rules cannot fight over bytes, only over one node's
  break — and head-symbol dispatch already makes that exclusive.
- **The emitter stays DUMB and total.** It walks forms + comments + Breaks and renders. It holds no
  style opinion, so a new rule needs no emitter change.
- ★ **It is what makes the acceptance testable at all.** *"A new style rule is a NEW FILE and nothing
  else"* is only checkable if a rule's entire output is data.

## WHAT SHIPS

```
1  :wat::core::read-string-with-comments   the verb. Mirrors read-string; ADDITIVE, does not
                                           touch read-string or ReadOutcome.
2  wat/fmt.wat                             Break + the emitter (forms + comments + Breaks → text)
3  wat-scripts/fmt/rules/defn.wat          R1 as ONE defrule file
4  wat-scripts/fmt/rules/siblings.wat      R11 — and it is the ACCEPTANCE, not extra work
```

## THE RULES THIS STONE ENCODES

**R1 · `defn` — RULED, unconditional.** Head + name + param-spec (`:- [P…]`) if present on line 1;
arg-spec on its own line(s), one argument per line, empty `[]` included; ret-type on its own line;
body on its own line.

**R11 · sibling breaking is ALL-OR-NOTHING.** If any child of a form starts a line, every child does.
This is the rule that fixes the corpus's worst damage — the 1,096-column half-broken `match` at
`tests/services/probe_arc170_m1_teeth_revoked.wat:95`, where inner arms were broken and outer arms
appended to the tail of the last one.

⛔ **R15 (the 120 budget) IS NOT IN THIS STONE.** It needs a form's rendered width, and
`[[NOTE-width-is-a-fact-not-a-rule]]` proved rete cannot derive it — the engine refuses recursion
through an aggregate. Width must be emitted by the walk as a fifth fact. **That is the next stone,
and it is named.** R1 and R11 are purely structural and need no width, which is exactly why they go
first.

## ★★ THE ACCEPTANCE — the arc's dominating requirement, made mechanical

> **Builder:** *"i will never have all the rules that matter.. but i will absolutely spot stuff i
> don't like... we fix them and the code fixes itself as we do... that's the most important thing."*

So the bar is **not** "the rules are right". It is:

> **R11 IS ADDED AS A NEW FILE AND NOTHING ELSE.** Ship the engine with R1 only, run it, then add
> `siblings.wat` — no edit to `fmt.wat`, no edit to `defn.wat`, no Rust recompile — and watch the
> output change. If R11 required touching anything but its own file, **the design has failed** and
> that is the finding, regardless of how good the output looks.

## IDEMPOTENCE IS THE OTHER HALF

`fmt(fmt(x)) == fmt(x)`, byte-identical. A formatter that is not a fixpoint cannot join a floor gate
(`wat fmt --check` is `fmt(x) == x`), and non-idempotence is the classic way a layout engine ships
broken while every hand-checked example looks fine.

## OUT OF SCOPE — cut affirmatively

- **R15 / the width fact** — the next stone, named above.
- **`wat fmt` as a CLI mode.** No `--fmt` flag, no argv change. The driver is a `.wat` program run
  the ordinary way. A mode is packaging; this stone is the mechanism.
- **Writing files.** Print to stdout. Nothing in the corpus is rewritten by this stone.
- **Comment REFLOW.** A comment stays on its own line; only its indent may change.
- **Literal-spelling normalisation** (`3.00`→`3.0`). Measured, real, and still awaiting a ruling.
  Change nothing.
- **R3-R10, R12-R14.** They are drafted in `[[STYLE-TABLE-draft]]` and most are still MINE, not
  ruled. Adding them later is the acceptance, not this stone's work.
