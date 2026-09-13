# DESIGN — the poison tears

Builder: *"draw #2"* — ranked second in
`the-crash-surface-is-enumerated/FINDING-the-crash-surface.md` §5: *"fix disrupt's Malformed arm — one arm."*

**Drawn 2026-09-13. NOT STRUCK.**

## Why — a chaos knob that cannot register a hit

`disrupt-hits`'s own comment (`circuit.wat:387`):

> *"disrupt-hits — the poisoned call came back **lost/closed**: it TORE"*

and the predicate (`:544`):

```
tore? (or (= poisoned "lost") (= poisoned "closed"))
```

But the poison is a **2000-byte pad** (`10 chars × 200`, `:531–534`) sent via `Seen/check` against
`:max-frame-bytes 256` (`:102`) — and an oversized frame yields **`RecvOutcome::Malformed`**, which is
neither. Worse, the `Malformed` arm (`:543`) is an **unmigrated placeholder that `assertion-failed!`s**, so
when the poison actually fires the **worker dies** and the run cannot report at all.

⛔ **So `disrupt-hits` is structurally incapable of incrementing**, by two independent mechanisms. A prior
session measured the same zero, attributed it to **shared seeds**, fixed those, and added `disrupt-fires`
precisely to separate *"never injected"* from *"injected and did nothing"* (`:388–391` says so). With seeds
fixed and the poison genuinely sent, the answer is still zero — **two causes, one fixed.**

## ⭑⭑ AND MALFORMED REALLY IS A TEAR — measured, not argued

This is the fact the fix rests on, and I proved it before drawing:
`wat-scripts/scratch-pad/probe-crash-surface-frame-cap.wat`, my own run —

```
a-small=Message/Ok ; a-big=Malformed ; b-other=Message/Ok ; a-again=Closed
```

- `a-big` — the oversized frame on connection **A** returns `Malformed`
- `b-other` — connection **B** still works, so **the service survives**
- ⭑ `a-again` — a further call on **A** returns **`Closed`**: **connection A is dead**

**An oversized frame costs the sender its connection and leaves the service healthy.** That is exactly the
condition `tore?` exists to detect, and the redial at `:546` is exactly the right response. The arm was
never a placeholder for a *momentary* failure — it was a tear, mislabelled.

## The one contract decision

> **`tore?` means "the poisoned call cost us our connection" — and the test for that is whether the
> connection survived, never which variant name came back.**

So `Malformed` joins `lost`/`closed` in `tore?`, and the comment at `:387` is corrected to say what the
predicate now means. ★ The counter keeps its **name** and gains an accurate **definition**; this is the
counter-unit rule applied in the one direction it permits — *documenting the unit*, not silently widening
it.

⚠ `TimedOut` already maps to `"lost"` (`:542`) and so already counts as a tear. That is **conservative and
left alone**: a redial of a connection that was merely slow is harmless, and changing it is not this stone.

## ⭑ The invariant this unlocks, which is the proof

After the fix the poison **deterministically** produces `Malformed`, and `Malformed` always tears. So:

```
disrupt-fires == disrupt-hits        on every run, and both > 0 at a live rate
```

**That is a testable equality, not an eyeball.** Before the fix `hits` is pinned at 0 while `fires` can be
positive — the exact gap `disrupt-fires` was added to expose. If the two differ after the fix, something
other than the poison tore a connection and I want to know.

★ And the second, blunter proof: **the run must COMPLETE at a rate that fires.** The FINDING measured that
at `disrupt-bp=10000` the fill stalls (`arrived=0`, polls=601) because the worker dies on the placeholder.
After the fix, a high rate must finish with `dup=0`.

## Scope

**IN:** the `Malformed` arm returns a tear instead of raising · `tore?` admits it · the `:387` comment
corrected · a run at a **firing** rate that completes, with `fires == hits > 0`.

**OUT = REJECTED:**
- ⛔ **The other 60 placeholder arms.** This migrates **one**, because the builder ranked it #2 and it is
  load-bearing. ⚠ **It sets no precedent**: the replacement here is the honest answer for *this* call site
  (a tear → redial), derived from a probe of *this* failure. The remaining 60 are still last, and each
  needs its own reading. A template applied to all 61 would be the fallback-that-collapses-failures defect.
- ⛔ **Renaming `disrupt-hits` → `disrupt-tears`.** More honest, and a corpus rename for one arm's worth of
  benefit. The comment carries the definition instead. Named so the builder can overrule cheaply.
- **`RecvOutcome::Stopped`'s raise** (`:541`, *"disrupt poison stopped"*). `Stopped` means a stop was
  requested while the read was parked — **nothing died** — so raising on it during shutdown is arguably
  also wrong. Different case, different reading, not this stone.
- **The `:max-frame-bytes 256` cap or the pad size.** The mechanism is correct; only the handling was wrong.

## The four questions

**Obvious?** YES — the comment already says what the predicate should mean; the code just disagreed with it.
**Simple?** YES — one arm, one `or`, one comment. **Honest?** YES, and doubly: it stops a counter from being
permanently pinned at zero, and it stops a *chaos* injector from killing the system it is meant to perturb.
**Good UX?** YES — `disrupt-bp` becomes a knob that can be turned, which is what it was shipped as.

## Trap-doors

1. ⛔ **Do not count Malformed as a fire.** `disrupt-fires` is already correct — counted on `hit?`, where the
   poison is SENT (`:574–576` says so explicitly). Touching it would destroy the very separation that
   exposed this bug.
2. ⛔ **`points'` (`:553`) is gated on `tore?` too** — it records which draw tore. It will start recording.
   That is correct and expected; do not "fix" it.
3. **A high rate means a redial per fire.** At `disrupt-bp=10000` every alarm tick tears and redials
   `seen-addr`. Expect the run to be slower and possibly noisier; report the wall clock rather than hiding it.
4. **The redial has its own raise** (`:547`, *"redial seen failed — peer is dead, not a broken pipe"*). That
   one is correct and must stay: if the *service* is gone, this is not a momentary failure.
5. **`circuit.wat` is `wat-scripts/`, not stdlib** — no bootstrap, no stash-dance. But it is in the
   `every_wat_scripts_file_loads` gate, so a type error reddens the floor.
