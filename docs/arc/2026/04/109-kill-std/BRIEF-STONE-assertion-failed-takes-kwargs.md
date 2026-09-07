# BRIEF — `assertion-failed!` takes KWARGS, and 2532 bare `:None`s stop existing

> The design is `NOTE-assertion-failed-should-take-kwargs.md` (builder catch 2026-07-20, ruled and
> queued, never built). Read it first. This brief re-measures it and sequences it.

## ⛔ FIRE THIS ONLY ON A GREEN TREE, AND WRITE THE PROBE FIRST

At the time of drawing, the stdlib does not parse (the match-arm reland is in flight), so **no probe
could be run and none is committed**. FM 2-bis is not waived — it is SEQUENCED:

> **The strike's FIRST act is to write and verify a RED probe**: the kwargs call form is accepted
> and the bare positional form is REFUSED. Do not begin the migration until that probe is red for
> the right reason on a green tree.

## THE FINDING, RE-MEASURED 2026-09-06

The NOTE measured **86 calls across 27 files**. Today:

```
2670  assertion-failed! call sites
 442  files
1266  of those calls end in TWO BARE `:wat::core::None`s  (= 2532 None sites)
```

**31× growth in seven weeks while the stone sat queued.** That is the interest on failure debt,
measured — and it is the argument for taking it before the Some/None sweep rather than after.

⚠ The NOTE's line citations have drifted: `src/assertion.rs:107` is now **`:127`**
(`eval_kernel_assertion_failed`, `args.len() != 3` at **`:135`**), and the registration at
`src/check.rs:16583` is now **`:17239`**. Both still exist; re-derive rather than trusting either.

## WHY THIS GOES BEFORE Some/None

`:wat::core::None` has 5784 sites. **2532 of them — 44% — are these two trailing args**, and the
NOTE's own fix deletes them:

> *"`:actual`/`:expected` default to `:None` so a plain fail is `(assertion-failed! :message "msg")`."*

Migrating Some/None first would rewrite those 2532 into `(wat.core/Option.None {})` — longer, and
then removed anyway. Same total work, one pass instead of two over the same 1266 call sites.

## THE WORK (from the NOTE, unchanged)

The bare name `:wat::kernel::assertion-failed!` becomes a **kwargs macro** (`:message` / `:actual` /
`:expected`) realizing into a **primed positional** primitive (`:wat::kernel::assertion-failed!'`).
Because the bare name flips, every site migrates atomically with the rename:

```clojure
(assertion-failed! "msg" :None :None)   ->   (assertion-failed! :message "msg")
(assertion-failed! "msg" a e)           ->   (assertion-failed! :message "msg" :actual a :expected e)
```

`:actual` / `:expected` default to `:None`.

## READ IN ORDER

```
NOTE-assertion-failed-should-take-kwargs.md   THE DESIGN — the finding, the fix, the rationale
src/assertion.rs:127 · :135                   eval_kernel_assertion_failed; the strict arity-3 gate
src/check.rs:17239                            the registration
wat-scripts/fixes/positional-to-kwargs.wat    the RECORDED codemod for exactly this shape — copy it
docs/arc/.../project_294_9a kwargs flip       the established bare-name-flips-atomically pattern
```

## STOP TRIGGERS

- **STOP-1 — the probe was not written first.** See the banner. A 442-file migration begun without a
  red probe has no way to tell "it worked" from "it compiled".
- **STOP-2 — a hand-edited `.wat` call site.** R21; `positional-to-kwargs.wat` is the recorded shape.
- **STOP-3 — both forms accepted at the end.** The positional form must be REFUSED once the flip
  lands, or the corpus sits in two calling conventions and nothing proves the migration finished.
- **STOP-4 — the two Nones are rewritten rather than DROPPED.** The whole point is that
  `:actual`/`:expected` default, so a plain fail carries neither. A migration that produces
  `(assertion-failed! :message "m" :actual :None :expected :None)` at 1266 sites has done the work
  and kept the defect.
- **STOP-5 — `assertion-failed!`'s sibling drags in.** `src/check.rs:17266` notes a sibling using the
  same panic_any mechanism. Out of scope unless the compiler forces it; report if so.
