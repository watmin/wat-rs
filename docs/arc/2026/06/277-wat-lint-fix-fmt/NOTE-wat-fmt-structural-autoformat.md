# NOTE — wat-fmt: structural source autoformatting (the "fmt" of lint-fix-fmt) (2026-06-21)

Builder ask: we've been hand-adhering to wat's indentation/layout conventions (and doing it
well), but we should **enforce it structurally** — a `gofmt`/`rustfmt`-style canonical
formatter. We have the tooling (wat-lint + wat-fix, AST → `write-forms` → span-offset edits);
we're missing the LOGIC (the layout rules). This is **autofixable** (`fix: Some(FixEdit)`) —
a deterministic structural rewrite, the opposite of the dead-code lint
([[NOTE-dead-code-lint-non-autofixable]], `fix: None`).

## The canonical `defn` shape (the first, load-bearing rule)
```
(wat.core/defn my/fn [a :- wat.type/i64 b :- wat.type/i64] :- wat.type/i64 (wat.core/* a b))
```
→ reformats to:
```
(wat.core/defn my/fn
  [a :- wat.type/i64
   b :- wat.type/i64]
  :- wat.type/i64
  (wat.core/* a b))
```
Rules, exact:
- **argspec = one parameter per line**, vertically aligned inside the `[ ]` (the `[` sits
  with the first param; continuation params align under the first).
- **return type (`:- T`) on its own line**, immediately after the argspec.
- **body begins on its own line**, after the return type.
- (the `defn` head + name stay on the opening line).

## Scope
- The `defn` shape above is rule #1. **More forms need their own layout rules** (let/match/
  do/defstruct/defenum/etc.) — the "unspoken rules" we've been following get codified
  incrementally, each its own small rule.
- Most real diffs are **off-by-one-space indentation fixes** — we already write structurally
  correct code; the formatter normalizes the spacing. So the autofix is low-risk and high-trust.

## Mechanism (tooling exists; logic doesn't)
wat-lint detects non-canonical layout (a rule per form); wat-fix applies the canonical
rewrite via the existing AST → `write-forms` → span-edit pipeline (the same machinery the
concat→format autofix uses, `wat/lint.wat`). The formatter is a **fix-only** family: it never
reports-without-fixing (a layout deviation is always mechanically fixable). Likely a dedicated
`wat fmt` entry that runs the layout rule-set in fix mode over a SourceFile (or the whole
corpus), idempotent (re-running yields zero edits).

## Status
Queued — not built. Rule #1 (`defn` shape) is specified above; the broader layout rule-set +
the `wat fmt` driver are the work. Autofixable (`fix: Some`).
