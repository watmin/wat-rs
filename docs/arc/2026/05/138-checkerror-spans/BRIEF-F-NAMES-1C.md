# Arc 138 F-NAMES-1c — Sonnet Brief: name wat::test! deftest threads

**Goal:** rename the wat::test! deftest worker thread from `<unnamed>` to `wat-test::<deftest_name>`. Single-file fix in crates/wat-macros/src/lib.rs at line 672.

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

**Driver:** the user's UX observation — `<unnamed>` in panic headers is unhelpful. F-NAMES-1c fixes it.

## Read in order

1. `docs/arc/2026/05/138-checkerror-spans/NAMES-AUDIT.md` — F-NAMES-1c spec.
2. crates/wat-macros/src/lib.rs lines 660-690 (the deftest thread spawn site).

## What to do

At crates/wat-macros/src/lib.rs:672, the current emit:
```rust
let __wat_handle = ::std::thread::spawn(move || {
    // deftest body
});
```

Replace with:
```rust
let __wat_handle = ::std::thread::Builder::new()
    .name(format!("wat-test::{}", #deftest_name))
    .spawn(move || {
        // deftest body
    })
    .expect("Thread::Builder::spawn failed");
```

Where `#deftest_name` is the deftest name string already in scope (visible at line 626 as `deftest_name = &site.name`).

After fix, panic header reads:
```
thread 'wat-test::my_deftest_name' panicked at <real-wat-file>:10:19:
```

Both pieces (thread name + file location) become navigable.

## Constraints

- ONLY crates/wat-macros/src/lib.rs modified. NO other files.
- All 7 arc138 canaries continue to pass.
- Workspace tests pass.
- NO commits, NO pushes.

## Verification spot-check

Pick any wat-test deftest that's known to panic; run it; confirm panic header shows `thread 'wat-test::<name>'` instead of `thread '<unnamed>'`.

## Reporting back

Compact (~150 words):

1. Diff stat (1 file).
2. Confirm `Thread::Builder::new().name(format!("wat-test::{}", deftest_name))`.
3. Verification: 7/7 canaries; workspace tests; spot-check panic header.
4. Honest deltas.
5. Four questions briefly.

## Why this is tiny

One quote! block change in one file. ~5-10 min sonnet.
