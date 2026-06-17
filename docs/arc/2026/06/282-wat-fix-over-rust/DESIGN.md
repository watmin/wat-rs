# Arc 282 — wat-fix over Rust: codemod the host language with borrowed eyes + wat rules

> **STATUS: STUB / HORIZON — banked to mull (2026-06-17). BLOCKED behind arc 278 (rules engine).**
> Surfaced by the builder while the arc-281 `Span`-gains-a-field change was hand-rippling into 8
> `Span { .. }` literal sites: *"do we extend wat-fix to write its own rust code? … write how to fix our
> entire code base rust or wat … and express it in wat."* The answer: **yes — but borrow the parser,
> never write it in wat.** Do NOT start before 278 ships; this is a fact-source for that engine, not a
> standalone machine.

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

### Fact-source options (borrow, don't build — decision deferred to open)

- **`rustc_lexer`** — the actual rustc lexer; tiny, robust, token-level (token kind + byte span). Cheapest;
  handles a large class of codemods (rename, add-field-to-struct-literal, add-arg, attribute insert) by
  token-pattern + the span seam. NO full AST.
- **`syn` + `proc-macro2`** — full Rust AST in Rust; walk it and emit nodes+spans as EDN. Heavier; gives
  real structure (match on item/expr/struct-literal shapes).
- **`tree-sitter-rust`** — incremental, error-tolerant, has a query language; strong for structural
  match over possibly-not-compiling source.

⚠ **Dependency decision:** Cargo.toml carries ZERO rust-frontend deps today (grounded 2026-06-17) — the
project is deliberately dep-light. Adding any of the above is a real call to weigh when this opens.

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
2. Decide + add the frontend dep (rustc_lexer first — token-level covers the most-needed codemods).
3. Build the Rust→EDN fact-bridge (nodes/tokens + spans). RED probe: a known Rust codemod (e.g. the
   Span-field-add) expressed as a wat rule produces the right `(off,old-len,new-text)` edits.
4. Run it on the wat-rs Rust corpus itself (dogfood); the diff is the proof — the substrate codemods its
   own host.
