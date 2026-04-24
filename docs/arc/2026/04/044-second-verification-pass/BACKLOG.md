# Arc 044 — BACKLOG

Two slices.

---

## Slice 1 — Seven drift fixes across three files

**Status: ready.**

- `wat-macros/src/lib.rs:379, 387` — `wat::test_suite!` →
  `wat::test!`.
- `examples/with-loader/wat-tests/test-loader.wat:2` —
  `wat::test_suite!` → `wat::test!`.
- `examples/with-loader/wat-tests/helpers.wat:3` — same.
- `wat/std/test.wat:144, 199, 292` — drop `set-dims!` lines from
  the three usage-comment example blocks (they're showing users
  how to write a test entry; under arc 037 the entry doesn't need
  set-dims!).

Five Edit calls (or fewer if multiple sites share a contiguous
block).

After the slice: re-run the surface-audit grep with broadened
patterns. If anything else surfaces, open Slice 1.5 for it; do
NOT silently bundle.

## Slice 2 — INSCRIPTION + cross-references

**Status: obvious in shape.**

- `INSCRIPTION.md` capturing the seven fixes plus the
  iterative-verification observation.
- `docs/README.md` arc index extended.
- 058 FOUNDATION-CHANGELOG row in lab repo.
