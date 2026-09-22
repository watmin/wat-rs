# BRIEF — STONE 255.10: the keyword requirement IS the defect

**Drawn 2026-09-22 against `main` @ `fe4c0923d`.** Floor 5962/5962, clippy 0, census `no STOP-8`.
Delta baseline **18**, on the committed list
`docs/arc/2026/06/251-types-as-forms/delta-sample-179.txt`. ⛔ **USE THAT FILE.**

## ⭐⭐ THE BUILDER'S RULING, 2026-09-22

> *"all of the conversions you detailed are correct — we need to break off of the keyword
> requirements onto symbols"*

**The symbol spelling is correct wat. A reader slot that REQUIRES a keyword is a defect by
definition.** Not a site to patch when it bites — **a class to eliminate.** This stone stops curing
instances and starts removing the requirement.

## The discriminator — already implemented, in the codemod

| keyword | what it is | converts to |
|---|---|---|
| `:wat::core::defrecord`, `:my::svc::Oops` | a **NAME** (carries `::`) | ⭐ symbol — `wat.core/defrecord` |
| `:messages`, `:features`, `:nature` | a **syntax MARKER** | ⛔ stays a keyword |
| `:Ok`, `:Bad` — a variant tag in a **declaration** | ⭐⭐ **A NAME.** ⛔ **RULED 2026-09-22, correcting this brief.** `(wat.core/defenum u/Whatever wat.enum/Pure :- [T] Thing [t :- T] Wut [])` mints `u/Whatever.Thing` / `u/Whatever.Wut` — the tags are **bare symbols**, not keywords | ⚠ **NOT YET** — measured: the codemod leaves `:Ok` alone, while the minted name already renders `wat.core/Option.Some`. **The declaration and what it mints disagree TODAY.** Closing it needs a CODEMOD change — ⛔ **OUT OF SCOPE for this `src/` stone; assert NOTHING about variant tags in either direction.** |
| a map key `{:value 7}` | **DATA** | ⛔ stays a keyword |

⭐ **The rule the readers must mirror:** a slot expecting a **NAME** accepts both spellings through
the identity door; a slot expecting a **marker** or **data** is untouched. ⛔ **Both halves are
load-bearing. A cure that lets `:messages` be read as a name is worse than the defect.**

## The evidence — 83% of the residue is ONE habit

Measured on the committed sample: **15 of the 18** failures are three user forms, and every one of
them — plus both greens 255.9 forged, plus the two tables that stopped 8d-ii — is **one slot that
pattern-matches a spelling instead of asking for identity.**

| form | slot | files |
|---|---|---|
| `defsurface :messages` | the message **NAME** (`surface.rs:746`, `Some(WatAST::Keyword(mn,_))` ⇒ list empty ⇒ **every declared type reported UNDECLARED**) | **8** |
| a `defn` inside a rete rule | the **head** (identity is right; not extracted) | **4** |
| a macro building a keyword from a name | `keyword-node`'s **input** | **3** |

⛔ **The `defsurface` one is a FALSE RED that also HIDES THE REAL ONE** — measured: the original
correctly names a genuinely-missing type; the converted reports a **correctly-declared** type instead
and never reaches the true defect.

**Known seeds, all measured** (`surface.rs:746` · `surface.rs:1003` · the rete `defn` head ·
`keyword-node`'s input · `edn/render.rs:2508`'s hardcoded type table · `function/subsume.rs`'s
`value_matches_type_by_name`). ⭐ **The last two are the same habit at the STRING level** —
`p.as_str()` against a literal — so they belong to this class, and they are what stopped 8d-ii.

## ⛔⛔ 634 KEYWORD READ-SITES. DO NOT CURE BLIND.

`grep -rn "WatAST::Keyword("` under `src/` returns **634**. ⛔ **A sweep over that is exactly the
shape that cost 255.4 thirty-two reds.**

⭐ **255.9 proved the filter and it is the method here:** classify by **REACHABILITY**, not by grep.
`normalize_symbol_refs` is **step 7**; anything at or after it in a code position can never see a
symbol, and data positions are never normalized at all. **That took 82 sites to 2.** Do the same.

⚠ **255.9 also named the limit of its own argument** — *"every UNREACHABLE row is a pipeline-order
argument, not a probe; re-order any pass relative to step 7 and they all go live silently, with no
gate on that ordering."* **That is still true. Say so again if it is still true after your census.**

## ⭐ THE GATE ON EVERY CURE: EVIDENCE, NOT COUNT

⛔ **No cure without a file or test that CHANGES STATE** — red→green, or a **lying diagnostic to a
true one**. Name it per site. A site you believe is wrong but cannot make fail is **REPORTED, NOT
CURED** — that is a wanted outcome, not a failure.

**If the reachable set is larger than you can cure with evidence, CURE WHAT YOU CAN AND STOP.**
A census plus three evidenced cures beats twelve speculative ones.

## ⛔ NON-VACUITY — pin BOTH halves, in one test

1. ⭐ A **name** slot accepts both spellings and produces the **same identity**.
2. ⛔ `:messages` / `:features` / `:nature` are **still markers** — a surface that spells one as a
   symbol is still refused.
3. ⛔⛔ **STRUCK 2026-09-22 by the builder's correction.** This row used to read *"a variant tag (`:Ok`) is still data"*. **It is a NAME.** ⛔ **Assert NOTHING about variant tags** — not this, not its opposite. Pinning either way pins behaviour that is mid-ruling. If the census finds a site where a variant tag's spelling is load-bearing, **REPORT it under "needs a ruling".**
4. ⭐ The `defsurface` wall still **FIRES on a genuinely undeclared type**, in **both** spellings —
   ⛔ **this stone must not turn a false red into a false green.** Use the measured probe:
   a `:messages` block whose response enum references a record absent from `:messages`.

## The work

1. **The reachability census** — every name slot a converted form can reach. Report it whole,
   including UNREACHABLE rows and why.
2. **Cure with evidence**, through the identity door. ⛔ **One door — not a spelling test per site.**
3. **Report the rest**, each with its reason and what it would take.

## The gate

- The four non-vacuity rows, **in one test**.
- ⭐ **The 8 `defsurface` residue files** — say how many close. **If fewer than 8, say which and why.**
- `scripts/floor.sh` green; clippy `-D warnings --all-targets --workspace` 0; census `no STOP-8`.
- Delta **< 18** on the committed list, ⭐ **and report the RECOVERY column (fail→clean)** —
  ⛔ **any non-zero recovery is a STOP until explained.** It is the gate that would have caught a
  green-forging conversion for eight stones and nobody was printing it.
- ⛔ **NOT ONE `.wat` CONVERTED** in the live tree. This is a `src/` stone.

## Doctrine

- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Capture whole, never re-run to green, name the arm.
  ⭐ The last three executors each disclosed a red and **all three stones were accepted because of it.**
- ⛔ **REPORT WHAT A DOOR CANNOT EXPRESS.** Eleven stones running.
- ⛔ **If this brief contradicts the code, THE CODE WINS.** ⭐ **Seventeen corrections across fifteen
  stones — and the last stone corrected this orchestrator THREE times**, including catching it
  measuring on a sample its own brief forbade. **Assume an eighteenth.**
- ⚠ **Reformat in its own commit, or not at all.**
- Work only in `/home/john/work/holon/wat-rs`. **Do not push.**

## Out of scope

The 416-test rete remainder (downstream of the two type tables) · 255.8's wrong-join acceptance ·
8d-ii (fourth draw) · 8d-iii · a gate on the step-7 ordering.
