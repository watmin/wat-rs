# NOTE — a name the READER manufactured has no text to rewrite

> **Found by the rider, on a STOP-1, 2026-08-25.** It refused to apply a codemod I had briefed and
> whose finder rules I had written, dry-run, and called *proven*. It was right. Verified
> independently below, against a census I ran an hour before the rider existed.

## The defect

`crates/wat-reader/src/parser.rs:390–410` — **five reader macros synthesize a keyword node and clone
the source literal's span onto it:**

```rust
Token::Char(c) => Ok(Some(WatAST::List(vec![
    WatAST::Keyword(":wat::core::char/of".into(), span.clone()),   // ← 19-char name
    WatAST::StringLit(c.to_string(), span.clone()),
], span)))                                                          // ← span covers `\a`, 2 columns
```

So `wat/grep.wat`'s fact base reports a `Node` of kind `"keyword"`, a `Named` fact reading
`:wat::core::char/of`, and a `Span` **two columns wide**. Both facts are true. They answer different
questions, and nothing in the fact base says so.

A rename codemod splices its replacement into `Span`. Nineteen characters into a two-character hole:

```diff
-    [c        \a                         -    [a \a
+    [c        :wat::core::char           +    [a :wat::core::char
                                          -     b \b
                                          +     b :wat::core::char
```

The right-hand pair is from `wat-tests/holon/char-round-trip.wat:45-47`, a test asserting two
*different* chars are unequal. Both literals become the same bare keyword. **The codemod does not
break the test — it silently inverts what the test measures.**

## ⛔ IT IS A CLASS, AND `char/of` IS ITS SMALLEST MEMBER

Asked of the whole corpus with `wat-scripts/scratch-pad/probe-span-narrower-than-name.wat` — *which
named keyword nodes carry a span too narrow to hold the name they claim?*

```
  980  :wat::core::unquote              ~
  310  :wat::core::quasiquote           `
  119  :wat::core::unquote-splicing     ~@
   50  :wat::core::char/of              \c
    2  :wat::holon::literal             #holon
 ────
 1461  nodes, corpus-wide.  span widths: 1290 are ONE column, 162 are two.
```

`:wat::core::quote` is absent — the corpus has no `'`-quoted forms today. It is one `'` away from
joining, and nothing marks that.

**Every one of those five names is a rename waiting to corrupt its own corpus.** The stone that hit
it happened to draw the 50, which is 3.4% of the class. A future rename of `:wat::core::unquote`
would have splice-corrupted 980 sites, in macro bodies, where the damage reads as valid syntax.

★ **This probe cannot be written in text.** The defect is a *disagreement between two facts about the
same node*. `grep` has no way to ask it, and neither does any tool that sees only characters. It took
the fact base to find a bug in the fact base.

## ⚠ AND IT INVERTS THE CENSUS I PUBLISHED AT `61dd04a3b`

I replaced the stone's `grep -c` with a rules-based census and reported *"char/of: 17 drawn → 67
derived, the draw was off by 50."* Measured now:

```
  67   fact-base nodes named :wat::core::char/of
  50   of them manufactured by the reader — no text to rewrite
  17   genuine textual call sites          ← the ORIGINAL DRAW'S NUMBER
```

**The `grep -c` was right and my "derived" census was wrong.** Not because grep is a good census — it
is not, and it was wrong about Uuid in the same table — but because *for this question* the two
instruments answer different things and I never asked which question the migration needs. A codemod
rewrites **text**; it needs the count of names that are *written*. The fact base counts nodes that
are *named*. I called the structural instrument better and stopped there.

`[[feedback_state_what_the_instrument_can_see_before_quoting_it]]` — and the sharper edge:
**an instrument that replaces a discredited one inherits none of its scepticism.**

## THE RULING THIS NEEDS — it belongs to `wat/grep.wat`, not to any codemod

The guard is expressible today; that is not in question:

```wat
(:wat::rete::where (:wat::rete::core::i64::= ?l ?el))
(:wat::rete::where (:wat::rete::core::i64::= (:wat::rete::core::i64::- ?ec ?c :undefined 0)
                                             (:wat::rete::string::length ?n)))
```

Empirically exact: 172/172 genuine matches across `uuid`/`regex`/`list-of` satisfy it; 50/50 phantoms
violate it. The question is **where it lives**, and the four options are not equal.

| | Obvious | Simple | Honest | Good UX |
|---|---|---|---|---|
| **(a)** every codemod adds the guard itself | **NO** — nothing tells the next author it exists | YES | **NO** — the fact base still lies; each codemod patches the same lie | NO |
| **(b)** `grep.wat` stops emitting `Named` for disagreeing nodes | YES | YES | **NO** — a `\a` node genuinely IS named `char/of`; suppressing that makes *"where are chars constructed?"* unanswerable. Destroys a true fact to prevent one misuse | NO |
| **(c)** a new fact `:wat::grep::Written`, emitted only when span width == name length | YES — two facts for two questions: *what is it called* vs *where is that name spelled* | YES — one predicate, one fact, nothing lost | YES — both questions get true answers | **partial** — a codemod that forgets to join it gets today's corruption |
| **(d)** a synthesized node gets no **own-span** fact at all | YES | YES | YES — a borrowed span is not this node's span, and saying so is the truth | **YES** — a span-consuming rule *cannot join*, so the phantom is **unrepresentable** rather than merely detectable |

**(d) is the only option on the top rung of the ladder** — it does not ask anyone to remember. Every
rename codemod already joins `Span` (it must, to compute the edit), so removing the phantom's Span
fact makes the wrong result structurally unreachable. Its cost is real and must be weighed: it
breaks `Span == Node` as the non-vacuity control (the control would become `Span == Node − 1461`,
which is still a control but no longer a free one), and the reader keeps the borrowed span for error
LOCATION, which is correct and should not change — so (d) is about what `grep.wat` *emits*, not about
what the AST *holds*.

**(c) and (d) are compatible**: `Written` is the honest name for what a codemod wants to join, and
(d) is what makes forgetting it impossible. Shipping (c) alone leaves the trap armed for whoever
forgets.

## STATUS

**BLOCKED on that ruling.** `wat-scripts/fixes/rename-four-families-to-their-homes.wat` is on disk,
`--check`-clean, and **never applied**. It is correct today for `uuid` (97), `regex` (13) and
`list-of` (62) — 172 sites, zero anomalies. Only `char-of` is blocked, and only because `\c` exists.

Not deferred, not out of scope: this is the stone's own STOP-1 and the stone does not land until it
is ruled. The `.rs` half was correctly not started — shipping retirement entries for a rename the
corpus has not made would leave the tree asserting a migration that did not happen.
