# BRIEF — STONE: `:wat::grep::run` (the driver — part A of `--grep`)

DESIGN: `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-the-grep-mode.md` — read it whole, first.

**This brief is PART A only: the wat driver.** The `--grep` CLI mode is part B and is NOT yours —
do not touch `src/distribution/`, `src/bin/`, or any argv parsing. The split exists because the
driver can be proven end to end without a single line of Rust, which is what makes part B pure
plumbing against something already working.

## Your role

You are a rider, not the orchestrator. **Ending your turn ENDS you** — nothing wakes you, no
notification is coming. Run every command in the FOREGROUND and block on it. Your turn ends when the
numbers are in your hands, not when a command is launched.

**You may not spawn sub-agents.**

Anchor: `/home/john/work/holon/wat-rs`. `pwd` first. You do not commit, push, stash, revert, or
checkout — leave your work uncommitted and report.

## Verification — the stdlib rule from the last stone still applies

`wat/grep.wat` is baked in by `include_str!` at RUST-compile time, so your edit is invisible until
the crate rebuilds. **Rebuild after every edit to `wat/grep.wat`, before any run** (~19s):

```
systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 timeout 900 cargo build --release
systemd-run --user --scope -q -p MemoryMax=6G  -p MemorySwapMax=0 timeout 180 ./target/release/wat <args>
```

Do NOT run `cargo nextest`, `scripts/floor.sh`, or clippy — the orchestrator takes those centrally.

⚠ **A stdlib file cannot pass a standalone `--check`.** `Privilege::Stdlib` comes from the
`STDLIB_FILES` pipeline, never from a CLI target — `wat/fix.wat` and `wat/deporder.wat` fail it
identically, and that is not a defect. Your verification is: the crate builds + a CONSUMER program
runs. Do not chase that red.

## The work in one paragraph

Add `:wat::grep::run` to `wat/grep.wat`: it takes a vector of rules, reads one EDN vector of file
paths from stdin, compiles the rules ONCE with the single query `:wat::grep::q-match`, and runs each
file through that network — facts in, fire, query, print each `Match`, reset — so that no file's
facts can reach another's.

## The rooms — read in this order

1. **`wat/grep.wat`** — whole. Your `run` goes here, beside `facts-of` and `q-match`. It shipped at
   `349a2ea52`; everything it declares is already live.
2. **`wat/rete/syntax.wat`** — `with-network` and `with-overlay`, and the header comment recording
   why they live there. **`with-overlay` is the one you want** — it hands the body a function
   `facts -> fired Session` always re-seeded from the compiled base, so the base is never in scope
   and threading one file's facts into the next HAS NO FORM. Read that header; it explains the
   difference from `with-network` precisely, and choosing wrong is how row 4 fails.
3. **`wat-scripts/fixes/angle-brackets-to-binder.wat:282-300`** — the stdin harness, labelled in its
   own comment *"identical shape to every recorded migration"*. Copy that shape for reading the EDN
   path vector; do not invent one.
4. **`wat-scripts/scratch-pad/rules-corpus-03-source-to-facts.wat`** — the `:fx::match-arrow` rule
   and its `q-match` consumption, shipped and working. Your probe's rules copy this shape.
5. **`wat/io.wat`** (or wherever `read-file` lives — grep for `:wat::io::read-file`) — how the
   driver gets a file's source.

## What to build

**In `wat/grep.wat`:**

```clojure
(:wat::core::defn :wat::grep::run
  [rules <- (:wat::core::PersistentVector :- [:wat::rete::Rule])]
  -> :wat::core::nil …)
```

- Reads ONE EDN vector of paths from stdin.
- Compiles `rules` + the single query `:wat::grep::q-match`. **The driver compiles — so the driver
  holds the lease, in one scope.** A caller that handed in a compiled Session would split ownership
  across a boundary; that is why the parameter is rules.
- Per path: read the source, `facts-of` it, insert the three fact vectors, fire, run `q-match`,
  print each `Match`, and move to the next file with the network back at its compiled base.
- Prints nothing else. No count, no header, no separator, no ranking. A file whose rules assert
  nothing produces no output for that file — that is the honest answer, not an error.

**And a probe** at `wat-scripts/scratch-pad/probe-grep-driver.wat`: declares two rules, calls
`:wat::grep::run` directly with them, and serves as the fixture for every row below. (Part B replaces
"the probe calls run" with "the CLI looks up `:user::grep` and calls run" — nothing else changes.)

## The acceptance rows YOU run

- **Row 1 — end to end.** `printf '["<file>"]' | ./target/release/wat <probe>` prints the Matches
  the probe's rules assert, and nothing else. Output verbatim.
- **Row 2 — a file whose rules match nothing prints nothing** for that file. Not an error, not an
  empty record, not a blank line with punctuation. Silence.
- **★ Row 3 — FACTS DO NOT LEAK BETWEEN FILES. This is the load-bearing row.** Craft file A so a
  rule fires on it and file B so the same rule does not. Run `["A" "B"]` — B's section must be
  empty. Then run `["B" "A"]` — so the result cannot be an artifact of ordering. Report both runs
  verbatim. **A green Row 1 with a failing Row 3 means the stone did not ship.**
- **Row 4 — the perturbation.** Prove Row 3 measures something: make the driver accumulate instead
  of reset (a one-line local change), re-run `["A" "B"]`, and show that B now DOES report A's match.
  Then revert the perturbation and confirm Row 3 again. **A control that cannot fail is not a
  control.** Report the perturbed output verbatim, and state plainly that you reverted it.
- **Row 5 — many files, one network.** Run 10+ real `.wat` paths through one invocation and confirm
  it completes and prints per-file. This is the shape part B will drive.

Report each row's command and its output **verbatim** — never a summary, never a `| head`/`| tail`
window. A row you could not run is reported as not-run, never as passed.

## Blast radius

- `wat/grep.wat` — edited (`run` added)
- `wat-scripts/scratch-pad/probe-grep-driver.wat` — created
- fixture files for rows 3–4 may live in the session scratchpad (`/tmp/...`), NOT in `wat-scripts/`

Nothing under `src/`. No other stdlib file. Nothing in `wat-scripts/lib/`.

## STOP triggers — each ships NOTHING and surfaces the gap

1. **Row 3 fails and you cannot make it pass without holding the base Session in scope.** STOP. That
   means the isolation cannot be structural here and the design's central claim is wrong — report
   what you observed; do not ship a version where the reset is the caller's discipline.
2. **`readln` cannot deliver the path vector** in the driver's context. STOP and report the exact
   outcome (`ReadlnOutcome::Eof` / `Stopped` / something else) — do not switch to a different input
   channel.
3. **The lease cannot be balanced** — something in `with-overlay`'s contract makes the driver hold
   or drop a lease it should not. STOP and report; the scoped-work surface is rete's and not yours
   to change.
4. **Anything requires editing `src/`.** STOP — that is part B, or a finding for the orchestrator.

A STOP means: leave the tree as it is, write the report, end your turn. It is never a licence to
ship a smaller version of a row.

## What you own that nobody can reconstruct

Row 4's perturbed output — the proof that Row 3 can fail — is the single most valuable thing in your
report. Beyond that: which of `with-network`/`with-overlay` you used and why, anything that
surprised you, and any place the driver had to do something the design did not anticipate.
