# A golden must not pin a Rust line

**A tax this session paid twice.** Excursus `001-sns-sqs`.

## The sentence

> **8 golden files assert the LINE NUMBER of a `src/*.rs` span. Source layout is therefore
> load-bearing for a test, and it has already bent the code.**

## The evidence, both from this session

1. **A `rustfmt` run reddened it.** `.floor/2026-09-19T01-13-46Z/` — three arms, one being
   `probe_supervisor_select_lost` because reformatting shifted `src/freeze.rs:1547 → 1559`. The
   sweep changed no behaviour whatsoever.
2. ⛔ **We then bent the source to satisfy it.** `src/freeze.rs:1313` is a **128-character** call
   left deliberately unwrapped so `apply_function`'s span stays on line 1547. Measured:
   `cargo fmt --check` wanted 7 changes in that file at HEAD and **8** with the engine merge — one
   new rustfmt-unstable site, added on purpose. **The fixture is dictating the formatting.**

## ⛔ The spans themselves are CORRECT — this is not that stone

`tests/lint/span_substitution_justified.rs` rules on exactly this:

> *"A **leaf** with no wat span in scope. `rust_caller_span!()` exists for exactly that case — a
> Rust-side helper genuinely holding no wat location, where **a Rust line beats nothing**."*

So a diagnostic pointing into `src/*.rs` at a leaf is doctrine, not a defect, and **the user should
keep getting it.** ⚠ Do not "fix" this by removing Rust spans, and do not weaken that lint.

**The defect is narrower: the golden asserts the line.** The test's claim is the error's identity and
message; *which line of `runtime.rs` raised it* is an implementation detail the test never meant to
pin.

## The work

### 1. Normalise `:line` / `:col` for `src/*.rs` spans in the golden comparison

`assert_edn_matches_file!` (`src/lib.rs:309`) is the one door. For a `#wat.core/Span` whose `:file`
matches a Rust source path, compare the **file** and normalise the line/col. ⛔ **Only Rust paths.**

⚠ **153 golden files carry `.wat` spans and MUST keep their lines** — there, the line is the entire
point (it is the author's own code). A normaliser that catches both is the failure mode.

### 2. The 8 goldens

`tests/process/probe_supervisor_select_lost__process_panics.edn` ·
2 × `tests/types/probe_arc293_*` · 5 × `tests/diagnostics/probe_diagnostic_value_snapshot_in_errors__*`

### 3. ⭐ Then unbend the source

Re-wrap `src/freeze.rs:1313` to whatever `rustfmt` wants, and confirm the golden stays green. **That
is the proof the tax is gone** — the line may move freely again.

### 4. The control

⛔ By MUTATION, and it is the whole point: **insert a blank line above one of the pinned spans, so
the line number shifts, and the golden must stay GREEN.** Then revert. A normaliser that cannot
survive a deliberate line shift has not been demonstrated.

⚠ And the inverse: shift a line inside a `.wat` fixture that a golden pins, and that golden **must
redden** — proving the normaliser did not over-reach into the 153.

## ⚠ The trade-off, named

Normalising the line loses one signal: a diagnostic that starts being raised from a **different
place in the same Rust file** would no longer redden. Within a file, that is precisely what we do not
care about — but it is a real loss and the SCORE must state it rather than pretend the change is
free. ⭐ The **file** is still pinned, so a diagnostic moving to a different module still reddens.

## Scope wall

⛔ Do not remove Rust spans from diagnostics. Do not weaken `span_substitution_justified` or
`unused_span_justified`. Do not touch the 153 `.wat`-span goldens. Not a `.wat` change at all.

## The four questions

- **Obvious** — ✅ the test asserts an error's identity, not a line of `runtime.rs`; and the tax is
  already documented in two floor artifacts.
- **Simple** — ✅ one normaliser at one macro, 8 goldens.
- **Honest** — ✅ it preserves the Rust span the user sees and the lint that defends it; it removes
  only the test's dependence on the line, and names what that costs.
- **Good UX** — ✅ nobody keeps a 128-character line unwrapped to hold a fixture's line number again.
