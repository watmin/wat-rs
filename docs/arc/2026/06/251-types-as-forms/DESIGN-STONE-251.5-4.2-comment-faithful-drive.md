# DESIGN — Stone 251.5 / Slice 4.2: the comment-faithful `.wat` corpus drive (span-edit codemod)

**Status: STRIKE-READY (drawn 2026-06-10). Four-questions-decided (A: span-edit codemod, builder-
ratified; B trivia-reader and C hand-migrate both failed O+S+H).** Migrates the 27 dirty `.wat` files
to faithful-Clojure surface while PRESERVING comments + formatting. The first *reversible* checkpoint
of Strike 4 (dual-read still parses both spellings). Home: a throwaway Rust codemod harness (retires
with the hard-cut) + the already-built wat `fix-form` for decisions.

## Why not the naive drive (the trap drawing 4.2 surfaced)
`read-string`→`fix-form`→`write-forms` is a pure-AST round-trip; comments are trivia, not AST, so it
**deletes every comment** (empirically confirmed). The dirty stdlib is heavily documented —
`test.wat` 668 comment-lines, `stream.wat` 236, `core.wat` 135, **2,000+ doc-lines total**. Deleting
them is below the bar. This is normal for a Lisp *reader* (Clojure's `read`/`pr` drop comments too);
codemods use a trivia-preserving layer (`rewrite-clj`), NOT the core reader. 4.2 is that layer.

## Grounded reality (the architecture constraint)
- `Span = { file, line: i64 (1-idx), col: i64 (1-idx, char-count) }` (`src/span.rs:48`) — char-based
  line/col, NOT byte-offsets. The splice maps `(line, col, char-len)` → byte range via the source's
  line-start offsets.
- **No wat span accessor** — the homoiconic bridge exposes `ast-kind`/`ast-name`/`ast->children` but
  NOT spans. So the codemod *application* (diff + splice) is **Rust**; the *decisions* stay wat.

## The strike — a throwaway Rust codemod harness
Reuses the wat `fix-form` (the single source of role-inversion decisions); the harness only diffs +
splices. Per dirty `.wat` file:
1. Read the file **text**; parse → old-tree (Rust AST with line/col spans).
2. For each top-level form: `new = eval(wat ":migrate::fix-form", old)` (decisions reused, no
   duplicated grammar).
3. **Diff old-form vs new-form → edits.** `fix-form` is structure-preserving EXCEPT `strip-if` (drops
   the `-> :T` of an annotated `if`). Decompose:
   - **strip-if as a deletion pass:** find annotated-ifs in old-tree; emit DELETION edits covering the
     `->` + type token spans. (The only arity-changing rule — handled once, separately.)
   - **everything else is leaf-text-only → a parallel-leaf-walk:** old & new now share shape; walk
     pairwise; where a leaf's rendered text differs, emit `(old-leaf-span-range → new-leaf-text)`.
4. Map each edit `(line, col, char-len)` → byte range (precompute line-start byte offsets; col is
   char-count so handle multi-byte UTF-8).
5. Apply edits **right-to-left** (offset-stable) → new text. Comments/blank-lines/formatting between
   edited tokens are untouched.
6. Write the file.

## The gate
- `cargo test --release --workspace --no-fail-fast` GREEN — migrated `.wat` still load + run (dual-read
  parses both spellings; this is the reversible checkpoint, so green is expected, not a cutover).
- **Comment-preservation verified:** a sample file (`core.wat`) — every `;;` line survives
  byte-identical; the `git diff` shows ONLY migrated tokens changed (eyeball-auditable minimal diff).
- An FM-2-bis probe FIRST: a fixture `.wat` (a few forms: a head, an annotated-if, a binder arrow, a
  typealias core-scalar target, an inline `;; comment`) → run the harness → assert comments survive +
  the tokens migrated. RED before the harness exists.

## Reusability (the payoff)
This harness IS the engine for 4.3 (the 267 rust-test-strings): extract each embedded wat string →
same `fix-form` + same splice (into the string's content). The 4.3 rune-marks decide WHICH strings;
this harness does the migration. So 4.2 builds the durable migration engine, not a one-off.

## Risks
- **strip-if span identification** — the deletion pass must precisely cover the `->`+type tokens
  (including inter-token whitespace). Probe it; STOP if the annotated-if shape varies.
- **line/col → byte mapping** — char-count cols + multi-byte UTF-8 (the corpus has `∀`, `→` in
  comments, but edits target code tokens which are ASCII; still, map via char-indices, not bytes).
- **idempotence** — re-running the harness on an already-migrated file must be a no-op (the faithful
  forms parse but `fix-form` leaves them unchanged → zero edits). Verify.

## Out of scope
- 4.3 (rust-strings — reuses this engine), 4.4 (hard-cuts). The `<>` lexer + keyword-type spellings
  stay live (dual-read) until 4.4.
