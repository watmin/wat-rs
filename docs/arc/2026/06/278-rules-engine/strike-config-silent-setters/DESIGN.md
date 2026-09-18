# DESIGN — Ω4: a config setter can be silently dropped, and you get the default

> Drawn 2026-09-05 at HEAD `0ee56325f`. Source: vigilia 2026-09-05 CLASS Ω4 (`circumspicere`).
> **Every claim below was driven by the orchestrator at THIS HEAD**, and the drive found a
> SECOND live defect the ward reported as one.

## The defect: you ask for 4096 and silently get 10000

```
set-dim-count! 4096                                  ->  "4096"    rc=0   (control)
setmax-fire-rounds! 5  (typo), then set-dim-count!   ->  "10000"   rc=0   ✗
setmax-fire-rounds! 5  (typo) alone                  ->  "10000"   rc=0   ✗
defrecord ..., then set-dim-count! 4096              ->  "10000"   rc=0   ✗
```

`dim-count` governs every vector in a VSA/HDC substrate. A wrong dimension at exit 0, with no
diagnostic, is a silent wrong answer for the whole program.

## TWO defects, one root

**Ω4a — a mistyped config head ends the setter section and is swallowed.**
`config.rs:465-469` accepts a setter only if the leaf `starts_with("set-")`.
`setmax-fire-rounds!` has no hyphen after `set`, so it takes the `_ =>` arm at `:470-473`
(`remainder_start = Some(i); break;`), the section ends, and **every valid setter behind it is
never processed**.

**Ω4b — `SetterAfterNonSetter` is unreachable, so a CORRECTLY SPELLED setter after any body form is
silently ignored.** `remainder_start` is assigned at exactly one site, immediately followed by
`break`, so `if remainder_start.is_some()` at `config.rs:477` can never see `Some`. The variant is
declared (`:222`), rendered, and **never constructed**. Driven: `defrecord` then a valid
`set-dim-count! 4096` → `10000`.

**The root of the silence is `cernere` C1 — the open `:wat::` vocabulary.** The orphaned form lands
in the program body, and `resolve/walk.rs:268` accepts any `:wat::`-prefixed head by prefix alone.
**Control, driven:** the same shape with a NON-reserved head (`:totally::bogus::head!`) fails
startup with `UnresolvedReferences`. So the vocabulary is closed everywhere except the namespace
this defect lives in.

## The one contract decision, pinned

**After the setter section ends, a `:wat::config::…!` form anywhere in the remainder is an ERROR.**

The discriminator already exists in this file: **`head.ends_with('!')`**. Config *accessors*
(`(:wat::config::dim-count)`) are legitimate body forms and do not end in `!`; setters do. So:

| remainder form | verdict |
|---|---|
| `:wat::config::…!` with a valid `set-` leaf | `SetterAfterNonSetter` — the variant becomes REACHABLE |
| `:wat::config::…!` with a non-`set-` leaf | malformed config setter — located, distinct |
| `:wat::config::dim-count` and friends | legal, untouched |

**Do NOT cure this by closing the `:wat::` vocabulary.** That is `cernere` C1, a much larger strike
with its own blast radius (two shipped fixtures ride the escape). This strike closes the config
namespace's own hole and leaves C1 standing and rowed.

## Scope

**IN:** both defects, the cure, and gates for each — **one strike, floor GREEN at the end.**

**OUT, affirmatively cut:** C1 itself; `RequiredFieldMissing`, which is also declared-and-never-
constructed (`circumspicere` L1-4, `intueri` 13) and is a *different* defect — the doc says `dims`
and `capacity-mode` are required while `config.rs:16/34-36` says every field has a default. Name it
in the SCORE; do not fix it here.

## ⛔ Cure and gate ship together

Floor green at the end, or the strike does not end. See `../strike-mode-parity/` for why this
sentence is in every DESIGN now.
