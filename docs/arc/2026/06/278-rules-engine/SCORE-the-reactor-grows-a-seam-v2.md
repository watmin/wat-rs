# SCORE — the reactor grows a seam (v2)

**NOT STRUCK. STOP-4 fired.** Executor: grok, 2026-09-04. Extraction on disk, uncommitted.
HEAD unchanged at `4e00c138f`.

```
Summary [ 363.318s] 5214 tests run: 5210 passed (3 slow), 4 failed, 15 skipped
FLOOR=100      .floor/2026-09-04T10-40-41Z/      run AFTER the edit; the red was NOT re-run
```

## ★ THE RED DOES NOT NAME A DIVERGED SEND SITE — because my proof was the wrong instrument

STOP-4 said *"a red is the extraction being unfaithful, and the failing test names which site
diverged."* **It named none.** All four are `probe_arc278_peers_bijection` EDN goldens:
message and reason identical, column unchanged, **lines shifted +13** — the helper was inserted
before `defservice`.

| golden | expected | actual |
|---|---|---|
| case1 / case4 (missing) | 896 / 903 | 909 / 916 |
| case2 / case5 (extra) | 913 / 921 | 926 / 934 |

Verified: those `.edn` files carry **`:line 896`** — a `service.wat` line number, snapshotted.

⛔ **So my contract decision was false as written.** I said the floor is *"necessary and sufficient
for faithfulness"* because every test expands through this macro. **Four of them also snapshot the
macro's line numbers**, so the floor conflates two different things:

- **behaviour changed** — what the stone must detect
- **lines moved** — irrelevant to behaviour, and unavoidable for any insertion

**Any** insertion into `service.wat` above line 896 reds those four, correct or not. The floor cannot
bless this stone until those goldens are in scope, and that is **a DESIGN gap, not a diverged site.**

## ★ THE EXECUTOR REFUSED TO PATCH THE GOLDENS, AND THAT IS THE RESULT

> *"I did not patch the four `.edn` files. STOP-5 / row 6 is one file; patching goldens after STOP-4
> is the improvisation v1 refused on `:- [R O]`."*

Same discipline, two stones running. v1 stopped rather than try `[R O]` uninstructed; v2 stopped
rather than update goldens uninstructed. **Both were the cheap, plausible move, and both would have
turned a design gap into a green.**

Ordering held too: the floor ran **after** the edit, per row 1 — the correction v2 carried from v1.

## ⛔ MY ROW 2 WAS WRONG AGAIN

I predicted `grep -n 'kernel::send'` → *"the helper plus the four exclusions"* — six. It is **fifteen**:

- **1** helper (`:104`)
- **5** exclusion lines — I named four *groups* and five *lines*, then predicted four
- **9 sends that were never candidates**: `peers-allowed`/`denied` (`2025 2045`), malformed-reply
  (`2100`), five client-face send-then-recv methods (`2239 2324 2368 2420 2470`), child-main status
  (`2620`)

**I described a subset and implied it was the total.** Tenth count of mine to miss this campaign, and
the same shape as the other nine: I enumerated what I had looked at and reported it as what exists.

Also: my BRIEF cited **pre-edit** line numbers, so every exclusion moved (`1828→1825`, etc.). The
executor mapped them rather than reporting a mismatch.

## What is right on disk

Five callers at `1672 1706 1789 1812 1851`. **No exclusion swept.** Probe re-run green
(`SEAM-EXPRESSES`). Circuit ×5 `total=8000; distinct=8000; dup=0` — run after the captured red, not
as a floor re-run. STOPs 1/2/3/5 did not fire. No hygiene failure. No BOOTSTRAP dance.

**And the R2 answer, stated not built:** the helper is `[peer payload] -> bool` and does not take the
rate or seed. Widening it — *drop? before the send, still returning `true`, because a drop is not
`Stopped`* — is R2. Wrapping at the five sites would defeat the seam.

## What v3 must carry

1. **★ Put the helper at the END of the file.** The goldens snapshot lines **896-934**, inside the
   macro. A `defn` appended after the macro shifts nothing before it, the goldens stay, and the floor
   becomes a pure extraction proof again. ⛔ **Probe the forward reference first** — the template
   would call a `defn` defined later in the file.
2. **The floor is not sufficient by itself.** Say so. It is necessary; the four goldens are a
   line-number tripwire that any insertion trips.
3. **Cite post-edit lines, or cite none.** My pre-edit citations forced the executor to map five
   exclusions by hand.
4. **Row 2's expected count is 15**, not 6 — helper + 5 exclusions + 9 never-candidates.

★ **And if the end-of-file placement does not work**, the goldens are the stone: they encode a
line number to assert a *span*, which makes them break on any edit above them. That is its own
finding and its own stone — not something to patch mid-strike.
