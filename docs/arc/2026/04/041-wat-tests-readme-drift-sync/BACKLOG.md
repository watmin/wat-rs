# Arc 041 — BACKLOG

Two slices total — the file is small enough that one
implementation slice covers everything plus a wrap-up.

---

## Slice 1 — §Layout external-crate addition + §In-process user-facing layer

**Status: ready.**

Two additions:

- **§Layout.** Add a paragraph (or table row) noting that consumer
  crates ship their own `wat-tests/` tree and discover/run via
  `cargo test -p <crate>` through the same `wat::test! {}` shape.
  Reference `crates/wat-lru/wat-tests/` as the workspace-internal
  example (arcs 013 + 036) and `examples/with-loader/` as the
  consumer-binary template.
- **§In-process vs hermetic.** Add a short paragraph noting that
  the user-facing path is the `deftest` / `deftest-hermetic` /
  `make-deftest` macro family (arc 029 + 031), and the
  `:wat::test::run` / `run-hermetic-ast` primitives this section
  describes are what those macros expand to. Reader knows which
  layer to start at.

## Slice 2 — INSCRIPTION + cross-references

**Status: obvious in shape.**

- `INSCRIPTION.md`.
- `docs/README.md` arc index.
- 058 FOUNDATION-CHANGELOG row (lab repo).

---

## Cross-cutting

- Verification: grep audit + spot-read after Slice 1.
- Commit per slice.
