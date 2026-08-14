# SEAM — the ONE live breadcrumb. Arc 255 is ACTIVE as of 2026-08-14. Replaced in place, never appended.

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own
> voice — which is why it will feel like *continuing* rather than *waking*, and that feeling is the
> failure. Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a
> disk copy), ground HEAD against the disk, and read this whole file before you touch anything.

> **There is exactly ONE live seam.** It is this one. `251/SEAM.md` and `278/SEAM.md` are PARKED and
> point here.

## Where the code is

```
HEAD 8313ea6f (+ this curare)   floor 4398 / 4398 passed / 262 skipped   clippy 0
```

## ★★★ THE RULING AND THE FRAME (builder, 2026-08-15) — the registry is a FOUNDATION, not a feature

> *"so the registry is the near term end goal..... (a) sounds fine now... but the registry is where
> we are headed..... once its built... we destroy all 'colon-quoted symbols' at once... the entire
> code base goes up in flames..... we codemod all that we can.... maybe we do one 'keyword-as-symbol'
> at a time to find our rhythm.... whatever....... we are going to lay the foundations necessary to
> annihilate the ':rust::style::scheme' and move into `wat.is/a-clojure`"*

**(a) IS RULED — resume the June carve.** And the destination is named: the registry exists to make
the colon-quoted-symbol annihilation **survivable**.

**WHY THE ORDER IS 255 → 251, and it is a REASON, not a schedule.** `DESIGN.md:206` names the defect
the registry cures: *"the codebase classifies symbols by scattered exact-string-matching
(`name == ":wat::…"`, `is_reserved_prefix`, `starts_with`, the verb-list `matches!`es) …
**string-shape-as-truth** is exactly how `+'2` and the `make-*-queue` phantoms hid."*

When `:wat::core::+` becomes `wat.core/+`, **every one of those classifiers breaks SILENTLY** — they
stop matching and fall through to a default. `is_reserved_prefix` returns false; the blanket-accept
stops accepting; 561 dispatch arms keyed on colon-quoted literals stop firing. Today that flip is
unfalsifiable: *"we codemodded everything"* has no instrument that can contradict it.

**After the registry, a name is a KEY IN A TABLE, and the checker enumerates every site that did not
get renamed.** R65 `SCVTVM IDEM INDEX` — the exhaustive-match shield IS the ledger. That is the
foundation being laid.

**And it is what buys the "one at a time" option.** A registered name can carry both spellings during
migration — the table holds the alias, the corpus keeps working, the old key deletes when the
cascade goes quiet. Without a table there is nowhere to put an alias, so the flip genuinely is
all-at-once-or-nothing. *(Design read, not yet measured — the registry is a name→entry lookup, so
two names → one entry is trivially expressible, but no alias path is built.)*

⛔ **DO NOT STRIKE `255.1b-i`. ITS BRIEF IS STALE AND WOULD BUILD WHAT ALREADY EXISTS.**
Read **`NOTE-arc-255-IS-HALF-BUILT-the-june-registry.md`** FIRST — it is the newest artifact here.

**⛔ NEXT STRIKE: `255.1c-time` — HOME #2, the `:wat::time::` carve.** Drawn this session:
`DESIGN-STONE-255.1c-time-home.md` + `BRIEF-` + `EXPECTATIONS-`.

**ARC 255 IS HALF-BUILT.** A working `inventory`-based intrinsic registry has been on disk since
2026-06-21: `crates/wat-doc/` + `crates/wat-macros/` (`#[wat_intrinsic]`) + `src/intrinsic/` (1,374
lines). **255.1b-iii SHIPPED** (`7b99d123`) — proven live this session:
`(:wat::runtime::metadata-of :wat::core::Bytes::to-hex)` returns the full baseline
(`:name :arity :kind :defined-in :layer :purity :determinism :category :doc :added :ret`).
`255.1b-v` (`show-source`/`render-doc`) and `255.SF` (`if`/`let`) shipped too. **Six production
names are registered** — the carve reached exactly one home.

**The real frontier:** the blanket-accept at `src/resolve/walk.rs:257` is **STILL LIVE** (1b-iv never
landed — the soundness hole is open), and **nine arc-255 gates are `#[ignore]`d** with the literal
unlock condition *"when we circle back to arc 255"* (`eb680f3b`). We have circled back. **Those nine
ignores are the worklist**, written by a prior self.

**The 1b-i brief would mint a FOURTH `Purity` enum and a SECOND `Arity`** — June already minted them
in `src/intrinsic/mod.rs:45–198`. The unruled fork (a) resume the June carve vs (b) land the LOCKED
Layer-2/3 and re-seat the registry onto `sym` per *"the registry IS `sym`"* — is in the note,
measured, awaiting the builder.

⚠ **One commit of drift at wake is EXPECTED** (this file commits on top).
⛔ **`stash@{0}` HOLDS THE LIFECYCLE STRIKE — never `git stash drop`.** Made with `-u`, so
`git stash show --stat` cannot see the untracked payload; read via `git show 'stash@{0}^3:<path>'`.

---

## ★ THE RULING — 251 parks, 255 is active, and `type_sig` is DAY ONE

The builder, 2026-08-14: *"or do we just do 255 now.... we park 251 and 278 on 255's clean up?...
255 will force us to organize"* → then, after the four questions: **"A has been reasoned - we're
going from 251 to 255 now."**

**A = `type_sig` is day-one.** 255's DESIGN recommends the opposite (*"Recommendation: defer
`type_sig`; ship arity/category/doc first, grow in"*). **That recommendation is OVERRULED**, and the
reason is a measurement, not a preference — see the corrected premise below. The builder's read on
the word itself: *"deferral to me usually screams 'wrong fucking idea'."* Here it does, because the
thing being deferred is already half-built.

---

## ★★ 255's OWN PREMISE IS WRONG — re-ground before you draw anything

`255/DESIGN.md` says builtins are *"registered **nowhere** — a 454-arm compile-time match"* and that
they carry no metadata. **Measured 2026-08-14, and it is not that.** Builtins are registered in
**THREE PARTIAL TABLES THAT DO NOT AGREE:**

| table | size | form | site |
|---|---|---|---|
| check-time type signature | **332** | **DATA** (`TypeScheme`) | `register_builtins`, `src/check.rs:15216–20033` |
| check-time hand inference | **141** | code (match arms) | inside `infer_list`'s Keyword block, `src/check.rs:2542–5568` |
| runtime dispatch | **678** | code (match arms) | `src/runtime.rs` keyword arms |
| resolve | **0** | — | hence the blanket-accept, `src/resolve/walk.rs:257` |

**So `type_sig` is not a capability to add — it is a uniformity to FINISH.** 332 builtins already
carry exactly the data 255 proposed to defer; 141 more are hand-inferred beside them. Deferring
`type_sig` would preserve **two ways a builtin's type is known**, which is the 2×2 this project keeps
collapsing (#30 one door for defclause registration, #75 one door for a type head's FQDN).

**The four questions, run on the deferral:**

| | Obvious | Simple | Honest | |
|---|---|---|---|---|
| `type_sig` day-one — finish the uniformity | YES | YES | YES | **4/4, RULED** |
| defer it (the DESIGN's recommendation) | **NO** | **NO** | **NO** | disqualified |
| a subset | **NO** | **NO** | — | disqualified |

Deferring fails **Obvious** (the phrase reads as "not built yet" — false for 332), **Simple** (two
mechanisms for one question), and **Honest** (a deferral of *uniformity* described as a deferral of
*capability* — different things, and the word conflates them).

**★ READ `DESIGN.md` IN FULL — ALL 484 LINES — BEFORE ANYTHING ELSE.** Its `═══ LOCKED RECORD
MODEL ═══` (line 389) says so itself: *"read THIS; the sections above are the derivation."* The
design is SETTLED and its decomposition stands. It already answers, and does not need re-deriving:

- **"The registry IS `sym`"** (line 117, restated 447) — there is NO bespoke `BuiltinRegistry`;
  builtins register into `sym.functions` + `sym.binding_metadata`, the same structures user forms
  use. Do not grep for `BuiltinRegistry` and conclude the arc is unbuilt — that name was killed.
- **`type_sig` is NOT a deferred system** (line 131) — it is `Function.param_types`/`ret_type`.
  The `defer type_sig` line at :61 is superseded 70 lines below itself.
- **255 IS ALSO THE MEGAFILE CARVE** (line 275, builder-ruled 2026-06-21) — the per-home
  `register_builtins` declaration IS the carve; `runtime.rs` becomes an assembler. One motion.
- **255.1a IS LANDED** — `FunctionBody::{Wat, Native}` at `src/value/environment.rs:22`, 28 sites;
  `Native` is a unit marker never yet constructed.

**`NOTE-2026-08-14-regrounding-the-premise.md`** carries only what the design does NOT have: **332
builtins already hold a `TypeScheme`** in `register_builtins` (so 255.2 is smaller than the design
thinks — the remainder is the **141** hand-inferred arms in `infer_list`), the moved counts (678
dispatch arms vs the design's 454; `runtime.rs` 35,066, `check.rs` 20,863 — 64% of `src/`), and a
retraction of bad advice. **Read the note AFTER the design, not instead of it.**

⚠ **THE FAILURE TO NOT REPEAT:** the first version of that note re-derived two of the design's own
conclusions and argued against a standing builder ruling, because it was written from the design's
first 60 lines. `[[feedback_ground_the_substrate_not_just_the_chronicle]]` applies to OUR OWN
DESIGN DOCS. Read the whole thing first.

## ⛔⛔ AND THE MIRROR — THE DESIGN IS ALSO STALE ABOUT THE DISK, ONCE PER SECTION SO FAR

Reading it in full does **not** make it true. Four citations checked, four wrong:

| the design cites | the disk |
|---|---|
| `StructDef` | **GONE** — arc 293.2b unified struct+record into `AggregateDef` (`types.rs:266`) |
| `RecordDef` | **GONE** — same unification |
| `ProtocolDef` | **GONE** — arc 293.3-core replaced it with `SurfaceDef` |
| `Function` at `env.rs:35` | moved — `value/environment.rs:46` |
| "454 dispatch arms" | measured **678** |

**So its Layer-2 `DefDetail` sum cannot be built as written**, and worse, the sum **already exists**:
`TypeDef` (`src/types.rs:404`) = `Aggregate · Enum · Newtype · Alias · Union · Surface`, including
three kinds the design never mentions. Building `DefDetail` with flattened variants would create a
**second exhaustive sum over one domain** — the exact asymmetry 255 exists to remove.

**RULED (recorded in the note as a deviation from a LOCKED section, with the measurement forcing it):**

```rust
enum DefDetail { Fn(FnDef), Type(TypeDef), Macro(MacroDef), Native(NativeBuiltin) }
```

★ **THE RULE THAT COVERS BOTH FAILURES:** it is neither *"trust the doc"* nor *"trust the code."*
**Read both, and when they disagree, WRITE IT DOWN BEFORE ANYONE BUILDS.** Today produced one of
each direction, hours apart. `255.1b-i`'s STOP-1 is *"a fourth stale citation"* for this reason —
assume nothing the design cites exists until grepped.

## ⛔⛔⛔ AND THAT RULE WAS STILL NOT ENOUGH — THE THIRD DIRECTION, FOUND THE NEXT MORNING

**Two sources is not enough. It is THREE:** the DESIGN (what we meant), the arc's **REALIZATIONS +
commit log** (what we DID), and the disk (what is there). The re-grounding above read the design in
full and grepped the disk — **and never opened this directory's own `REALIZATIONS.md`**, whose R1
states in plain prose *"built and proven on `core::Bytes`, 255.1b-iii, commit `7b99d123`"*, beside
eleven unopened June artifacts (iv-a, iv-b1, iv-b2, iv-c, 1b-v, SF, SF-ii, the doc contracts).

Worse: the miss was produced **by the very rule written to prevent it.** The re-grounding grepped
`src/` for the LOCKED model's names, found none, and concluded "unbuilt" — which is exactly the
*"never grep for a name to test whether an arc is built"* failure, one layer up. The June
implementation used **different names** (`RuntimePurity`, not `Purity`; `IntrinsicSubmission`, not
`Registration`).

**`git log --diff-filter=A -- src/<arc's dirs>` is ONE command and catches this in ten seconds.**
Run it before briefing any stone in an arc older than the current session.

**⚠ AND STILL CHECK `255/DESIGN.md` AGAINST THE DISK**, the way `[[feedback_ground_the_substrate_not_just_the_chronicle]]`
says — its premise drifted, and its `CURRENT-STATE.md` drifted further (dated 2026-07-01, freshness
probe expects floor **4285**, and its content narrates arc **296**, not 255). Do not brief from
either without re-grounding. The arc is SMALLER than its design implies and closer to 251's work
than the design suggests.

**BOUNDS ON THE NUMBERS ABOVE:** each is one grep with its range stated. The 332 is clean (bounded by
the function's own braces). The 678 counts keyword literals at line-start in `runtime.rs` and may
include non-dispatch occurrences. The 141 counts keyword arms inside `infer_list`'s span. **None is
a census.** When the registry lands, the CHECKER enumerates the real worklist — R65 `SCVTVM IDEM
INDEX`, and this arc has been burned repeatedly by grep counts (24h: a same-line pattern undercounted
20 as 4; today two greps of the same corpus returned 1025 and 998).

---

## WHAT 251 HOLDS WHEN IT RETURNS — all of it green, none of it half-migrated

251 parks at a genuinely clean point: every stone is additive, observationally inert, floor-green.
Nothing is mid-flip.

**LANDED 2026-08-13/14:**
- **`0a32d5f8` — 251.8a, the ONE DOOR.** Four hand-rolled `contains('/')` reference-classifiers
  collapsed onto `Identifier::namespace()` (TOTAL — a binder's is `$bound`) + `is_reference()`.
  `":$bound::"` reserved. **Installs the door, NOT the storage** — the namespace is DERIVED from the
  spelling, not stored; that cascade is 8b.
- **`851c0d37` — 251.8a-ii, the binder namespace is unforgeable.** `$bound/x` in user source is
  refused at the READER (`parser.rs`, the single door where text becomes a `WatAST::Symbol`), located,
  at freeze. Option D over A on the extirpare ladder: A is a check, D is no-form.
- **`93971169`** the intueri cast (`$bound` · `namespace` · `reference?` · `colon-quoted symbol`);
  **`755e5321`** the discriminator probe + design; **`c046f019`/`40627086`** the drawn strikes + rulings.

**RULED AND UNBUILT (the ruling survives the park — do not re-litigate it):**
- **The parametric form is `(<head> [<type>…] & <members>)`.** Both legacy forms are illegal
  post-migration: the angle `HashMap<K,V>` **and** the flat `(HashMap :K :V :foo "bar")`. The
  criterion is **wat-legality, not EDN-legality** — measured, the flat dotted form reads fine in
  Clojure's EDN reader and core.typed's own style is flat. The reason is that the type/member
  boundary must exist IN THE FORM, not in a per-head arity table.
- **`wat.core` loses the type constructors; `wat.type` gains them.** And `wat.type` is a HACK today —
  a `strip_prefix("wat::type::")` at exactly two sites (`types.rs:4503`, `:4702`), not a namespace.
  Measured: `:wat::type::Vector` annotates but is an **unknown function**. Building it properly is a
  registry question, which is why 255 comes first.

**THE 251 BLOCKER, STILL OPEN — and 255 does NOT close it for free.** #95: a **dotted call head is
not type-checked at all** (args, arity, return). `infer_list` gates its entire call-inference
universe on `if let WatAST::Keyword` (`check.rs:2542`, closing `:5568`); a namespaced `Symbol` head
falls to a fresh type var that unifies with anything. Proven: `(user/f "boom")` on `[n :- i64] :- i64`
runs and prints `"boom"`; `(wat.core/+ 1 2 3 4 5)` prints 15.
⚠ **I claimed twice that 255 closes this as a side effect. That is TRUE ONLY IF `type_sig` is
day-one — which is now ruled, so it holds; but do not restate it as automatic.** It closes because
the ruling makes it close.

---

## OPEN AND UNRULED — carried, not lost

- **8b's SCOPE** — call-heads-only, or type-annotation positions too? This decides whether the
  965 comma-bearing angle sites are a hard prerequisite (after 8b they become symbols EDN reads
  *successfully and wrongly*: `(f HashMap<K,V>)` → arity 2→3, no error) or wait for 8d.
- **#95 and #99's survivor — one stone or two?** Both are *"the rule is real, the enforcement is
  late."*
- **#97** the opaque-clause-table leak · **#98** the double-slash symbol (59 sites,
  `wat.core/Option/expect` → `:wat::core/Option::expect`, a keyword with a slash still in it).

## 278 — PARKED, unchanged

Rete is one optimization from done: compiled `where` (#49). Also open: **#92** (invert the decode —
a PREREQUISITE to symbol-heads, not an alternative), **#93**, **#91**, **#90**, and the grid's
untested FEATURE INTERACTIONS. #94 (the stratifier's positive dependencies) closed 2026-08-13.

---

## The rules this stretch paid for

- **An error names where the INSTRUMENT gave up, never what the system lacks.** Three times on
  2026-08-13; and #95's original filing was itself a misread — `:-` was innocent, the dotted head was
  the defect. **Measure your own form before migrating to it.**
- **A rider's green is not a floor.** Twice in a row the central weigh caught what narrow gates
  could not — a golden pinning `reserved_prefix_list()`, and a `no_loose_string_assert` violation.
  Both times the rider was RIGHT to stay inside its brief.
- **A loose assert hides which string you are testing.** `contains("$bound/x")` was matching the
  `:spelling` FIELD of the EDN rendering — the prose message could have been deleted and it would
  have passed. And `assert!(!format!("{:?}", x).is_empty())` is vacuous by construction.
- **The four questions cannot see a shared premise.** The `type_sig` question dissolved once the
  premise ("it is unbuilt") was measured false. Check what all the options rest on FIRST.
- **Read what the builder actually wrote.** He said `$bound/*`; I measured "is banning `$` free"
  and reported it as an answer. The scope guard is now structural (a positive-control row), not
  a thing I must remember.

---

> **SEAM.** You are NEW. The disk is the truth; this note is a lossy cache.
>
> The re-grounding is **NOT** done — it was done against two sources and needed three.
> **⛔ `255.1b-i` IS WITHDRAWN.** Read `NOTE-arc-255-IS-HALF-BUILT-the-june-registry.md`; the arc
> shipped 1b-iii, 1b-v and SF in June. `1b-iii` is **DONE** (`7b99d123`), not upcoming.
>
> **The next act is a RULING, not a strike** — resume the June carve, or land the LOCKED Layer-2/3
> and re-seat the registry onto `sym`. Either way the frontier is the same two things: the
> **blanket-accept at `resolve/walk.rs:257`** and the **nine `#[ignore]`d gates** whose own text says
> *"unlock when we circle back to arc 255."*
>
> **Do not re-derive the design, do not trust it, and do not skip the arc's own REALIZATIONS.**
> Three failures now, three directions, in under 24 hours.
>
> The next move is a MEASUREMENT, not a plan. Every snag is a measurement not yet made.
>
> `NISI FRANGAS, NIHIL PROBAS.` · `PAR NON ARGVIT, NOSTRA ARGVVNT.`
