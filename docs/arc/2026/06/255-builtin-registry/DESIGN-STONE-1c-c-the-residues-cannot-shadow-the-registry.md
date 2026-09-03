# DESIGN — STONE 1c-c: a residue may not name a registered verb, and a gate says so

> **Builder, 2026-09-03:** *"the registry is forged into the sole authority for these lookup
> problems... you have named another stepping stone.... a small step... so... we take it."*

## The defect, in the residues' own words

Two functions consult the registry first and fall through to a hand-list:

```
src/macros/eval.rs   is_expand_time_legal   registry at :425,  arms from :476
src/rete/purity.rs   intrinsic_meta         registry at :472,  arms from :504
```

Both define their list the same way, and both say so themselves:

> `is_expand_time_legal`: *"every name below is one for which `registry().lookup_entry` returns
> `None`… **A REGISTERED verb does not belong here** — if one is ever added below alongside a real
> registration, the derivation above is being **shadowed by a copy, which is the exact defect this
> stone exists to remove**."*
>
> `intrinsic_meta`: *"for a head this lookup misses (`lookup_entry(head) == None` — unregistered,
> not a name-list)."*

**Measured 2026-09-03 — 53 rows violate that rule:**

```
is_expand_time_legal    34 of 55 named FQDNs are registered
intrinsic_meta          19 of 42 named FQDNs are registered
effectful_by_prefix      0 of  8  (a prefix list, not a name list — correctly out of scope)
```

More than half of the expand-time residue is dead text. Its own header still claims **58**.

## ⛔ DETECTION HAS BEEN ACCIDENTAL, TWICE — that is the case for a gate

The expand-time residue's header records the previous occurrence verbatim:

> *"~~Option/Result unwrappers~~ — DELETED 2026-08-31. All four … are now `#[wat_intrinsic]`-registered,
> so the `registry().lookup_entry` door above answers first and these arms were unreachable dead
> text — precisely the 'shadowed by a copy' defect this residue list's own header names. **Found by
> a rider that was homing three of them and noticed the fourth had been stale since earlier the
> same day.**"*

And `intrinsic_meta` carries its own hand-verification: *"`registry().lookup_entry(name).is_some()`
for every deleted name, verified against every…"*.

★★★ **Twice the rule has been enforced by a human noticing, and twice it drifted again.** That is
the definition of a convention, and this campaign's own doctrine says the answer is to climb one
rung: `[[feedback_impose_the_check_and_read_the_screams]]`. A rule a file states about itself, that
nothing checks, is decoration.

## The stone

**One gate**, asserting: no FQDN named in either residue hand-list resolves in the registry. Then
delete the rows it names and correct both stale count comments.

The rows are **unreachable by construction** — a registered name is answered by the consult above
the list — so deleting them changes no behaviour. That is exactly why nothing caught them.

## ⚠ THE GATE'S OWN INSTRUMENT MUST BE PROVEN, OR IT IS WORTHLESS

Both lists are `matches!(head, | "…" | "…")` chains in source, so the gate must read source as
data — the technique `registry_first_door_owns_every_handler_row_no_literal_arm_survives` already
uses (`include_str!`, bounded to a function, comments stripped).

⚠ **This orchestrator got that boundary wrong twice inside ten minutes today** — once by
brace-counting past a function's end and collecting names from its neighbours, once by testing
only the eight names it happened to care about and reporting eight when the answer was
thirty-four. A source-parsing gate that silently parses nothing returns a **vacuous green**, which
is worse than no gate.

So the gate must carry its own non-vacuity proof: it asserts it found a plausible number of names
in each list AND that it can see a specific name known to be there. `[[feedback_a_green_test_can_prove_nothing]]`.

## THE FOUR QUESTIONS

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **one gate over both residues + delete what it names** | YES | YES | YES | YES | ✅ |
| delete the 53 rows, no gate | YES | YES | **NO** | — | ⛔ |
| gate only the expand-time residue | YES | YES | **NO** | — | ⛔ |
| convert both lists to `const` arrays first | YES | **NO** | YES | — | ⛔ |

- **no gate — Honest NO.** It has been cleaned by hand twice and drifted both times. A third
  hand-clean claims a cure it cannot deliver.
- **one list only — Honest NO.** The class is "registry-first consult + hand-list fallback", and
  it is present in both. Fixing the one that was noticed leaves the one that was not.
- **`const` arrays first — Simple NO.** A worthwhile refactor and a *separate* one; a const array
  can hold a registered name just as easily, so it does not remove the defect the gate does. Doing
  both at once mixes a shape change with a correctness gate.

## Acceptance — DERIVED

```
                        before   after   why
shadowed rows              53       0    34 expand-time + 19 intrinsic_meta, each named by the gate
is_expand_time_legal       55      21    named FQDNs remaining
intrinsic_meta             42      23
the gate                    —       1    with its own non-vacuity assertions
stale count comments        2       0    the expand-time header says 58; intrinsic_meta has its own
GAP_A / GAP_B / DEBT   49/52/111  same   ⬅ nothing is registered or unregistered by this stone
KNOWN_UNREVIEWED           14      14
floor                5127/5127   5128/5128   ⬅ +1: this stone DOES mint a test
clippy                              0
```

★ **`+1` on the floor is derived, not estimated** — a new `#[test]` fn is exactly what this stone
adds, unlike a registration stone where the count cannot move. Two acceptance tables this campaign
got that wrong by assuming; this one states the mechanism.

## Out of scope — CUT

- `effectful_by_prefix` — a prefix list, not a name list. Zero shadowed. Naming it in the gate
  would be a category error.
- Converting the lists to `const` arrays. Separate shape change.
- Any registration. This stone moves no name into or out of the registry.
