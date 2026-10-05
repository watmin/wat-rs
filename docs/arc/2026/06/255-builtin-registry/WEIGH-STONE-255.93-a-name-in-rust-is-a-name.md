# WEIGH — STONE 255.93: a name in Rust is a name — REJECTED

**Executor: grok via pulsare, solo.** Commits `80ecb7108`, `e6158af7c`, `34f9a45aa`, kept on the local branch
`stone-255.93-rejected`; `main` reset to `84bc32638` by the builder. Weighed by the orchestrator on 2026-10-04.

## Why it is rejected (read in the code, not the report)

- **Dispatch kept the `::` spelling.** "The hot dispatch match stays `&str` arms. A slash head is rewritten to the
  colon spelling once, ahead of the match." The approved form was a match on the pair.
- **The case rule returned.** `slash_names_a_type_member` (`src/name_map.rs:109`) decides a type member by a capital
  letter; `function_under_member_slash` (`src/runtime.rs:1899`) by an uppercase byte. Ruled out on 2026-10-04.
- **A `Name` renders `::`.** `Name::keyword()` (`crates/wat-reader/src/identifier.rs:228`) builds `:ns::name`; each
  name is indexed under four spellings (keyword, display, path, the other join). The ruling allows one stringification,
  `"{namespace}/{name}"`.
- The floor was red (677), the cure unfloored, the cost gate missed by 0.08 s.

## The cause is the plan, not the executor

255.92 and 255.93 keyed the registries by `Name` while the AST, the dispatchers and `src/` still carried `::` text.
Every stone then had to reconcile text with `Name` at lookup and compare sites, so the string machinery grew instead of
dying, and a strict per-stone cost gate rewarded keeping the old fast paths. That reconciles two spellings at the
compare site, which **K1 (2026-09-28)** already ruled against: *canonicalize at the door where a name enters.*

## Next

The door is the reader: a keyword that is a name becomes its pair when it is read, through the one
`Name::from_keyword`, so nothing downstream ever holds a `::` name. First, measure whether any `::` keyword is data
rather than a name (255.94).
