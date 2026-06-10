# DESIGN — Stone 251.5: THE unified corpus sweep, via a wat-to-wat fixer

**Status: foundation PROVEN (2026-06-10); decomposition drawn. The keystone of arc 251.**

After the dual-read enabler phase (251.1–251.4: every core.typed surface reads alongside its
legacy spelling), 251.5 migrates the WHOLE corpus to the new surface in one churn and then
HARD-CUTS the legacy surfaces. The realization (builder, 2026-06-10): this is **nothing but a
structural data-swap over EDN** — so it is a **program, not a sed**, and the program is written
**in wat itself**. The arc that makes wat genuine EDN uses wat's own EDN-ness to migrate itself.

## Why a program, not a sed

The transform is a position-aware AST rewrite. A sed is fragile exactly where structure decides:
the `->` overload (return arrow vs `(-> …)` threading head), `:wat::core::i64` (type) vs
`:wat::core::i64::+` (op), and any token inside a string/comment. A structural walk knows the
position; all of that dissolves.

## Why in wat (the proof)

Every primitive needed already exists — ZERO new Rust for the fixer core:

| need | primitive |
|---|---|
| read file → text | `:wat::io::read-file` |
| text → forms-as-data | `:wat::edn::read` |
| transform forms | the arc-249 macro engine + quasiquote/`forms` |
| forms → clean EDN text | `:wat::edn::write` |
| write text → file | `:wat::io::IOWriter/open-file` + `write-string` |

A wat-to-wat fixer is the homoiconic-Lisp claim made executable: read wat-as-data, rewrite the
tree, write wat-as-data.

## Foundation — PROVEN

`tests/probe_arc251_stone5_roundtrip.rs`: `program_to_edn → edn_to_program` is a faithful
identity (span-agnostic) over a representative program — defn, `Vector<>` parametrics, `foldl`,
`HashMap`, maps, sets, vectors, quoted forms. So `read → write` corrupts nothing; the fixer
changes ONLY what the transform changes. (This is the `wat_edn_bridge` round-trip the
`:wat::edn::read`/`write` primitives wrap.)

## The role-inversion transform (the data-swap rules)

Position-aware rewrite over the forms (the crux — the fixer encodes wat's grammar, the same
knowledge the resolver has):

| position | from (legacy) | to (clojure-faithful) |
|---|---|---|
| call head (head of a list) | `:wat::core::map` (keyword) | `wat.core/map` (symbol) |
| binder arrow | `<-` | `:-` |
| return arrow | `->` (sig position, NOT `(-> …)` threading) | `:-` |
| type atom (type position) | `:wat::core::i64` / `:i64` | `wat.type/i64` (symbol) |
| parametric type | `:wat::core::Vector<T>` | `(wat.type/Vector T)` (form) |
| fn type | `:wat::core::Fn(A)->R` | `[A :-> R]` (bracket) |
| rust interop | `:rust::a::b::C` | `rust.a.b/C` (symbol) |
| **DATA keyword (untouched)** | `:foo` `:keys` `:else`, field accessors | unchanged |

The hard discriminant: a keyword in **head position** is a call → symbol; a keyword **elsewhere**
is data → stays. Type slots (binder type, return type, field types) are known grammar positions.

## Decomposition (each gated by the full suite; hard-cuts strictly LAST)

- **251.5a — the fixer core, in wat.** `fix-source : String -> String` (`edn::read` → role-invert
  → `edn::write`). Probe: a known dirty form in → the exact clean form out (per the table). The
  transform built rule-by-rule, each gated.
- **251.5b — drive over the `.wat` corpus** (114 files) via `read-file`/`write-file`. Full suite
  green after (dual-read means both spellings work, so the migrated corpus still runs).
- **251.5c — the Rust-test-string adapter** (the hard part, builder-flagged): 256 files / ~7,179
  lines of wat embedded as Rust string literals in non-uniform wrappers
  (`startup_from_source`×424, `parse_one!`×227, `eval_in_frozen`×198, …). A thin Rust harness
  extracts each wat-source string, routes it through the wat fixer's `fix-source` (via eval),
  re-escapes, reinjects. The transform is REUSED; only the IO adapter is Rust.
- **251.5d+ — the HARD-CUTS** (irreversible, LAST, only after 5a-c verify): delete the `<-`/`->`
  arrows, the keyword type spellings, the `<>` `angle_depth` lexer machinery (lexer.rs:637-730),
  the `:wat::core::Fn(...)->...` parser; flip the internal canonical `:wat::core::`→`:wat::type::`.
- **251.6** (separate): native symbol dispatch + ANNIHILATE `src/resolve/normalize.rs`.

## Open / to resolve while building 5a

- Faithful round-trip preserves structure but NOT formatting/comments (EDN write reflows). For
  `.wat` files this is acceptable (they're data); for Rust-embedded strings, reflow inside a
  string literal is fine. Confirm no test asserts on exact source text.
- Whether to also extract the 7k lines of embedded wat to `.wat` fixtures (permanent surface
  reduction) vs in-place rewrite — a 251.5c sub-decision; default in-place (faster), revisit if
  the adapter proves fragile.
