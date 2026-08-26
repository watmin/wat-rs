# DESIGN — STONE: the host surface renames

> **Builder, 2026-08-25:** *"this is performed when the shadowdancer returns"* → HOME #7 landed
> (`36a473b39`) → *"draw the rename stone and cast intueri on it"*.
>
> ⚠ **THIS DESIGN DOES NOT NAME THE NAMES.** The ward does. This stone draws *what moves, how big it
> is, and what will break*; the replacement names come from the cast recorded in
> `wat-scripts/intueri/host-surface-renames.rs.intueri`. Drawing the shape and choosing the words
> are two acts, and the builder ruled the second one belongs to `intueri`.

## WHY — three findings, none of them mine to overrule

**1. `Harness` is a LEVEL 1 LIE** (ward, `0c1667524`). Its own module doc must spend a bullet fencing
off the first reading — *"A test runner. That's `wat test <path>`."* **A name that needs its doc
comment to say "I am not the thing you just assumed" has failed Obvious.** What it does: freezes wat
source into a `FrozenWorld` and invokes `:user::main` from a host Rust process.

**2. `compose` is a LEVEL 2 MUMBLE** (ward). Every public function compounds it —
`compose_and_run`, `compose_and_run_with_loader` — *the author compounded the verb every single
time*, which is the file telling you the bare word was never enough. And composing is roughly half
the file; the other half is OS stdio + signal wiring, which "compose" does not name.

**3. `Outcome` is the only verbless member of a large, consistent family** — and this is the finding
**four per-file casts structurally could not see**, because a cross-file pattern is invisible to a
per-file ward:

```
RecvOutcome 195 · SelectOutcome 100 · SendOutcome 79 · ReadFrameOutcome 56 · CosineOutcome 46
AcceptOutcome 43 · CloseOutcome 41 · ConnectOutcome 38 · VectorDecodeOutcome 33 · DeftestOutcome 33
ReadlnOutcome · ReadOutcome · FormOutcome · ConnectOutcome · …
```

Every one is `<Verb>Outcome`. Harness's is bare **`Outcome`**, and `lib.rs:142` re-exports it to the
crate root — so an external caller writes `wat::Outcome` beside a dozen verbed siblings.

## THE SURFACE — measured, and half of it must NOT move

```
identifier                    src+crates  tests+ex  docs(live)   docs/arc
Harness                            29        19         11         101
HarnessError                       30         7          3          38
compose_and_run                    22         0          1          50
compose_and_run_with_loader         6         0          1          15
                                  ───       ───        ───         ───
                LIVE SURFACE      137                             204  IMMUTABLE
```

⛔ **204 of the 341 references live under `docs/arc/**` and are untouched.** A past INSCRIPTION
naming `Harness` is a true statement about the world when it shipped. **More than half of what a
naive grep would "fix" is history.**

`Outcome`'s own surface is tiny — 2 qualified sites — but its crate-root re-export is the whole
problem, so it moves with the cluster.

## THE THREE TRAPS — all three already known, all three will fire

This is the fourth surface-move in a row, and the traps are no longer discoveries:

**TRAP 1 — the bare crate-root re-export.** `src/lib.rs:113` and `:142` are the *definition* of the
public surface being renamed:

```
pub use host::compose::{compose_and_run, compose_and_run_with_loader};
pub use host::harness::{Harness, HarnessError, Outcome};
```

Hit by HOME #5 (once), HOME #6 (twice), HOME #7 (twice). **Seven instances over four stones.**

**TRAP 2 — the proc-macro emits these names as generated text.** `crates/wat-macros/src/lib.rs`
writes `::wat::host::harness::HarnessError` and `::wat::host::test_runner::run_single_deftest` into
code it generates. **Nothing in `wat-rs` type-checks a string the macro writes.** It fails at
*expansion*, in `examples/console-demo` and `examples/with-loader` — and, corrected by HOME #7's
rider and verified by my own run, **a plain `cargo build --release` reaches them**, because
`Cargo.toml` pins both as workspace `default-members`. That pin exists precisely so downstream rot
cannot hide; its own comment says so.

**TRAP 3 — `README.md` teaches this API.** `README.md:485` has a section headed
`### Rust embedding — wat::Harness` with a worked example (`use wat::Harness;`,
`Harness::from_source(src)?`). A rename that leaves the README teaching the old name ships a lie to
the first thing a new reader opens. 11 live doc references; they are in scope.

★ **And the fourth trap is mine, from one commit ago:** `git add -A <dir>` followed by a pathless
`git commit` commits **the index**, not the directory — it shipped a broken `main` in `0a81149ac`.
This stone commits by path or not at all.

## THE FOUR QUESTIONS

- **Obvious?** YES — three names that a reader must correct after reading the body.
- **Simple?** YES — a rename. No logic, no structure, no behaviour.
- **Honest?** YES, and it is the whole point: `Harness` makes a reader believe something the file
  itself denies.
- **Good UX?** YES — and `wat::Harness` is what an *external crate* writes, so this is the one
  surface in the campaign where the reader is not us.

## ACCEPTANCE

1. **Zero live references to the old names** outside `docs/arc/**`. Derived: 137 at HEAD.
2. **`docs/arc/**` is byte-identical.** Derived: 204 references there, all staying.
3. **The macro's emitted strings are updated**, and `cargo build --release` (not merely
   `--all-targets`) is green — that is the build that reaches expansion.
4. **`README.md`'s `Rust embedding` section teaches the new name**, worked example included.
5. **No behaviour change** — `git diff` shows only identifiers and prose; no expression moves.
6. Floor green **accounted BY NAME** (baseline 5057/5057, 19 skipped); clippy 0.
7. **The commit contains everything it needs to build** — verified by building HEAD itself, not the
   working tree. A floor proves the tree; only a commit's own build proves the commit.

## OUT OF SCOPE

- **`test_runner`** — the ward called it *"honest for two thirds of itself"*, silent only about its
  reporting half. That half is the diagnostics-home question, not a rename.
- **The diagnostics home** — `panic_hook.rs` plus `test_runner.rs`'s ~350 reporting lines, converged
  on by two independent readers. Still the largest thing the cast found.
- **The remaining ward findings** — `StdioSnapshot` never constructed, `source_has_config_setter`
  ORing two conditions, the `failure_to_diagnostic` doc-link, `AssertionPayload::raised_error`.
