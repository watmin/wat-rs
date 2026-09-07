# EXPECTATIONS — 296 H-2: a variant is a tagged map

Written BEFORE the strike. Every row's bar is derived from a control that already exists on disk —
a record's rendering, the committed probe, or H-2a's proven regen method — never from what I expect
the output to look like.

⚠ **A PASS IS THE DEFECT'S OWN SIGNATURE HERE.** Today a record and a variant render byte-identical
tags, so any row that merely asserts "the variant renders something sensible" is satisfied by the
bug. Rows 1–3 are written against the record CONTROL for that reason.

| # | what | command | expected |
|---|---|---|---|
| 1 | a variant's tag is not a record's | `cargo nextest run --release -E 'test(probe_arc296_h2)'` | `a_variant_tag_is_not_a_records_tag` PASSES (RED at HEAD: both `#usr.Shape/Circle`) |
| 2 | a variant's body is EXACTLY a record's, for the same declared field | same | `a_variant_body_is_rendered_exactly_like_a_records` PASSES (RED at HEAD: `[2]` vs `{:r 2}`) |
| 3 | the probe's `#[ignore]`s are GONE | `grep -c '#\[ignore' tests/types/probe_arc296_h2_variant_tag_and_body.rs` | `0` — the stone un-ignores its own rows; leaving them skipped is a false green |
| 4 | a unit variant is `{}` | a fixture printing `(:usr::Shape::Dot)` | `#usr/Shape.Dot {}` — an empty MAP, not `[]` |
| 5 | ⛔ records did not move | `git diff --stat` over the `.edn` goldens, filtered to record tags | ZERO record tags changed. STOP-1 fires if any did |
| 6 | round-trip: written → read → same value | the enum coerce's own tests | green; unit arm accepts `{}` and REFUSES `[]` (STOP-5: one wire, not two) |
| 7 | the 395 goldens regenerate and are DATA-EQUAL | `UPDATE_EDN=1` over the suite, then a standalone `wat-edn` comparator over `git show HEAD:<path>` vs post-image, per file | every regenerated golden DATA_EQUAL — formatting normalised, not one datum moved. H-2a's exact method, already exercised at 208 sites |
| 8 | the 33 `.wat` sites moved BY CODEMOD | `git log -p` on the fix + a `/tmp` dry-run diff | a `wat-scripts/fixes/*.wat` exists and is committed; no hand edit to a `.wat` |
| 9 | foreign variant keeps its keys | a foreign round-trip probe | keys survive read→write. STOP-2 fires if `ForeignVariantValue` cannot hold them |
| 10 | the floor | `./scripts/floor.sh` unpiped, read the Summary line | `5206+ passed, 0 failed` (23 skipped MINUS the 2 this stone un-ignores) |
| 11 | clippy | `cargo clippy --release --all-targets --workspace` | 0 errors |

## RUNTIME PREDICTION

**Rust half 25–40 min.** Seven rooms, one of them (`ForeignVariantValue`) a struct field plus its
constructors. **Regen + prove 15–25 min** — mechanical but the per-file data-equal proof is the
expensive part and is NOT optional. **Codemod 20–30 min** including the mandatory `/tmp` dry-run
diff. **Total 60–95 min.**

## TRAP DOORS — named before the strike

- **The regen can go green while silently normalising a real change.** That is exactly why row 7
  demands the per-file DATA-EQUAL proof rather than a green floor. H-2a's own SCORE says it: *"the
  gate, and it is the point — converting 208 sites without firing the regen would leave us believing
  in a capability proven on one file, the same shape as the defect being fixed."*
- **`tag_from_type_path` is shared by records, structs, capabilities and foreign values (13
  callers).** The cheapest-looking edit — make it dotted — moves all of them. Row 5 is the guard.
- **The unit-variant `[]` wall (`render.rs:3065`, arc 278 A.0) will go red, correctly.** It is not
  an obstacle; it is the old wire refusing the new one. Moving it is the work. Adding a second arm
  that accepts both is the failure (STOP-5).
- **H's design references a file that no longer exists** — `edn_shim.rs:2727` is now
  `src/edn/render.rs`, and the `enum_variant_field_names` re-derivation it describes is ALREADY GONE
  (G′ carries the names). Do not go looking for work that has landed.
- **The design's migration numbers are stale.** It says 213 across 103 files, measured 2026-08-15.
  Re-measured 2026-09-06: **539 across 351**. Size against the new number.
- **A green floor does not prove the migration finished.** If the reader accepts both the old vector
  body and the new map body, every golden passes and nothing moved. Row 6's REFUSES clause is what
  makes the floor mean something.
