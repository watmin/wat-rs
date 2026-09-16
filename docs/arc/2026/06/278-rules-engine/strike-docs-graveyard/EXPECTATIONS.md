# EXPECTATIONS — the docs graveyard gate

> Written **before** the strike. Scored against the orchestrator's own re-run.

| # | what | command | expected |
|---|---|---|---|
| 1 | the gate is RED before any fix | `cargo nextest run --release --no-capture -E 'test(docs_wat)'` | **FAIL**, naming **five** files from a walk of ten: `surface-field-dispatch`, `red-owner-signals-child`, `experiri-acc-head`, `experiri-then-match`, and the two `complected-2026-05-02` files |
| 2 | the gate walks TEN files | the failure message or a printed count | **10**. Fewer means the walk is missing a directory; more is a finding to surface |
| 3 | non-vacuity | read the gate | an explicit `assert!(!entries.is_empty(), …)` with its reason, cited to `no_ceiling_raise_in_rete.rs:92` |
| 4 | the gate is GREEN after | as row 1 | **1 passed** |
| 5 | the migration RESTORES the proof | `./target/release/wat docs/…/probes/surface-field-dispatch.wat` | prints **142** — its header's own promise. **Not "it loads"** |
| 6 | `red-by-design` files still fail | drive each of the three | still refuse, for their own stated reasons — a marked file that started *passing* means the thing it proved is gone |
| 7 | `historical` files are byte-unchanged | `git diff --stat` on the two | **header comment only**; not one line of their body moves |
| 8 | the sibling gate still holds | `cargo nextest run --release -E 'test(every_wat_scripts)'` | green |
| 9 | the floor | `./scripts/floor.sh`, Summary from the captured log | **5,180 / 5,180**, 21 skipped, exit 0 |
| 10 | clippy | `cargo clippy --release --workspace --all-targets -- -D warnings` | silent, exit 0 |

## The mutation proof

Row 1 → row 4. Then **two independent mutations**, because this gate has two arms:
- strip the rune from one `red-by-design` file → **that file alone** reddens; restore
- revert `:nature` → `:holder` → **`surface-field-dispatch` alone** reddens; restore

⚠ If a mutation reddens *nothing*, that is a finding about coverage, not a null result — the
previous rider on this arc established that, and it is how an unproven line gets found.

## Runtime prediction

30–45 minutes. Two or three release builds; the gate itself is ~50 lines modelled closely on its
sibling, and the edits are one keyword plus five header comments.

## Trap doors named in advance — with the step

- **Row 5 is the one that can be silently lost.** "It loads" is not the bar; the file's header
  promises it prints 142. **Step:** run the binary on it and read stdout, do not infer from the
  gate passing.
- **The `historical` pair is a trap in the other direction.** Their bodies must not move.
  **Step:** `git diff` them specifically and confirm only the header changed.
- **The walk may find `.wat` in arcs nobody has looked at in months.** That is the gate working.
  **Step:** surface them with their verdicts; do not narrow the walk to make the strike smaller.

## What would make this a failure even if every test passes

Marking `surface-field-dispatch.wat` instead of migrating it. That is rot wearing a declaration,
which rebuilds the graveyard **inside** the gate and leaves it looking enforced — the precise
outcome the ★ decision exists to prevent.
