# SCORE — the reactor grows a seam

**NOT STRUCK. STOP-1 fired.** Executor: grok, 2026-09-04. Nothing landed; tree clean of code.

The DESIGN's seam has the wrong type, and its site table has the wrong shape. Both are mine.

## ⛔ STOP-1 — the type I did not probe

```
:pr::send-keep-serving?: parameter #1 expects (:wat::kernel::Peer :- [:wat::core::Never :?32]);
  got (:wat::kernel::Peer :- [:pr::Reply :pr::Op])

:wat::kernel::send: parameter payload expects :wat::core::Never; got :R
```

`:- [R]` **expressed fine** — these are not parse errors. The failure is orientation: **`send`
projects `I` from `Peer :- [I O]`**, and I wrote `Peer :- [Never R]`.

★ **That is the timer shape.** `service.wat` says it in its own comment: *"`after` is honestly typed
`(Peer :- [Never O])` — it can never RECEIVE a Reply."* I lifted the timer's orientation and applied
it to a reply peer, so the helper tried to send `:R` on a peer whose `I` is `Never`. The selectable
is `Peer :- [Reply Op]` (`service.wat:1297`). **Both errors are that one swap.**

The seam needs **two type parameters** — `Peer :- [R O]` — not one. The executor **did not try it**:
*"That is a different seam."* Correct. STOP means stop, and improvising a second design mid-strike is
how a brief becomes fiction.

## ⛔ AND MY SITE TABLE WAS WRONG — verified by me

I wrote *"ten sites, two shapes."* Classified before any edit, and I confirmed each:

| lines | what |
|---|---|
| `1659 1697 1784 1811 1854` | **the bool shape** — `Stopped → false`. **Five, not ten** |
| `1828` | bool-shaped but **`Stopped → true`**, result discarded by `do` — a different disposition |
| `1939 1950` | same logic, but arm bodies are **recursive tail calls / nil**, not bool |
| `2006 2012` | `send self` status; all four arms `nil` — leave |

19 `kernel::send` in the file. I named ten and mis-shaped four of them.

★ **Third factual error about this one file in two messages.** *"3120 lines, exactly one top-level
form"* (it is nine, and 1422 lines are comments). *"Ten sites, two shapes"* (five in the shape).
*`Peer :- [Never R]`* (needs `[R O]`). I have been reasoning about a template I kept not reading —
and the line count I used as an excuse was the only part I had measured.

## ★ THE EXECUTOR REFUSED A GREEN ROW, AND WAS RIGHT

> *"Floor / circuit: not run. Nothing changed. **A green floor of the unextracted corpus is not proof
> of an extraction.**"*

My EXPECTATIONS made the floor *"both necessary and sufficient for faithfulness."* That is true
**after** the extraction and vacuous before it. Running it would have produced a green row on a
scorecard for work that did not happen — the most expensive kind of green, because it reads as
evidence.

The probe was also **removed from `wat-scripts/scratch-pad/`** so the `every_wat_scripts_file_loads`
gate would not inherit a red from a deliberately-failing file — the same relocation lesson Stone A
learned with `.wat.bad`, applied unprompted.

## Can the seam carry a drop?

**Not as-is.** A send-shaped helper needs `Peer`'s `I` to equal the payload type, and the selectable
is `Peer :- [R O]` — **two type parameters.** Said, not built.

## What the next draw must carry

1. **`Peer :- [R O]`**, parametric over both — and probed before briefing, not after.
2. **Five sites, not ten.** `1659 1697 1784 1811 1854`.
3. **`1828` is a genuine variant** (`Stopped → true`): either a second helper, an argument, or an
   affirmative exclusion. Not silently swept in.
4. **`1939 1950` are not extractable to a bool helper** — their arms are the serve loop's own tail
   calls. A seam there is a different shape, or none.
5. **The floor is the proof only after the edit.** Do not schedule it as a pre-check.

## Still open

- **R1, re-drawn** — the seam, with the right type and the right five sites.
- **Stone C** · **S33/S34** · **S15**–**S32** · the arc-109 NOTE.
- **3d, the select pool, server-side handle killing** — all still behind the seam.
