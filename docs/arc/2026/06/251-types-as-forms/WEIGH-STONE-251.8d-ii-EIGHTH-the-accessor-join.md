# WEIGH — 251.8d-ii (EIGHTH): ACCEPTED. ⭐ **112 → 107**, and ⛔⛔ **the corpus is the wrong side.**

Weighed against `4b1fdae8e` by independent re-run. Tree clean, `wat/` byte-identical to baseline,
nothing pushed. Trajectory **3 747 → 416 → 283 → 161 → 112 → 107**.

## ⭐⭐ MY ADDRESS REPRODUCED — the first in five draws

After four consecutive wrong addresses, ⭐ **the proven-probe discipline worked the first time it was
applied properly.** The rider re-ran my table and it held: `wat/spawn.wat` converted alone reproduces,
narrowed by hunk to **ONE TOKEN** — the `defn` NAME at `wat/spawn.wat:130`.

## ⛔⛔ THE FINDING THAT CHANGES THE CAMPAIGN — THE JOIN IS *UNSPELLABLE*

I verified this myself rather than take it on report:

```
:my::ns::a/b    →  my.ns.a/b
:my::ns::c::d   →  my.ns.c/d
```

⭐⭐ **BOTH keyword spellings collapse to the SAME faithful symbol.** A faithful-Clojure symbol
carries exactly one `/` and it is the namespace/name split. **`:wat::spawn::process/post-spawn` and
`:wat::spawn::process::post-spawn` have one image between them.**

⛔ **So the `/` join at a NON-TYPE parent is not spelled differently — it is UNSPELLABLE.**
`rekey_type_member_functions` can only disambiguate when the parent is a **type** (proved by making
`process` a record: the defect vanishes). ⭐ **That is why `wat/cache.wat`'s 8 `Lru/*` names
self-heal and `wat/spawn.wat`'s 7 do not.**

⛔⛔ **THE CORPUS IS THE WRONG SIDE. This cannot be fixed in the checker** without making two
distinct keyword spellings ambiguous.

## ⭐⭐ IT REFUSED THE PERMISSIVE CURE — WHICH IS EXACTLY WHAT THE BRIEF ASKED

A permissive cure was available: teach the keyword call head to try the other join. ⛔ **It would
also make the WRONG-JOIN row resolve — the hole 255.8's WEIGH ruled must be closed before 8d-iii.**
⭐ **It did not take it**, and corroborated independently: 255.4's `one_member_join` lint already
refuses that spelling corpus-wide — **and caught its own first draft control.**

⭐ **My warning — "do not assume this one is permissive" — was the load-bearing line in the brief.**
The rider says plainly it *"would have shipped the wrong door without it."* **Four stones of
permissive cures had built exactly the wrong prior.**

## ⭐ AND IT DECLINED TO LAND THE RESTRICTIVE CURE — CORRECTLY

Respelling the spawn seven is a **documented public API** (`wat/spawn.wat`'s own header, lines 90-98)
across **27 live files**, needing a recorded codemod, a replay fixture and an oracle — and two of the
files are another migration's byte-exact `.post`. ⭐ **That is a builder ruling, and 255.8's
already-scheduled stone.** ⛔ **It built the GATE that stone needs instead.**

## What it DID cure — `Fault/of`, the opposite direction

`Fault` **is** a type, so the join is legitimate — but `Fault/of` is a **defmacro**, and the rekey
pass walks `functions_iter()` only; macros register and expand **before the `TypeEnv` exists**.
⭐ **Measured in situ: the stdlib's own call at `wat/spawn.wat:475` fails, so every program loading
the stdlib inherits it.** Cured by asking `other_join_spelling` when the primary key misses, `/`→`::`
only, Keyword success path byte-identical.

## Re-derived here

| | mine |
|---|---|
| converted floor | **107 / 6013** |
| ⭐ FIXED / NEW vs the seventh draw | **6 / 1** |
| ⭐ the NEW | ⭐⭐ **its own gate**, `every_stdlib_declaration_name_survives_the_faithful_surface` |
| `Fault/of` occurrences | ⭐ **6 → 0** — gone, not reduced |
| landable floor | ⭐ **6013 / 6013 GREEN ×3** |
| clippy | 0, **forced fresh** (8.32 s) — ⚠ the IDE's `dead_code` diagnostic is **stale**, third time today |
| ledger | **220 → 220, UNMOVED** |

⭐⭐ **THE 1 NEW IS THE HONEST OUTCOME.** Its own gate's non-vacuity floor (`slash_joined >= 16`)
fires on the converted tree because **the conversion takes 17 `/`-joined declarations to 1.**
⭐ **It counted that as a NEW failure and did not tune it away** — and the gate's doc says it is
**retired or re-aimed when 8d-iii lands, never loosened.** ⭐ **A second, whole-corpus confirmation
of the unspellability finding, arrived at from the other end.**

## ⛔ The ledger did not move, and it says why

**220 → 220.** Its cure's deciding expressions carry **no keyword literal**, so the discriminator
cannot see it — ⛔ **the same blind spot the seventh draw's `match_qq_head` cure hit.** ⭐ **Stated,
not papered over.** **Two cures in two stones are now invisible to the countdown that schedules the
terminal cut. That gap is itself a stone.**

## The brief was wrong three ways

1. ⛔ **My control row does not hold as written.** I wrote *"target FAIL, control PASS"* and ⛔ **never
   named the control in the brief** — it was a deliberately unrelated rete test, not a sibling. **Both
   siblings in the target's own file also FAIL**; the whole post-spawn surface goes down at once.
   ⭐ **A control's identity is part of the measurement. Name it.**
2. ⛔ **"Accessor" was the wrong word and aimed the stone at the wrong code** —
   `canonical_identity`/`reconstruct_call_path`, when the deciding code is `declare::parse`'s name
   slot and `freeze::env::rekey_type_member_functions`. The real accessor in that fixture survives.
3. ⚠ **"33 of the 112" is 31** attributed per test.

⭐ **It also says what the brief got right**: the address reproduced, *"check `Fault/of` separately"*
found a genuinely different defect with the opposite direction, and the permissive warning saved the
door.

## ⭐ Every `UnresolvedReference` in the converted floor is now ONE defect

Eight distinct paths, all `Namespace::…/member` surface joins — **seven are `wat/spawn.wat`'s builder
constructors, the eighth is the deliberate bogus field.** ⭐ **The class is fully attributed.**

## VERDICT

**ACCEPTED.** census `no STOP-8` 213 = 213, delta **3 / RECOVERY 0**, 0 live `.wat`.
**Twenty-six corrections across twenty-four stones.**

⛔⛔ **THIS NEEDS THE BUILDER.** The spawn seven are **fenced, not cured**, and **8d-iii cannot run
until they are respelled** — a public-API rename across 27 live files. **That is a ruling, not a
rider's call.**

**Ninth draw (either order):** the 140 `defservice`-minted members over 11 lower-case parents
(255.8's `stdin-svc/start` family) and the 416 `Enum.Variant/field` accessors — ⭐ **the rider
narrowed its census to declaration names BY ARGUMENT, NOT MEASUREMENT, and flagged it** · or shape
**E**, ⛔ **125 sites, 78 in `check.rs`, untouched across eight draws.**
