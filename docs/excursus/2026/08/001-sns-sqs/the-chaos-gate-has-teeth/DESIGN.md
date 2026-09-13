# DESIGN — the chaos gate has teeth

Builder's ruling **D2-b**: *"a FLOOR harness drives circuit.wat with chaos argv and fails on an inert
injector."* Chosen first of the five because it is a **gate**, and the chaos path should be watched by the
floor before D1-c changes what happens on it.

**Drawn 2026-09-13. NOT STRUCK.**

## The measured ground

**Nothing automated has ever set a chaos knob.** All three floor harnesses that drive `circuit.wat` —
`probe_ex001_fanout.rs`, `probe_arc278_sane_circuit.rs`, `probe_async_publish.rs` — contain **zero**
mentions of `disrupt` or `bp`. The chaos line is a hand-typed command.

⭑ **And the reason is one line of wat, not a missing feature.** `circuit.wat` already has the entry point:

```
(:wat::core::defn :user::chaos [] -> :wat::core::nil
  (:wat::core::let [triple (:user::run-chaos* 2000 4 3 200 42)] … println … println … println))
```

`run-chaos*`'s 4th argument lands on `:fanout::run-with`'s **`rate`** (position 5 of 18) — **the disrupt
rate.** So `:user::chaos` is the chaos gate, at **200 bp**, at n=2000 — and it is exactly the *"chaos gate's
own 200 bp"* whose measured pair `circuit.wat:391` records as **`draws=256, hits=0`**.

⛔ **It is unharnessable because it returns `nil` and PRINTS.** `:user::compute` returns a `String` and every
harness parses it with a `field(summary, key)` helper. `:user::chaos` hands a Rust caller nothing to assert
on. ⚠ `:user::phased` and `:user::deadline-redial-is-fresh` are driven by **nothing** either — same shape,
same reason.

## ⭑⭑ THE GATE MUST CONTAIN NO PROBABILITY, AND MY OWN FOUR-QUESTIONS GOT THIS WRONG

I proposed *"asserts `fires > 0`"*. **That is a flake**, and the arithmetic says so: at the shipped 200 bp
with **256 draws**, `P(zero fires) = 0.98²⁵⁶ ≈ 0.6 %`. One run in ~170 goes red on a correct tree — on a
floor that runs many times a day. *Can this row fail while the change is correct?* **Yes.** So it is the
wrong row.

The deterministic facts are **relations**, not thresholds:

| assertion | why it cannot flake |
|---|---|
| `disrupt-draws > 0` | the alarm is **time**-based and the run is **work**-based, so under load the run takes longer and draws go **UP**. The flake direction is a run finishing too fast, and n=2000 gives ~256. |
| `disrupts == disrupt-fires` | **rate-independent.** Every fire tears (`the-poison-tears`, `15ddc5e35`), so hits must equal fires at *any* rate, including zero. |
| `disrupt-fires == disrupt-draws` **at 10000 bp** | a 100 % rate fires on every draw. Measured 5/5/5 (mine) and 4/4/4 (grok's). |

★ **So "inert" gets a precise definition the floor can hold:** `draws == 0` (never armed or never ticked) or
`hits ≠ fires` (fired and did nothing). Both are the failures that hid for two sessions, and neither
involves a die roll.

## The one contract decision

> **Two scenarios, and both return their summary as a `String`.** One at the **shipped** 200 bp asserting
> the two rate-independent relations; one at **10000 bp** asserting `fires == draws`. Neither asserts a
> threshold on a random quantity.

★ Returning a `String` rather than printing is what makes an entry point harnessable, and it is the
established shape — `:user::compute` does exactly this and every existing harness parses it the same way.

## The four questions

**Obvious?** YES — an injector that never fired should be a red, and "inert" is defined as two equalities.
**Simple?** YES — one entry point returns instead of printing, plus one sibling and one harness file.
**Honest?** YES, and it is the point: the knobs currently *look* covered because they exist. It is also
honest about its own limits — it gates that the injector **fires and tears**, not that the system survives
anything in particular. **Good UX?** YES — a future inert injector cannot ship, which is the failure this
campaign paid two sessions for.

## Scope

**IN:** `:user::chaos` (or a returning sibling) yields its summary `String` · a 10000 bp sibling · one
harness file asserting the three relations · the harness named so its purpose is legible from the file list.

**OUT = REJECTED:**
- ⛔ **Tuning the 200 bp rate.** The builder ruled D2-b, not D2-a, and the rate is a separate question. This
  stone makes inertness *visible*; it does not choose a number. ⚠ At 200 bp `fires` will usually be small
  and occasionally 0 — **which is why `fires > 0` is not a row here.**
- ⛔ **Harnessing `:user::phased` / `:user::deadline-redial-is-fresh`.** Same unharnessable shape, real gap,
  **not this stone** — naming them so the pattern is on record.
- ⛔ **Asserting the system SURVIVES chaos** (`dup=0` at a firing rate). Tempting and out: at 10000 bp the
  circuit's behaviour is the subject of other stones, and conflating "the injector works" with "the system
  tolerates it" is how a gate ends up testing two things and pinning neither.
- **Changing any counter.** `disrupt-draws` / `-fires` / `disrupts` are correct as of `15ddc5e35`; this
  stone only reads them.

## Trap-doors

1. ⛔ **`fires > 0` at 200 bp is a 1-in-170 flake.** Named above because it is the obvious row and it is
   wrong.
2. ⛔ **Do not assert a draw COUNT.** `draws` was 4 and 5 on two runs of the same command at n=50, and ~256
   at n=2000. `> 0` is the only safe form.
3. **`:user::chaos` currently returns `nil`.** If its signature changes, check for callers — including
   `:user::main`'s dispatch and any `wat-scripts/` caller — before assuming it has none.
4. **The floor grows by a chaos run.** ~10 s at n=50, and `:user::chaos` is n=2000 (~25 s). Report the floor
   time delta; if it is large, say so rather than hiding it. A 10000 bp run also **redials per fire**.
5. **`circuit.wat` is in `every_wat_scripts_file_loads`** — a type error reddens the floor immediately.
6. **`5237` will change.** New deftests/harnesses move the count; state the new number rather than letting a
   reader think the floor shrank.
