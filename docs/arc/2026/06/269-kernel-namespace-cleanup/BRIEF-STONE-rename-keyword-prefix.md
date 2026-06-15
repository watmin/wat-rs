# BRIEF — Stone (arc 269 vehicle): `:wat::fix::rename-keyword-prefix`

Executor: Shadowdancer (sonnet). Anchor: `/home/watmin/work/holon/wat-rs/` (verify `pwd`; ONLY here;
ignore `.claude/worktrees/`). Run `cargo` PLAINLY (no setsid/timeout). Trust your own clean build over
rust-analyzer. **Do NOT commit — the Inquisitor weighs.** Report HONESTLY what you changed (the
Inquisitor reads the diff). Full rationale: `DESIGN-STONE-rename-keyword-prefix.md` (this dir).

## Work in one paragraph
Add `:wat::fix::rename-keyword-prefix` to `wat/fix.wat` — a comment-faithful keyword-PREFIX rename on
the existing fix-text engine. It is `fix-text` with a simpler leaf rule: every keyword leaf whose name
starts-with `old-prefix` gets its prefix spliced to `new-prefix` (suffix preserved). Pure wat, two
small fns, reusing the engine verbatim.

## Rooms (read the models, then add beside them in `wat/fix.wat`)

1. **`wat/fix.wat:175` `fix-text-leaf-edits`** — the edit-shape model: `(Tuple off old-len new-text)`
   where `off = (fix-text-offset-of (ast-span node) lines)`, `old-len = (string::length (ast-name node))`.
2. **`wat/fix.wat:231` `fix-text-seq-edits`** — the recursive concat-over-children model (yours is
   simpler — no `prev-arrow?` thread).
3. **`wat/fix.wat:310` `fix-text`** — the entry model: split lines, read-string, collect edits over
   forms, reverse, `fix-text-apply`.

Add TWO defns at the end of the rule library (after `fix-macro-param-types`):

```wat
;; rename-prefix-edits — recursive collector: every keyword leaf whose name starts-with
;; old-prefix → one prefix-swap edit; structural nodes recurse; other leaves → no edit.
(:wat::core::defn :wat::fix::rename-prefix-edits
  [node <- :wat::WatAST  old-prefix <- :wat::core::String  lines <- :wat::core::Vector<wat::core::String>]
  -> :wat::core::Vector<(wat::core::i64,wat::core::i64,wat::core::String)>
  ;; if structural? → concat over (rename-prefix-edits child old-prefix lines) for each ast->children
  ;; else if (ast-kind node)="keyword" and (string::starts-with? (ast-name node) old-prefix):
  ;;      emit (Tuple off old-len (concat new-prefix (subs name (length old-prefix) (length name))))
  ;;   ⚠ new-prefix is NOT in scope here — pass it too, OR build the new name in the caller.
  ;;      Simplest: give this fn BOTH old-prefix + new-prefix params.
  …)

;; rename-keyword-prefix — the rule: old-prefix new-prefix src → migrated-src.
(:wat::core::defn :wat::fix::rename-keyword-prefix
  [old-prefix <- :wat::core::String  new-prefix <- :wat::core::String  src <- :wat::core::String]
  -> :wat::core::String
  ;; lines = (string::split src "\n"); tree = (read-string src); forms = (ast->children tree)
  ;; edits = concat over forms of (rename-prefix-edits form old-prefix new-prefix lines)
  ;; (fix-text-apply src (reverse edits)))
  …)
```
(Give `rename-prefix-edits` BOTH `old-prefix` and `new-prefix` params so it can build the new name at
the leaf — that's cleaner than threading. Use the EXACT helper names the engine uses:
`:wat::core::structural?`? — there's no public `structural?`; use `:wat::fix::structural?` if it
exists, else dispatch on `(:wat::core::ast-kind node)` being a structural kind / use
`:wat::core::ast->children` returning non-empty. GROUND `:wat::fix::structural?` at fix.wat before
relying on it; if absent, recurse when `ast->children` is non-empty and the node isn't a leaf kind.)

## Gate (run all; report verbatim from YOUR runs)
```
cargo test --release -p wat --test probe_arc269_rename_keyword_prefix      # 1 passed (RED→GREEN: prefix swapped, comment byte-identical)
cargo test --release -p wat --test probe_arc251_fix_text_comment_faithful  # passes (engine unbroken)
cargo test --release -p wat --test probe_arc251_fix_macro_param_types      # passes (sibling rule unbroken)
cargo test --release -p wat --lib -- --test-threads=1                      # zero NEW vs baseline 917/36
cargo test --release -p wat --test nursery -- --test-threads=1             # zero NEW vs baseline 895/4
cargo test --release --workspace --no-run                                  # compiles
```

## STOP triggers (REJECT — surface; do not improvise)
1. There's no clean way to recurse structural-vs-leaf (no `structural?`/the kind set is unclear) →
   STOP; report how `fix-text-seq-edits`/`fix-text-struct-edits` decide structural-vs-leaf (mirror it).
2. The prefix-swap produces a malformed keyword (e.g. `subs` char-index vs byte-index mismatch on the
   suffix) → STOP; report (the probe's accessor forms catch this).
3. Comment fidelity breaks (the probe's `;; KEEP THIS COMMENT` is altered/moved) → STOP.
4. Any engine fn (`fix-text-apply`/`fix-text-offset-of`/`fix-text`) needs changing → STOP (this rule
   only ADDS two defns; it consumes the engine).

## Blast radius
`wat/fix.wat` ONLY (+2 defns at the end of the rule library). NO Rust changes, NO engine changes, NO
other files. The probe is already committed.

## Return
Report: the two defns (file:line) + how you decided structural-vs-leaf recursion + char-index handling
for the suffix; every gate command's counts from YOUR runs; confirm the engine + sibling rule still
pass; any honest delta. If a STOP fires, STOP and report. Do NOT commit.
