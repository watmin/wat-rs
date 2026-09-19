# EXPECTATIONS — a golden must not pin a Rust line

Scored against `DESIGN.md`. Write `SCORE.md` beside it.

## The null

⛔ **"Normalising cannot distinguish a Rust span from a `.wat` span at comparison time"** is a full
delivery — show why, land nothing. Over-reaching into the 153 `.wat`-span goldens would be far worse
than the tax being removed.

## Rows

| # | what | how it is judged |
|---|---|---|
| 1 | **Rust-only normalisation** | `:line`/`:col` normalised for spans whose `:file` is a Rust path; **`:file` still compared**. ⛔ `.wat` spans untouched. |
| 2 | **The 8 goldens** | All eight updated and green. Name them. |
| 3 | ⭐ **Then unbend `freeze.rs:1313`** | Re-wrap it to rustfmt's preference and show the golden still passes. State the `cargo fmt --check` count for `src/freeze.rs` before and after — it was 7 at HEAD before the engine merge and 8 after. |
| 4 | ⭐ **Control by MUTATION, both directions** | (a) insert a blank line above a pinned Rust span → the golden stays **GREEN**; (b) shift a line in a `.wat` fixture a golden pins → that golden **REDDENS**. ⛔ Both runs shown. (a) alone proves nothing about over-reach. |
| 5 | **The trade-off stated** | A diagnostic moving within the same Rust file no longer reddens. Say it plainly; do not present the change as free. |
| 6 | **Scope wall** | Rust spans still emitted; `span_substitution_justified` and `unused_span_justified` untouched; the 153 `.wat` goldens untouched; no `.wat` edits. |
| 7 | **Floor** | `scripts/floor.sh`, release. Summary verbatim + `.floor/<stamp>/` + the tree it ran against. A red: do not re-run, capture whole, name the arm. Clippy over the WHOLE output. |

## What would make this stone wrong

- **Over-reach into `.wat` spans.** Row 4(b) exists solely for this, and it is the one that matters.
- **Removing the Rust span** instead of un-pinning it — that inverts the doctrine the lint defends.
- **Normalising the `:file` too.** Then a diagnostic could move modules unnoticed.
- ⚠ **Leaving `freeze.rs:1313` bent.** The unbending is the proof; without it the stone claims a tax
  removed while still paying it.
- **A control that only inserts a line and sees green** — green is also what a no-op normaliser
  produces. Row 4 needs the reddening half.

## Deliverable

`SCORE.md`: the normaliser and its Rust-path test, the 8 goldens, the `freeze.rs` re-wrap with fmt
counts before/after, mutation evidence in **both** directions, the trade-off stated, floor Summary +
`.floor/` path + tree statement.

⚠ Paths in pulsare messages are repo-root-relative (`wat-rs/docs/…`).
