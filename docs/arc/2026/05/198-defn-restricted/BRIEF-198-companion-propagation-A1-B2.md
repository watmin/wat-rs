# BRIEF — 198 strike 2: companion propagation (A1 + B2) — RULED

> Read `DESIGN-STONE-a-restriction-governs-mention-not-head-position.md` first, **including its
> `⚖ RULING` section at the end** — the decision is made and the losing options are recorded with the
> axis that killed each. Builder, 2026-08-15: *"B2 and A1 - they have been reasoned."*

## ⛔ STATE OF THE TREE — YOU ARE INHERITING WORK

`src/check.rs` is **already modified and uncommitted** by a prior flight. It holds the mention-rule
change: the `WatAST::List` + `items.first()` guard is deleted, the walker fires on every
`WatAST::Keyword`, and the doc is rewritten (including a false claim it used to make about `let`
bodies). **That change is correct. Do not redo it, revert it, or re-litigate it.** Build on it.

With it in place the floor is:

```
Summary [ 193.457s] 4531 tests run: 4528 passed (2 slow), 3 failed, 154 skipped
```

Those 3 are your worklist. Nothing else in 4531 moved; no `wat/` corpus file screams.

## THE WORK IN ONE PARAGRAPH

Two synthesized companions name their own type in their bodies, and their FQDNs are not on the type's
whitelist, so they trip the type's own gate. **A1:** make those companions inherit the type's
restriction (closing a real escape hatch the substrate currently advertises). **B2:** record on the
`Function` which type it was synthesized for, and let the walker exempt mentions of that owning type.
Then fix the retirement message that teaches the bypass.

## THE MEASURED FACTS — do not re-derive, do not doubt, verify cheaply if you like

All three failures are identical: exactly two errors, from exactly two sites.

| enclosing fn | mint site | names |
|---|---|---|
| `:my::Token'` — positional prime ctor | `src/runtime.rs:1550` mints it; body pushes the type at **`:1557`** | `:my::Token` |
| `:my::is-Token?` — membership predicate (arc 237.6) | minted near `src/runtime.rs:1940`; body names the type at **`:2006`** | `:my::Token` |

- **The accessors are INNOCENT.** `:my::Token/id` (`runtime.rs:1600`) uses `Record/field-at` and a
  *string* `class_no_colon`; it never names the type as a keyword. **The companion set is two.**
  If you find a third, that is a finding — report it, do not silently widen.
- **`contract_03_defstruct_with_field_metadata` declares `:restricted-to []`** — the empty whitelist
  that by design matches nothing — **and trips too. The propagation must be UNCONDITIONAL**, not
  gated on "the whitelist is non-empty".
- **Restriction registration itself is correct** (`runtime.rs:1452-1467`, keys on `struct_def.name`
  and `T/field`). Do not touch it except to add the companions.

## A1 — the companions inherit the type's whitelist

At the mint sites, register the type's `:restricted-to` for `T'` and `is-T?` in
`sym.binding_metadata`, exactly as `runtime.rs:1453-1458` does for the type itself.

**This closes a hole that exists today, independent of any of this work:** `(:my::Token' 7)` from
`:user::` currently constructs a restricted type with no gate at all. **The floor cannot see it** —
nothing in 4531 tests asks — so it needs its own test (gate 2 below).

### A1 includes fixing the message that teaches the bypass

`src/runtime.rs:15835` currently emits:

> *"bare-positional construction of `X` is retired (the bare name is the kwargs macro); write kwargs
> `(:ns::P :field value …)` **or use the positional prime `:ns::P'`**"*

After A1 that advice walks a non-whitelisted user into a wall. **Rewrite it so the prime is not
offered unconditionally** — the honest remedy for a restricted type is the kwargs form from a
whitelisted caller. Ship this in the same strike; it is part of A1, not a follow-up.

## B2 — the companion may name its own type

Add ownership to `Function` — e.g. `synthesized_for: Option<String>` set to the owning type's FQDN at
**both** mint sites. In `walk_for_restricted_call`, when the enclosing function carries
`synthesized_for == Some(T)`, a mention of `T` is exempt.

**Exempt the OWNING TYPE ONLY.** A companion mentioning some *other* restricted binding must still be
refused — that is the whole reason B4 ("skip synthesized bodies") lost on Honest.

## ⛔ THE RULED-OUT OPTIONS — do not drift to these when it gets awkward

| | option | killed by |
|---|---|---|
| B1 | append companion FQDNs into T's `:restricted-to` list | **Obvious + Honest** — the diagnostic prints the whitelist back to the user; B1 quotes entries they never wrote |
| B3 | exempt by name pattern (`ends_with("'")`, `is-…?`) | **all three, one is a FORGERY** — a user-authored fn named `:my::Token'` would inherit the exemption |
| B4 | don't walk synthesized bodies at all | **Honest** — exempts generated code from *every* restriction; a companion naming `write-fd-raw` would pass |
| B5 | make the body not name the type | **Obvious + Simple** — a ctor that doesn't say what it builds |
| A2 | leave `T'` unguarded | **Honest** — the type says restricted; a public alias constructs freely |

**If B2 turns out to be hard, that is STOP-1 — report the exact obstacle. It is NOT licence to fall
back to B1 or B4.**

## THE GATE — five proofs

1. **The three current failures go green.** `contract_03_defstruct_with_field_metadata`,
   `struct_restricted_form_parses_and_accessors_callable_from_whitelist`,
   `struct_restricted_public_accessors_unrestricted`.
2. **NEW — the prime escape closes (proves A1).** A `:user::` fn calling `(:my::Token' 7)` on a
   type restricted to `[:my::issuer::]` must be **refused**. ⛔ **Run this BEFORE your change and
   confirm it currently PASSES clean** — that is the negative control proving the test can fail.
   Report both observations.
3. **NEW — a whitelisted caller still constructs.** `:my::issuer::mint` calling `(:my::Token :id 7)`
   and `(:my::Token' 7)` must both succeed. Without this, A1 is indistinguishable from a total ban.
4. **The original stone's gates still hold.** The value-position alias
   (`(:wat::core::let [f :wat::kernel::str-double] (f "AB" 3))` from `:user::`) is still **refused**,
   and the `write-fd-raw` alias is still refused — **CHECK ONLY, never execute an arbitrary-fd write.**
5. **B2's exemption is provably load-bearing.** Temporarily disable the `synthesized_for` exemption,
   confirm the three tests go red again, restore, confirm green. Report how you verified it, and
   confirm `git diff` shows no residue.

## STOP TRIGGERS — rejections. Report and leave the site.

- **STOP-1 — B2 cannot be implemented cleanly.** Report the obstacle. Do **not** fall back to B1/B3/B4.
- **STOP-2 — a third companion names the type.** The measurement says two. A third is a finding:
  report it with its mint site; do not widen silently.
- **STOP-3 — the corpus screams.** The floor says it does not. If it does, report the list and stop.
- **STOP-4 — you are tempted to widen a whitelist, weaken a restriction, or make a test assert less
  to reach green.** Never. That inverts the stone.

## BLAST RADIUS

`src/runtime.rs` (the two mint sites + the remedy message), `src/check.rs` (the B2 exemption only —
the mention rule is already there and correct), `src/value/symbol_table.rs` or wherever `Function`
lives (the new field), and new tests. **No `.wat` corpus rewrites** — report if that turns out false.

**Affirmatively cut, still in the stone, do NOT build here:** W1 (startup wall — every registered
`:restricted-to` must be enforceable) and W2 (sweep every *"X is safe because Y cannot be authored"*
claim).

## VERIFY

`cargo build --release --tests`, then `cargo clippy --workspace --all-targets --release -- -D warnings`
(expect 0), then `scripts/floor.sh` and read the **Summary line** — never a piped exit code.

Expect **`3 failed` → `0 failed`** plus your new tests. Report the real arithmetic.

**On any red you did not intend: do NOT re-run.** `scripts/floor.sh` keeps the untruncated log. Copy
the failing test's whole stdout+stderr block **verbatim** — never a `| head` window — name the exact
assertion, and report.

## HOW TO WORK

You are a rider. **Ending your turn ENDS you** — nothing wakes you and no notification is coming.
**Run every build and test in the FOREGROUND and block on it.** The previous rider on this exact
strike backgrounded the floor, ended its turn, and died mid-flight; its run had to be recovered by the
orchestrator. Do not background anything. Do not set a monitor and wait.

Anchor at `/home/watmin/work/holon/wat-rs`; `pwd` first. **Leave the work uncommitted** — the
orchestrator commits. Never `git commit`/`push`/`stash`/`revert`/`checkout --`; `stash@{0}` holds
unrelated work.

## REPORT

- the diff at each of the two mint sites, and the B2 exemption
- **gate 2 both ways** — the prime escape passing before, refused after
- **gate 5** — how you proved the exemption load-bearing, and that no residue remains
- each of the five gate proofs with its exact diagnostic
- the floor Summary line verbatim with the arithmetic
- every STOP that fired
- **the honest deltas — especially anywhere this brief did not match the disk.** Every rider on this
  arc has found a defect in the orchestrator's brief; this one already corrects two of its own
  author's earlier claims.
