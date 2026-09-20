# FINDING — a SYMBOL whitelist entry gets RESOLVED; a keyword one never was

**Found 2026-09-20 during the orchestrator's weigh of 251.8d-i**, on the built binary at `22118e145`.
⛔ **This is a SEMANTIC change hiding inside what 8d promises is a SPELLING change.**

## The measurement

`{:restricted-to [my.kernel/specific-caller]}` where `my.kernel/specific-caller` is **not defined in
the file**:

```
#wat.resolve/UnresolvedReferences {:message "1 unresolved reference"
  :path ":my::kernel::specific-caller"
  :context "namespaced symbol ref — not a builtin, not a registered function (arc 251)"}
```

The entry is **data** — a name in a whitelist — and it is being resolved as a **reference**.
The keyword spelling `[:my::kernel::specific-caller]` never was.

## ⭐ AND THE TWO ENTRY KINDS NOW BEHAVE DIFFERENTLY

This falls straight out of the first-slash ruling, and nobody predicted it:

| entry | `Identifier::bare` gives | `is_reference()` | resolved? |
|---|---|---|---|
| `my.kernel` — a **namespace**, no `/` | `[$bound, my.kernel]` | **false** | ❌ no — works as data |
| `my.kernel/specific-caller` — an **exact FQDN**, has `/` | `[my.kernel, specific-caller]` | **true** | ✅ **yes — must exist** |

So a **prefix** entry may name anything; an **exact** entry must name a function that exists. Same
vector, same position, two rules. ⛔ **That asymmetry is invisible at the call site** — the author
writes two names in one list and one of them is checked.

## What it breaks, concretely

`tests/kernel/wat_arc198_def_restricted_bad_exact_fqdn_denied.wat` — arc 198's **negative control**,
and the fixture 8d-i's brief nominated as the codemod's own acceptance test. Its whitelist names
`:my::kernel::specific-caller`, and **that function does not exist in the file** (measured: 0
definitions). It does not need to — as a keyword it is inert data.

After 8d-iii converts it, the file fails with `UnresolvedReference` instead of
`DefRestrictedCallerNotAllowed` — **the right colour for the wrong reason**
(`[[feedback_a_stale_assertion_goes_red_for_the_wrong_reason]]`).

✅ **The test will catch it.** `assert_restricted_call_rejected` matches
`CheckErrorKind::DefRestrictedCallerNotAllowed` and panics on any other kind, so this reds loudly
rather than passing hollow. **The guard works. The question is what the cure should be.**

## THE FORK — the builder's, weighed

### Option A — accept resolution: a whitelist entry must name something that exists

| question | |
|---|---|
| Obvious | **NO** — nothing at the call site says one entry kind is checked and the other is not |
| Simple | **NO** — two entry kinds, two resolution rules, decided by a slash |
| Honest | **NO** — it makes a security mechanism's failure mode depend on definition order and compilation unit; you cannot whitelist a caller defined elsewhere or later |
| Good UX | **NO** — the arc-198 fixture would have to gain a function it exists precisely to not have |

### Option B — `:restricted-to` values are DATA and are exempt from resolution

| question | |
|---|---|
| Obvious | **YES** — a metadata value is data; it is not in call position |
| Simple | **YES** — one rule for both entry kinds, no slash-dependent behaviour |
| Honest | **YES** — **exact parity with today.** It keeps 8d a pure spelling change, which is 8d's whole premise |
| Good UX | **YES** — whitelist any name regardless of where or when it is defined |

⭐ **B is 4-YES; A is 0-YES. The orchestrator recommends B.**

⚠ **What B costs:** a typo in a whitelist stays silent, as it is today. Mitigations already exist and
are unchanged — the match fails **closed** (an unmatched caller is denied, verified) and the
diagnostic prints the whole whitelist (`whitelist [my.kernel]`), so a typo surfaces as a denial that
names the bad entry.

## Scope note

The fix is in the resolver/normalizer — `:restricted-to` metadata values must not be walked as
references — and it belongs to **8d-iii** (or a small stone before it), **not** 8d-i, whose gate is
the census and which writes no corpus file. 8d-i is unaffected: it is green on this measurement
because nothing is converted yet.
