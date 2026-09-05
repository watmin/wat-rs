# PARKED — the doc-comment migration waits on wat-fmt (arc 277). This is WHY, not an abandonment.

> **Builder, 2026-09-05:** *"we are parking 255 to go work on 277 ..... we build the tooling such
> that we can render example exprs in metadata-maps beautifully..... it also means we can observe
> when they violate desired form."*

⛔ **A next self will find the corpus MID-MIGRATION — one row converted, 575 not — and must not
read that as unfinished work to resume.** It is a deliberate stop with a named dependency.

## WHAT IS DONE AND GREEN

```
the doctest gate       ARMED at zero, in scripts/floor.sh, runs FIRST and unconditionally
the holon wall         ARMED at zero, sabotage-proven twice
#wat.doc/Row           the form is REAL. src/intrinsic/char.rs is the one converted row.
the round-trip gate    read(print(doc)) == doc, over char · hologram · map · rest-arg ·
                       a constructed @deprecated
the heredoc            string-local docstring margin; an indented code sample survives
the EDN keyword bug    edn::write no longer emits what edn::read refuses
```

## ⛔ WHY IT STOPPED HERE

The sweep would print **609 examples — median 67 columns, p90 188, max 1515 — all on one line**,
because the `@example` grammar is line-oriented and forbids breaking them. Migrating without a
formatter bakes every one of those into the new form, and the formatter then has to rewrite all of
them. **Do not sweep before the thing that would make you re-sweep exists** — the same lesson the
doctest gate and the round-trip gate each cost a cycle to learn.

★ And the builder's second reason is the one that makes it more than cosmetics: once a canonical
renderer exists, **an example that violates the desired form becomes a FINDING.** The examples stop
being prose nobody checks and become lintable — which is this whole effort's thesis applied to the
last unguarded surface.

## WHAT STILL BLOCKS THE SWEEP, besides the formatter

```
87 of 576 rows CANNOT take the new form.  #[wat_special_form] routes through
  parse_special_form -> DocSpecialForm, which has NO from_metadata-equivalent.
  Of those: 52 carry @alias, 36 carry @syntax.
  #wat.doc/Alias is therefore designed and NOT WRITABLE.

the @-form ratchet is not built.  Until it is, a new row can still be written in the old form
  behind the sweep.

from_metadata never enumerates keys (13 targeted metadata_lookup calls, no key walk), so an
  unknown key is SILENT — in the EDN form AND in the wat-side maps. A printer never emits a
  stray key, so the sweep will not trip it; that is exactly why it should be closed on its own
  merits rather than discovered later by a hand-written row.
```

## HOW TO RESUME

Read `[[DESIGN-the-registry-prints-its-own-replacement]]` and
`[[DESIGN-the-tagged-edn-doc-row]]`, then this file's blockers. The order is unchanged:

```
1  wat-fmt exists (arc 277)                    ← the current work
2  DocSpecialForm gets a metadata-map reader    ← unblocks 87 rows and #wat.doc/Alias
3  the @-form ratchet, shrink-only, day one
4  the sweep — ONE pass, all 576
```

The registry onslaught itself is separately parked at
`[[RESUME-the-registry-is-blocked-on-three-named-decisions]]` — three DECISIONS, not labour.
