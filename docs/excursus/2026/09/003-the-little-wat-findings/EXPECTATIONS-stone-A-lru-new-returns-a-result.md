# EXPECTATIONS — STONE A

Written before the strike. Scored against my own re-run, never the executor's report.

| # | what | command | expected |
|---|---|---|---|
| 1 | a non-positive capacity is a wat error | `wat` on a program calling `(:wat::cache::Lru/new 0)` | a wat error naming **`:wat::cache::Lru/new`** |
| 2 | ⭐ no Rust panic reaches the user | same, stderr | **no** `RUST_BACKTRACE`, **no** `thread 'main' panicked` |
| 3 | it carries the user's span | same | `:file` names the user's `.wat`, not a `.rs` |
| 4 | HolographicLru propagates | same via `HolographicLru/new 0` | a wat error, not a panic |
| 5 | the happy path is unchanged | `wat-tests/cache/HolographicLru.wat` | passes |
| 6 | the durable rebuild still works | `lru-svc` exercised by its tests | passes |
| 7 | ⭐ the gate can fail | restore the `panic!`, re-run the new test | RED, naming the case |
| 8 | …and recovers | restore the cure | green |
| 9 | F-083's row still pins its defect | the findings board | `every_row_holds_its_measured_verdict` passes with F-083 present |
| 10 | the floor | `scripts/floor.sh` → `.floor/latest/clean.log` | 0 failed |
| 11 | clippy | `cargo clippy --release --all-targets` | clean |
| 12 | the note no longer asks for a mandate | `head docs/arc/2026/04/109-kill-std/NOTE-the-cache-lru-…md` | status records the mandate and what shipped |

⛔ Read every verdict from the `Summary` line, never a piped exit code — a `cargo nextest … | tail`
returns `tail`'s status, and this session has already seen `[exited with code 0]` printed over a
run whose Summary said **2 failed**.

## Runtime prediction

Rust edit + wat call-site moves: 45-75 min. ⚠ **Build cost dominates:** a cold release harness
build measured **3m 15s** here, `cargo build --release` alone **1m 07s**, and the floor is now
**~25 minutes** (the wat-scripts gate alone is 425s). Budget 2-3 hours, most of it compiling.

⛔ **Do not poll for a build with `pgrep -f 'cargo …'`** — the pattern matches its own command line
and the loop never exits. That cost this session an hour.

## Trap doors, named in advance

1. **`Result<Self, E>` through `#[wat_dispatch]` had a real bug once** — the macro re-quoted `Self`
   into a generated free fn where it does not resolve. It was fixed at the root (`result_ok_is_self`)
   and `Sqlite::open` proves the arm works. If you hit anything resembling it, you are off the
   supported path: STOP.
2. **A service `:init` may not be able to express a failing rebuild.** That is STOP-2 and it is the
   most likely blocker: the durable path is the motivating case, so an `:init` that cannot fail
   would mean the cure cannot reach the very defect it was mandated for.
3. **`HolographicLru/new` has no guard of its own** — it inherits `Lru/new`'s. Do not add a second
   guard there; propagate the one Result.
4. ⚠ **The F-083 board fixture is a caller.** See the BRIEF. Changing it is in scope; curing F-083
   is not.
5. **Two `.wat` corpora that merely NAME the verb** (`wat-scripts/fixes/type-member-colon-to-slash.wat`,
   `src/remedy/retirement.rs`) are string tables, not call sites — check before editing either.
