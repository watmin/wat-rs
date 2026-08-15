# 296 · DESIGN STONE I — the gate PERFORMS the registration

> **STATUS: DRAWN, NOT BUILT.** Ruled by the builder 2026-08-15: *"why do we have N things making
> independent decisions?"*

## THE DEFECT

`resolve::gate` returns **advice**. Thirteen callers then each decide what to do with it.

Twelve write the same three-arm match. One propagates the verdict as `Err`. And one level further
out, `check/env.rs:158` receives that correctly-propagated `Err` and **prints it**:

```rust
// Loud on purpose while we learn what the corpus holds.
eprintln!("GATE-REJECT\t{path}\t{verdict:?}")
```

A door you can walk past is not a door. It is a doorman you may ignore, and the ignoring is invisible
because it looks like fourteen independent local decisions rather than one missing wall.

## WHY IT ESCAPED TWICE — the shapes do not resemble each other

Both escapes produce the identical outcome — *the door said no and nothing stopped* — and no single
search finds both:

| | shape | how it was found |
|---|---|---|
| **H-1c** | never called the gate at all (`register_aggregate_methods` inserted straight into `sym.functions`) | a fixture that should have gone red stayed green |
| **this one** | called it, received `Err`, logged it | reading the file while chasing the first |
| *(and)* | `_ => {}` wildcard at the defclause door | caught mid-flight by the H-1 rider |

Three appearances, three different silhouettes. Hand-discipline held twelve times out of fourteen —
which is exactly what a convention does right up until the afternoon it doesn't.
**`[[feedback_a_match_with_identical_arms_is_a_discard]]`: a gate can force you to LOOK at a value;
nothing forces you to ACT on it.**

## THE SHAPE — inserting IS passing the door

```rust
resolve::register(&name, privilege, existing, &span, || sym.register_function(name, f))?;
```

The door performs the insert. Callers `?` it. There is no verdict to drop, no arm to forget, and no
way to insert without passing through — because inserting *is* passing through.

**`resolve::gate` becomes private to its module.** That is the actual wall: if `gate` is not callable
from outside, "ask the door, then separately do the thing" has **no form**. `register` is the only
public entry.

### Bridging the seam — measured, 2026-08-15

The thirteen sites are genuinely heterogeneous, and the design must carry that rather than flatten it:

| axis | what is actually there |
|---|---|
| **registries (6)** | `sym.register_function` (×4) · `self.types.insert` · `env.schemes.insert` · `env.register_defined_value_ast` · the macro registry · the defclause table |
| **error taxonomies (4)** | `RuntimeError` · `TypeError` · `MacroError` · `CheckError` — H-1 already mirrored `DottedName` into all four |
| **span sources (3)** | `form.span()` · `span.clone()` · `rust_caller_span!()` — and some sites have none |

So: the insert arrives as a **closure**; the span arrives as a **parameter**; and the rejection is a
`Rejection { verdict, name, span }` with `From<Rejection>` for each of the four error types. Then `?`
does the taxonomy conversion at every site and no caller writes an arm at all.

## ⛔ THE RUNG THAT LOOKS LIKE THE FIX AND IS NOT

**`#[must_use]` on `Registration` is not this stone.** It forces a caller to *look* at the value; it
does not force a caller to *act* on it. The proof is already on disk: twelve sites do look — one of
them looked via a `_ => {}` wildcard, and `check/env.rs:158` looks via an `eprintln!`. Both satisfy
must-use. Both are the bug.

Climb to no-form or the class regrows in a fourth silhouette.

## THE FOUR QUESTIONS

- **Obvious?** YES — one function named for what it does, and you cannot register without it.
- **Simple?** YES — thirteen hand-rolled matches become one implementation; callers become one line.
- **Honest?** YES — `gate` is *named* like a door while behaving as an advisor. That name is a lie,
  and `check/env.rs:158` is a caller that believed it.
- **Good UX?** YES — the right path becomes the only path.

## PRECEDENT — this collapse has run twice here

- task **#30** — one door for defclause registration (the 2×2 collapsed)
- task **#75** — one door for a type head's FQDN (17 hand-rolls collapsed, both defensive branches deleted)

Same move, same reason, and both stuck.

## STOP TRIGGERS

- **STOP-1 — a site cannot express its insert as a closure** (borrow conflict: the registry is
  borrowed for the `existing` lookup and again for the insert). Report the site; do not leave it on
  the old `gate` as an exception. An exception is the hole re-opening under a rationale.
- **STOP-2 — a site has no span to give.** Do not invent one and do not pass a fabricated default:
  `rust_caller_span!()` is already the honest answer at sites with no form, and it is used elsewhere in
  these files. Report anything that fits neither.
- **STOP-3 — making `gate` private breaks a caller outside `resolve`** that is not one of the
  thirteen. That is a fourteenth entry the census missed, and it is the finding.
- **STOP-4 — the floor moves.** This stone is a refactor, not a behaviour change: every rejection that
  fired before must still fire, and none that did not must start. A moved count means a site's
  semantics changed in translation. Capture it whole before touching anything.

## WHAT THIS STONE IS NOT

Not a widening of what the gate rejects. `has_dotted_name`, `Reserved`, `Unnamespaced` and the
idempotent-before-reserved ordering are all correct and stay exactly as they are. This changes only
**who is allowed to ignore them** — from *anyone* to *no one*.
