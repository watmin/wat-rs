# BRIEF — STONE 255.1c-retire-kernel-drop · delete one verb, and only that verb

## You are a rider

You edit and report. **The orchestrator builds the floor, runs clippy, and commits.** Do not run
either; do not commit, push, stash, or revert. **Ending your turn ENDS you** — nothing wakes you.
Run everything in the FOREGROUND and block on it.

Anchor `/home/watmin/work/holon/wat-rs/`; `pwd` first. Any path with `.claude/worktrees/` is harness
state — never operate on it.

**Two commands are yours:**

```bash
systemd-run --user --scope -q -p MemoryMax=8G -p MemorySwapMax=0 timeout 900 cargo build --release
systemd-run --user --scope -q -p MemoryMax=8G -p MemorySwapMax=0 timeout 900 \
  cargo nextest run --release -E 'test(/intrinsic::tests::/) + test(/collection/) + test(/seq/)'
```

The second filter deliberately includes the **collection and seq** suites — they are your proof you
did not kill the wrong `drop`.

⚠ Your filter does not reach `tests/diagnostics/`. Deleting a body shifts `src/runtime.rs`, so five
line-pinned `.edn` goldens will go red. Expected, mine to ratify. **Report a scoped run, not a floor.**

## ⛔⛔ THE ONE WAY THIS STONE GOES WRONG — READ THIS TWICE

**There are TWO functions named `infer_drop`, and one of them is load-bearing with 38 callers.**

```
src/check.rs:9448              fn infer_drop(…)   ← THE KERNEL ONE.   DELETE.
src/collection/infer.rs:1069   infer_drop(…)      ← THE SEQABLE ONE.  DO NOT TOUCH.
```

```
check.rs:4253   ":wat::kernel::drop" => infer_drop(…)                            ← DELETE this arm
check.rs:4431   ":wat::core::drop"   => crate::collection::infer::infer_drop(…)  ← LEAVE THIS ALONE
```

`collection/infer.rs`'s version type-checks `(:wat::core::drop xs n)` — arc **118.2a**, the lazy
`Seqable<T> × i64 → Stream<T>` path that arc 118 spent four months building and inscribed this
morning. **A grep-and-delete on the name `infer_drop` destroys it.**

Likewise `:wat::core::drop` (38 corpus call sites) is ALIVE and is NOT your target. Your target is
`:wat::kernel::drop` and nothing else. **Every deletion you make must name the full
`:wat::kernel::` path or be inside a body you have confirmed is the kernel one.**

## Why the verb is being retired (so you can recognise a surprise)

`:wat::kernel::drop` is a no-op that accepts only `Sender`/`Receiver`. **Nothing in the corpus
constructs either** — `:wat::kernel::Channel<T>` is a *typealias*, not a verb. It has had **zero
callers in its entire four-month history**, its co-born sibling `try-recv` is already gone, and the
raw-channel-ends world it served was replaced by typed Peers (arc 170 C1/C2), where `close` is the
successor. It is unreachable, not merely unused.

## The work

Delete, in this order, verifying each is the kernel one:

1. `src/runtime.rs` — the `":wat::kernel::drop" => …` dispatch arm **and its held-back comment block
   above it** (a `⊘ drop is HELD BACK…` paragraph), plus the `eval_kernel_drop` fn body.
2. `src/check.rs:4253` — the `":wat::kernel::drop"` inference arm.
3. `src/check.rs:9448` — the LOCAL `fn infer_drop` and its doc comment (it says
   *"Type-check `(:wat::kernel::drop handle)`"* — that sentence is your confirmation you have the
   right one).
4. `src/check.rs:19481` — a comment referencing it; update or delete as its context requires.
5. `wat/runtime-meta.wat` — `:Resource`'s prose. It names `drop` in the member list (`:159`) and
   carries *"`drop` is a documented NO-OP — it does not force teardown while other references
   remain"* (`:161`). **Remove both**, leaving the sentence grammatical. Do not touch any other
   variant's prose.
6. `src/intrinsic/kernel_resource.rs` — the module doc explains why `drop` was held back from the
   carve. Replace that with a one-line note that the verb was **retired**, citing this stone. The
   home's fourteen rows are now `:Resource`'s whole population.

## MUST NOT TOUCH

`src/collection/infer.rs` · `check.rs:4431` · anything spelled `:wat::core::drop` · `wat/seq.wat` ·
`Value::wat__kernel__Sender`/`Receiver` and the crossbeam plumbing (they build peers internally; only
the wat-facing verb goes).

## STOP triggers

1. **STOP-1 — a corpus `.wat` file calls `:wat::kernel::drop`.** My census says zero. If you find
   one, my census was wrong — report it and stop; do not migrate it.
2. **STOP-2 — deleting the kernel `infer_drop` breaks the collection/seq suites.** That means the two
   got crossed. Stop immediately and report; do not "fix" the seqable path.
3. **STOP-3 — a test asserts on `:wat::kernel::drop`'s behaviour.** Report it; do not delete a test
   to make a deletion pass.
4. **STOP-4 — the blast radius is insufficient.** Name the file and why.

## What "done" looks like

- `cargo build --release` exits 0
- the scoped run's **full Summary line**, labelled scoped — **the collection and seq suites green is
  the load-bearing evidence** that the surviving `drop` is intact
- `grep -rn ':wat::kernel::drop' src/ wat/ tests/` returns only intentional historical mentions, each
  listed with a one-line disposition
- `git status --short` **and `git diff --numstat src/runtime.rs`** — I need the net delta for the
  goldens ratification
- the honest deltas

Runtime band: 20–35 minutes.
