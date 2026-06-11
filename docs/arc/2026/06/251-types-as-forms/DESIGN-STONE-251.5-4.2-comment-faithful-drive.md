# DESIGN — Stone 251.5 / Slice 4.2: wat's `rewrite-clj` — the comment-faithful codemod

**Status: STRIKE-READY (drawn 2026-06-10, REDRAWN around the wat capability after a reach-stumble
correction). Four-questions-decided (A: span-edit codemod, builder-ratified).** Migrates the 27 dirty
`.wat` files to the faithful surface PRESERVING comments + formatting — **driven in wat**, by giving
wat the one capability it was missing (source-location access). The first *reversible* checkpoint of
Strike 4 (dual-read).

## The reach-stumble correction
The first draft put the codemod in a throwaway **Rust** harness because "spans are Rust-side." That
was working around an absent wat capability — exactly what `feedback_reach_stumble_is_the_signal` +
extirpare forbid (*close the gap, don't nest the workaround*). And it hollowed the "wat writes wat"
proving point (Rust doing the real migration). So the gap gets CLOSED: expose source locations to
wat, and the codemod is **pure wat** — wat's equivalent of Clojure's `rewrite-clj`, a **durable,
foundational** capability (every future formatter / refactor / migration uses it), not a throwaway.

## Why not the naive drive
`read-string`→`fix-form`→`write-forms` is a pure-AST round-trip; comments are trivia, not AST, so it
**deletes every comment** (confirmed). The dirty stdlib carries 2,000+ doc-lines (`test.wat` 668,
`stream.wat` 236, `core.wat` 135). Below the bar. A Lisp *reader* drops comments by design; codemods
use a trivia-preserving layer (`rewrite-clj`) that splices into the ORIGINAL text. 4.2 is that layer,
in wat.

## The one substrate addition — `ast-span` (intueri-named)
- **`(ast-span node) → {:line N :col N :file "…"}`** — a plain map (keyword access `(:line loc)`),
  the node's source START location. Rhymes with `ast-kind`/`ast-name` (property-read of a node; a
  plain hyphen, NOT `ast->span` which would imply a structural conversion). Mirrors the existing Rust
  `Span { file, line, col }` (`src/span.rs:48`); the map's `:line`/`:col` keys self-describe it as a
  point. The ONLY new Rust verb — line/col already live in the AST, just unexposed.
- A leaf's char-EXTENT is `ast-span` start + `char-len(ast-name)` — wat computes it from the token
  text it already has (`ast-name`). No range type needed; the codemod edits leaves (+ strip-if
  deletes leaf tokens), all of which have a known text length.

## ⚠️ CORRECTION (2026-06-11) — `:file` omitted, and the first reason for it was WRONG

> Append-only record: we wrote the rationale one way, built it another, and the build's *stated reason*
> was itself wrong. Keeping all three visible so the knowledge isn't lost (don't hide the fault — learn).

**As designed (the bullet above, unedited):** `ast-span → {:line :col :file "…"}` — file *included*.
**As shipped (4.2a, `2a7dacff`):** `ast-span → {:line :col}` — file *omitted*, and the code comment
(`edn_shim.rs`) justified the omission as *"a mixed-value map is un-typeable in wat's ADT model."*

**The builder caught that justification, and it is wrong.** The file is not unknowable, and the typing
wall is not the real reason. Traced to the disk:
- `parser::parse_all_with_file(src, file)` (`src/parser.rs:164`) **already accepts a real file label** —
  the slot exists.
- `(:wat::io::read-file path)` (`src/io.rs:1337`) **HAS the path**; it returns content-only, discarding it.
- `(:wat::core::read-string str)` has **no file param** (faithful to Clojure's reader, which has none),
  so it passes the constant `"<read-string>"`.
- The path therefore dies at the **read-file → read-string seam** — *discarded*, not *unknowable*.

**The honest reason to omit `:file` (the one that actually holds):** this codemod drives exactly ONE file
at a time; the driver holds `path` in scope and threads it itself, so `ast-span` reporting file would hand
back what the consumer already has. **Omitted because the single-file consumer owns its path — NOT because
it can't be typed.** (If it were always-one-file *and* the reason were honest, the omission would be
unremarkable; the fault was the false reason, not the omission.)

**Banked capability — absent + named, NOT can't-know:** a file-aware read (read-file → forms carrying the
real path through the existing `parse_all_with_file` slot) + `ast-span` returning a **`Span` RECORD** (a
product type — file/line/col each typed: the ADT-correct shape, not a homogeneous `HashMap<keyword,i64>`).
The *map* shape is what forced the false "un-typeable" framing in the first place; a record dissolves it.
Build it when a **multi-file** refactor / error-report tool needs cross-file provenance — see the 4.2
out-of-scope bank below.

**The lesson:** a *"we can't know X"* claim deserves the same disconfirming scrutiny as a *"X exists"*
claim. Here a discarded parameter got rationalized as a law of nature. Trace the param to its source
before declaring a value unknowable. (Sibling of `feedback_absence_claim_needs_all_forms`.)

## The codemod, in wat (reuses everything that exists)
Per dirty `.wat` file:
1. `read-file` → the source **text**; `read-string` → the **tree** (nodes carry spans).
2. Walk the tree with `fix-form` (the decisions — reused), collecting **edits**: where a leaf changes,
   an edit `{loc (ast-span leaf), old-len (char-len (ast-name leaf)), new-text (render of the fixed
   leaf)}`; `strip-if` → deletion edits over the `->`+type leaf spans (the only arity-change).
3. **line/col → flat char-offset:** precompute line-start offsets from the text (`split` on `"\n"`,
   cumulative `length`); `offset = line-start(line) + (col - 1)`.
4. **Splice** each edit: `(concat (subs text 0 off) new-text (subs text (+ off old-len) (length text)))`,
   applied **right-to-left** (highest offset first, so earlier offsets stay valid).
5. `write-file` the result. Comments / blank lines / formatting between edited tokens: untouched.

String vocab: `subs` (char-indexed), `split`, `concat`, `length` exist. **Build-feasibility check
first:** does wat have `reverse`/`sort` for right-to-left application (or build the edit list in
reverse)? If absent → reach-stumble, flag it (do NOT nest a workaround).

## The gate (FM-2-bis probe FIRST)
- Probe: a fixture `.wat` with a `;; comment`, an annotated-if, a binder arrow, a typealias core-scalar
  target → run the codemod → assert (a) every `;;` line survives byte-identical, (b) the tokens
  migrated, (c) re-running is a no-op (idempotent: faithful forms produce zero edits). RED before
  `ast-span` + the codemod exist.
- `cargo test --release --workspace --no-fail-fast` GREEN — migrated `.wat` still load + run
  (dual-read; reversible checkpoint).
- Comment-preservation on the real corpus: `core.wat`'s 135 `;;` lines survive byte-identical; the
  `git diff` shows ONLY migrated tokens — eyeball-auditable minimal diff.

## Reusability (the payoff)
This wat codemod IS the engine for 4.3 (the 267 rust-test-strings): extract each embedded wat string →
same codemod (`ast-span` works on the extracted source) → splice back. And it is permanent tooling —
wat can now refactor wat. The "wat writes wat" proving point is whole: wat migrates its own corpus,
comments and all, in wat.

## Risks
- **strip-if span identification** — the deletion edits must cover the `->`+type leaf extents exactly
  (incl. inter-token whitespace if the close must collapse). Probe it.
- **char vs byte** — `col` is char-count; `subs` is char-indexed; stay in char-space throughout (the
  corpus has multi-byte `∀`/`→` in comments — never touched, but the offset math must be char-based).
- **right-to-left application** — verify wat can reverse/sort the edit list (build-feasibility above).
- **idempotence** — a second pass must yield zero edits.

## Out of scope
- 4.3 (rust-strings — reuses this engine), 4.4 (hard-cuts). `ast-span` is a permanent verb (NOT
  throwaway — it's foundational tooling); the codemod *driver* retires with the hard-cut, the
  *capability* stays.
- **File-aware source provenance (`Span` record + path-carrying read)** — see the CORRECTION above.
  `ast-span` reports `{:line :col}` only because read-string nodes carry the placeholder
  `"<read-string>"`, and the single-file codemod owns its path. A multi-file refactor / error-report
  tool would need: (a) a path-aware read that threads the real path through `parse_all_with_file`, and
  (b) `ast-span` returning a `Span` RECORD (product type, file/line/col typed). Threadable today (the
  parser slot exists); built when a cross-file consumer surfaces. Affirmative cut, not a deferral.
