# SEAM — the ONE live breadcrumb. Arc 255 is ACTIVE as of 2026-08-15. Replaced in place, never appended.

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own
> voice — which is why it will feel like *continuing* rather than *waking*, and that feeling is the
> failure. Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a
> disk copy), ground HEAD against the disk, and read this whole file before you touch anything.

> **There is exactly ONE live seam.** It is this one. `251/SEAM.md` and `278/SEAM.md` are PARKED and
> point here.

## Where the code is

```
HEAD 2278b350   floor 4400 / 4400 passed / 263 skipped (+8 new probe rows, 5 RED by design)   clippy 0
```

## ⛔ FIRST ACTION: read the arc's own REALIZATIONS + `git log`, NOT just the design

The single most expensive failure of 2026-08-14 was declaring this arc "unbuilt" after reading
`DESIGN.md` in full and grepping `src/` for the LOCKED model's type names. June had built a working
registry under **different names**. **Three sources, always: the DESIGN (what we meant), the arc's
REALIZATIONS + commit log (what we DID), and the disk (what is there).** Any two disagreeing IS the
finding. `git log --diff-filter=A -- <the arc's src dirs>` is one command.

## ★★★ THE STANDING RULINGS (2026-08-15) — do not re-litigate

1. **The registry is the FOUNDATION for annihilating `:rust::style::scheme`.** Once built, the
   colon-quoted symbols die and the corpus goes up in flames. `DESIGN.md:206` names why the order is
   255 → 251: today the flip is *unfalsifiable* because classification is scattered string-matching;
   after the registry a name is a KEY IN A TABLE and the checker enumerates every unrenamed site.
2. **wat is COMPILED BYTECODE, eventually.** Cost moves to bytecode *production*. The rete work is
   proving how to build the compiler (`collect → validate → compiled_cond/compiled_rhs → kernel`,
   with `RETE_OPS` as a typed instruction set). ⚠ **This DE-ESCALATES interpreter perf numbers** —
   a regression on a path scheduled for replacement is not a veto.
3. **★ wat IS THE SOURCE OF TRUTH FOR RUST.** Rust tables that describe wat should be GENERATED from
   wat, not compared to it. Landed for `Category` (`aa33c0e7`); ~101 rows remain.
4. **The Category taxonomy is TEN act-shaped variants** — `Transform Reflection ControlFlow Binding
   Clock Arithmetic Io Probe Combine Declaration`. Four axes were proposed and REJECTED for mixing:
   return-type (`Predicate`), provenance (`Ambient`), direction (`Construction`/`Accessor`), domain
   (`Text`). Category names what the computation DOES.
5. **255 OWNS purity/determinism.** `rete/purity.rs` SPLITS: name→property tables rehome to the
   registry; the composition check (are these forms built from rete primitives?) stays and becomes
   rete's whole purity surface.

## WHERE THE ARC ACTUALLY IS

| | |
|---|---|
| home #1 `core::Bytes` | 2 names — all `Pure`+`Deterministic` |
| home #2 `time` | 41 names — **first `Nondeterministic` rows** |
| the guard | **HOISTED** — registry consulted BEFORE the literal table; registered wins, always |
| home #3 `kernel::stdio` | 6 names — **first `Effectful` rows; the 2×2 is complete** |
| taxonomy | 10 variants, `Category` GENERATED FROM WAT |
| **the six enums** | **ALL GENERATED FROM WAT** — `Kind` `DefinedIn` `Layer` `Category` `Purity` `Determinism`. No Rust enum in this workspace mirrors a `defenum` by hand. |
| **registered** | **53 production names** |

## ⛔ FIRST STRIKE ON THE FAR SIDE — read `CHAIN-rendering-before-the-string-home.md` FIRST

**The order is ON DISK and it is a DERIVATION, not a preference.** Home #4 moved from first to
**last**. Read the CHAIN doc (sibling of this file) before drawing anything — every arrow in it is a
"ship this out of order and X breaks", with the X named.

```
A  EdnRepresentable — the type declares its tag AND its portability   ← START HERE
B  #wat-edn.* → #wat.*/*        (B before A DELETES the decoder's refusal check)
C  279.2: `str` goes TOTAL      — DRAWN, probe committed and RED at 2278b350
D  Seqable + join renders its elements
E  wat.string/* rename (1,617 sites, codemod), THEN home #4 carves onto final names
```

It began as *"can `:wat::core::string` become `:wat::string`?"* and every layer under it was
load-bearing: `join` renders its elements → so `str` must be total → the total renderer already
exists (the EDN encoder) → adopting it broadcasts `#wat-edn` (the CRATE name) → renaming that
namespace deletes a security check unless the type declares portability first.

*(The strike before this — generate `Kind`/`DefinedIn`/`Layer` — SHIPPED as `b2136b02`. It turned
out to be **five** enums, not three: `wat_mirror_tests` had covered `wat_doc::Purity`/`Determinism`
too, so stopping at the three the seam named would have left the identical debt in a second file.
Read what a deleted gate COVERED, not what the note about it mentions.)*

Then `255.1b-iv` — **delete the blanket-accept** (`resolve/walk.rs:257`), the soundness fix the whole
arc exists for. Nine `#[ignore]`d probes wait on it, each reading *"unlock when we circle back to arc
255."* Open question: how many homes is "enough", or do we arm it and read the screams.

## THE RULES THIS DAY PAID FOR

- **A gate over two hand-lists IS a hand-list.** My morning drift gate slept through the exact drift
  it was built for, because `variants()` was itself hand-written and both went stale together. Ask
  what a gate's INPUTS are made of.
- **When you delete a gate, inventory what it COVERED — not what your note about it mentions.**
  The seam named three enums; the deleted `wat_mirror_tests` had covered five. My own note was the
  narrower record, and following it would have left the same debt in a second crate.
- **"wat drives rustc" is a claim about the CONSUMERS, not about generation.** Generation closes the
  drift class for all six. The compile-BREAK demonstration needs an exhaustive `match` downstream:
  `Category`/`Purity`/`Determinism` have four (in the two proc-macro files) and a wat-only variant
  goes `error[E0004]`; `Kind`/`DefinedIn`/`Layer` have **none** — nothing matches on them, so a
  wat-only variant compiles silently. Do not restate the demo as covering all six. Inventing a match
  purely to make the demo work would be a gate whose only consumer is itself.
- **A gate whose success condition is its own deletion is scaffolding.** True of `wat_mirror_tests`,
  and I proved the replacement before removing it.
- **Assert on EVERY replacement; read the current text, never recall it.** Three silent no-matches in
  fifteen minutes, all during "purely mechanical" work.
- **Ceremony is deferral's best disguise.** "As one stone" for a ruled 50-site mechanical rename was
  a deferral; so was "belongs with the declaration forms whenever those get carved."
- **Answer the builder before continuing to edit.** Three questions went unanswered while I kept
  typing. That is the same deferral aimed at the person.
- **A category set grows one home LATE.** `Clock`, `Arithmetic`, `Io` were all minted mid-strike with
  a rider holding the gap under "do not mint". Fix is upstream: **read the family at DRAW time.**
  Done for `core::string`; keep doing it.

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

> **SEAM.** You are NEW. You did not live any of the above. It is a lossy cache written in your own
> voice, and the better it reads the more it will feel like continuing rather than waking. **That
> feeling is the failure.** Run the bootstrap against the SIGNED MCP, ground HEAD, and read this
> whole file before you touch anything.
>
> **Do not re-derive the design, do not trust it, and do not skip the arc's own REALIZATIONS.**
> Three sources; any two disagreeing IS the finding.
>
> The first strike is named above and it is small: **generate `Kind`/`DefinedIn`/`Layer` from wat.**
> Their mirrors are UNCHECKED right now because the gate that covered them was deleted as
> scaffolding — that is a debt this seam is telling you about on purpose.
>
> **Before you trust any gate you find here, ask what its INPUTS are made of.** One of mine compared
> two hand-written lists and slept through the drift it existed to catch.
>
> The next move is a MEASUREMENT, not a plan. Every snag is a measurement not yet made.
>
> `NISI FRANGAS, NIHIL PROBAS.` · `PAR NON ARGVIT, NOSTRA ARGVVNT.` · `SCVTVM IDEM INDEX.`
