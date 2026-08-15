# BRIEF — 296 G-1b: finish the kill (`Record::of` + `ThreadPeer`)

> **Builder's ruling, 2026-08-15:** *"Record::of ...... we killed these names a long time ago .....
> that's how you declare a record .. and constructing a record is just a map."* And: *"yeah -
> thread-peer is dead, record-of is dead -- finish the kill."*

G-1's carrier landed green (floor 4417/4417). This closes the two **retired surfaces** it exposed.
Both were found by the migration screaming at them, which is what the stone is for.

## WHY — these are zombies, not design questions

`Record::of` was retired by **arc 294.c.2a**. `wat/Record.wat:91` records it: *"base defrecord macro
routes through `aggregate-new` (the ONE nature-dispatched ctor). **Drop the `:wat::core::Record::of`
wrapper**."* The primitives were never removed, so they kept answering.

Measured census, whole corpus:

| | live callers |
|---|---|
| `:wat::core::Record::of` | **zero** |
| `:wat::holon::Record::of` | **one** — its own probe fixture |

The three live paths, none of which need it:

- **declare** — `:wat::core::defstruct` / `:wat::core::defrecord` / `:wat::holon::defrecord`
- **construct in source** — the defrecord-generated ctor → `aggregate-new`, the one nature-dispatched
  door, already registry-guarded at `runtime.rs:15812`
- **construct from data** — `#ns/Name {:field val}`, *"the substrate's uniform EDN rule"*
  (`wat/process.wat:21`)

**This deletes G-1's one blocker rather than deciding it.** Both positional-index-label fallbacks
(`{:0 v :1 v}` — `field-N` with the prefix filed off) live *inside* these two evaluators. Remove the
primitives and there is nothing left to fall back from. No ruling needed on what names a retired
constructor should carry; the question was always "why does it exist?"

## PART 1 — delete both `Record::of` primitives

| room | what |
|---|---|
| `src/runtime.rs:4799-4800` | the two dispatch arms |
| `src/runtime.rs:15598` `eval_record_of` | the evaluator (contains one positional fallback) |
| `src/runtime.rs:15678` `eval_holon_record_of` | the evaluator (contains the other) |
| `src/check.rs:4517-4530` | the two check arms |
| `src/check.rs:11731`, `:12359` | the two arg-checkers (`CALLEE` consts name them) |
| `src/check.rs:19691-19693` | the registration comment block |
| `src/rete/purity.rs:1931`, `:2019` | the two purity rows |
| `src/stdlib.rs:125` | the comment naming them |

12 mentions in `runtime.rs`, 11 in `check.rs`, 2 in `purity.rs`, 1 in `stdlib.rs`. `aggregate-new`
does **not** depend on either — they are sibling dispatch arms, and its HolonRecord arm builds its own
hologram via `build_holon_hologram`.

## PART 2 — the probe pair: keep the coverage, drop the door

`tests/types/probe_arc234_stone2a_record_primitives.{rs,wat}` — six probes. **Every one measures a
surface that OUTLIVES the primitive**, reached through it:

| probe | what it MEASURES |
|---|---|
| 1 | construction returns an Aggregate with the right class + arity |
| 2 | `:wat::core::type` returns the class FQDN |
| 3 | field-at index 0 on the constructed value |
| 4 | multi-field construction (`:myapp::Point`, 2 fields) |
| 5 | `Record/field-at` positional access |
| 7 | equality via `holon_form` |

**Re-express, do not delete.** Declare `:myapp::Voltage` (1 field) and `:myapp::Point` (2 fields) with
`(:wat::holon::defrecord …)` in the fixture, construct through the generated ctor, and keep every
assertion byte-for-byte where the value is unchanged. The fixture stops hand-building holograms —
`aggregate-new` derives them — which is itself the point: the probes then measure the live path.

Ask what each probe MEASURES before deciding how it constructs. A test whose subject is the retired
constructor itself goes; a test whose subject is `type` / `field-at` / hologram equality stays and
changes only how it builds its input. **STOP-2 below covers the case where a probe's own subject
turns out to be retired too.**

## PART 3 — `ThreadPeer` is ALREADY DELETED — do not redo, do not revert

The orchestrator deleted `make_thread_peer_pair_for_test` and its `pub use` in the working tree and
measured it: build clean, `clippy -D warnings` 0, floor **4417/4417**. Ruled dead because it is a
**superseded representation**, not because it lacked callers: the live thread peer is
`make_rust_opaque(PEER_TYPE_PATH, ThreadOwnedCell::new(Peer::from_thread(tx, rx)))` — the same two
crossbeam endpoints, sealed behind an opaque with an owner-thread invariant, inside the derive graph
(`Thread`/`Process` → `Peer` → `ThreadSelfPeer`, `wat/spawn.wat:235-272`). `ThreadPeer` exposed them
as wat-visible fields and the checker has never heard of it.

Leave that deletion exactly as you find it.

## ⛔ WHAT STAYS — two positional labels that are CORRECT

Do not "clean up" either of these. Both were weighed:

1. **`src/runtime.rs:13881` — a Newtype's `"0"`.** A newtype declares no field name; arc 049 says it
   holds exactly one inner value referred to positionally as `<Type>/0`, and
   `register_newtype_methods` bakes that same `/0` into the accessor. `"0"` is the field's **real
   name** — what a user writes to read it — not a fallback guess. Its sibling arm correctly raises
   for an unregistered class.
2. **`src/runtime.rs:35044` — a co-located Rust unit test.** Synthetic never-registered class
   `my::Pt`; the contract under test is `to_holon_inner`'s error message. The names never render.

## STOP TRIGGERS — rejections. Report and leave the site.

- **STOP-1 — a live `Record::of` caller outside the probe fixture.** The census says there is none.
  If the compiler or a `.wat` load disagrees, the census was wrong and that caller is the finding —
  do not migrate it silently.
- **STOP-2 — a probe's SUBJECT is itself retired.** If re-expressing probe N means calling another
  verb that turns out to be dead (check it has live callers before you lean on it), stop and report
  rather than reviving one zombie to keep another alive.
- **STOP-3 — re-expression would weaken an assertion.** If a probe cannot be re-expressed without
  dropping or loosening what it asserts, report it. Losing coverage quietly while deleting a
  primitive is the failure this part exists to prevent.

## BLAST RADIUS

`src/runtime.rs`, `src/check.rs`, `src/rete/purity.rs`, `src/stdlib.rs`, and the probe pair. **No
`.wat` corpus files** — nothing in `wat/` calls these. **Leave `src/edn_shim.rs`'s seven
`format!("field-{}", i)` fallbacks untouched** — still G-2; verify the count is still 7 when you
finish.

## VERIFY

`cargo build --release --tests`, then `cargo clippy --workspace --all-targets --release -- -D
warnings` (0), then `scripts/floor.sh` — read the **Summary line**. Baseline is **4417 passed / 0
failed / 263 skipped**; expect it to drop by however many probe tests you retire outright, and say
exactly which and why.

**On any red: do NOT re-run.** A re-run that goes green destroys the only evidence. Copy the failing
test's whole stdout+stderr block verbatim — never a `| head` window — name the exact assertion that
fired, and report.

## HOW TO WORK

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Run
every build and test in the FOREGROUND and block on it. Anchor at `/home/watmin/work/holon/wat-rs`;
`pwd` first. Leave the work uncommitted; the orchestrator weighs and commits.

Report: what was deleted, each probe's disposition (re-expressed / retired, with the reason), the
floor Summary line verbatim, every STOP, and the honest deltas — especially anywhere this brief did
not match the disk.
