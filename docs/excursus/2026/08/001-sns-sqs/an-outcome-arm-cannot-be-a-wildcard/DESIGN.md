# DESIGN — an outcome arm cannot be a wildcard

**Drawn 2026-09-16.** Builder's item **3**, sequenced after item 1 (`every-status-arm-is-named`,
`4413fff0f`) and before **B**. **NOT STRUCK. PHASE 1 IS REPORT-ONLY** — see §The gate is a later ruling.

## Why

Two stones this session removed wildcard arms by hand: `the-blind-sends-face-their-outcome`
(`e03fc7f46`, 3 send sites) and `every-status-arm-is-named` (`4413fff0f`, 9 generated `Status` sites).
Both were found by grepping, and **both nearly weren't**:

- my textual sweep pronounced the corpus clean while **12 of `service.wat`'s 14 wildcards were
  macro-generated** and invisible to it — their arm heads are unquoted symbols (`~status-stopped-kw`),
  so no regex can attribute them to an enum;
- the same sweep read its own comments as code, and a fixed window missed `(_` at end of line.

⭐ **The property both stones bought is LOUDNESS: a variant added later becomes a checker error rather
than being silently absorbed.** Today nothing preserves it. The next wildcard written into a generated
body restores the blindness, and the only reason we know about the nine is that a crash was hunted.

⚠ **And this lint would have caught NONE of the three missing-form failures** found this session
(a blocked `send` has no variant; `recv-all`'s timeout has no form; `after`'s ring refusal raises).
Those are absent variants, not unread ones. Stated so the lint is not oversold: it guards one
direction of arc 109's painted-brick doctrine — *a variant nothing READS* — and is silent on the other.

## The instrument — and this is the whole design decision

| | text lint in `tests/lint/` | ⭐ checker diagnostic |
|---|---|---|
| sees literal sites | ✅ | ✅ |
| sees **macro-generated** sites | ⛔ **no** — the blind spot that hid 12 of 14 | ✅ works on expanded forms, knows the real scrutinee type |
| exemption mechanism | ✅ `// rune:lint(name)` read from the source line (`tests/lint/no_loose_string_assert.rs:98`) | ⛔ the checker does **not** see comments |
| convention | the repo's established lint shape | new ground |

**Choose the checker.** The blind spot is the reason this stone exists; an instrument that shares it is
theatre. The cost is that exemptions cannot be comments, which §Exemptions answers.

The hook already exists: `src/check.rs:~6925` is where a match on an enum is checked for exhaustiveness
and where **a wildcard currently BLESSES a missing arm** (*"…missing arm(s) for variant(s): X (or
include `_` wildcard)"*). That is the exact point that knows the enum path and that a wildcard was
used.

## Scope — which enums, and why not all of them

**Only the FIXED, registered outcome/event enums** — `RecvOutcome`, `SendOutcome`, `TrySendOutcome`,
`ConnectOutcome`, `CloseOutcome`, `ServiceEvent`, `StopOutcome`, `GateOutcome`, `CallOutcome`,
`ReadlnOutcome`, `AcceptOutcome`, `SignalOutcome`, `NextOutcome` (confirm the list against
`src/types.rs` rather than trusting this one).

⛔ **NOT the per-surface generated enums** (`<S>::Reply`, per-op response enums). Their variant sets are
generated per surface, so "name every variant" has no fixed meaning and their wildcard is the **desync
detector** — a reply for a *different* op. Three such sites are already documented as deliberate
(`service.wat` `:2785`, `:2980`, `:3091`). They are out of scope **by construction**, not by exemption.

⚠ **`Status` is the interesting case**: it is generated *per service* but has a **fixed six-variant
shape** (`Started · Stopped · Hibernated · PeersAllowed · PeersDenied · Faulted`). Item 1 drove it to
zero wildcards by hand. Decide and state whether the rule can reach it — if it can, it guards the
highest-leverage surface in the tree; if it cannot, say so plainly, because that is where the nine
lived.

## ⛔ PHASE 1 IS A CENSUS. THE GATE IS A LATER RULING.

Emit the diagnostic **as a collected report, not an error**, and expose it through one test that
**prints and passes**. Then report:

1. total sites where the rule would fire, **per enum**;
2. split **live** (`wat/`, `wat-scripts/` services) from **tests/probes/scratch-pad**;
3. how many are in **macro-generated bodies** — the number that justifies choosing the checker;
4. the **known-remaining live two**: `service.wat:2978` / `:3089`, `(RecvOutcome::Message …)` + `_` in
   the generated paging path, filed by item 1 as *"five behaviour decisions × two sites"*.

**Do not turn it into a hard error in this stone.** A corpus-wide error is a ruling about churn the
builder has not been asked for, and the two censuses that preceded this one (the recoverable-crash
census `2ea49a6bd`, the waiter census `e7816e543`) were both report-only and both worth more than the
strikes they authorised. ⭑ **Textual estimate for sanity-checking your count, NOT for quoting:** 1440
forms attributed to an enum tree-wide; 1 live `SendOutcome` wildcard (a scratch probe); 3 live
`ServiceEvent`; 2 live `LociDiedError`; 5 in `service.wat` of which 3 are per-surface. The checker
should find MORE than this, and the difference is the point.

## Exemptions — needed only if phase 2 gates

The checker cannot read comments. Options, to be **reported not chosen** in phase 1:

- an in-form annotation the macro can emit and the checker can see;
- an allowlist keyed by enum path + enclosing form name (brittle; say so if you recommend it);
- **no exemption at all** — the fixed enums are exhaustible by definition, so a wildcard is always a
  latent bug and the corpus is driven to zero. ⭑ This is the cleanest and may be viable precisely
  because the live count is small; the count from phase 1 decides it.

## Trap-doors

1. ⛔ **Do not let the rule fire on non-outcome enums.** A wildcard over a domain enum
   (`:my::Color`, `PeerKind`) is ordinary code. The rule is about **outcome/event** enums, where an
   unread variant is a swallowed failure.
2. ⛔ **`Option`/`Result` are not in scope.** `(_ …)` over an `Option` is idiomatic.
3. **Prove the diagnostic sees a GENERATED site.** If it cannot flag one of `service.wat`'s
   quasiquoted bodies, the instrument choice failed and the whole justification collapses — report
   that as the finding rather than shipping a lint with the blind spot it was built to remove.
4. **The floor must stay green**: phase 1 adds a reporting test, not an error.
