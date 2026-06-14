# BRIEF — Stone 251.5-4.2: `fix-text`, wat's comment-faithful span-edit codemod (rewrite-clj)

**Executor:** Shadowdancer (sonnet). **Anchor:** `/home/watmin/work/holon/wat-rs/` (verify `pwd`;
operate ONLY here; `git -C /home/watmin/work/holon/wat-rs`; any `.claude/worktrees/` path is harness
state — ignore). **Design (read fully):** `docs/arc/2026/06/251-types-as-forms/DESIGN-STONE-251.5-4.2-comment-faithful-drive.md`
§ "The codemod, in wat" (the algorithm) + § "Risks". The RED probe is on disk + verified RED
(`tests/probe_arc251_fix_text_comment_faithful.rs` — `UnknownFunction :wat::fix::fix-text`). Do NOT
commit — the Inquisitor weighs.

## The work in one paragraph

Add ONE runtime defn `:wat::fix::fix-text` to `wat/fix.wat`: `(fix-text src) -> migrated-src` — a
**comment-faithful** codemod. The naive `read-string → fix-source → write-forms` round-trip deletes
every comment (a reader drops trivia). Instead, parse only to **locate** edits, then splice the
**original text** so comments + formatting survive byte-identical. This is a **runtime defn** (like
the existing `fix-source`), NOT a defmacro — so it is OUTSIDE the macro purity fence; use
`read-string` / `ast->children` / `ast-span` / `ast-name` / `ast-kind` freely.

## The algorithm (from the design — follow it)

```
fix-text(src):
  tree  = (read-string src)               ;; forms; every node carries a span (ast-span)
  edits = walk tree, reusing fix-source's per-leaf DECISIONS, but EMIT edits instead of rebuilding:
            - a leaf the rule rewrites (head-keyword?→symbol, arrow?→:-, type-shaped?→type-form):
                edit = { off: (offset-of (ast-span leaf) src),
                         old-len: (string::length (ast-name leaf)),
                         new-text: <rendered fixed leaf> }
            - strip-if (annotated-if): DELETION edits over the `->` + type leaf spans (the only arity change)
  ;; line/col → flat char offset:
  offset-of(loc, src) = (line-start (:line loc) src) + ((:col loc) - 1)
       line-start precomputed: (string::split src "\n") → cumulative (string::length + 1) per line
  ;; apply RIGHT-TO-LEFT (highest offset first, so earlier offsets stay valid):
  for edit in (reverse (sort-by off edits)):
     src = (string::concat (string::subs src 0 off)
                           new-text
                           (string::subs src (+ off old-len) (string::length src)))
  return src
```

A thin `(fix-file path)` wrapper (`read-file → fix-text → write-file`) is fine to add too, but the
PROBE only needs `fix-text` (string → string). Keep `fix-file` minimal if you add it.

## Read in order (the rooms)

1. `wat/fix.wat` — `fix-source` / `fix-seq` (the per-leaf DECISIONS you reuse) + `strip-if`
   (the deletion case) + `structural?`/`arrow?`/`head-keyword?`/`type-shaped-keyword?`.
2. `tests/probe_arc251_fix_text_comment_faithful.rs` — the gate (comment byte-identical + `-> :T`
   stripped). Make it GREEN.
3. The DESIGN § "The codemod, in wat" (steps 1-5, verbatim) + § "Risks".
4. Primitives (ALL exist — grounded): `:wat::core::ast-span` (→ `{:line :col}` map; `(:line loc)`
   reads it), `ast-name`, `ast-kind`, `ast->children`, `read-string`, `:wat::core::string::subs`,
   `string::split`, `string::length`, `string::concat` / `concat`, `:wat::core::reverse`,
   `:wat::core::sort'`. `(:wat::io::read-file path)` + `write-file` (wat/io.wat) for `fix-file`.

## Blast radius

`wat/fix.wat` (the new `fix-text` defn + optional `fix-file`). NO Rust change (every primitive
exists). NO change to `fix-source` / `fix-seq` (reuse them). NO change to the probe.

## STOP triggers (rejection — ship nothing, report)

1. **STOP-1:** a primitive you need is missing or behaves differently than the design assumes
   (e.g., `string::subs` is byte- not char-indexed; `ast-span` doesn't give the start col) — report
   the exact gap; do NOT nest a workaround (that's the reach-stumble the design was redrawn to avoid).
2. **STOP-2:** right-to-left application can't be expressed (`reverse`/`sort'` don't compose over the
   edit vector) — report; do NOT fall back to the comment-DELETING `write-forms` round-trip.
3. **STOP-3:** char-vs-byte — `:col` is a char count; `subs` must be char-indexed; if they disagree,
   the corpus has multi-byte `∀`/`→` in comments and offsets would corrupt — report rather than ship
   a byte-offset version.

## The gate (report each exact line; do NOT commit)

```
cargo test --release -p wat --test probe_arc251_fix_text_comment_faithful   # 1 passed
cargo test --release -p wat --lib -- --test-threads=1                       # 915 passed / 36 failed (PRE-EXISTING; zero new)
cargo test --release -p wat --test nursery -- --test-threads=1              # 895 passed / 4 failed (zero new)
cargo test --release --workspace --no-run                                   # full surface compiles
```
NOTE: lib has 36 PRE-EXISTING failures (`check::`/`runtime::tests`) — confirm it stays 36. Run
`cargo test` PLAINLY (no setsid/timeout). Stale rust-analyzer diagnostics may contradict a clean
`cargo build` — trust your own build.

## Prior comparable (copy the shape)

`wat/fix.wat` itself (the `fix-seq`/`fix-source` walk you adapt). For the strike-cycle shape,
`BRIEF-STONE-C0b.3b-e.md`.
