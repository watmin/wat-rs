# BRIEF — STONE 299.3-entropic · rename the `Category` variant `:Clock` → `:Entropic`

## You are a rider

You edit and report. **The orchestrator builds the floor, runs clippy, and commits.** Do not run
either; do not commit, push, stash, or revert. **Ending your turn ENDS you** — nothing wakes you, no
notification is coming. Run everything in the FOREGROUND and block on it.

Anchor: `/home/watmin/work/holon/wat-rs/`. `pwd` first. Any path containing `.claude/worktrees/` is
harness state — never operate on it.

**Two commands are yours:**

```bash
systemd-run --user --scope -q -p MemoryMax=8G -p MemorySwapMax=0 timeout 900 cargo build --release
systemd-run --user --scope -q -p MemoryMax=8G -p MemorySwapMax=0 timeout 900 \
  cargo nextest run --release -E 'test(/intrinsic::tests::/) + test(every_enum_variant_reaches_both_hand_lists)'
```

Read exit codes directly, never through a pipe.

## The work in one paragraph

`Category`'s `:Clock` variant named a **device**. The builder ruled that `time::now` and
`SecureRandom.uuid` are the same category — a syscall that samples an unpredictable source, effects
nothing, and returns a value that can only be bounded, never pinned. Rename the variant **in place**
to `:Entropic`, rewrite its prose, and follow the rename through every mirror and all 17 rows that
declare it. **No `@Purity` and no `@Determinism` changes anywhere in this stone.**

## Read in order — why you are sent to each

1. **`docs/arc/2026/07/299-entropic-values/DESIGN-STONE-299.3-entropy-is-a-CATEGORY-Entropic.md`** —
   the stone. Its "The prose to ship" section is the **exact text** for the variant; its "The one
   contract decision, pinned" says why this is a rename and not add-and-retire.
2. **`git show ec11f6ac`** — 255.1c-taxonomy, the last `Category` change. **This is the shape you
   copy**: it went 10 → 15 and had to touch every mirror. Your rename touches the same set.
3. **`wat/runtime-meta.wat`** — `:127` the variant, `:70` the header's list, `:165` `:Ambient`'s prose
   which says *"NOT `:Clock`"* and must follow the rename.
4. **`crates/wat-doc/src/lib.rs`** — `:71` `CATEGORY_LEGAL_VALUES`, `:1159` the drift gate's `all`
   array, `:1170` its match arm.
5. **`crates/wat-macros/src/wat_intrinsic.rs:380`** and **`wat_special_form.rs:84`** — one arm each.
6. **`src/intrinsic/time.rs`** — 17 rows declaring `@Category      Clock`.

## The prose to ship

Take it **verbatim from the design stone's "The prose to ship" section.** Do not paraphrase it — it
records why the variant was renamed and what it is NOT, and that text becomes the generated Rust
variant's `///` doc via `wat_enum_derive`.

## Two corrections this stone makes true

`src/intrinsic/kernel_stdio.rs:36-39` justifies `readln'`/`read-frame`'s nondeterminism as *"the
returned value depends on ambient state outside the call's arguments, **exactly as
`:wat::time::now` reading the wall clock does**"* — and `src/intrinsic/kernel_ambient.rs` repeats the
same claim for its four readers. **This stone is what makes those sentences false**: `readln'` is
`:Io` (the world hands it DATA, and you inject it in a test) while `time::now` is `:Entropic` (it
samples, and you conform it to a bound). Correct both comments to say they are different cells. Do
not change any declared axis value in either file — comments only.

## Blast radius

```
EDIT  wat/runtime-meta.wat                        the variant + prose, the header, :Ambient's "NOT :Clock"
EDIT  crates/wat-doc/src/lib.rs                   legal values, the gate's array, its match
EDIT  crates/wat-macros/src/wat_intrinsic.rs      one arm
EDIT  crates/wat-macros/src/wat_special_form.rs   one arm
EDIT  src/intrinsic/time.rs                       17 @Category rows
EDIT  src/intrinsic/kernel_stdio.rs               comment only
EDIT  src/intrinsic/kernel_ambient.rs             comment only
```

Nothing else. No new files. No `@Purity`/`@Determinism` edits. No `.edn`, no test fixture, no other
`.wat`.

## STOP triggers — SHIP NOTHING FURTHER AND REPORT

1. **STOP-1 — a `Clock` reference outside the blast radius.** I measured that no `.edn` golden, test
   fixture, or corpus `.wat` pins `Clock` as a Category. If you find one, my census was wrong —
   report it, do not fix it.
2. **STOP-2 — the rename would change a variant's ORDINAL.** It must not: fifteen variants in,
   fifteen out, same order, one renamed. If anything renumbers, stop.
3. **STOP-3 — a row's `@Purity` or `@Determinism` has to change to make something compile or pass.**
   That would mean the axes are entangled in a way the stone did not predict. Report; change neither.
4. **STOP-4 — the blast radius is insufficient.** Name the file and why. Do not widen alone.

## What "done" looks like

- `cargo build --release` exits 0 — a missing/unknown `@Category` is a `compile_error!`, so a green
  build already proves every one of the 17 rows and every mirror was reached
- the scoped run green; report its **full Summary line** verbatim, and any failure's whole block.
  `every_enum_variant_reaches_both_hand_lists` is the drift gate that exists **because these lists
  drifted once before** — it is the load-bearing row here
- `grep -rn "Clock" src/ crates/ wat/` returns only intentional historical mentions, and you list
  them for me with a one-line disposition each
- the honest deltas — what surprised you, anything you inspected that this brief did not send you to

Runtime band: 25–40 minutes, mostly the two builds.
