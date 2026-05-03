# Arc 138 F-NAMES-1e — Sonnet Brief: name wat-side spawned threads

**Goal:** name the 3 unnamed `thread::spawn` sites in the substrate that run wat-side spawn primitive workers. After this, every wat-side spawned thread has a meaningful name (`wat-thread::<derived>` or similar), and `thread::current().name()` returns Some(...) wherever wat code panics.

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

**Driver:** F-NAMES-1d-asserthook spot-check revealed upstream gap. Wat assertions on `:wat::kernel::spawn` workers still show `<unnamed>` because those workers have no name. F-NAMES-1e closes the upstream gap.

## Read in order

1. `docs/arc/2026/05/138-checkerror-spans/SCORE-F-NAMES-1D-ASSERTHOOK.md` — predecessor; identifies these 3 sites.
2. `docs/arc/2026/05/138-checkerror-spans/SCORE-F-NAMES-1C.md` — the wat::test! Builder::new().name(...) pattern to mirror.
3. The 3 spawn sites:
   - src/spawn.rs:183 — `:wat::kernel::spawn` thread/process worker
   - src/runtime.rs:12421 — Thread<I,O> spawn primitive
   - src/runtime.rs:18780 — service spawn worker

## What to do

For each `std::thread::spawn(move || { ... })`, replace with:
```rust
std::thread::Builder::new()
    .name(format!("wat-thread::{}", <derived-name>))
    .spawn(move || { ... })
    .expect("Thread::Builder::spawn failed")
```

Where `<derived-name>` is the most informative identifier in scope. Options:
- The body lambda's name (if the wat code did `(spawn :my::handler)` — the handler keyword IS the name).
- The call-site span (file:line:col format).
- A combination like `<lambda-name>@<file>:<line>` when both are available.

Investigate per-site to pick the cleanest derivation. Each site likely has different context.

## Constraints

- 3 files modified: src/spawn.rs + src/runtime.rs (2 sites).
- All 7 arc138 canaries continue to pass.
- Workspace tests pass: `cargo test --release --workspace 2>&1 | grep FAILED | grep -v trading` returns empty.
- NO commits, NO pushes.

## Verification spot-check

After fix, run `RUST_BACKTRACE=1 cargo test --release --workspace 2>&1 | grep -E "^thread '"` and confirm:
- Wat-test deftest assertion failures show `wat-thread::<name>` instead of `<unnamed>`.
- The full chain: wat::test! deftest worker (`wat-test::<deftest>`) spawns a wat-spawn worker (`wat-thread::<lambda>`) which runs the assertion. The assertion-hook output should now show `wat-thread::<lambda>` (the actual panicking thread).

## Reporting back

Compact (~250 words):
1. Diff stat (3 files, 6+/3- approximately).
2. 3 spawn sites updated with derivation chosen for each.
3. Verification: 7/7 canaries; workspace; spot-check confirms `wat-thread::<name>` renders.
4. Honest deltas (any spawn site where derivation was ambiguous).
5. Four questions briefly.

## Why this is small

3 spawn sites + Builder pattern from F-NAMES-1c. ~10-15 min sonnet.
