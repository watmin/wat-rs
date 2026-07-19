# BRIEF — the `no_inlined_edn` crusade (drive to zero)

> The shared steel every rider carries (R39 `VNA CAEDE PROBATA, FRATRES MITTIMVS`). One rider per
> `tests/<dir>`. **EDIT-ONLY: never run cargo / nextest / any build** — the orchestrator weighs
> centrally (FM 18: N riders each running cargo = target/ lock thrash).

## The task

In your assigned `tests/<dir>/`, drive the `no_inlined_edn` lint to zero: every EDN-esque string
literal it flags becomes **either** a co-located pretty-printed `.edn` golden **or** a per-offense
rune, per the decision-tree. The detector (`tests/lint/no_inlined_edn.rs`) already excludes every
non-EDN false positive (format messages, prose, output/search positions), so **every literal it
still flags in your dir is GENUINE EDN** and falls into exactly one bucket below.

To see your dir's offenders, read the offender list handed to you in your prompt (file:line each).
Do NOT run the lint yourself.

## Decision-tree (apply per offending literal)

**① STRUCTURAL GOLDEN** — an EXPECTED VALUE compared against some `actual` (`assert_eq!(rendered,
"#wat.check/CheckErrors {…}")`, a raw-string `r#"#wat…"#` golden, a bare `{:…}`/`[…]` expected
value) whose correctness does NOT depend on exact whitespace/formatting.
→ **Convert to a co-located pretty-printed `.edn` file:**
  1. Create `tests/<dir>/<probe-basename>__<label>.edn` — basename = the `.rs` file stem; label = a
     short snake_case name from the assert message / the value's role (`__unbound_symbol.edn`,
     `__two_type_errors.edn`). Multiple goldens in one file → distinct labels.
  2. Write the golden **PRETTY-PRINTED**: multi-line, 2-space indent (see the exemplar). NOT the
     single-line inline form. (Builder rule: "the .edn files must be pretty printed.")
  3. Replace the inline literal with `include_str!("<probe>__<label>.edn")` and make the assertion
     `wat::assert_edn_eq!(<actual>, include_str!("…edn") [, "<existing msg>"])`.
     - `<actual>` must be a `String`. If it's `&str`, append `.to_string()`; if already `String`, use
       as-is.
     - `assert_edn_eq!` is whitespace-BLIND (parses both sides as EDN), so the `.edn` file's trailing
       newline is fine — **no trim needed**.

  **①-doubled — the "doubled-format" golden (ratified in wave 0, the fleet MUST copy this).** Many
  error-golden probes build the actual as `let err = format!("{}\n---\n{:?}", err, err)` (Display +
  `\n---\n` + Debug) compared against a golden holding both faces. **That combined string is NOT one
  valid EDN document** → a single `assert_edn_eq!(combined, golden)` trips STOP-1 ("ACTUAL not valid
  EDN"). Convert to the **two-call** form against ONE golden `.edn`:
  ```rust
  let golden = include_str!("<probe>__<label>.edn");
  wat::assert_edn_eq!(format!("{err}"),   golden, "<msg> (Display)");
  wat::assert_edn_eq!(format!("{err:?}"), golden, "<msg> (Debug)");
  ```
  This preserves both properties the original tested (Display-renders-as-EDN AND Debug-renders-as-EDN
  — arc 296's no-`{:?}`-impostor guarantee) and upgrades to structural comparison. **Verify the two
  faces are byte-identical (same EDN data) before collapsing to one golden**; if they genuinely
  differ, keep two goldens. A single-face golden (`format!("{:?}", err)` only) → one
  `assert_edn_eq!(format!("{err:?}"), include_str!(…), "<msg>")`.

**② GENUINE EDN INPUT** — fed to a reader/parser/eval/lint as INPUT under test (`let edn_src =
"(1 2 3)"; parse(edn_src)`, `parse_one!("[1 2 3]")`, argspec `parse_triples("[x <- :wat::core::i64]")`,
inline wat SOURCE fed to a lint/reader).
→ **Per-offense rune:** add `// rune:lint(no-inlined-edn) — input under test: <what it feeds>` on the
  offending line (trailing) OR the line directly above it (for a multi-line raw string).

**③ EDN-TOOLING EXACT-FORMAT GOLDEN** — an EXPECTED VALUE whose correctness DEPENDS on exact
whitespace/newlines: a pretty-printer's output, a framer's exact bytes (`assert_eq!(pretty,
"{\n  :a 1\n  :b 2\n}", "…exact multi-line form")`).
→ **Per-offense rune:** `// rune:lint(no-inlined-edn) — is the EDN tooling correct: exact-format
  output under test (assert_edn_eq is whitespace-blind)`. Do NOT convert (structural comparison would
  defeat the formatting test).

## STOP-TRIGGERS (never guess — leave the literal untouched, report it)

- You cannot confidently classify a literal (compared golden vs reader input?).
- The golden is NOT a simple `assert_eq!(actual, golden)` convertible to `assert_edn_eq!` — it's in a
  `vec![]`, a `match` arm, a data structure, a multi-value comparison. Report; do not force it.
- The `actual` side can't be made a `String` cleanly.
- A literal looks like a NON-EDN false positive the detector missed (a message/glue/pattern) — report
  it as a suspected detector gap; do NOT rune or convert it.

## Exemplars (already on disk — copy the pattern)

- `.edn` golden (pretty-printed): `tests/services/probe_arc278_rst_peer_notify_baseline__peer_crashed.edn`
- assert idiom: `tests/services/probe_arc278_rst_peer_notify_baseline.rs:31` —
  `wat::assert_edn_eq!(format!("{err}"), include_str!("…edn"), "msg")`
- macro: `src/lib.rs:176` `assert_edn_eq!(actual: String, expected: &str [, msg])` — parses both as
  EDN, compares structurally.

## Output (report back)

For each offending literal: `file:line → converted-to-<name>.edn | runed-input | runed-tooling |
STOPPED-<reason>`. List every `.edn` file created. Do NOT run any build command.
