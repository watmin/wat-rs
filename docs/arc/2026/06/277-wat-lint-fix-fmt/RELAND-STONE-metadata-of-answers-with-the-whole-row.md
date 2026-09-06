# RELAND — STONE: `metadata-of` answers with the whole row

**The stone is right and I would accept the substance.** The round trip closes on both branches, the
vectors are populated, `:ret` is the pair, `:yields`/`:syntax` are correctly absent, the negative
controls are real, and the refute was taken cleanly — the three `.wat` annotations are back to
`:wat::core::Value` and this stone's `.wat` carries zero `HolonAST`.

## ⛔ TWO LINT REDS, BOTH IN THE STONE'S OWN NEW TEST FILE

```
FAIL wat::lint no_inlined_edn::tests_carry_no_inlined_edn
     Offenders:  tests/reflection/probe_stone_metadata_of_whole_row.rs:227

FAIL wat::lint no_loose_string_assert::tests_carry_no_loose_string_assert
     Offenders:  tests/reflection/probe_stone_metadata_of_whole_row.rs:192
                 tests/reflection/probe_stone_metadata_of_whole_row.rs:253
                 tests/reflection/probe_stone_metadata_of_whole_row.rs:257
                 tests/reflection/probe_stone_metadata_of_whole_row.rs:261
```

Both diagnostics carry their own prescription; follow them rather than this file:

- **`no_inlined_edn`** — move the literal into a co-located pretty-printed
  `probe_stone_metadata_of_whole_row__<label>.edn`, `include_str!` it, compare with
  `wat::assert_edn_eq!`. The rune is an *"EXTREMELY hard bar"* and this is not it.
- **`no_loose_string_assert`** — four `contains`/`starts_with`/`ends_with` where an exact
  `assert_eq!` belongs. *"A loose check passes on reordered fields."*

★ **These are exactly the wrong lints to rune around**, because this stone's whole subject is a
map whose SHAPE is the claim. A `contains` on a rendered row passes when `:args` reorders, when a
key is dropped, when `:ret` loses a half — the failures rows 3, 5 and 6 exist to catch.

## STOP TRIGGERS

- **STOP-1 — do not rune either lint.** Both bars are hard and neither is earned here. If a literal
  genuinely cannot be a file, **STOP and report** rather than writing the rune.
- **STOP-2 — the exact compares must not weaken the rows.** Row 5 asserts `:args` has FIVE entries;
  an `assert_eq!` on a whole structure is stronger than a `contains`, not a substitute for the count.
- **STOP-3 — nothing else changes.** The substance is accepted; this is the two lints and nothing
  more.

## ★ AND TWO CENSUS CORRECTIONS THE SCORE MADE THAT ARE MINE, RECORDED HERE

- **`yields` IS a field.** The DESIGN said *"`IntrinsicEntry` has no `yields` field."* It is at
  `src/intrinsic/mod.rs:510`; my enumeration windowed `NR>=415 && NR<=490` and **clipped the
  struct**, then read the absence as a fact. So the carried-but-unput count is **eight**, not seven
  — and I made that error *inside the correction that raised five to seven*.
  `[[feedback_a_truncating_pager_makes_absence_unfalsifiable]]`
- **The `HolonAST` count is 6, not 9.** My refute's pattern was looser than the site. The strike's
  6-live figure is the right one; the surviving `.wat` annotations and the wall's Rust-only scope
  remain real, separate work.

Both are recorded because the SCORE's row 8 is right and the DESIGN it disagreed with is mine.
