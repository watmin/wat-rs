# DESIGN — Strike 4: the corpus drive + hard-cuts (251.5b → 251.5d)

**Status: PLAN drawn 2026-06-10 (grounded surface counts). The keystone leg of arc 251 — the
irreversible one. Executes the 251.5 master decomposition (`DESIGN-STONE-251.5.md`) with the
current reality folded in.** The fixer spine (251.5a — `read-string`/`write-forms`/`ast->children`/
`with-children`/`ast-kind`/… + `fix-source`'s position-aware `fix-seq`) is BUILT + green. Generics
(251.7 `0c95ae2c` + 256 `96968bf4`) are SHIPPED, which newly unblocks dropping the `<T,U>` suffix.

## Surface (grounded 2026-06-10)
- **72 `.wat` files; 27 carry dirty rust-scheme forms.**
- **267 rust test files** embed wat as string literals (non-uniform wrappers — `startup_from_source`,
  `parse_one!`, `eval_in_frozen`, …).
- **351 `<-`, 338 `->`, 35 `<T,U>`** across `.wat`.

## The forced ordering (why hard-cuts are LAST)
The dual-read (251.1–251.4: every faithful surface reads ALONGSIDE its legacy spelling) is what lets
a half-migrated corpus keep running. The hard-cuts DELETE the legacy reader. Therefore **every byte
of corpus migration (4.2 + 4.3) must land green BEFORE any hard-cut (4.4).** 4.4 is the one-way door,
opened only when everything green sits behind it. Slow is smooth.

```
4.1 complete-the-transform  →  4.2 drive .wat (reversible checkpoint)  →  4.3 rust-strings  →  4.4 HARD-CUTS
```

## Slice 4.1 — complete the transform: grammar-bearing migrator + `<T,U>`-drop
`fix.wat` is deliberately **grammar-free** (local, position-from-the-walk rules). It SKIPS the slots
that require knowing wat's declaration grammar: declaration type-slots (`typealias`/`newtype`/
`typeunion`/`struct`/`enum` — the name `<T>` + field/target types) and ctor type-args. Per the four-Q
keystone decision, that grammar is NOT re-encoded in perpetual `fix.wat` (it lives once, in the
resolver/parser; duplicating it = the 251.1 keystone braid). So a **separate, non-blessed, one-time
migrator** handles those slots — it carries the bounded "which arg is a type" knowledge and RETIRES
with the hard-cut, so it leaves no perpetual drift. Folds in the **`<T,U>`→bare-var-signature drop**
(now safe: 251.7/256 generalize free signature vars; `(defn :map<T,U> [x <- :Stream<T>] …)` →
`(defn map [x :- (wat.type/Stream T)] …)`). Probe-gated per declaration kind. **Must precede the
drive** — otherwise declarations migrate half-way and the hard-cut breaks them.

## Slice 4.2 — drive the `.wat` corpus (27 dirty of 72) — COMMENT-FAITHFUL (span-edit codemod)
**DESIGN: `DESIGN-STONE-251.5-4.2-comment-faithful-drive.md` (STRIKE-READY).** The naive
`read-string`→`fix-form`→`write-forms` drive DELETES all comments (pure-AST round-trip; trivia isn't
AST) — disqualifying given 2,000+ stdlib doc-lines. Four-Q-decided (A, builder-ratified): a **span-edit
codemod** (the `rewrite-clj` pattern). REDRAWN after a reach-stumble correction (the first draft's Rust
harness was working around an absent wat capability — extirpare/reach-stumble forbid that): the codemod
is **driven in wat** by closing the gap — one new substrate verb **`ast-span`** (intueri-named: node →
`{:line :col :file}` plain map; the only missing piece, line/col already in the AST). The wat codemod
then reuses everything — `read-file`+`read-string`+`fix-form` (decisions)+`ast-span` (locations)+
`subs`/`split`/`concat` (splice into the ORIGINAL text, right-to-left) → minimal diff, comments
preserved. This is **wat's `rewrite-clj`** — durable foundational tooling, the engine for 4.3 too, and
the "wat writes wat" proving point made whole. `ast-span` is PERMANENT (not throwaway); the codemod
driver retires at the hard-cut. Gate: FM-2-bis probe (comments survive byte-identical + idempotent) →
`--workspace --no-fail-fast` green.

## Slice 4.3 — the Rust-test-string corpus (267 files — the hard part)

The embedded wat (~7k lines across 267 files) is NOT one thing. The insight: **extracting full-program
strings to `.wat` fixtures collapses the fragile rust-string adapter into the uniform 4.2 `.wat`
drive** (the fixer already handles `.wat`) AND is a permanent surface-reduction (embedded-wat-as-
escaped-Rust-string is itself a smell; a `.wat` file is the honest home). But it only applies to FULL
PROGRAMS — fragments and intentionally-dirty inputs don't extract. So 4.3 begins with a **triage**.

**The triage mechanism = a bespoke RUNE (intueri names it at 4.3-open).** The marks I'd reached for
are structurally runes (`// rune:<spell>(<category>) — <reason>`), so use the rune discipline, don't
invent a comment grammar:
- **Grammar:** `// rune:<name>(<class>) — <reason>` ; **reason REQUIRED** (a rune without a reason
  fails — this forces the judgment onto the page, which matters most for the dangerous class).
- **Region-granular** (bracket the embedded string), NOT file-level — one test file holds several
  embedded strings of different classes.
- **Classes (the triage buckets):**
  - `program → fixtures/X.wat` — full program → EXTRACT to a `.wat` fixture, point the test at it;
    it then rides the 4.2 drive. (transient rune — the extractor consumes it.)
  - `fragment` — a single form / sub-program (`parse_one!`, `eval_in_frozen`) → migrate IN-PLACE
    (not a standalone file). (transient.)
  - `inline` — a tiny full program where inline beats a fixture-file indirection → migrate in-place.
    (transient; size is the discriminator vs `program`.)
  - `dirty-keep — <why>` — an intentionally-malformed/legacy input a test feeds to assert REJECTION
    → **DO NOT migrate** (migrating it could flip a should-fail test to passing). **The silent-hole
    class.** (may EARN permanence — a standing justification, like an excusare override.)

**Pipeline:** fan-out sonnet **MARKS** (writes runes, reason mandatory; 267 files = a clean fan-out,
disjoint slices, same embedded rubric) → **EXCUSARE-grade the runes** (the audit IS excusare — weigh
each reason; scrutinize every `dirty-keep` AND every un-marked dirty string hardest) → a
**DETERMINISTIC throwaway script** extracts/migrates by class (NOT a 2nd sonnet — mechanics, no
judgment in the 267-file pass; the script parses the existing rune grammar) → full-suite gate.

The judgment (mark) is separated from the irreversible mechanics (extract) by an auditable artifact
(the runes) — the examinare weigh, applied to 267 files at once.

**Settle BEFORE building 4.3:** EDN-write reflows formatting/whitespace — grep for tests asserting on
exact source text (a `dirty-keep`-adjacent risk). Empirically ground the program/fragment/dirty split
(my ~424 full-program / ~425 fragment estimate is from wrapper COUNTS, not inspection) before sizing.

## Slice 4.4 — the HARD-CUTS (irreversible, LAST)
Only after 4.1–4.3 are all green. Each cut gated; this is where dual-read dies:
- delete `<-` / `->` annotation arrows (binder + return);
- delete the keyword type spellings (`:wat::core::i64` as a TYPE; the `:i64` bare-legacy);
- delete the `<>` `angle_depth` lexer machinery (`lexer.rs:637–730`);
- delete the `:wat::core::Fn(…)->…` parser path;
- delete the `<T,U>` suffix parser (`split_name_and_type_params`, `runtime.rs:2634` — redundant once
  generics are inline and the suffix is dropped corpus-wide);
- flip the internal canonical `:wat::core::`→`:wat::type::`.

## After Strike 4
251.6 (separate): native symbol dispatch + ANNIHILATE `src/resolve/normalize.rs`. Then 251.N
inscription. Banked tangent (NOT on this spine): bounded polymorphism / intrinsic retirement
(see `docs/arc/2026/06/256-generic-defclause/DESIGN.md` out-of-scope — parked).

## Risks ledger
- **4.3 is the fragile one** (builder-flagged) — non-uniform wrappers + escaping + 267 files.
- **Formatting reflow** — acceptable for `.wat` (data) + embedded strings, BUT verify no exact-text
  assertions before 4.3.
- **The one-way door** — 4.4 cannot be partially done; a single un-migrated dirty form anywhere
  becomes a hard error the moment its reader is deleted. The full-suite gate after 4.2 + 4.3 is the
  proof that the door is safe to open.
