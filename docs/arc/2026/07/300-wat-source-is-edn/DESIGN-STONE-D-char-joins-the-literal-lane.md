# DESIGN — Stone D: `\c` joins the literal lane (`WatAST::CharLit`)

> **Ruled by the builder, 2026-08-25:** *"wow — that's a crazy flaw to find this late in wat's
> maturity — make it."*
>
> Surfaced sideways: a rider hit a STOP-1 on arc 255's four-homes codemod
> (`255/NOTE-a-name-the-reader-manufactured-has-no-text-to-rewrite.md`), and the builder's question
> — *"how can there be a line of code with no line?"* — is what turned a codemod guard into this.

## THE THESIS

**A char is first-class at both ends of the substrate and absent only in the middle.**

```
wat_edn::Value::Char(char)          crates/wat-edn/src/value.rs:58     ← the DATA layer has it
WatAST::??                          crates/wat-reader/src/ast.rs        ← THE HOLE
Value::wat__core__Char(char)        src/value/value.rs                  ← the RUNTIME has it
```

Every other scalar literal in wat owns an AST variant. `\c` is the only one that becomes a **call**:

```rust
Token::Int(n)          => WatAST::IntLit(*n, span)
Token::Float(x)        => WatAST::FloatLit(*x, span)
Token::Rational(r)     => WatAST::RationalLit(r.clone(), span)   // "NOT a desugared constructor call"
Token::BigInt(n)       => WatAST::BigIntLit(n.clone(), span)     // "NOT a desugared constructor call"
Token::Bool(b)         => WatAST::BoolLit(*b, span)
Token::Str(s)          => WatAST::StringLit(s.clone(), span)
Token::Symbol("nil")   => WatAST::NilLit(span)                   // arc 244: "the asymmetry is annihilated"
Token::Char(c)         => WatAST::List([Keyword(":wat::core::char/of"), StringLit], span)   ← alone
```

## ★ THE CLINCHING MEASUREMENT — wat's own reader cannot see a char literal

```
(:wat::core::read-string "\\a")
  ->  #wat.core.ReadOutcome/Forms [((:wat.core/char/of "a"))]
```

**Read a char literal back and you get a function call.** Every wat program that reads wat — every
`wat-fix` codemod, `wat/lint.wat`, `wat/grep.wat`, anything that re-emits from the AST — is told the
user wrote `(:wat.core/char/of "a")`. They wrote `\a`.

Arc 300's law is `VNVS LECTOR NE DIVIDANTVR` — *one reader, lest they be divided*. A literal the one
reader silently rewrites into a call is that law's own counterexample, which is why this stone is
300's and not a new arc's.

## ⛔ THE GAP IS SHIPPED IN FOUR PLACES AS A LAW

Not hidden. **Written down, four times, as a fact of the language, with a workaround built around it**
— `[[feedback_a_comment_can_ship_a_gap_as_a_law]]`, verbatim, at a new site:

| site | what it says / does |
|---|---|
| `crates/wat-reader/src/parser.rs:404` | the desugar itself |
| `src/runtime.rs:21366` | *"**WatAST has no CharLit variant**; render as `(:wat::core::char/of "c")` so that `(eval-ast! (to-wat char-holon))` round-trips"* |
| `src/closure_extract.rs:1999` | *"Char is portable: encode as a `char/of` call on a length-1 String"* |
| `src/wat_edn_bridge.rs:816` | `Edn::Char(c) => Err(UnsupportedEdnForm)` — **it refuses**, and `:540` lists Char among forms with *"no WatAST counterpart"* |

Three route around the hole; the fourth declines to cross it.

## ★ AND A TOTALITY CLAIM RESTS ON IT — asserted twice, false as written

`src/edn_shim.rs:3996` and `:4651`:

> *"A WatAST is a parsed form — by definition an EDN value (`watast_to_edn`/`edn_to_watast` are a
> **total bijection**)."*

`edn_to_watast` refuses `Char`, `Inst`, `Uuid`, `BigDec`, `Tagged`, and namespaced `Symbol`. The claim
is true only on `watast_to_edn`'s *image* — and it is true there **precisely because no WatAST can
produce an `Edn::Char` today.** The hole is what makes the claim survive.
`[[feedback_a_totality_claim_is_only_as_good_as_its_sampling]]`

★ Note the direction: this stone makes the bijection **more** true (a `CharLit` renders to `Edn::Char`
and decodes back), and in the same motion makes the `Err` arm wrong. Both move together or the claim
gets falser, not truer.

## ⛔ ARC 300 LOOKED STRAIGHT AT THIS AND ROUTED AROUND IT

`DESIGN-STONE-rational-B-runtime.md`, in its own words:

> *"Rational follows the Int/Float precedent … a real `*Lit` variant — **NOT the Char/Uuid precedent**
> (desugar → `(:wat::core::char/of "x")`, parser.rs:355)."*
>
> *"(Deliberate divergence from the **NEWEST scalar precedent** (Char, Arc 220, which desugars).
> Rationals follow the OLDER numeric-literal precedent instead — grounded in decision-freeness,
> ratified in this design.)"*

The desugar was **seen, cited, and diverged from** — as a *legitimate alternative design*, never as a
hole. Being the newest is what made it read as authoritative. Arc 244 had already annihilated the same
asymmetry for `nil` and said so; 300 B and C1 chose the literal lane twice more. **`\c` is the last
survivor of a class this repo has declared annihilated three times**, and it survived by being cited
as precedent for the thing that was replacing it.

## THE FORM

```rust
/// Character literal, as in `\a`, `\newline`, `A`. The lexer resolves
/// named and unicode forms before this point. Scalar-literal lane, NOT a
/// desugared constructor call (arc 244 / arc 300 B / C1 precedent).
CharLit(char, Span),
```

```rust
Token::Char(c) => Ok(Some(WatAST::CharLit(*c, span))),
```

The lexer already resolves `\newline` / `\space` / `\tab` / `A` to a `char` (`Token::Char(char)`),
and the runtime already has `Value::wat__core__Char(char)`. **Nothing new is being represented.** The
AST stops being the one layer that cannot hold what both of its neighbours hold.

## THE FOUR QUESTIONS

- **Obvious?** YES — every other scalar literal is a `*Lit`; a reader that turns `\a` into a call is
  the surprise, and it surprises `read-string`'s callers today.
- **Simple?** YES — it *deletes* a desugar. One node where there were three. The `char/of` verb keeps
  working; it simply stops being the parse target.
- **Honest?** YES — it removes a claim the substrate makes four times and cannot support, and it makes
  a twice-asserted totality claim true where it is currently true only by omission.
- **Good UX?** YES — `\a` reads back as `\a`. A codemod, a formatter, and a linter all see what was
  written.

## COST — a compiler-named cascade, precedented three times

Adding an AST variant breaks every exhaustive `match`. **That is the worklist, not a crisis**
(`docs/SUBSTRATE-AS-TEACHER.md`, FM 15): each error names the next site; watch the count waterfall.

Measured, from the three prior literal-lane additions:

```
NilLit      (arc 244)    42 arms across 18 files
BigIntLit   (arc 300 C1) 33 arms
RationalLit (arc 300 B)  31 arms
```

Expect **~30–45 arms across ~16–18 files**, each mechanical: span accessor, hash, constructors,
type-string, an eval arm, a check arm, an EDN arm.

## WHAT THIS DELETES

- **The 50 phantom-span nodes** measured by `wat-scripts/scratch-pad/probe-span-narrower-than-name.wat`
  — corpus-wide 1461 → **1411**. Not detected: *gone*. The remaining 1411 (`~`, `` ` ``, `~@`,
  `#holon`) are genuine synthesized call heads and are **not** a defect — a macro head carrying its
  call site's span is what every macro system does, Rust's `Span::call_site()` included.
- **Arc 255's four-homes STOP-1, in its char half** — the codemod's worst door stops existing rather
  than needing a guard.
- Two workaround comments (`runtime.rs:21366`, `closure_extract.rs:1999`) and one refusal
  (`wat_edn_bridge.rs:816`).

## WHAT STAYS

`:wat::core::char/of` **the verb**. A runtime String→char conversion is a real operation with real
call sites (17 textual, measured) and its own error surface (length-1, BMP-only). It stops being what
the reader emits; it does not stop existing. Arc 255's four-homes stone renames it to
`:wat::core::char` on its own schedule — **the two stones do not block each other**, and this one
makes that one strictly easier.

## ACCEPTANCE — every bar derived on a freshly-built binary, this session

1. **`(:wat::core::read-string "\\a")` yields a CHAR LITERAL, not a call.** Measured at HEAD:
   `#wat.core.ReadOutcome/Forms [((:wat.core/char/of "a"))]`. This row is the thesis.
2. **`\a` `\newline` `\space` `\tab` `A` all still evaluate and print.** Measured at HEAD: they do.
3. **`(:wat::kernel::println \x)` prints `\x`.** Measured at HEAD: it does. A round-trip regression here
   is this stone's own doing.
4. **The span-disagreement probe drops 1461 → 1411**, and `:wat::core::char/of` disappears from its
   output entirely. Measured at HEAD: 1461, of which 50 are `char/of`.
5. **`(:wat::core::char/of "x")` still works** when written explicitly.
6. **`src/wat_edn_bridge.rs`'s `Edn::Char` arm decodes to `CharLit`** instead of `Err`, and Char leaves
   the `:540` "no WatAST counterpart" list — the same motion arc 300 C1 already performed for `BigInt`.
7. **Floor green, accounted BY NAME** (baseline 5043/5043, 19 skipped); clippy 0 under `-D warnings`.

## OUT OF SCOPE — affirmatively cut

- **The other 1411 synthesized heads.** They are correct. If a codemod ever needs to rename one, the
  answer is `:wat::grep::Written` (255's note, option (c)) — a fact meaning *the name is spelled here*.
  Tracked there; not this stone's work and not blocked by it.
- **`Inst` / `Uuid` / `BigDec` / `Tagged` / namespaced `Symbol`** — the bridge's other four refusals.
  Each is its own question about whether that form belongs in wat source at all; `Char` is separable
  because a char literal already has surface syntax the reader accepts.
- **Renaming `char/of` → `char`.** Arc 255's four-homes stone owns that.
