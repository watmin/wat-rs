# BRIEF — STONE: the host surface renames

DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-the-host-surface-renames.md`
THE CAST: `wat-scripts/intueri/host-surface-renames.rs.intueri` — read its `⊘ CAST 2026-08-25`
section; it carries the evidence for every name and every rejection.

PRIOR ART: **HOME #7** (`36a473b39`, `src/host/`) and its **repair** (`36a473b39`'s parent
`0a81149ac` shipped broken — read why).

## Your role

Your cwd is `/home/john/work/holon/wat-rs`. Run `pwd` first. **Ending your turn ENDS you** — every
command FOREGROUND, blocking. **No sub-agents.** Do not commit, push, stash, revert, or
`git checkout`. `git stash@{0}` must never be touched. **Use `git mv`.**

You may run `cargo build --release` and `--all-targets`, and single named tests. **Not** the floor,
**not** clippy.

---

## The work

```
src/host/harness.rs  ->  src/host/guest.rs
src/host/compose.rs  ->  src/host/entry.rs

Harness                      ->  Guest
HarnessError                 ->  GuestError
Outcome                      ->  RunOutput
compose_and_run              ->  run_program
compose_and_run_with_loader  ->  run_program_with_loader
```

Names ruled by three independent `intueri` casts plus the builder. **You are not re-litigating
them** — the cast file carries the evidence, including a same-crate collision for every rejected
candidate. Your job is to land them without breaking anything and without touching history.

**137 live references. 204 more live under `docs/arc/**` and DO NOT MOVE.**

---

## ⛔ THE LINE THAT MATTERS MOST — `docs/arc/**` IS UNTOUCHABLE

A past INSCRIPTION naming `Harness` is a **true statement about the world when it shipped**. More
than half of what a naive tree-wide `sed` would "fix" is history. If your final count needs you to
edit a file under `docs/arc/` to reach zero, your count is wrong, not the history. **STOP-1.**

In scope: `src/`, `crates/`, `tests/`, `examples/`, `README.md`, and `docs/` **excluding**
`docs/arc/`.

---

## THE THREE TRAPS — all measured, all will fire

**TRAP 1 — `lib.rs:113` and `:142` ARE the public surface being renamed.**

```
pub use host::compose::{compose_and_run, compose_and_run_with_loader};
pub use host::harness::{Harness, HarnessError, Outcome};
```

They name their targets **unprefixed**, so no `crate::`/`wat::` grep finds them. Seven such
instances over the last four stones — this is the eighth and ninth.

**TRAP 2 — `crates/wat-macros` EMITS these names as generated text.** It writes
`::wat::host::harness::HarnessError` and `::wat::host::test_runner::run_single_deftest` into code it
generates. **Nothing in `wat-rs` type-checks a string the macro writes.** It fails at *expansion*,
in `examples/console-demo` and `examples/with-loader` — and a **plain `cargo build --release`
reaches them**, because `Cargo.toml` pins both as workspace `default-members`. (HOME #7's rider
corrected me on that and I verified it; do not skip the plain build believing it blind.)

**TRAP 3 — `README.md:485` TEACHES this API.** A section headed `### Rust embedding — wat::Harness`
with a worked example: `use wat::Harness;`, `Harness::from_source(src)?`. A rename that leaves the
README teaching the old name ships a lie to the first thing a new reader opens. **Update it,
including the heading and the worked example.**

---

## ⚠ AND FIX A THIRD LYING DOC — it is in a file you are renaming

`src/host/compose.rs` (becoming `entry.rs`) asserts at **line 25**, in its module doc, that the
function wires `crate::io::RealStdin` / `RealStdout` / `RealStderr` directly onto the frozen world.
The same claim repeats in the fn doc around `:85-88`.

**That doc line is the only occurrence of `crate::io` in the entire file.** The body at `:192-199`
says the Real* IO construction *"retires alongside the four-arg main_args plumbing"* — arc 170
slice 1f's substrate services own fd 0/1/2 now.

Make both docs say what the function actually does today: assemble the world, install process-global
state (panic hook, dep registry), install real OS **signal** handlers, invoke `:user::main`. **The
signal half is true; the stdio half is not.**

This is the third lying doc found in this family — the other two were fixed in HOME #7.

---

## STOP triggers — each means SHIP NOTHING and report

1. **STOP-1 — you need to edit `docs/arc/**` to reach zero.** History does not move. Report the
   count and the files.
2. **STOP-2 — a name you were given collides with something the cast did not check.** Every reject
   died on a grep-verified collision; if an *accepted* name hits one, that is a finding and I want
   it before the rename lands.
3. **STOP-3 — fixing anything beyond the two file renames, the five identifier renames, and the
   `compose.rs` doc.** The remaining ward findings (`StdioSnapshot` never constructed,
   `source_has_config_setter`, the `failure_to_diagnostic` doc-link, `AssertionPayload::raised_error`)
   are their own stone.
4. **STOP-4 — a room's line number does not hold what this brief says.** Written against `a527c8ba8`.

---

## Acceptance you can check yourself

```bash
ls src/host/                     # entry.rs guest.rs mod.rs test_runner.rs
git ls-files | grep -vE '^docs/arc/' \
  | xargs grep -ohwE 'Harness|HarnessError|compose_and_run|compose_and_run_with_loader' | wc -l   # 137 -> 0
git ls-files | grep -E '^docs/arc/' \
  | xargs grep -ohwE 'Harness|compose_and_run' | wc -l                                            # 204, UNCHANGED
grep -n 'wat::Harness\|Rust embedding' README.md    # -> the new name
grep -rn 'crate::io' src/host/entry.rs              # -> nothing (the doc lie is gone)
cargo build --release                                # the build that reaches macro expansion
cargo build --release --all-targets
```

⚠ Validate any pattern you invent before quoting its count. Four of my censuses today returned
confident wrong numbers: `\|` inside a `grep -E`; grepping `wat::` across `src/` and calling it
external; `grep -c` (lines) against a total built with `grep -o` (occurrences); and `Harness`
word-grepping to 162 because `.gitignore` says *"Harness state"* about the **agent** harness.

## Report back with

- The cascade's waterfall, and for each number **which build produced it**.
- The acceptance checks above — especially the `docs/arc/` count, **before and after, proving it did
  not move.**
- Confirmation the two file moves show as renames.
- **The before/after text of the `compose.rs` doc fix.** An honesty repair; I want to read the words.
- **Every site you edited that was not one of the five renames or that doc**, with `file:line`.
- Anything the brief got wrong.
- What you did NOT do, and why.
