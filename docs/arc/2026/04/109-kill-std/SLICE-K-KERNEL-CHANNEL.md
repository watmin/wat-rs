# Arc 109 Slice K.kernel-channel — `:wat::kernel::Queue*` → `Channel`/`Sender`/`Receiver` family rename

**Compaction-amnesia anchor.** Read this first if you're picking
up slice K.kernel-channel mid-flight.

## Provenance

Surfaced 2026-05-01 during K.holon-lru's anchor work. Gaze ward
flagged `GetReplyPair` as needing rename to `GetReplyChannel` for
substrate-wide consistency BUT blocked the rename: `GetReplyPair`'s
body is `:wat::kernel::QueuePair<T>` — the kernel-level typealias.
Renaming HolonLRU's wrapper to `GetReplyChannel` while the body
stayed `QueuePair` would have created a Level 1 lie (Channel name,
Pair body).

The honest fix: rename the kernel layer first; everything else
falls into place.

User direction (2026-05-01):

> if we have identified a blocker we pivot - always - we do not
> compromise if a path is carved. we work on whatever kernel
> changes are necessary - now

## What this slice does

Substrate-wide rename of the kernel's channel-primitive vocabulary
to drop the `Queue*` prefix (which leaks crossbeam's
data-structure name) in favor of the canonical Channel /
Sender / Receiver family.

| Today | After |
|---|---|
| `:wat::kernel::QueueSender<T>` | `:wat::kernel::Sender<T>` |
| `:wat::kernel::QueueReceiver<T>` | `:wat::kernel::Receiver<T>` |
| `:wat::kernel::QueuePair<T>` | `:wat::kernel::Channel<T>` |
| `:wat::kernel::make-bounded-queue` | `:wat::kernel::make-bounded-channel` |
| `:wat::kernel::make-unbounded-queue` | `:wat::kernel::make-unbounded-channel` |

| File | Today | After |
|---|---|---|
| Substrate stdlib | `wat/kernel/queue.wat` | `wat/kernel/channel.wat` |

Plus `src/stdlib.rs` include path; plus the substrate's
scope-deadlock walker (arc 117) has hardcoded recognition
strings `"wat::kernel::QueueSender"` / `"wat::kernel::QueuePair"`
that need updating.

The 5 typealiases / verbs above are the substrate's published
`Queue*` family. A handful of unrelated typealiases live in
`wat/kernel/queue.wat` (`ProcessPanics`, `ThreadPanics`,
`CommResult<T>`, `Chosen<T>`) — those move with the file but
keep their own names; they're not in the Queue* rename family.

## Why this fixes the K.holon-lru block

After this slice ships:
- HolonLRU's `GetReplyPair` body becomes `:wat::kernel::Channel<T>`
- Renaming HolonLRU's typealias to `GetReplyChannel` becomes honest
  (name and body both say Channel)
- K.holon-lru can include the rename naturally as part of its
  scope, no Level-1 lie

This slice is the prerequisite. K.holon-lru rides immediately
after.

## Pattern 3 walker

**`CheckError::BareLegacyKernelQueuePath`** — fires on any keyword
matching one of the five retired patterns:
- `:wat::kernel::QueueSender` (with optional `<T>` parametric)
- `:wat::kernel::QueueReceiver`
- `:wat::kernel::QueuePair`
- `:wat::kernel::make-bounded-queue`
- `:wat::kernel::make-unbounded-queue`

Walker shape: keyword-prefix detection (same as slice 9d
stream-walker). Single function checks five prefixes; canonical
replacement is mechanical (strip `Queue` from typealias names;
rename `*-queue` → `*-channel` for verbs).

## What to ship

### Substrate (Rust + wat-stdlib)

1. **Move file**: `git mv wat/kernel/queue.wat wat/kernel/channel.wat`.
2. **Internal renames in `wat/kernel/channel.wat`** — sed-style:
   - `:wat::kernel::QueueSender` → `:wat::kernel::Sender`
   - `:wat::kernel::QueueReceiver` → `:wat::kernel::Receiver`
   - `:wat::kernel::QueuePair` → `:wat::kernel::Channel`
   - `:wat::kernel::make-bounded-queue` → `:wat::kernel::make-bounded-channel`
   - `:wat::kernel::make-unbounded-queue` → `:wat::kernel::make-unbounded-channel`
   - Inner-no-colon forms (`wat::kernel::QueueSender<T>` etc.)
   - Header doc rewritten to identify the post-K.kernel-channel
     namespace.
3. **Update `src/stdlib.rs`**:
   - `path: "wat/kernel/channel.wat"` (was `"wat/kernel/queue.wat"`)
   - `include_str!("../wat/kernel/channel.wat")` (was queue)
4. **Update `src/check.rs::ScopeDeadlock` walker**: the substrate's
   compile-time scope-deadlock detector (arc 117) has hardcoded
   recognition for `"wat::kernel::QueueSender"` / `"wat::kernel::QueuePair"`
   in `type_contains_sender_kind` / `type_is_thread_kind` helpers
   plus the `offending_kind` diagnostic field. Rename:
   - Recognition strings: `"wat::kernel::QueueSender"` → `"wat::kernel::Sender"`;
     `"wat::kernel::QueuePair"` → `"wat::kernel::Channel"`.
   - `offending_kind` field strings: `"QueueSender"` / `"QueuePair"`
     → `"Sender"` / `"Channel"`.
   - Doc comments referencing the retired names.
5. **Mint `CheckError::BareLegacyKernelQueuePath`** in
   `src/check.rs`: variant + Display + Diagnostic + walker
   `validate_legacy_kernel_queue_path` + replacement helper.
   Wire into `check_program`.

### Verification

Probe coverage:
- `:wat::kernel::QueueSender<i64>` → fires; canonical is
  `:wat::kernel::Sender<i64>`
- `:wat::kernel::Sender<i64>` → silent
- `(:wat::kernel::make-bounded-queue 1)` → fires; canonical is
  `:wat::kernel::make-bounded-channel`
- `(:wat::kernel::make-bounded-channel 1)` → silent
- ScopeDeadlock walker still fires on the new shapes (post-rename)
  — same lockstep detection, just with new names

## Sweep order

Same four-tier discipline.

1. **Substrate stdlib** — `wat/kernel/channel.wat` (just moved +
   renamed); other `wat/` files using kernel queue family
   (`wat/stream.wat`, `wat/console.wat`, `wat/holon.wat`).
2. **Lib + early integration tests** — `src/check.rs` walker
   (already updated), `src/runtime.rs` lib tests, `src/freeze.rs`
   if any.
3. **`wat-tests/`** + **`crates/*/wat-tests/`** —
   `wat-tests/stream.wat`, `wat-tests/service-template.wat`,
   `crates/*/wat-tests/**/*.wat`.
4. **`tests/`**, **`examples/`**, **`crates/*/wat/`** —
   `tests/*.rs` embedded wat strings,
   `examples/console-demo/wat/main.wat`,
   `crates/wat-telemetry/wat/telemetry/*.wat`,
   `crates/wat-lru/wat/lru/CacheService.wat`,
   `crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat`,
   `crates/wat-telemetry-sqlite/**`.

Final gate: `cargo test --release --workspace` 1476/0;
`grep -rln 'wat::kernel::Queue\|make-bounded-queue\|make-unbounded-queue'`
returns empty (or only the substrate's own legitimate recognizer
strings in src/check.rs).

## Estimated scope

- 22 files identified by survey
- ~165+ occurrences across the codebase
- Plus substrate file move + walker mint + scope-deadlock walker
  rename
- New typealiases: 0 (this is a pure rename slice)

Comparable to K.console / K.lru in size. Sonnet-tractable single
sweep with substrate-as-teacher diagnostic stream as the brief.

## Side benefit (gaze observation, slice 1d earlier)

Memory captures the shorter half-names solve the "half the
codebase spells `:rust::crossbeam_channel::*` because
`QueueSender` is too long" leak. Post-K.kernel-channel:

- `:wat::kernel::Sender<T>` is short enough to use everywhere
- The Rust-path leak (`:rust::crossbeam_channel::Sender<T>`) goes
  away as consumers prefer the substrate name
- Substrate provides the canonical name; users use it

## What does NOT change

- `:rust::crossbeam_channel::Sender<T>` / `Receiver<T>` (the Rust
  crate types themselves) — those stay; this is a kernel-typealias
  rename, not a Rust-side change. Substrate's typealiases are
  thin wrappers over crossbeam.
- The 4 unrelated typealiases in queue.wat (`ProcessPanics`,
  `ThreadPanics`, `CommResult<T>`, `Chosen<T>`) — they move with
  the file but keep their names.
- Channel semantics — `make-bounded-channel(1)` is still
  bounded(1) rendezvous; `Sender<T>` still wraps
  `crossbeam_channel::Sender<T>`. Pure naming.
- arc 117's scope-deadlock RULE — the lockstep detection still
  works; just the names of what it detects change.

## Closure (slice K.kernel-channel step N)

When sweep is structurally complete:

1. Update `INVENTORY.md` — add a "K.kernel-channel" subsection
   to § K (or its own section); mark ✓ shipped.
2. Update `J-PIPELINE.md` — slice K.kernel-channel done; remove
   from independent-sweeps backlog.
3. Update `SLICE-K-KERNEL-CHANNEL.md` — flip from anchor to
   durable shipped-record.
4. Update task #183 → completed.
5. Add 058 changelog row noting:
   - First slice to rename a substrate primitive family (kernel
     channels)
   - Unblocks K.holon-lru's `GetReplyPair` → `GetReplyChannel`
     rename
   - Solves the `:rust::crossbeam_channel::*` leak (short
     substrate names usable everywhere)
6. Then ship K.holon-lru (now unblocked).

## Cross-references

- `docs/arc/2026/04/109-kill-std/INVENTORY.md` § K — channel-
  naming-patterns subsection (the doctrine this slice extends).
- `docs/arc/2026/04/109-kill-std/SLICE-K-LRU.md` — the slice that
  rehearsed `ReqPair` → `ReqChannel`; same rationale at the
  substrate level.
- `docs/SUBSTRATE-AS-TEACHER.md` — Pattern 3 walker mechanism.
- `wat/kernel/queue.wat` (pre-K.kernel-channel) →
  `wat/kernel/channel.wat` (post).
- task #183 — this slice supersedes the follow-up task.
