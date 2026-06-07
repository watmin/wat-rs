# BRIEF — Stone 214.4.6i-lexer: primed type heads with generic params lex

One focused lexer change. The 4.5 peer types are primed + parametric
(`:wat::kernel::Thread'<I,O>`), and the builder's direction is: make this pass
the lexer (the symbolic type-annotation surface is arc-251 territory; the
keyword grammar carries it today).

## The mechanism (already root-caused)

`src/lexer.rs` `lex_keyword`, the `'<'` arm (~line 723): `<` increments
`angle_depth` only when the previous emitted char is alphanumeric or `_`
(`prev_alpha`). After a primed head (`Thread'`) the previous char is `'`, so the
`<` is not recognized as a type-head opener, `angle_depth` stays 0, and the comma
between the params hits `CommaInKeywordBody` (and whitespace hits
`UnclosedBracketInKeyword`'s sibling break).

Disambiguation safety (why the fix is sound): operator `<` in a keyword path
always follows `::` (`:wat::core::<`); arc-171 discriminator apostrophes come
AFTER an op name (`<'2`, `op'i64'i64`). So `'` immediately before `<` can only be
a primed type head. `parse_type_expr` already accepts the primed parametric form
— only the source lexer lags.

## The work

1. In the `'<'` arm's `prev_alpha` computation, treat `'` as a valid
   type-head-final char: `ch.is_ascii_alphanumeric() || ch == '_' || ch == '\''`.
2. Update the `lex_keyword` doc comment (the `'` paragraph ~626 and the arc-072
   disambiguation note ~646) to document the primed-type-head case
   (`Thread'<I,O>`) and the `'<` disambiguation argument above.
3. Add a focused unit test next to the existing `lex_keyword` tests in
   `src/lexer.rs` mirroring the probe's three cases (primed+comma, primed+space,
   unprimed control).

## Verify (report exact numbers)

- `cargo test --release --test nursery probe_arc214_lexer_primed_generic_head` → **3 passed**.
- `cargo test --release --lib -p wat` → the green band (~940/0/1), with the
  lexer's own test module green.
- `cargo clippy --release` → no new warnings in `src/lexer.rs`.

## Expectations (scored by the orchestrator against an independent re-run)

| # | Claim | Check |
|---|---|---|
| 1 | probe 3/3 | orchestrator re-runs the probe |
| 2 | lib band green | orchestrator re-runs `--lib` |
| 3 | one character-class change + docs + unit test; nothing else | read the diff |
| 4 | tree left dirty (no commit) | `git status` |

Runtime band: 5–10 min. Do NOT commit — the orchestrator scores and commits.
