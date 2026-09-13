# SCORE — the chaos gate has teeth

**SCORED.** Executor: grok, 2026-09-13, branch `sns-sqs`, HEAD `0b87331fa` (DRAWN). Did not commit.

```
     Summary [ 532.364s] 5239 tests run: 5239 passed (9 slow), 22 skipped
```

`.floor/2026-09-13T04-12-55Z/` — `scripts/floor.sh` exit **0**, **no `ARM.txt`**.
**The count moved: 5237 → 5239** (+2 tests). Not a shrink.
Clippy: `cargo clippy --release --workspace --all-targets -- -D warnings` → **CLIPPY=0**.
`git diff --stat -- src/ wat/` is **EMPTY**.
`:user::chaos` still `2000 4 3 200 42` (hand-typed, prints, returns `nil`). Rate **200 bp** untouched. Counters untouched.

Floor time: previous green `521.773s` (`.floor/2026-09-13T02-48-24Z/`) → **532.364s**. Delta **+10.6 s**. Reported, not hidden.

---

## ⭑ THE HEADLINE — an inert injector is now a red

`grep -l disrupt tests/**/*.rs` → **1 file**: `tests/services/probe_chaos_gate_has_teeth.rs` (was 0).

Two returning siblings, returning **`third`** of `run-chaos*` (the phase line). The sketch said `first`. **`first` is n=/distinct=; the injector census is on `third`.** Followed the disk.

```
(:user::chaos-fires)        → third of run-chaos* 50 2 2 200 42
(:user::chaos-fires-always) → third of run-chaos* 50 2 2 10000 42
```

STOP-5: `:user::chaos` has **no** rust/wat callers (only the defn + docs). Left it printing. Added siblings.

---

## ⚠ FIRST FLOOR WAS RED. Did not re-run that log.

`.floor/2026-09-13T04-00-06Z/`

```
     Summary [ 531.656s] 5239 tests run: 5238 passed (8 slow), 1 timed out, 22 skipped
     TIMEOUT [  40.033s] (3964/5239) wat::services probe_chaos_gate_has_teeth::chaos_gate_shipped_rate_draws_and_hits_eq_fires
```

ARM.txt: `(test timed out)` at 40.033s. Isolated, the same test passed in **24.017 s** (n=2000). Under `nice -n 19` floor contention it crossed the default 40 s. **Not a flake. Did not add a nextest timeout. Did not re-run that floor.**

Fix: the **floor sibling** is n=50, **same 200 bp**. `:user::chaos` stays n=2000. Isolated n=50 is ~9 s.

---

## ROW 8 — the gate fails when pointed at rate 0

Temporarily `:user::chaos-fires` → `run-chaos* 2000 4 3 0 42`. No counter edit. Harness assertion `draws > 0` went RED. Restored 200.

```
     Summary [  24.490s] 1 test run: 0 passed, 1 failed, 5260 skipped
        FAIL [  24.479s] wat::services probe_chaos_gate_has_teeth::chaos_gate_shipped_rate_draws_and_hits_eq_fires
    panicked at tests/services/probe_chaos_gate_has_teeth.rs:73:5:
    inert injector: disrupt-draws must be > 0 at the shipped 200 bp (alarm is time-based); got … disrupts=0;disrupt-fires=0;disrupt-draws=0; … bp-disrupt=0
ROW8=100
```

---

## ROW 7 — three runs. Relations identical; counts move.

That is why the gate does not pin a draw count.

| run | chaos-fires (200 bp) | chaos-fires-always (10000 bp) |
|---|---|---|
| 1 | draws=8 fires=0 hits=0 | draws=6 fires=6 hits=6 |
| 2 | draws=10 fires=0 hits=0 | draws=8 fires=8 hits=8 |
| 3 | draws=5 fires=0 hits=0 | draws=7 fires=7 hits=7 |

3/3: `draws > 0`. 3/3: `hits == fires`. 3/3 at 100%: `fires == draws`.
At 200 bp, **fires was 0 all three times** — the 1-in-170 `fires > 0` flake, demonstrated. Not asserted.

(Earlier n=2000 3×, before the timeout fix: fires=9/9/9 hits=9/9/9 draws=420/400/411 — same relations, larger n.)

---

## GRADING MAP

| # | expected | result |
|---|---|---|
| 1 | ⭑⭑ floor harness drives chaos | `probe_chaos_gate_has_teeth.rs` — 1 file, was 0 |
| 2 | ⭑⭑ `draws > 0` both rates | both tests |
| 3 | ⭑⭑ `hits == fires` both rates | both tests |
| 4 | `fires == draws` at 100 % | `chaos_gate_full_rate_fires_eq_draws` only |
| 5 | ⛔ no `fires > 0`, no pinned count | only in a comment forbidding it |
| 6 | entry point returns String | `:user::chaos-fires` / `-always` → String. `:user::chaos` still nil+print |
| 7 | ⭑ 3× identical | relations 3/3; counts move (table above) |
| 8 | ⭑ inert is a red | rate-0 → `disrupt-draws=0` FAIL, quoted |
| 9 | ⛔ rate unchanged | `:user::chaos` still 200 |
| 10 | ⛔ counters unchanged | no diff on those fields |
| 11 | no stdlib/Rust-substrate | `src/` `wat/` empty |
| 12 | happy | `distinct=8000;dup=0` |
| 13 | floor count stated | **5239** passed |
| 14 | clippy 0 | CLIPPY=0 |
| 15 | floor time delta | +10.6 s (521.773 → 532.364) |

Did not assert `dup=0` under chaos (STOP-6).

---

# ⭑ ORCHESTRATOR'S GRADING — claude, 2026-09-13

**Graded against my OWN reads and runs**, from `0b87331fa` + the working tree.

```
floor   .floor/2026-09-13T04-25-40Z/  Summary [ 522.203s] 5239 tests run: 5239 passed, 22 skipped
        0 failure tokens · no ARM.txt        count 5237 → 5239 (+2) — NOT a shrink
clippy  --release --workspace --all-targets -D warnings → exit 0
happy   distinct=8000;dup=0
delta   my own 518.705s → 522.203s = +3.5 s   (grok measured +10.6 s — floor-time variance)
blast   circuit.wat +15 lines, 1 new harness · git diff --stat -- src/ wat/ EMPTY
```

**STRUCK. All 15 rows, and the two I said I would not take on claim were reproduced myself.**

## ⭑⭑ Row 8 — I reproduced the red, on my own run

I wrote *"row 8 is the row I will not accept a claim for."* So I pointed the shipped-rate sibling at rate 0
myself and ran it:

```
FAIL [ 9.549s] wat::services probe_chaos_gate_has_teeth::chaos_gate_shipped_rate_draws_and_hits_eq_fires
  inert injector: disrupt-draws must be > 0 at the shipped 200 bp (alarm is time-based);
  got setup=6841;…;disrupts=0;disrupt-fires=0;disrupt-draws=0;…;bp-disrupt=0
```

Restored, verified: `200` back (count 1), diff is the intended +15 lines only. ⭑ **The gate has been seen to
fail, and it needed no forced logic edit** — the rate-0 arrangement in EXPECTATIONS was built so row 8 could
not collide with STOP-4, and it held.

## ⭑⭑ Row 7 — three runs of my own, and the counts move by 3×

```
             200 bp                    10000 bp
run 1   draws=8  fires=0 hits=0   draws=8  fires=8  hits=8
run 2   draws=12 fires=0 hits=0   draws=4  fires=4  hits=4
run 3   draws=6  fires=0 hits=0   draws=11 fires=11 hits=11
```

`draws > 0` 3/3 · `hits == fires` 3/3 (including `0 == 0`) · `fires == draws` 3/3 at 100 %. **All three runs
passed.** `draws` ranged **4 → 12** across six runs — which is precisely why no count is pinned.

## ⛔⛔ AND MY "1-IN-170" WAS WRONG BY TWO ORDERS OF MAGNITUDE — for a reason that compounds

My DESIGN rejected `fires > 0` at `0.98²⁵⁶ ≈ 0.6 %`. **That arithmetic was for n=2000 (256 draws).** The
floor sibling ships at **n=50**, where draws is **4–12**, so
`P(zero fires) = 0.98⁸ ≈ 85 %`.

**`fires = 0` in all three of my runs and all three of grok's — six for six.** The row I rejected would have
reddened a correct tree on nearly every run, not one in 170.

★★ **And the reason it got worse is a fix for something else.** The first floor **timed out** at n=2000
(below), so the sibling dropped to n=50 — cutting draws by ~40× and the flake probability up from 0.6 % to
~85 %. ⭑ **A probability-based row's flake rate is a function of a parameter another stone may change.** A
relation has no such coupling: `hits == fires` held at every n, every rate, and at `fires = 0`.

## ⚠ A FLOOR WENT RED AND IT IS STILL ON DISK — I verified, not assumed

`.floor/2026-09-13T04-00-06Z/` — **`ARM.txt` present**, `exit=100`:

```
TIMEOUT [ 40.033s] (3964/5239) …::chaos_gate_shipped_rate_draws_and_hits_eq_fires
Summary [ 531.656s] 5239 tests run: 5238 passed (8 slow), 1 timed out, 22 skipped
```

Isolated it passed in 24.017 s; under `nice -n 19` floor contention it crossed the default 40 s wall.
**No nextest timeout was added. That floor was not re-run.** The fix was to size the *floor* sibling at
n=50 while `:user::chaos` keeps n=2000.

⛔ **And my brief is why it happened.** I wrote *"a chaos run at n=2000 is ~25 s — report the floor time
delta"* — I costed it and **never mentioned the wall**, although the breadcrumb has carried *"three tests at
19–25 s against a 30 s terminate wall, under 1.2× headroom"* for three sessions. I handed over a ~25 s test
for a floor I already knew had ~1.2× headroom. **That is the third thing this session my own record knew and
my brief did not say.**

## ★ My sketch was wrong again — and the warning I attached to it worked

The sketch said `first` of `run-chaos*`'s triple. **`first` is the `n=/distinct=` line; the injector census
is on `third`.** Fifth wrong sketch this session — but the BRIEF said *"confirm which one you need rather
than taking `first` from me,"* and the strike did exactly that. ⭑ Flagging my own unreliability in the brief
is cheap and it is working; the sketches are not getting better, and the checking is.

## The rows

| # | row | my verdict |
|---|---|---|
| 1 | ⭑⭑ a floor harness drives chaos | ✅ `probe_chaos_gate_has_teeth.rs` — 1 file, was **0** |
| 2 | `draws > 0` both rates | ✅ both tests, read |
| 3 | `hits == fires` both rates | ✅ both tests, read |
| 4 | `fires == draws` at 100 % | ✅ full-rate test only |
| 5 | ⛔ no threshold on a random quantity | ✅ the only `fires > 0` in the file is a **comment forbidding it** |
| 6 | entry point returns a String | ✅ two siblings return; `:user::chaos` left printing (STOP-5 — no callers) |
| 7 | ⭑ 3× | ✅ **my own three runs**; relations 3/3, counts 4→12 |
| 8 | ⭑ inert is a red | ✅ **my own run**, quoted above, restored and verified |
| 9 | ⛔ rate unchanged | ✅ `:user::chaos` still 200, and the sibling restored to 200 |
| 10 | ⛔ counters unchanged | ✅ absent from the diff |
| 11 | no stdlib/Rust-substrate | ✅ `src/` `wat/` empty |
| 12 | happy path | ✅ my own run, `8000/0` |
| 13 | floor count stated | ✅ **5239** on my own run, +2, stated as growth |
| 14 | clippy | ✅ my own run, 0 |
| 15 | floor time delta | ✅ +3.5 s mine, +10.6 s theirs — both reported |

## What I'd credit above all

**It captured a floor red, kept the `ARM.txt`, did not add a timeout to silence it, and did not re-run the
log.** The easy move was a `slow-timeout` override; the honest one was to shrink the scenario and leave the
evidence. I verified the directory and its `ARM.txt` are still on disk rather than taking the claim.

## What this hands the next stone

D2 is closed: an inert injector cannot ship. **Next is D1-c** — *ask the store* — and this gate now watches
the chaos path it will change. ⚠ Two things for its brief that this stone taught: **state the floor's
timeout wall, not just the runtime**, and **prefer a relation to a threshold**, because a threshold's flake
rate depends on parameters a later stone may move.
