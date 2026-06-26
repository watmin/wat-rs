# NOTE — wat-fixes-rust (+ wat-fixes-wat-in-rust): the migration toolkit, over Rust

**Status: DEFERRED CAPABILITY — NOT an arc (2026-06-23).** The builder's call: *"we'll do the
wat-fixes-rust whenever we have a mammoth refactor before us — no need to arc it, we'll make it when we need
it."* This is the don't-build-the-forcing-function discipline (capabilities arrive precisely when they must):
the next **mammoth** `.rs`/wat-in-rust refactor is the forcing function; build it then, not before. This NOTE
is the design-on-the-shelf so that build is cheap when it comes. Surfaced during arc 291
strike-3a-ii-β: the qualified annihilation of the client-stop op had a blast radius of **5 probe `.rs`
files** (tests asserting the removed `Op::Stop`/`Reply::Stop`/`/stop c`), every one **hand-edited** because
the wat-fix toolkit (`wat/fix.wat`, `wat-scripts/fixes/`) rewrites **`.wat` source**, not **wat embedded in
Rust string literals**. That boundary is the documented wat-fix limit. This NOTE captures the capability
that would close it — the builder's idea, with the apparatus's grounded read — so it survives compaction.

## The idea (the builder's)

> *"should we pick a rust lexer and just …. do it? … there's some lexer who yields rust-as-values … we could
> shim that into wat and wat rewrites rust /and delegates to fix.wat for wat strs/ ? … we could make a
> wat-fixes-rust and wat-fixes-wat-in-rust?"*

Two tiers: **`wat-fixes-rust`** (rewrite Rust source) and **`wat-fixes-wat-in-rust`** (rewrite the wat
inside Rust string literals, by delegating the inner wat to `fix.wat`).

## Why it is small-in-principle (the apparatus's read, grounded in `fix.wat`)

The editing core is **already source-language-agnostic**. `fix.wat`'s `fix-text-*` verbs
(`fix-text-offset-of` / `fix-text-span-len` / `fix-text-node-edits` / `fix-text-deletion-edit` /
`fix-text-apply`, with `fix-source`) operate on **source text + byte-spans** — comment-faithful, idempotent
span replacements. The wat parser merely *feeds* them nodes-with-spans. **The only new part for Rust is the
parse that yields Rust-as-values-with-spans.** The edit machinery is reused untouched.

This is the **narrow-waist / N-loci-one-interface law** (291 R4) applied to the codemod tool: the
`fix-text` core is the invariant waist; the source-language parse is a pluggable member. A new language
joins with one shim. **wat-fix becomes locus-over-source-language** — the same move as 3a-ii-β-foundation's
`Thread'/Process' <: Peer'` (a new member joins the family without touching the core).

## Lexer choice — `proc-macro2` (tokens), not `syn` (AST), for the motivating case

- **`syn`** = full typed Rust AST (`ItemFn`, `Expr`, …). Huge surface to shim into wat values; needed only
  for *semantic* rust rewrites ("rename this method respecting scope"). Defer until a semantic transform
  actually needs it.
- **`proc-macro2`** = a `TokenStream` of `TokenTree` (`Ident`/`Punct`/`Literal`/`Group`), **each carrying a
  byte-`Span`**. Lightweight, lexical. Sufficient to: find a wat-bearing `Literal` (string) token, find/rename
  `Ident`s. **This is the right first shim** — token-level, span-bearing, matches how `fix-text-*` already works.

## The composition — it's a specialization, not two systems

- **`wat-fixes-rust`** = `proc-macro2` tokens → wat values (tokens + spans) → wat pattern-matches tokens →
  emit `fix-text` edits → `fix-text-apply`.
- **`wat-fixes-wat-in-rust`** = `wat-fixes-rust` locates the `Literal` token whose content parses as wat →
  hand its **inner byte-span** to **`fix.wat`** (the existing wat codemod) → splice the rewritten wat back
  via a `fix-text` edit. **`wat-fixes-wat-in-rust` = wat-fixes-rust ∘ fix.wat.** The delegation the builder
  described, falling out for free.

## Scope honesty (why it's an arc, not "a single func")

A `proc-macro2`-token shim into wat values (a new `Value`/walkable surface or a `:rust::*` rust-deps shim) +
the token→fix-text bridge + the wat-string delegation + tests/oracle. That is **arc-sized**, not a single
`fix.wat` func. Hence ARC-CANDIDATE, deferred ("not yet"), surfaced for the builder's call — per
`feedback_dont_greedily_create_arcs` (don't unilaterally mint).

## Motivating cases (the forcing function accruing)

- **291 strike-3a-ii-β** — 5 probe `.rs` files hand-edited (the removed `Op::Stop` assertions).
- **293.2-rename (`60d7d99a`)** — the `Record::def → defrecord` rename: **12 `.wat` files** flowed through the
  self-hosted fix-wat codemod cleanly, but **81 `.rs`** wat-in-string fixtures were swept with a blunt `sed`
  — and `sed` **corrupted prose comments** that intentionally named the OLD form (a probe doc-comment, an
  arc-237 *"RED today: X does not exist"* historical state, an arc-209 mixed doc). Tests stayed green; the
  *record lied*. The weigh (not the suite) caught it; 3 docs were repaired by hand. **This is the sharper
  argument for the tool:** a `proc-macro2`-token shim renames only inside the wat-bearing `Literal` tokens and
  **never touches a `//`/`///`/`//!` comment** — the exact distinction `sed` is blind to. The boundary isn't
  merely "tedious to hand-edit"; an unguided textual sweep is *unsound over prose*.

## Until it exists (the standing boundary)

wat-in-Rust-string `.rs` fixtures are **Rust edits** (hand-edited or via Rust tooling), NOT wat-fix targets.
`.wat` source is wat-fix's domain. **A blunt `sed`/text-replace over `.rs` is NOT a safe substitute** — it
cannot tell a wat-string-literal from a prose comment, so it corrupts historical/contrast comments (the
293.2-rename incident above). Until the tool exists: rename inside string literals only, and audit comment
substitutions by hand in the weigh. This NOTE is the marker that the boundary is *known and intentional*, with
a designed exit.

## Pairs
`feedback_lean_on_wat_migration_toolkit` (the toolkit + the .rs boundary) · `project_wat_is_linux_best_of_breed`
(best-of-breed: proc-macro2/syn are the Rust-native parsers) · 291 R4 (narrow waist / N-loci) ·
`STRIKE-3a-facet-split.md` §"3a-ii-β SHIPPED" (the 5-probe hand-edit that motivated this).
