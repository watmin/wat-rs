# SCORE — the harness takes a record; (Some 0) is zero

**SCORED.** Executor: grok, 2026-09-14, branch `sns-sqs`, HEAD `08b3234d5` (DRAWN). Did not commit.

⛔ The headline is that **`None` and `(Some 0)` are different on `:fanout::Input`**.
`vis-ms` / `inbox-vis-ms` / `inbox-cap` no longer have an unreachable value.
Positional argv is gone. No fallback.

```
     Summary [ 553.539s] 5247 tests run: 5247 passed (9 slow), 22 skipped
```

`.floor/2026-09-14T23-31-14Z/` — `scripts/floor.sh` exit **0**, **no `ARM.txt`**.
Count **5247**, unchanged. Clippy **0**. NORUN **0**.
`git diff --stat -- src/ wat/ tests/ scripts/ docs/` **EMPTY**.

Hand-edited `wat-scripts/fanout/circuit.wat` only (4059 → 4126). Not a wat-fix.
Delay knobs (`delay-bp`, `delay-ms`) are in the record — they had landed
(`08b3234d5`).

---

## ⭑ THE HEADLINE — a record, and zero is reachable

Happy path (every Option named; `nil` = None; `:vis-ms 1000` = Some 1000):

```
printf '#fanout/Input {:n 2000 :m 4 :j 3 :sub-cap 8192 :fill-first? true :vis-ms 1000 :p nil :rate nil :seed nil :drop-check-bp nil :drop-mark-bp nil :drop-seed nil :drop-after? nil :drop-recv-bp nil :drop-ack-bp nil :inbox-vis-ms nil :inbox-cap nil :chaos-bp nil :delay-bp nil :delay-ms nil}\n' \
  | ./target/release/wat wat-scripts/fanout/circuit.wat
```

```
distinct=8000;dup=0
vis-ns=1000000000;inbox-vis-ns=200000000;inbox-cap=64
bp-delay=0;delay-draws=0;delays-fired=0
```

Row 2 — `(Some 0)` reaches the service as **0**, not the default:

```
:vis-ms 0   →  vis-ns=0
```

That value was unreachable under `> vis-ms 0`. It is real: n=12 fill-not-first
comes back `distinct=24;dup=2` (zero visibility redelivers). The dups are the
consequence; `vis-ns=0` is the gate.

Row 3 — `None` still means default. Wire spelling is **`nil`**, not omit:

```
:vis-ms nil  →  vis-ns=1000000000000   ;; 1000 s, no drops
```

Row 4 — a record missing `:n` is **refused**:

```
struct :fanout::Input has no field named n
```

---

## The four overloaded zeros, each read (row 5)

HEAD line numbers. Not a blanket `0 → None`.

| site | what 0 meant | conversion |
|---|---|---|
| `:2920` | vis-ms sweep; `> vis-ms 0`; **0 = default** (200 ms if any drop rate, else 1000 s) | `None` → that default. `(Some ms)` → `ms * 1e6` **including 0** |
| `:2929` | inbox-vis ms→ns; **0 = the default** | `None` → `:fanout::inbox-vis-default-ns`. `(Some ms)` → `ms * 1e6` including 0 |
| `:3666` | argv 11 inbox-vis-ms; **"0 means the default, NOT no redelivery"** | same arm as `:2929`. The sharp one. Callers that passed 0 meaning "I don't care" became `None`, not `(Some 0)` |
| `:3699` | argv 12 inbox-cap; **0 means 64** | `None` → 64. `(Some 0)` = ZERO (was unreachable) |

Other knobs (rate, seed, drop-*-bp, chaos-bp, delay-*) already meant 0 as a
value. Those stay `opt-i64 … 0`. `p` None→1. `drop-after?` None→false.

---

## STOP-4 — `run-with` takes the record

```
(:wat::core::defn :fanout::run-with [in <- :fanout::Input] …)
```

Unpacks once. No second positional shape. `:fanout::input` is the
bare-required / every-knob-None constructor the `:user::run*` family uses.

`p` is a field. Still not a reachable CLI knob (wiring is its own stone).
`:user::run-p*` still takes it.

---

## STOP-3 — Eof / Stopped raise

Copied the migration shape (`angle-brackets-to-binder.wat:282`). Both arms
`assertion-failed!`. A harness without a record is a usage error, not a
defaulted n=2000.

```
./target/release/wat wat-scripts/fanout/circuit.wat < /dev/null
```

```
circuit: stdin is #fanout/Input {…} — got EOF
```

`readln` still blocks on an open unread pipe (30 s floor wall). Callers must
provide stdin. `/dev/null` is Eof, not a hang.

---

## Findings named, not papered over

1. **Omit is refused; `nil` is None.** `reconstruct_record` requires every
   declared field. DESIGN's printf (required keys only) errors
   `has no field named vis-ms` — same path as missing `:n`. EXPECTATIONS row 3
   said "omit `:vis-ms`". That spelling cannot be done on this type. The
   contract (`None` = default) holds via `nil`.
2. **`(Some 0)` for vis-ms is not a no-op run.** vis-ns=0 redelivers. The
   phases key is how you see it without guessing from dups.
3. **The four Rust files do not pipe main.** They call `:user::compute` /
   `:user::chaos-fires` / `:user::pending-only-loses` / `:user::durable-ok`
   (and siblings). `:user::` is the test-harness rendezvous; they never saw
   argv. Left unchanged. Isolated 5/5 pass. Porcelain `tests/` **empty**.
4. **`scripts/capped.sh` is arg-agnostic.** Header comment still shows the
   old positional example. Not a caller. Not rewritten (not `docs/**` either).

Phases line gained `vis-ns`, `inbox-vis-ns`, `inbox-cap` so Some 0 is
observable. Existing completeness counters on the happy path hold
(`distinct=8000;dup=0`; delay zeros). Timing numbers always move.

---

## WHAT LANDED

`wat-scripts/fanout/circuit.wat`
- `:fanout::Input` — required `n m j sub-cap fill-first?`; Option for every
  knob that had a default, including delay-bp/delay-ms.
- `:fanout::opt-i64` / `:fanout::opt-bool` / `:fanout::input`.
- `:fanout::run-with` takes `[in <- :fanout::Input]`.
- `:user::main` — `readln` Datum → `run-with`; Eof/Stopped raise. No argv.
- `:user::run*` / `run-p*` / `run-chaos*` / `run-drop*` / `drop-recv-tiny` /
  `drop-ack-tiny` / `delay-full-rate` construct Input. Positional 0-meaning-
  default → None; explicit rates → Some.
- `:user::vis-zero-reaches` — in-process row 2 (n=12, vis-ms Some 0).

Namespace is `:fanout::`, not `:user::`.

---

## Floor delta (row 12)

**553.539 s** vs baseline **535.565 s**. Delta **+18.0 s**. Parsing one record
vs 15 strings; ~0 expected. The number contains the measurer. Observation,
not a gate. Count unchanged.

---

## GRADING MAP

| # | expected | result |
|---|---|---|
| 1 | ⭑⭑ equivalence | `distinct=8000;dup=0`; delay zeros; floor callers pass. New keys only |
| 2 | ⭑ defect fixed | `:vis-ms 0` → `vis-ns=0` |
| 3 | ⭑ None = default | `:vis-ms nil` → `vis-ns=1000000000000`. Omit refused (finding 1) |
| 4 | ⛔ missing `:n` refused | `has no field named n` |
| 5 | four zeros named | `:2920` `:2929` `:3666` `:3699` above; not a blanket |
| 6 | Eof/Stopped decided | both raise. `/dev/null` → the Eof string |
| 7 | callers converted | 0 test files (they do not pipe main). `capped.sh` untouched |
| 8 | ⛔ docs untouched | `git status -- docs/` empty |
| 9 | tests compile | NORUN=0 |
| 10 | floor | **5247 passed**, 0 FAIL |
| 11 | clippy | 0 |
| 12 | floor delta | **+18.0 s** |
