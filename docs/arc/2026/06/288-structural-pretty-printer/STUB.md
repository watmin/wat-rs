# Arc 288 — structural pretty-printer (STUB, banked — do NOT build yet)

**Status:** STUB / banked. Future arc. Captured 2026-06-19. Builder: *"we are getting so incredibly close to
needing a pretty printer — the rete engine is a requirement for it — don't worry about this now."*

## Trigger
The arc-278 EXPLAIN `DerivationNode` tree renders via raw `println`→EDN as a nested tagged-map structure:
legible (the `:constraints` show real forms `(:wat::core::< -5 0)` once the WatAST-render fix lands), but
**tag-noisy** — `#wat.core/PersistentVector […]`, `#wat.core/PersistentMap {…}`, the record tags repeat at
every level. Raw EDN is readable but not *operator-pretty*. This is general (every nested record renders this
way), not rete-specific — but EXPLAIN is what made the need obvious.

## The insight: pretty-printing IS rules-over-structure → built ON rete
A pretty-printer makes layout **decisions** — when to break a line, when to indent, when to elide a structural
tag, how to render each shape, when to collapse vs expand. Those decisions are **rules that fire on structural
patterns** → exactly a rete application. So:
- **rete is a REQUIREMENT for it** (the builder's call): the pretty-printer is downstream of the engine, not
  independent of it. Same shape as lint-as-rete and the structured→surface sugaring renderer (arc 287/lint).
- Rules over the value/AST structure → layout directives → rendered text. Pull-query / forward-fire over the
  shape, emit the rendering.

## Scope sketch (decide at arc-open)
- A `pretty` / `explain-str` surface: structural value → indented, tag-elided, operator-legible text.
- Likely rete rules over the structure (record shape, depth, collection kind) → layout.
- Shares the sugaring pretty-printer with the lint structured→surface renderer (don't build twice).
- The EXPLAIN why-tree is the first marquee consumer (`DerivationNode` → a proof-tree render).

## Relations
- Depends on arc 278 (rete — the engine it's built on) closing.
- Sibling of arc 287 (WorkQuery) + the lint structured→surface renderer (shared sugaring printer).
- NOT a now-thing. Do not build until rete closes and a real consumer is ready.
