# BRIEF — 294.k · a fabricated home is a lie; make it RAISE

**You are a rider, not the orchestrator. Ending your turn ENDS you** — nothing wakes you, no
notification is coming. Run every verification in the **FOREGROUND** and block on it. Your turn ends
when the numbers are in your hands, not when a command is launched.

Work in `/home/watmin/work/holon/wat-rs/`. **Do not commit, push, stash, or revert.** Leave work in
the tree.

## Read first

`DESIGN-STONE-294.k-a-fabricated-home-is-a-lie.md` (sibling) in full. For the shape of a finished job
in this arc, `BRIEF-294.j-RELAND-render-data-not-source-forms.md` and its strike (`f6f8df3b`).

## The work, in one line

`tag_from_type_path` and its decode-side mirror `struct_tag_for` invent a namespace when a type path
has no `::`, and erase a type's name to the word `"unnamed"` when that fails. **Both fabrications
become a RAISE that names the offending path.**

## Rooms, in order

1. **`src/edn_shim.rs:3959` — `tag_from_type_path`.** The two fallbacks (`:3964`, `:3968`, `:3969`).
2. **`src/edn_shim.rs:2653` — `struct_tag_for`.** The decode-side mirror, same `.local` fallback
   (`:2660`). **It must move in the same change** — the two are one concept implemented twice.
3. **The six live callers** of `tag_from_type_path`: `edn_shim.rs:3688, 3705, 3729, 3744, 3827, 3871`.
   `struct_tag_for` has one: `:2473`.

## ★ THE METHOD — impose the check, read the screams. Do NOT survey first.

Replace both fallbacks with a raise, **then run the floor.** That is the instrument.

Do **not** pre-enumerate which paths might lack a home. Do **not** add a temporary log and take a
census. My surveys have been wrong five separate times in this arc and the imposed wall has been right
every time. `[[feedback_impose_the_check_and_read_the_screams]]`

Two outcomes, **both are a successful strike**:

- **Floor silent** → the arms were dead. Report that, with the Summary line.
- **Floor screams** → the screams ARE the finding. Report **every** distinct offending path verbatim.
  Do not fix them, do not add a fallback, do not widen the raise to let them through. That list is
  the next stone and it is worth more than a green floor.

## ⚠ A SIGNATURE DECISION YOU MUST MAKE AND REPORT

`tag_from_type_path` returns a bare `Tag`; `struct_tag_for` returns `(String, String)`. Neither can
express failure today. So "raise" is one of:

- **`panic!` with the path in the message** — the house pattern 294.j used for its encode wall
  (`edn_shim.rs`, the `from_holon_item` `Err` arm). Simplest; no caller churn.
- **`Result<Tag, _>` threaded through the seven call sites** — more honest, more churn.

**Pick one, say which, and say why in your report.** If threading `Result` cascades *outside*
`edn_shim.rs`, that is **STOP-2** — do not push a signature change through unrelated modules to serve
this stone.

Whichever you pick: **the error MUST name the offending path.** A generic "invalid type path" message
reproduces the exact defect — a value losing its identity silently — one layer up.

## The gate

| # | assertion |
|---|---|
| 1 | `grep -rn 'wat-edn\.local' src/ crates/ tests/ wat/ wat-scripts/ wat-tests/` → **0** |
| 2 | `grep -rn 'wat-edn\.opaque' src/ crates/ tests/ wat/ wat-scripts/ wat-tests/` → **0** |
| 3 | ★ **a DIFFERENTIAL test**: feed `tag_from_type_path` and `struct_tag_for` the same set of paths; assert identical `(ns, name)` on every success **and that both raise on the same inputs**. This row has never existed and it is the row that would have caught the same class in task #102 the day it diverged. |
| 4 | a path with no derivable home **RAISES**, and the message **names the path** |
| 5 | the raise is covered by a **kept** test — `[[feedback_a_negative_control_that_can_be_kept_must_be_kept]]`. If you build a reproduction, it ships; do not delete it after writing the finding in prose. |
| 6 | floor via `scripts/floor.sh` — report the **Summary line**, never a piped exit code |
| 7 | `cargo clippy --release --all-targets` → **0** |
| 8 | `grep -rnE '^[[:space:]]*#\[ignore' tests/ src/ crates/ benches/ --include=*.rs \| wc -l` → **13** |

Row 3 is the load-bearing one. Rows 1–2 are the visible outcome; row 3 is what stops the pair
diverging again.

## What you report

- the `git diff` of both functions
- **which raise mechanism you chose and why** (panic vs Result), and whether STOP-2 fired
- the floor **Summary line verbatim**
- **if the floor screamed: every distinct offending path, verbatim, with the test that surfaced it.**
  This is the most valuable thing you can bring back and it outranks a green floor.
- clippy count; `#[ignore]` count
- honest deltas — surprises, anything you could not do, anything you did beyond the brief and why

## STOP triggers — rejection criteria. Ship nothing on that axis; report and stop.

- **STOP-1 — the raise fires on a path that is clearly LEGITIMATE** (a real type with a sensible home
  that the split simply mishandles). Then the bug is in the *derivation*, not the fallback, and the
  fix is a different stone. Name the path and stop.
- **STOP-2 — threading `Result` escapes `edn_shim.rs`.** Do not push a signature change through
  unrelated modules. Fall back to the panic form, or report and stop.
- **STOP-3 — the `#[ignore]` count moves off 13.** That is a finding about this brief, not a step.
- **STOP-4 — an unintended red. Do NOT re-run.** A re-run that goes green destroys the only evidence.
  `scripts/floor.sh` has already kept the untruncated log at `.floor/latest/`. Copy the failing test's
  **entire** stdout+stderr block **verbatim** — never a summary, never a `| head`/`| tail` window —
  and name the exact assertion or match arm that fired. There is no such thing as a known flake.

## Out of scope — RULED, do not touch

- **`wat-edn.cap`** (2 sites) — a **security boundary**; a refusal predicate keyed on the namespace
  string plus its emitter, which must move atomically. The builder's ruling, not this stone's.
- **`wat-edn.float`** (1 site, `crates/wat-edn/src/parser.rs:361`) — the EDN crate's own NaN/±Inf
  sentinel, with Clojure interop tests. The builder's ruling.
- **structs on the wire** — already law and already proven
  (`tests/comms/probe_arc293_W2a_struct_no_cross.rs`). The shim's struct arms serve **local
  rendering**; `value_to_edn_with` also backs `str`, diagnostics and chain envelopes. **Leave them.**
