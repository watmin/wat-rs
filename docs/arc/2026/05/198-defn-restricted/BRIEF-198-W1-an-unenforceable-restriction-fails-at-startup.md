# BRIEF — 198 W1: an unenforceable restriction fails at startup

> Read `DESIGN-STONE-a-restriction-governs-mention-not-head-position.md` first — W1 is the extirpare
> rung it names and affirmatively cut from the fix strikes. Baseline HEAD `8f0e3939`, tree clean,
> floor **4534 run / 4534 passed / 154 skipped**, clippy 0.

## THE CLASS

A `:restricted-to` can be **registered under a key that resolves to nothing**. The declaration parses,
registers, and is consulted by no one. Nothing errors. The capability is decorative and silent — which
is exactly how the mention-position bug survived 44 days.

**This wall would have caught its sibling.** When `310aa793` annihilated `Name/new`, it also moved the
whitelist's key from `T/new` to `T`. It got that right. **Had it not**, the metadata would have sat
under a dead name forever, enforcing nothing, with no signal. W1 makes that outcome impossible.

## THE WORK IN ONE PARAGRAPH

After registration completes, walk every key in `SymbolTable.binding_metadata` that carries a
`:restricted-to` entry and assert the key **resolves to a live binding**. A key that resolves to
nothing is an unenforceable declaration → **startup fails**, naming the orphaned key.

## ⛔ FIRST — MEASURE, THEN DECIDE WHETHER YOU CAN ARM IT

**Before writing the wall, write the census.** Enumerate every `:restricted-to` key at startup and
report which ones resolve and which do not.

- **If the count of orphans is ZERO** — arm the wall at zero offenders. This is the strongest form:
  the gate is turned on against a clean tree and can only ever fire on new drift.
- **If the count is NON-ZERO** — ⛔ **STOP-1.** Every orphan is a live finding: a capability someone
  declared that has never been enforced. Report the list verbatim with each key and why it fails to
  resolve. **Do not fix them and do not arm the wall over them** — the orchestrator rules on each.

Report the census either way. It is the deliverable even if the wall lands.

## WHERE

`src/restriction_entry.rs`'s module doc gives the pipeline order verbatim — read it, do not
reconstruct it:

> *"The startup pipeline (`startup_from_forms_post_config` in `freeze.rs`) iterates
> `inventory::iter::<RestrictionEntry>` **AFTER all `register_defines` calls complete and BEFORE
> `check_program` runs**."*

The wall goes **after the inventory drain and before `check_program`** — later than any registration,
earlier than any use. Placing it earlier reads a half-built table and produces false orphans.

## ⛔ "RESOLVES" MUST BE MEASURED, NOT ASSUMED

This is the load-bearing definition and getting it wrong makes the wall either useless or a false
alarm. A `:restricted-to` key may name any of several things. **Enumerate the kinds from the disk**,
by looking at what actually gets keys written into `binding_metadata`:

- a registered function (`sym.functions_iter()`)
- an aggregate type name (`runtime.rs:1453`)
- a synthesized companion — `T'` and `is-T?` (arc 198 strike 2, `8f0e3939`)
- a field accessor `T/field` (`runtime.rs:1460`)
- whatever else the census turns up — **if you find a kind this list does not name, that is a finding**

Do **not** hardcode a list of name shapes and pattern-match on them. That is the B3 forgery shape the
stone already ruled out. Ask the registry whether the key is live; do not ask a string what it looks
like.

## KNOWN SURFACE — verify these counts yourself before trusting them

- **5 Rust-side `#[restricted_to(...)]` sites**: `src/io.rs:1275` (`IOWriter/from-fd`),
  `src/io.rs:1315` (`IOReader/from-fd`), `src/kernel/spawn.rs:452` (`spawn-thread`),
  `src/kernel/spawn.rs:524` (`spawn-process`), `src/runtime.rs:26993` (`close`).
  **These are the orphan-risk channel** — the FQDN is a `&'static str` literal typed by hand, with
  nothing today checking it names a real binding. A typo is invisible.
- **7 wat-side `:restricted-to` declarations** in `wat/` (`core.wat` prose ×2, `spawn.wat:329`,
  `stdio.wat` ×3 + 1). These are attached to the form they restrict, so they are structurally
  lower-risk — confirm that rather than assume it.

**Count THINGS, not files** (`[[feedback_a_file_count_is_not_an_item_count]]`), and re-measure: every
number in this brief has been wrong at least once somewhere on this arc.

## THE GATE — the wall must be proven able to fire

1. **The census**, reported.
2. **Negative control.** Introduce a deliberate orphan — e.g. temporarily point one
   `#[restricted_to(...)]` at a misspelled FQDN — rebuild, confirm **startup fails** and the error
   **names the orphaned key**. Then revert and confirm green. `git diff` must show no residue.
   Without this the wall proves nothing (`[[feedback_a_green_test_can_prove_nothing]]`).
3. **A persisted test** for the orphan case, so the wall cannot be silently removed later.
4. **The floor stays green** at 4534, or the delta is explained.

## STOP TRIGGERS — rejections. Report and leave the site.

- **STOP-1 — the census finds orphans.** Report the list; do not fix, do not arm over them.
- **STOP-2 — "resolves" cannot be defined without pattern-matching name shapes.** Report the obstacle.
  A string-shape test is the ruled-out B3 forgery.
- **STOP-3 — the wall fires on something legitimate** you cannot classify. Report it; do not add an
  exemption to make it pass.
- **STOP-4 — you are tempted to delete or weaken a restriction to reach green.** Never.

## BLAST RADIUS

`src/freeze.rs` (the wall, at the pipeline point named above), possibly `src/restriction_entry.rs`
(its doc, if the pipeline order shifts), and new tests. **No `.wat` corpus changes.** W2 (the
safety-claim sweep) is a **separate strike — do not start it here.**

## VERIFY

`cargo build --release --tests`, then `cargo clippy --workspace --all-targets --release -- -D warnings`
(expect 0), then `scripts/floor.sh` and read the **Summary line** — never a piped exit code.

Baseline `4534 / 4534 / 154 skipped`. Report the real arithmetic.

**On any red you did not intend: do NOT re-run.** Copy the failing test's whole stdout+stderr block
**verbatim** — never a `| head` window — name the exact assertion, and report.

## HOW TO WORK

You are a rider. **Ending your turn ENDS you.** **Run every build and test in the FOREGROUND and block
on it — do not background anything, do not set a monitor and wait.** A rider on this arc died exactly
that way and its floor run had to be recovered by the orchestrator. Anchor at
`/home/watmin/work/holon/wat-rs`; `pwd` first. **Leave the work uncommitted.** Never
`git commit`/`push`/`stash`/`revert`/`checkout --`; `stash@{0}` holds unrelated work.

## REPORT

- **the census, in full** — every `:restricted-to` key, resolving or not
- the wall's placement and why that point in the pipeline
- how "resolves" is defined, and how you derived the list of kinds from the disk
- **the negative control both ways**, with the error text the orphan produced, and proof of no residue
- the floor Summary line verbatim with the arithmetic
- every STOP that fired
- **the honest deltas — especially anywhere this brief did not match the disk.**
