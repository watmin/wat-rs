# Arc 282 — wat-fix over Rust: codemod the host language with borrowed eyes + wat rules

> **STATUS: STUB / HORIZON — banked (2026-06-17). BLOCKED behind arc 278 (rules engine).**
> Surfaced by the builder while the arc-281 `Span`-gains-a-field change was hand-rippling into 8
> `Span { .. }` literal sites: *"do we extend wat-fix to write its own rust code? … write how to fix our
> entire code base rust or wat … and express it in wat."* The answer: **yes — but borrow the parser,
> never write it in wat.**
>
> **ATTESTED DECISION (builder, 2026-06-17): we WILL take a dependency on Rust's lexer (`rustc_lexer`)
> and EXPOSE IT AS A wat INTRINSIC** — a `:wat::core::rust::lex` (name TBD) that takes Rust source
> (`String`) and returns the token stream as EDN facts (token kind + byte/line/col span). That is the
> Rust fact-source; everything else (rules, apply) is the existing wat machinery. Do NOT start before
> 278 ships — this is a fact-source for that engine, not a standalone machine — but the dep direction is
> now decided, not open.

## The split — the folly vs the real idea

- **THE FOLLY (rejected):** writing a Rust lexer/parser *in wat*. Rust's grammar is enormous and a
  moving target; chasing rustc in wat is years of work for a thing that already exists. Extirpare: do
  not construct the situation that needs the re-implementation.
- **THE REAL IDEA:** wat-fix is three halves — **find** (rules over facts), **fix** (emit a
  `(offset, old-len, new-text)` edit), **apply** (string surgery). Grounded 2026-06-17: **`fix-text-apply`
  (`wat/fix.wat`) is already pure, language-AGNOSTIC string surgery** —
  `subs[0..off] + new-text + subs[off+old-len..]`, with no idea whether the source is wat or Rust. **The
  apply half works on Rust source today, unchanged.** The only Rust-shaped gap is a **fact-source**:
  turn Rust source → structured facts (tokens / AST nodes WITH spans) as EDN.

## The architecture

```
borrowed Rust frontend → EDN facts (nodes + spans)    ← the EYES   (Rust, BORROWED, not built)
        ↓
   wat defrules match facts → emit fix edits            ← the BRAIN  (wat, homoiconic rules)
        ↓
   fix-text-apply over the source                       ← the HANDS  (wat, already language-agnostic)
```

The wat expresses the **policy** (what to fix) and does the **surgery**; the borrowed parser provides
the **structure**. "Express it in wat" = the rules + the apply are wat; the parser is not.

### Fact-source: DECIDED — `rustc_lexer`, exposed as a wat intrinsic

**Chosen (builder-attested 2026-06-17): `rustc_lexer`** — the *actual* rustc tokenizer, published as a
standalone crate; tiny, dependency-light, robust, the real thing rustc itself uses. Token-level (token
kind + byte length per token); the intrinsic walks the token stream accumulating byte→line/col to stamp
each token with a span. This is the cheapest *and* most faithful fact-source, and it handles the large
class of codemods we actually need (rename, add-field-to-struct-literal, add-arg, attribute insert,
import-line edits) via token-pattern + the span seam. NO full AST — token + span is enough for the
text-splice apply model, and it tolerates not-yet-compiling source (lexing ≠ parsing).

**The intrinsic** — `:wat::core::rust::lex` (name TBD at open):
`(String) -> Vector<{:kind String :text String :line i64 :col i64 :end-line i64 :end-col i64}>` (or a
typed `RustToken` record). EDN facts, the no-magic shape: a wrong/garbled token is a visible bad fact,
not a silent coercion. Mirrors how `read-string`/`ast-span` already expose the wat frontend to wat — this
exposes the *Rust* frontend the same way. Impl beside the other reflection intrinsics
(`src/edn_shim.rs` / `runtime.rs` dispatch / `check.rs` scheme); pure + total (lexing is deterministic,
no IO), so it earns the macro-eval allow-list too if a macro ever needs it.

**Why a dep is acceptable here** — Cargo.toml carries ZERO rust-frontend deps today (the project is
deliberately dep-light), but `rustc_lexer` is the minimal, canonical exception: it is *rustc's own
lexer*, not a third-party reimplementation, so it cannot drift from the language it tokenizes. Taking it
is taking the source of truth, not a parallel guess. (Rejected alternatives: `syn`/`proc-macro2` —
heavier, full-AST, more than the splice model needs; `tree-sitter-rust` — a separate grammar that *can*
drift from rustc. `rustc_lexer` is the no-drift choice.)

## This IS arc 278 with a second fact-source

A rules engine is facts + rules → fired actions. Arc 278 (RETE/Clara-shaped) runs wat `defrule`s over a
fact base. Swap the fact base from "wat forms (read-string)" to "Rust AST nodes (borrowed frontend, as
EDN)" and the SAME engine drives Rust codemods. arc 282 is not a new machine — it is arc 278 pointed at
a Rust fact-source. Hence the block: build the engine first, prove it on wat (lint), THEN add the Rust
fact-source.

## The motivating worked example (live, 2026-06-17)

arc 281 added `end_line`/`end_col` to `Span` → every **direct struct-literal** `Span { file, line, col }`
broke (E0063) and was hand-edited across ~8 sites (test files). A wat-fix-over-Rust expresses that as
ONE rule: *"every `Span { .. }` literal → splice in `end_line: <v>, end_col: <v>`."* The pain of the
manual ripple IS the use case. And the loop closes: the substrate that self-corrects its own **wat**
form (arc 277, the self-fixing toolchain) would self-correct its own **Rust** form too — self-hosting
reaching down into the host language.

## Prior art (collision, recorded straight)

`comby`, `ast-grep`, `semgrep`, `rerast`, rust-analyzer's SSR (structural search/replace), tree-sitter
codemods. **Genuinely ours:** the rule language is our own homoiconic Lisp — the same one the substrate
is written in — driven by the self-fixing-toolchain doctrine; and (278's horizon) the LHS can be
**VSA/coincidence-matched** — a *fuzzy* structural matcher, similarity over equality. "ast-grep, but the
rules are your Lisp and the match can be coincidence, not exact."

## Four questions (sketch)

- **Obvious?** YES once split — borrow the frontend, rule in wat, apply already works. The folly
  (parser-in-wat) is the only un-obvious path, and it's rejected.
- **Simple?** YES if scoped to borrow-the-frontend (a bounded Rust→EDN bridge + existing wat machinery).
  NO (folly) if writing the parser in wat.
- **Honest?** The framing must be "wat RULES OVER borrowed Rust facts," never "wat parses Rust." The
  fact-bridge must emit clean EDN (no magic that lets a wrong fact through).
- **Good UX?** YES — one `defrule` surface over Rust facts, the same CLI, the same apply; a Rust codemod
  reads like a wat codemod.

## When opened (the sequence)

1. arc 278 ships + is proven on wat (lint runs on the engine).
2. Add the `rustc_lexer` dep + build the **`:wat::core::rust::lex` intrinsic** (DECIDED — token stream →
   EDN facts with spans; the byte→line/col accumulation is the only real work; impl + dispatch + check
   scheme + allow-list, mirroring `read-string`/`ast-span`). RED probe first: `rust-lex` of a known
   Rust snippet → the expected token+span facts.
3. Wire the Rust facts into the rules engine (arc 278) as a second fact-source; rule over them in wat,
   emit `(off,old-len,new-text)` edits, apply via the existing `fix-text-apply`. RED probe: a known Rust
   codemod (e.g. the `Span`-field-add) expressed as a wat rule produces the right edits.
4. Run it on the wat-rs Rust corpus itself (dogfood); the diff is the proof — the substrate codemods its
   own host.
