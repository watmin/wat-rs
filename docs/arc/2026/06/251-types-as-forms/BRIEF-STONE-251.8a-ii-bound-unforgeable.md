# BRIEF — STONE 251.8a-ii: the binder namespace is unforgeable

Read `DESIGN-STONE-251.8-symbol-proper.md` §251.8a-ii first — it carries the ruling, the four
questions on all four options, and why D was chosen over A. This brief is the strike.

## THE WORK, in one paragraph

`$bound` is the reserved namespace the substrate gives every local binder. User source can currently
write it — `($bound/x 1)` parses, is treated as a binder, and dies at runtime as `UnboundSymbol`.
Close that at the reader: a symbol token whose **namespace segment is exactly `$bound`** is a
located parse error. Only the substrate's own binder construction may produce that namespace.

## THE ROOM — one site

**`crates/wat-reader/src/parser.rs:348`**

```rust
Token::Symbol(s) if s == "nil" => Ok(Some(WatAST::NilLit(span))),
Token::Symbol(s) => Ok(Some(WatAST::Symbol(Identifier::bare(s.clone()), span))),   // ← here
```

This is the single door where user text becomes a `WatAST::Symbol`. The arm above it (`nil`) is the
precedent for special-casing a symbol token's spelling at this exact position — copy that shape.

Supporting reads:
- **`crates/wat-reader/src/identifier.rs`** — `BOUND_NAMESPACE` (the `$bound` constant, added by
  251.8a) and `namespace()`. Use the constant; do not re-spell the string.
- **`crates/wat-reader/src/parser.rs:26-90`** — `ParseError { span, kind }` and `ParseErrorKind`
  with its `Display`. Your error is a new `ParseErrorKind` variant with the span of the offending
  token, in the voice of the variants already there.

## ★ SCOPE — read this twice, the orchestrator already got it wrong once

The rule is about the **namespace `$bound`**. It is **NOT** about the character `$`.

- `$` is an ordinary identifier character (`is_symbol_break`, `lexer.rs:519`, does not list it).
- `$` is in live use as `:<name>$impl` — a macro-minted **name suffix inside a keyword**, never a
  namespace, across 17 files.
- `$impl`, `$x` as a plain binder, and every other `$` use must keep working. Only a symbol whose
  **namespace segment** (the part before the last `/`) is exactly `$bound` is refused.

If your change makes anything containing `$` fail that is not `$bound/…`, that is STOP-2.

## IMPLEMENTATION SKETCH — the shape

```rust
// crates/wat-reader/src/parser.rs, at the Token::Symbol arm

Token::Symbol(s) if /* the namespace segment is exactly BOUND_NAMESPACE */ => {
    Err(ParseError { span, kind: ParseErrorKind::ForgedBinderNamespace { /* the spelling */ } })
}
```

The message should say what is true and what to do: the namespace is substrate-minted for local
binders, user source may not write it, and a local is written bare (`x`, not `$bound/x`).

## BLAST RADIUS

`crates/wat-reader/src/parser.rs` and whatever one variant your error needs. **No changes to the
lexer.** No changes to `identifier.rs` beyond reading `BOUND_NAMESPACE`. No `.wat` corpus changes —
if the corpus contains a `$bound/…` symbol, that is STOP-3.

## STOP TRIGGERS — each means ship nothing, report the gap

**STOP-1 — the namespace split is not available at the parser.** If, at `parser.rs:348`, you cannot
determine the namespace segment without duplicating logic that lives elsewhere, stop and report what
you would have had to copy. A second hand-rolled `rfind('/')` is exactly the class 251.8a just
collapsed; do not re-introduce one.

**STOP-2 — collateral on `$`.** If any existing `$` use breaks — `$impl`, a `$`-prefixed binder,
anything — stop and report which. The rule is one namespace, not one character.

**STOP-3 — the corpus already contains it.** If any `.wat`, `.wat.bad`, or other corpus file has a
`$bound/…` symbol, stop and report the sites. That would mean the namespace was already in use and
the ruling needs revisiting before the wall goes up.

**STOP-4 — the reader cannot report locatedly here.** If a `ParseError` at this site does not carry
the offending token's span through to the user, stop. An unlocated refusal is not the win; a located
one is.

## THE GATE

1. A RED probe, written **before** the change and **mutation-tested**: `$bound/x` in source is
   refused with a located error naming the spelling. Then break the check (accept it again), confirm
   the probe goes red, restore, confirm green. **Report the mutation result explicitly** — a gate you
   have not watched fail is a claim, not a proof.
2. A **positive control in the same probe**: `$impl`-style names and a plain `$x` binder still parse.
   Without this the probe cannot tell "refused `$bound/`" from "refused `$`".
3. `cargo build --release` — exit 0.
4. `cargo clippy --release --all-targets` — zero warnings.
5. `./target/release/wat` on a file containing `($bound/x 1)` — refused, and the message names the
   spelling and says a local is written bare.

Run everything in the **foreground** and block on it; your turn ends when the numbers are in your
hands. The orchestrator runs the full floor centrally and weighs by its own re-run.

## A PRIOR RESULT TO COPY FOR SHAPE

251.8a itself (`0a32d5f8`) — one door, small diff, mutation-tested probe, and its honest delta
reported rather than smoothed. Its report is the register to write yours in.
