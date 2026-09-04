# BRIEF — the reactor grows a seam (v2)

Executor: grok. Anchor at `/home/john/work/holon/wat-rs`; `pwd` first. Branch `sns-sqs`, HEAD
`c84cf1339`, tree clean. Read `DESIGN-the-reactor-grows-a-seam-v2.md` first, and
`SCORE-the-reactor-grows-a-seam.md` for why v1 died.

## THE WORK

`wat/service.wat` sends a reply at five sites with the same four-arm `SendOutcome` match. Extract it
to one parametric `defn` and call it from those five. **No behaviour change.** One file, no codemod,
no stash.

## ROOMS

1. **`wat-scripts/scratch-pad/probe-send-seam-parametric.wat`** — **run it first.** `SEAM-EXPRESSES`,
   3/3. The signature is proven: `:- [R O]`, `peer <- (Peer :- [:R :O])`, `payload <- :R`. You are not
   discovering whether it expresses.
2. **`wat/service.wat:1854-1858`** — the shape, read cold. Four arms, `Stopped → false`.
3. **`wat/service.wat`** at **`1659 1697 1784 1811`** — the other four. Confirm they match before
   touching anything.
4. **`wat/service.wat:64`** — *"A vanished waiter (absent conn-id, or send Closed/Lost) is not an
   error — keep serving."* That sentence **is** the `Closed → true` arm. Do not change it.
5. **`wat/service.wat:67-95`** — the eight sibling top-level forms your `defn` joins.
6. **`wat/seq.wat:277`** — `:wat::core::foldl-spec :- [T U]`, the two-parameter precedent.

## ⛔ DO NOT TOUCH — and name each in the report

- **`1828`** — `Stopped → true`, discarded by `do`. Different disposition.
- **`1939 1950`** — arm bodies are the serve loop's recursive tail calls / `nil`.
- **`2006 2012`** — `send self` status, all arms `nil`.

## STOP TRIGGERS

1. **Any of the four arm dispositions changes.** `Sent`/`Closed` → true, `Stopped` → false,
   `Lost` → true. STOP.
2. **You are about to sweep an excluded site in** to make the count rounder. STOP.
3. **You are about to add the drop.** R2. STOP.
4. **The floor moves off `5214/5214`.** Every test expands through this macro; a red is the
   extraction being unfaithful, and the failing test names which site diverged. Capture whole, do not
   re-run.
5. **You are about to touch `src/` or any other `.wat`.** One file. STOP.

## HOW TO WORK

Foreground everything. Floor is `scripts/floor.sh`; **Summary line, never a piped exit code.**

⛔ **Do not run the floor before the edit.** A green floor of the unextracted corpus proves nothing
about an extraction — you said so last time and you were right; v1's EXPECTATIONS invited that green
and this one forbids it.

⚠ **Do not write `(:wat::core::None <Type>)`** — phantom form. Arc-109 NOTE.

Leave your work uncommitted. Prior comparable: `SCORE-the-call-site-reads-as-english.md`.

## REPORT

- the probe from room 1, re-run
- `grep -n 'kernel::send' wat/service.wat` after — the helper plus the four exclusions, **each named**
- the floor Summary line verbatim, run **after** the edit
- the circuit, five runs
- **what a rate-gated drop would need from this helper** — say it, do not build it
- every STOP that fired
- **the honest deltas.** Nine of my censuses have died on token-vs-form, and v1 of this stone
  mis-shaped four of ten sites. What you find is the fact.
