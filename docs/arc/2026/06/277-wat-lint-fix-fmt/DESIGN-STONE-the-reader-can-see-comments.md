# DESIGN — STONE: the reader can SEE comments. (It cannot today, and nothing downstream is designable without it.)

> First stone of `[[DESIGN-wat-fmt-the-rule-set-is-the-product]]`. It is not in the formatter.

## THE FACT, from the lexer's own test

```rust
// crates/wat-reader/src/lexer.rs:1069
lex_tokens("; a comment\n()")  ==  vec![Token::LParen, Token::RParen]
```

**Comments are discarded at lex time** — `lexer.rs:351`, `if c == ';' { skip to '\n'; continue; }`.
No token, no AST node, nothing downstream can see them. A canonical reprinter emits from the AST,
so **every comment in the corpus would vanish.**

★ It is also why `wat/fix.wat` is span-based: it never re-emits, so comments survive by not being
touched. The fix-text discipline is a workaround for this exact gap, not a preference.

## THE DECISION — a SIDE CHANNEL, not a token variant

| | Obvious? | Simple? | Honest? | Good UX? |
|---|:---:|:---:|:---:|:---:|
| **a `Token::Comment` variant** | YES | **NO** | YES | **NO** |
| **a side channel beside the token stream** | YES | YES | YES | YES |

`Token::` appears **111×** in `lexer.rs` and **32×** in `parser.rs`. A new variant makes every one
of those a site that must now skip comments — and a consumer that forgets is a silent parse bug.
**Simple fails, and Good UX fails: every existing consumer pays for a feature only the formatter
wants.**

The side channel costs nothing to anyone who does not ask:

```rust
pub struct Comment { pub text: String, pub span: Span }   // text VERBATIM, including its `;`s

lex(src, file)                -> Result<Vec<SpannedToken>, LexError>          UNCHANGED
lex_with_comments(src, file)  -> Result<(Vec<SpannedToken>, Vec<Comment>), LexError>
```

`lex` delegates and drops the comments, so **every existing caller is byte-identical** and the
parser is not touched at all.

## ★ AND ATTACHMENT IS NOT PARSE-TIME — it is a SPAN COMPUTATION, later

The instinct is "attach each comment to its AST node during parsing." **Do not.** `SpannedToken`
already carries spans (`lexer.rs:73`), `WatAST` nodes carry spans, and `ast-span`/`ast-end-span`
are already the machinery `fix.wat` navigates by. So *"this comment lies between node X's end and
node Y's start"* is computable **after** the parse, from spans alone.

That matters because attachment is **policy, not fact** — leading vs trailing-inline vs
section-break (the May draft's Rules 7–10) is a style decision the builder will want to change.
Baking it into the parser would freeze a policy inside a component that has no business holding
one, and every future change to it would be a parser change.

⛔ **So attachment is OUT of this stone.** This stone makes comments VISIBLE. What owns a comment
is the next decision, and it belongs beside the style rules.

## CORRECTNESS IS INHERITED, NOT RE-DERIVED

A `;` inside a string literal must not open a comment. **The lexer already gets this right** — the
string branch (`lexer.rs:459`) consumes a literal atomically through its closing quote, so an
interior `;` never reaches the top of the loop. Capturing at the existing skip site inherits that
exactly.

⚠ A rider must NOT re-implement string-awareness at the capture site. If it feels necessary, the
capture has been put in the wrong place.

## Scope

**In:** a `Comment` type · `lex_with_comments` · the capture at `lexer.rs:351` · a witness proving
comments are captured with byte-exact text and correct spans · proof that `lex`'s output is
unchanged.

**Out, affirmatively:** attachment to AST nodes (policy — the next stone) · any parser change ·
any formatter work · `;; @format-off` directives · comment REFLOWING. This stone changes what the
reader can see and nothing else.
