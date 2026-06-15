# DESIGN — Stone (arc 269 vehicle): `:wat::fix::rename-keyword-prefix`

> Opened 2026-06-15. The reusable keyword/namespace rename codemod on the fix-text engine. Built now
> to drive the spawn-coherence move (`:wat::kernel::{Bound,Spawned,ServiceEvent}` → `:wat::spawn::…`
> over the `.wat` stdlib) and banked as arc 269's tool for the big kernel split. Use-the-tool, not
> hand-fix wat ([[feedback_use_the_tool_not_hand_fix]]). Grounded against HEAD `1681deba`.

## What it is

```wat
(:wat::fix::rename-keyword-prefix old-prefix new-prefix src) -> migrated-src
```
For every keyword **leaf** in `src` whose name **starts with** `old-prefix`, splice the prefix →
`new-prefix`; the suffix (incl. `/accessor` and `::Variant`) is preserved. Comments + formatting
survive byte-identical (rides `fix-text-apply`'s right-to-left span splice). A PREFIX rename so one
call catches `:p::Bound`, `:p::Bound/listener`, `:p::ServiceEvent::Shutdown` together.

## The one contract decision

It is **`fix-text` with a simpler leaf rule.** `fix-text` (fix.wat:310) walks forms collecting
position-aware edits via the arrow/type/head-keyword rules; this rule swaps that for a single,
context-free leaf predicate — *"keyword leaf whose name starts-with `old-prefix` → emit a prefix-swap
edit."* No arrow/position context needed. Everything else (the span→offset mapping, the right-to-left
splice, comment fidelity) is reused verbatim.

## Build shape (mirror fix-text + fix-macro-param-types)

All primitives exist: `string::starts-with?` (string_ops.rs:40), `string::subs`, `string::concat`,
`string::length`, `ast-kind`/`ast-name`/`ast-span`/`ast->children`, `fix-text-offset-of`,
`fix-text-apply`, `read-string`, `reverse`.

1. `rename-prefix-edits [node old-prefix lines] -> Vector<(i64,i64,String)>` — a recursive collector:
   - if `(structural? node)` → `concat` over `(rename-prefix-edits child …)` for each child
     (`ast->children`); 
   - else if `(= (ast-kind node) "keyword")` and `(string::starts-with? (ast-name node) old-prefix)`
     → one edit `Tuple(off, old-len, new-name)` where `off = (fix-text-offset-of (ast-span node) lines)`,
     `old-len = (string::length (ast-name node))`, `new-name = (concat new-prefix (subs name
     (string::length old-prefix) (string::length name)))`;
   - else → empty Vector.
   (Model: `fix-text-leaf-edits` (fix.wat:175) for the edit shape; `fix-text-seq-edits` (fix.wat:231)
   for the recursive concat — but simpler, no `prev-arrow?` thread.)
2. `rename-keyword-prefix [old-prefix new-prefix src] -> String` — `lines = (split src "\n")`,
   `tree = (read-string src)`, `forms = (ast->children tree)`, `edits = (concat over forms of
   rename-prefix-edits)`, `rev = (reverse edits)`, `(fix-text-apply src rev)`. (Model: `fix-text`,
   fix.wat:310.)

Home: `wat/fix.wat` (the rule library), beside `fix-macro-param-types`.

## Scope / out

- **Running it** on the spawn-coherence move + the Rust/test edits — the NEXT stone (the move itself).
  This stone ships + proves the RULE only (on a synthetic src).
- **Not** a full symbol-rename (only keyword leaves) — the kernel names are all keywords; a symbol
  variant is YAGNI until a caller needs it.
- **No engine change** — `fix-text-apply`/`fix-text-offset-of` consumed as-is.

## Probe

`tests/probe_arc269_rename_keyword_prefix.rs` (committed RED) — renames `:my::old::Bound` →
`:my::new::Bound` over a src with both accessor forms + a comment; asserts both swapped, no old
prefix remains, comment byte-identical. RED at HEAD (`UnknownFunction`). GREEN once the rule ships.
