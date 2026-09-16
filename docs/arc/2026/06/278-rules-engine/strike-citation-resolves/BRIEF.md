# BRIEF — make a cited name resolve, or make it say why it cannot

Two kinds of citation in rete's comments are unchecked: a backticked identifier that names nothing,
and a bare `*.rs` filename that names no file. One walker answers both. Read `DESIGN.md` first — its
⛔ records that **my own first instrument reported zero because it let prose resolve against prose**,
and its ⚠ says the noise vocabularies must be declared rather than hand-filtered.

## Read in order

1. `tests/lint/no_stale_path_in_doc.rs:47` — `tok.contains('/')`, the line that makes a bare filename
   invisible. Decide whether to extend this gate or sit beside it, and say which and why.
2. `src/rete/kernel/mod.rs:4` — *"Tests are `tests.rs`."* The file became `tests/` on 2026-08-30.
3. `tests/lint/rete_names_in_wat_scripts_resolve.rs` — landed two strikes ago. Its
   **comment-stripping is string-literal aware in both directions** and its universe is split by
   namespace so neither half is subsumed. **Both problems recur here; read how it solved them.**
4. `tests/lint/every_walking_gate_declares_non_vacuity.rs` — your gate is in its population.
5. `src/rete/validate/typing.rs:88` and `:283` — two citations rotted by strikes in this same arc
   (`check_field_at`, `keyword_constant_segment`). Fixing them is part of the work.

## The measurement to reproduce and then improve

Mine, for you to beat: backticked identifier-shaped tokens in comments under `src/rete`, resolved
against **code positions only** (comments stripped) across `src` + `crates` + `tests` + `benches` +
`examples` → **732 candidates, 33 unresolved**. The seven the work-list row predicted are all in
there.

**Report your own number and how your classifier differs.** If yours disagrees with mine, yours is
probably right — mine is a throwaway regex and yours is the instrument.

## Traps named in advance — each with its step

1. **★ The resolution universe must exclude comments.** Otherwise a name that appears only in prose
   resolves against itself and the gate reports zero. **Step:** strip comments string-literal-aware,
   as the `wat-scripts` gate does; then drive it — a name you invent and put ONLY in a comment must
   come back unresolved.
2. **★ Too narrow a universe manufactures findings.** Test-only names (`probe_arc278_…`,
   `axis_variant_names_round_trip`) resolve in `tests/`, not `src/`. **Step:** state the universe in
   the gate's header and drive one name from each corner of it.
3. **Noise vocabularies get DECLARED.** clippy lint names, memory slugs, `_`-prefixed placeholders.
   **Step:** a named set per vocabulary with a one-line reason, or a per-site rune. An unexplained
   exclusion is what this class exists to remove.
4. **A rotted citation is not always a rename.** Some may name a type that was deleted, where the fix
   is rewording the sentence. **Step:** for each, decide rename-or-reword and say which; do not
   force a rename that makes the sentence false.
5. **Your gate is a walking gate.** **Step:** it must carry a `NON-VACUITY` declaration with a real
   floor, or the gate landed two strikes ago reds.
6. **`binary_id(wat::lint)` is not clippy.** Last strike's rider was 153/153 green and clippy went
   RED on a new unit test. **Step:** run the lint binary, and keep your test code idiomatic —
   `contains_key` over `get(..).is_none()` was the exact arm.

## STOP triggers

- **STOP-1** — if a flagged name turns out to be correct under a universe you cannot justify in the
  header, STOP and report the shape rather than widening until it goes green. *Widening until the
  finding disappears is how a gate becomes decorative.*
- **STOP-2** — if any currently-green test goes red, STOP and report which.
- **STOP-3** — if extending `no_stale_path_in_doc` would change its existing verdicts, STOP and
  report. A new gate beside it is fine; silently altering a live gate's population is not.

## Shape to copy

`docs/arc/2026/06/278-rules-engine/strike-phantom-rete-names/` — two strikes back, the same
code-vs-prose problem, and its gate solved both traps 1 and 2.

## The one thing worth more than the fix

**Tell me where this brief was thin.** Twenty-three riders before you each returned a prescription of
mine that did not survive contact. The last found that my stated contract was false — a codemod's old
column is code and must name what it removes — and that a naive union would have made half the
resolver decide nothing. If a step here is wrong, unnecessary, or impossible, say it plainly.
