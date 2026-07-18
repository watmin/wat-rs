# DESIGN-STONE — `no_inlined_edn` lint + the EDN-golden conversion campaign

> **Origin (builder, this session):** it emerged from the RST `.edn`-golden fix — *"its doing a string equality
> instead of an edn equality… lints should be forcing us to .edn files paired with tests."* The sibling of
> `no_inlined_wat`: don't inline EDN in `.rs` string literals; EDN lives in a co-located `.edn` file loaded via
> `include_str!` + compared via `wat::assert_edn_eq!` (structural, not string-eq). **Its own lint** (builder #3):
> *"string literals which contain edn /must be/ loaded from .edn files."*

## The rule (builder-pinned)
A string literal that is **EDN-esque** — content, after trimming leading whitespace, opens with `#` / `{` / `[`
/ `(` — must be an `.edn` file, not inline. *"if a string looks like edn at the start, it must be edn — no
exceptions — non-edn things must tweak their MATCH CONDITIONS"* (i.e. tighten the DETECTOR so genuine non-EDN
doesn't fire — NOT restructure hundreds of call sites), and *"any attempted runes here must meet an extremely
hard bar."* The `.edn` goldens are **pretty-printed** (multi-line, indented — the reviewed-artifact convention;
`assert_edn_eq!` ignores whitespace but humans don't).

## State — BUILT, UNCOMMITTED in the tree, RED (not yet committed — the detector over-fires)
`tests/lint/no_inlined_edn.rs` exists (auto-discovered into `wat::lint` via `build.rs`; mirrors
`no_inlined_wat`/`no_loose_string_assert` architecture: `collect_rs` walk, line scan, FAIL-listing offenders,
`// rune:lint(no-inlined-edn) — <reason>` file-scoped exemption). Currently **RED at 1306** (raw 1653 → 1306
after two structural tightenings the rider added: `is_lone_delimiter` [a bare unmatched `#`/`{`/`[`/`(`],
`is_bare_format_scaffold` [strips non-nested `{…}`; excludes if nothing survives — `"{}"`, `"{:?}"`, `"{e}"`]).
Breakdown: **517 `#` · 360 `{` · 122 `[` · 307 `(`** (a further 878 `(`-openers already excluded as genuine
wat call-forms — the wat/EDN boundary is grounded: it re-derives `no_inlined_wat`'s `is_inline_wat_form` reader
verdict and skips wat, so the two lints are COMPLEMENTARY, never double-flagging).

## ★ FAR-SIDE TASK #1 — annihilate the false positives (more ignore-conditions)
The 1306 is **~90% false positives**, especially the 506 in `src/` — GROUNDED samples:
- `src/check.rs:3209` `format!("{}::{}", enum_path, variant_name)` → builds `"Enum::variant"` (FQDN glue, NOT EDN)
- `src/check.rs:4093` `param: "#1".into()` / `:6441` `format!("#{}", i+1)` → positional `#N` markers (NOT EDN)
- `src/runtime.rs:721` `format!("{}/{}", …)` / `:1166` `format!("{}'", agg.name)` → `"Type/method"`, `"Foo'"` (glue, NOT EDN)

These START EDN-esque but are NOT EDN — you cannot "restructure" `format!("{}::{}")` (the checker's FQDN builder,
hundreds of sites). Per the builder's rule, **tighten the DETECTOR**. Ignore-conditions to add (find more as they
surface):
1. **`#` is an EDN tag only if followed by a letter / `{` / `_`** (a tag name like `#wat.…` or a `#{` set) —
   exclude `#`+digit and `#{}`-format-placeholder (`"#1"`, `"#{}"`, `format!("#{}", …)`).
2. **Exclude when the format-stripped residue is only identifier punctuation** (`::`, `/`, `'`, `-`, `/0`, `@`) —
   that is glue, not EDN structure. (The current `is_bare_format_scaffold` only excludes when NOTHING survives;
   `"{}::{}"` → `"::"` survives → wrongly flagged. Extend it: residue of pure-glue-punctuation ⇒ not EDN.)
3. Likely more: diagnostic/`panic!` templates with placeholders; sentinel strings. Sample the residual list and
   pull each false-positive CLASS out by the root (a detector predicate), never one-off runes.
Re-run after each tightening; the count should collapse toward the GENUINE-EDN offenders — overwhelmingly the
**`tests/` goldens** (`#wat.check/CheckErrors {…}`-style, real expected values). THAT is the real worklist.

## ★ FAR-SIDE TASK #2 — the carve-out
`crates/wat-edn/tests/*` (`comprehensive.rs`, `spec_conformance.rs`, `spec_strict.rs`, `round_trip.rs`,
`accessors.rs`, `pretty.rs`) inline tiny EDN literals as the **input under test** (`parse("()")`,
`parse("#{1 2 3}")`) — this is the EDN reader/writer's OWN test corpus, the exact parser-test carve-out
`no_inlined_wat` already grants. File-scoped rune, justified.

## ★ FAR-SIDE TASK #3 — the conversion campaign (only AFTER the detector is honest)
Once `no_inlined_edn` fires on GENUINE EDN only, run the drive-to-zero (same shape as the `no_inlined_wat`
crusade): every offender → a pretty-printed co-located `<probe>__<label>.edn` golden + `assert_edn_eq!(actual,
include_str!("…edn"))`. **Edit-only riders (forbid cargo) + central weigh (FM 18)**; runes at the extremely-hard
bar (only genuine-EDN-that-truly-can't-be-a-file, justify hard). Commit the tightened lint + the conversions
together (or the lint first once it's honest-red, then the campaign — builder's call).

## Regenerating the worklist
Do NOT rely on the scratchpad lists (`edn_offenders_v2.txt` — session-specific). Re-run the lint:
`cargo nextest run --release -E 'binary_id(wat::lint)'` → `no_inlined_edn` prints every `file:line`. That IS the
live list.

## What is DONE (do not redo)
The lint is written + the wat/EDN boundary grounded + the two initial tightenings landed + the 1306 measured +
the false-positive classes diagnosed (above). The RST that motivated this (`RecvError::PeerCrashed`) is
committed + pushed (`f0230bbc`) with its own `.edn` golden as the exemplar of the target form.
