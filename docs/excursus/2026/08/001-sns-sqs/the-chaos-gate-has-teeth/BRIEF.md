# BRIEF — the chaos gate has teeth

**Read `DESIGN.md` beside this first.** It carries why `fires > 0` is a **flake** (1 in ~170 at the shipped
rate), the three relations that are not, and the four things out of scope.

## The work, in one paragraph

`circuit.wat`'s chaos knobs are set by **nothing** in the floor. `:user::chaos` already drives them at
200 bp but returns `nil` and **prints**, so no Rust harness can assert on it. Make it (or a sibling) return
its summary `String`, add a 100 %-rate sibling, and write one floor harness asserting three relations that
contain **no probability**: `draws > 0`, `hits == fires`, and — at 100 % — `fires == draws`.

## The rooms

| where | why you are going there |
|---|---|
| `wat-scripts/fanout/circuit.wat:3568` | ⭑ **THE PROVEN SHAPE.** `(:wat::core::defn :user::compute [] -> :wat::core::String …)` — a zero-arg entry point that **returns** its summary. Every existing harness parses that return. Copy this shape. |
| `wat-scripts/fanout/circuit.wat` `:user::chaos` | the existing chaos entry point: `(:user::run-chaos* 2000 4 3 200 42)` then three `println`s, returning `nil`. **This is the room.** |
| `wat-scripts/fanout/circuit.wat:3529` | `:user::run-chaos* [n m j rate seed]` → `(:fanout::run-with n m j 1 rate seed 0 0 0 false 0 0 32 false 0 0 64 0)`. ⭑ Confirm for yourself that `rate` lands on `run-with`'s **`rate`** — position 5 of 18, the **disrupt** rate — before trusting the DESIGN on it. |
| `wat-scripts/fanout/circuit.wat:2878–2893` | `run-with`'s 18 named parameters, in order. This is the authority on which position is which knob. |
| `wat-scripts/fanout/circuit.wat:385–392` | the counter definitions, and the measured pair *"at the chaos gate's own 200 bp … draws=256, hits=0"* — the number this gate exists to make impossible to ship again. |
| `tests/services/probe_ex001_fanout.rs` | ⭑ **COPY THIS HARNESS.** `startup_from_source` + `FsLoader` (because `circuit.wat` does relative `load-file!`), `apply_function` on the entry point, and the `field(summary, key)` helper that splits on `;` then `=`. Its header explains why `startup_from_file` cannot work here. |
| `tests/services/probe_arc278_sane_circuit.rs` | the sibling that drives `:user::compute-calls` — a second worked example of one harness per scenario. |

## Implementation sketch

```
;; circuit.wat — a returning sibling, beside :user::chaos
(:wat::core::defn :user::chaos-fires [] -> :wat::core::String
  (:wat::core::first (:user::run-chaos* 2000 4 3 200 42)))      ;; shipped rate

(:wat::core::defn :user::chaos-fires-always [] -> :wat::core::String
  (:wat::core::first (:user::run-chaos* 50 2 2 10000 42)))      ;; 100 % — every draw fires
```

⚠ **The sketch is a convenience; the disk is the contract.** Check which element of `run-chaos*`'s
`(Tuple :- [String i64 String])` carries the **phase line** with `disrupt-draws`/`-fires`/`disrupts` —
`:user::chaos` prints `first` and `third`, so confirm which one you need rather than taking `first` from me.
My sketches have been wrong four times in this campaign and the disk was right every time.

## ⭑⭑ The assertions — three relations, zero probability

| assert | at | why it cannot flake |
|---|---|---|
| `disrupt-draws > 0` | both | the alarm is **time**-based, the run **work**-based: under load the run is longer, so draws go **UP**. n=2000 gives ~256. |
| `disrupts == disrupt-fires` | both | rate-independent — every fire tears (`15ddc5e35`), so this holds even at rate 0 |
| `disrupt-fires == disrupt-draws` | 100 % only | a 100 % rate fires on every draw |

⛔ **Do NOT assert `fires > 0` at 200 bp.** `0.98²⁵⁶ ≈ 0.6 %` — one run in ~170 reds a correct tree.
⛔ **Do NOT assert a draw count.** It was 4 and 5 on two runs of one command; `> 0` is the only safe form.

## Verify

- `./scripts/floor.sh`, read the **Summary line**. ⚠ **The count will move** — new harnesses change it; state
  the new number explicitly so nobody reads it as a shrink.
- ⛔ **Never a piped exit code** (a type-error run this session reported `$?` = 0 through `| head`; true exit
  **3**, on stderr).
- `cargo nextest run --release --no-run` — the build does NOT compile tests.
- `cargo clippy --release --workspace --all-targets -- -D warnings` → 0.
- **Run the new harness 3× and show all three results identical.** A gate whose whole claim is "no
  probability" owes that.
- Unperturbed happy path unchanged: `circuit.wat 2000 4 3 8192 true 1000` → `distinct=8000;dup=0`.
- `git diff --stat -- src/ wat/` must be **EMPTY**.
- Report the **floor time delta** — a chaos run at n=2000 is ~25 s.
- Everything heavy through `./scripts/capped.sh --limit 8g`.

## STOP triggers

1. **STOP-1 — if `draws` can be 0 on a legitimately-configured run**, STOP and report what makes it 0. The
   whole gate rests on `draws > 0` being structural, not lucky.
2. **STOP-2 — do not assert `fires > 0`, and do not assert any count.** If you believe a threshold is needed,
   STOP and say which, with its probability of failing on a correct tree.
3. **STOP-3 — do not change the rate.** 200 bp is the shipped value and tuning it is a separate ruling
   (D2-a, explicitly **not** taken).
4. **STOP-4 — do not change any counter** (`disrupt-draws`/`-fires`/`disrupts`). They are correct as of
   `15ddc5e35`; this stone reads them.
5. **STOP-5 — if `:user::chaos` has callers** that a signature change would break, STOP and name them; add a
   sibling instead of changing it.
6. **STOP-6 — do not assert the system SURVIVES chaos** (`dup=0` at a firing rate). Out of scope: this gate
   proves the injector fires and tears, not that the system tolerates it.

## Shape to copy

`tests/services/probe_ex001_fanout.rs` for the harness, and
`the-poison-tears/SCORE.md` for how the three counters relate and why the equality was chosen over a
threshold there too.
