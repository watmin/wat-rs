# BRIEF — STONE 255.1c-kernel-resource · HOME #7: `:Resource`'s fifteen

## You are a rider

You edit and report. **The orchestrator builds the floor, runs clippy, and commits.** Do not run
either; do not commit, push, stash, or revert. **Ending your turn ENDS you** — nothing wakes you, no
notification is coming. Run everything in the FOREGROUND and block on it.

Anchor `/home/watmin/work/holon/wat-rs/`; `pwd` first; `git -C <anchor>` for git reads. Any path with
`.claude/worktrees/` is harness state — never operate on it.

**Two commands are yours:**

```bash
systemd-run --user --scope -q -p MemoryMax=8G -p MemorySwapMax=0 timeout 900 cargo build --release
systemd-run --user --scope -q -p MemoryMax=8G -p MemorySwapMax=0 timeout 900 \
  cargo nextest run --release -E 'test(/intrinsic::tests::/)'
```

Read exit codes directly, never through a pipe.

## ⚠ Your filter is BLIND to part of the blast radius — report a SCOPED run, not a floor

`test(/intrinsic::tests::/)` does not reach `tests/diagnostics/`. Five `.edn` goldens there pin an
exact `src/runtime.rs` line, and deleting fifteen arms — **the largest shift of this campaign** — will
turn them red. Expected, mine to ratify, not yours to fix. Say *"scoped suite green; diagnostics
goldens not covered by my filter."* Do not write "the floor is green."

## ★★ THIS STONE HAS TWO DELIVERABLES AND THE SECOND MATTERS MORE

1. Fifteen verbs registered into `src/intrinsic/kernel_resource.rs`.
2. **A STRAIN REPORT: every verb you had to ARGUE into `:Resource` rather than one that landed in it.**

The builder's standing ruling: *"we continue with the names we have as seek failures to classify as we
move forward."* A naming argument in the abstract is taste; **a verb that will not classify is data.**
Fifteen bodies is the largest sample this taxonomy has faced.

**A verb that fits only after a paragraph of justification is a FINDING, not a success.** Write the
paragraph, then say plainly that it took one. Do not smooth it into a clean scorecard — the smoothing
is how a wrong taxonomy ships and stays wrong.

### Four strain candidates, named up front so you cannot quietly resolve them

`:Resource`'s axis: *"acquires, releases, or ADMINISTERS a handle whose lifetime is tracked outside
value scope."* These four test that sentence:

- **`allow` / `deny`** — grant/revoke a CAPABILITY. Is a capability a handle? The prose records that
  `:Mutate` was refused for these, which settles what they are NOT.
- **`pipe`** — CONSTRUCTS a reader/writer pair. Acquiring implies taking custody of something extant;
  `pipe` makes one. Is construction acquisition?
- **`after`** — SCHEDULES a timer. The handle is time; nobody holds it.
- **`drop`** — ⚠ **a documented NO-OP.** Its own prose: *"does not force teardown while other
  references remain."* A verb that administers nothing, in a category about administering. **Deriving
  from the name will get this wrong — read `runtime.rs:26662`.**

Report each of the four explicitly, even if it lands cleanly. "Landed without argument" is a real
answer and I need it stated.

## Read in order

1. **`docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-255.1c-kernel-resource.md`** — the stone.
2. **`src/intrinsic/kernel_error.rs`** — home #6, the most recent shape. Copy it. Note how a home can
   carry rows of different categories, and how a row whose type the gate cannot check says so.
3. **`src/intrinsic/kernel_message.rs`** — home #5, the shape for rows with NO registered scheme:
   a `//` maintainer comment naming the `infer_*` fn as the real type authority.
4. The fifteen arms and bodies — the table in the design stone maps every one. **Read all fifteen
   bodies before declaring anything.**
5. **`wat/runtime-meta.wat`** — `:Resource`'s prose. It names all fifteen. Read; do not edit.

## ⚠ Three structural traps, measured

- **`pipe`'s body is NOT in `runtime.rs`** — it is `crate::io::eval_kernel_pipe` (`src/io.rs:1573`),
  already `pub`. A rider assuming every body is in `runtime.rs` will hunt for it.
- **`spawn-thread` and `spawn-process` are INLINE BLOCKS**, not single delegate calls
  (`runtime.rs:6794`, `:6797`). Either lift each body to a named `pub(crate)` fn or wrap the block
  as-is — **your call, but say which you did and why.**
- **Gate coverage is MIXED — the first home where it is.** Five have registered schemes and WILL be
  checked (`pipe`, `drop`, `HandlePool::{new,pop,finish}`); ten have bespoke `infer_list` arms
  (`check.rs:4003–4245`) and will be SKIPPED by `None => continue`. **A green gate verifies five rows,
  not fifteen.** The ten each get a `//` comment naming their `infer_*` arm as the authority.
  **No stub schemes** — a stub existing only to be agreed with is a gate reading a copy of the truth.

## Blast radius

```
NEW   src/intrinsic/kernel_resource.rs
EDIT  src/intrinsic/mod.rs   one `mod kernel_resource;` line
EDIT  src/runtime.rs         delete 15 arms (+ replacement comments); widen delegates to pub(crate)
```

Nothing else. No `check.rs`, no `src/io.rs`, no `wat/`, no `.edn`, no test edits.

## STOP triggers — SHIP NOTHING FURTHER AND REPORT

1. **STOP-1 — a body's DOING is not "acquires, releases, or administers a handle."** This is the
   stone's PURPOSE, not its failure. Report the verb, the deciding body line, and what its DOING
   actually is. **Do not file it under `:Resource` anyway, and do not invent a new variant.**
2. **STOP-2 — routing changed.** Registration moves the lookup, never the handler.
3. **STOP-3 — you need to touch `check.rs`.** Including a stub scheme. Report and stop.
4. **STOP-4 — the blast radius is insufficient.** Name the file and why; do not widen alone.

## What "done" looks like

- `cargo build --release` exits 0
- the scoped run's **full Summary line**, labelled scoped; any failure's whole block verbatim
- **the strain report** — all fifteen, each marked `LANDED` or `ARGUED`, with the deciding body line;
  the four named candidates addressed explicitly
- your axis table (Purity / Determinism / Category) per verb, with dissent where you have it
- `git status --short` **and `git diff --numstat src/runtime.rs`** — I need the net delta for the
  goldens ratification
- the honest deltas

Runtime band: 60–90 minutes. This is the largest home of the campaign; do not rush the body-reads.
