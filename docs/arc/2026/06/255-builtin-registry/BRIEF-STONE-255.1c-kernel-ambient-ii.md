# BRIEF — STONE 255.1c-kernel-ambient-ii · THE RULING: the registry answers, the prefix guesses

## You are a rider

You edit and report. **The orchestrator builds the floor, runs clippy, and commits** — do not run
either, and do not commit, push, stash, or revert. **Ending your turn ENDS you**; nothing wakes you.
Run everything in the FOREGROUND and block on it.

Anchor: `/home/watmin/work/holon/wat-rs/`. `pwd` first. Any path with `.claude/worktrees/` is harness
state — never operate on it.

**Two commands are yours:**

```bash
systemd-run --user --scope -q -p MemoryMax=8G -p MemorySwapMax=0 timeout 900 cargo build --release
systemd-run --user --scope -q -p MemoryMax=8G -p MemorySwapMax=0 timeout 900 \
  cargo nextest run --release -E 'test(/intrinsic::tests::/) + test(/rete::purity/)'
```

Read exit codes directly, never through a pipe.

## Where you are picking up

**The tree already carries a previous rider's work, and it is CORRECT — do not redo or revert it.**
`src/intrinsic/kernel_ambient.rs` exists with seven registered verbs; seven literal arms are gone
from `runtime.rs`; five diagnostics goldens are bumped `:line 25353 → 25333` (ratified by the
orchestrator, delta reconciled). Floor is **4818/4819** — the single red is
`intrinsic::tests::pure_declared_matches_is_effectful_op`, and that red is the finding this stone now
acts on.

**Read first, in order:**

1. `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-255.1c-kernel-ambient.md` — **its final section,
   `⊘ RULED 2026-08-19 — OPTION B`, is your specification.** It names the three sites and why each one.
2. `git diff` — the previous rider's work, so you know what is already in place.
3. `src/runtime.rs:29726` `is_effectful_op` · `:29724` `derive_pure_deterministic`
4. `src/intrinsic/mod.rs:544` the gate · `:300–316` the registry's lookup API
5. `src/intrinsic/reflect.rs:40–80` — `derive_pure_deterministic`'s one caller
6. `src/rete/purity.rs:1055–1068` — the other consumer

## The work — the builder's ruling, applied at three sites

> **Builder: "the registry is the truth now... that's why we built it. the registry must answer these
> kinds of questions... forms and funcs must be registered in a central authority who can resolve
> such questions.. that's 255 purpose."**

### Site 1 — `is_effectful_op` splits: a registry door over a named prefix fallback

```rust
/// Extract the existing prefix chain here, UNCHANGED, under its honest name.
/// This is a GUESS about a namespace, not a fact about a body.
fn effectful_by_prefix(head: &str) -> bool {
    head.starts_with(":wat::kernel::")
        || head.starts_with(":wat::io::")
        || head.starts_with(":wat::eval-")
        || head.starts_with(":wat::load")
        || head.starts_with(":wat::config::")
}

pub(crate) fn is_effectful_op(head: &str) -> bool {
    if let Some(e) = crate::intrinsic::registry().lookup_entry(head) {
        return matches!(e.purity, wat_doc::Purity::Effectful);
    }
    effectful_by_prefix(head)
}
```

`Pure` and `Preserving` both mean not-effectful — `matches!(.., Effectful)` is the whole test.
`effectful_by_prefix` must stay reachable from `src/intrinsic/mod.rs`'s test module for site 3.

### Site 2 — `reflect.rs` reads the entry it already holds

`reflect.rs:75` calls `derive_pure_deterministic(entry.name)` from inside
`for entry in registry().all_entries()` — a prefix guess for a row whose `entry.purity` and
`entry.determinism` are in the same struct. Read them directly instead.

Then **measure what is left of `derive_pure_deterministic`** and report it. Do **not** delete it or
its `NONDETERMINISTIC` hand-list on your own judgement — zero consumers is not evidence of deadness,
and the disposition is the builder's. Its doc comment is false twice (it claims two callers; it
claims the hand-list is for unregistered verbs) — if the fn survives, its doc must stop lying.

### Site 3 — the gate becomes a CENSUS, and keeps the one direction with teeth

After site 1, `pure_declared_matches_is_effectful_op` would compare `entry.purity` against a function
that returns `entry.purity`. It cannot fail. Shipping that is a gate reading a copy of the truth.

Re-point it so both halves are honest:

- **The census (registered rows).** For each registered entry, compare its declared purity against
  **`effectful_by_prefix(entry.name)`** — still genuinely independent. Every disagreement is an
  inventory entry, **not** a failure. Collect them all (do not `assert!` inside the loop — that is
  exactly why the old gate could only ever surface one), and make the test **print the full inventory**
  so a reader sees which rows the prefix rule gets wrong. **Expect the four readers carved by the
  previous rider to be its first four entries** — `stopped?`, `sigusr1?`, `sigusr2?`, `sighup?`.
- **The assertion that survives (unregistered verbs).** `Effectful ⇒ effectful_by_prefix` still has
  teeth where the registry is silent — a doc could lie about an effect the runtime cannot refuse.
  Keep that direction asserting.

Rename the test if its current name no longer says what it does; a name that describes a
biconditional over a census is its own small lie.

## Blast radius

```
EDIT  src/runtime.rs           split is_effectful_op; derive_pure_deterministic's fate + doc
EDIT  src/intrinsic/mod.rs     the gate → census + surviving assertion
EDIT  src/intrinsic/reflect.rs read entry.purity / entry.determinism directly
```

Nothing else. No new files. No change to `kernel_ambient.rs`, the goldens, `wat/runtime-meta.wat`,
`check.rs`, `crates/`, or any `.wat` fixture.

## STOP triggers — SHIP NOTHING FURTHER AND REPORT

1. **STOP-1 — the census comes back EMPTY, or without all four readers.** The four are the reason
   this stone exists; an empty census means site 1 or site 3 is wired wrong, or the census is reading
   the registry on both sides. Report it; do not adjust the census until it looks right.
2. **STOP-2 — `rete::purity` or any `intrinsic::tests::` test beyond the gate changes state.**
   `rete/purity.rs:1058` calls `is_effectful_op` and will now get a declared answer for registered
   rows. If that moves a test, that is a real behaviour change — capture it verbatim and stop.
3. **STOP-3 — `derive_pure_deterministic` turns out to have consumers you did not expect**, or
   removing its one call site breaks something. Report; do not delete, do not improvise.
4. **STOP-4 — the blast radius above is insufficient.** Name the file and why. Do not widen alone.
5. **STOP-5 — an init-order or borrow problem calling `registry()` from `is_effectful_op`.** It is a
   `OnceLock` returning `&'static` and should be safe from anywhere; if it is not, that is a finding.

## What "done" looks like

- `cargo build --release` exits 0
- the scoped run is green, and you report its **full Summary line**; any failure verbatim, whole block
- **the census inventory printed in full** — every row where declared purity and the prefix guess
  disagree, which is the artifact this stone exists to produce
- what `derive_pure_deterministic` has left, measured, with your recommendation and no action taken
- the honest deltas — what surprised you, what you inspected that this brief did not send you to

Runtime band: 30–50 minutes, mostly builds.
