# BRIEF — 296 H-2b: finish H-2's tail

> H-2's MECHANISM LANDED and is correct. `tests/types/probe_arc296_h2_variant_tag_and_body.rs`
> is **3/3 green with no `#[ignore]`s**: a variant tags `#ns/Enum.Variant`, a record still tags
> `#ns.Enum/Name`, bodies are the same map, a unit variant and a zero-field record share `{}` and
> differ only in the tag. `ForeignVariantValue` carries names (STOP-2, closed). Records did not move.
>
> **The floor is RED at 31.** None of it is the writer. This strike finishes the corpus.

## ⛔ WHAT THE ORCHESTRATOR GOT WRONG — read this first, it is why the tail exists

H-2's EXPECTATIONS row 7 demanded *"every regenerated golden DATA_EQUAL — not one datum moved"*,
copied from H-2a. **That proof cannot hold for this stone.** H-2a was an INERT conversion, so
data-equality was the right invariant. H-2 deliberately CHANGES the wire, so a datum's rendering
must move. The row imported a control from a stone of the opposite nature, and the executor
correctly declined to claim it.

The right invariant, and this strike's contract decision:

> **In a golden, ONLY the variant tag and its body shape may change. LAYOUT MUST NOT MOVE.**

## THE THREE CLASSES — measured 2026-09-06 after the strike

```
1. LAYOUT DRIFT           40 of 311 changed goldens gained lines. The rewriter parses and
                          re-writes, so it pretty-printed what was one line. Harmless where the
                          test compares STRUCTURALLY; fatal where it compares the golden text
                          VERBATIM — e.g. wat_arc144_uniform_reflection::special_form_lookup_
                          define_smoke, whose left/right differ ONLY in whitespace, both carrying
                          the new tag. THIS IS THE BIGGEST CLUSTER AND THE EASIEST TO MISREAD as
                          a tag problem.
2. EDN INSIDE STRINGS     31 occurrences across 18 .wat files — `read-foreign "…"` payloads,
                          assert needles, scratch-pad probes. A form-tree codemod CANNOT see
                          these (the executor named this as STOP-4 and hand-fixed one).
                          UPDATE_EDN cannot either: it regenerates goldens, not string literals.
3. PROSE + NEEDLES        ~49 in 19 .rs files. Measured split: in src/ it is 20 of 24 PROSE
                          (doc comments, @example) — src/comms/mod.rs:522-525 is the only
                          non-prose group and it is a to_wire/from_wire round-trip where the tag
                          is an OPAQUE STRING PAYLOAD, not a rendered value.
```

★ **No live substrate site still mints an old-form tag.** Verified: every `src/` hit outside
`comms/mod.rs`'s test is a comment. The writer is done; this is expectations and documentation.

## READ IN ORDER

```
tests/reflection/wat_arc144_uniform_reflection.rs:147   the LAYOUT-DRIFT exemplar. Its golden went
                                                        one-line → pretty. Read the assertion: it
                                                        compares TEXT.
tests/reflection/wat_arc144_uniform_reflection__special_form.edn   `git show HEAD:<path>` vs now
tests/value/probe_arc278_read_foreign.wat               class 2 exemplar (already hand-fixed once)
src/comms/mod.rs:522                                    class 3's only non-prose site
```

Full file lists for classes 2 and 3 are reproducible in one command each; they are in the SCORE of
this strike, not pre-copied here, because **the orchestrator's hand-built room maps have been wrong
four times this campaign** — derive them, do not trust a list.

## THE WORK

1. **Restore layout on the 40.** For every changed `.edn`, the diff against `git show HEAD:<path>`
   must be the tag and the body shape ONLY. Where the rewriter reflowed, put the layout back.
2. **Class 2 by hand, deliberately.** EDN inside a string is not reachable by the codemod
   (`wat-scripts/fixes/variant-vector-to-tagged-map.wat` correctly cannot see it). Hand edits here
   are the RIGHT tool, not a violation of R21 — R21 governs FORM rewrites.
3. **Class 3:** update prose and `@example`s so the docs stop teaching the dead wire.
4. Leave `src/comms/mod.rs:522-525` alone or update it as prose — it is an opaque payload; say
   which you chose and why.

## STOP TRIGGERS

- **STOP-1 — a golden whose diff vs HEAD is anything but the tag + body shape.** That is the
  contract decision. If a datum moved, report it; do not accept it because the floor went green.
- **STOP-2 — a test that only passes because its golden was reformatted to match.** The fix is the
  golden's layout, not the test's assertion. Changing a verbatim comparison to a structural one is
  H-2a's stone, already done for 208 sites, and is NOT this strike's licence.
- **STOP-3 — a `.wat` FORM (not a string) still carrying an old tag.** That IS codemod work; the
  fix exists. Report it rather than hand-editing.
- **STOP-4 — the H-2 probe's 3 rows go red.** They are green now. A red there means the writer
  moved, which this strike must not touch.

## OUT OF SCOPE, AFFIRMATIVELY

H-3 (Option/Result into wat declarations) · the match arm · the constructor form · the
record-steals-a-variant's-constructor NOTE. All named in H-2's BRIEF and unchanged.
