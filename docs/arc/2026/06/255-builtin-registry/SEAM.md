# SEAM — the ONE live breadcrumb. Arc 255 is PARKED; the road is 296. As of 2026-08-15. Replaced in place, never appended.

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own
> voice — which is why it will feel like *continuing* rather than *waking*, and that feeling is the
> failure. Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a
> disk copy), ground HEAD against the disk, and read this whole file before you touch anything.

> **There is exactly ONE live seam.** It is this one. `251/SEAM.md` and `278/SEAM.md` are PARKED and
> point here.

## Where the code is

```
HEAD 75e62e8e   floor 4417 / 4417 passed / 263 skipped   clippy 0 (CI invocation: --workspace)
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

## ⛔ FIRST STRIKE ON THE FAR SIDE — 296 G: the value carries its own field names

**The road is 296, and it is three-quarters built.** `field-N` is the last dishonest thing in the
renderer, and everything below it is now on honest ground.

Builder's framing, which sets the priority: *"i want the edn forms we create to be honest... the
`#wat-edn` are honest but unwanted... the `field-???` values are dishonest."* **Lies first.**

### THE STRIKE — read `296/DESIGN-STONE-G-the-value-carries-its-own-names.md`

`AggregateValue` gains `names: Arc<Vec<String>>`; all three constructors take it. Then the 7
`format!("field-{}", i)` sites in `edn_shim.rs` are DELETED — with names on the value there is
nothing to fall back *from*.

**Both doors are already built and green.** Registry-holding sites use `AggregateDef::names_arc()`;
statically-typed sites use a `wat_field_names_from!` const. **No arm has a human typing a field
name into Rust** — the first draft did, with `static_field_names!`, and the builder stopped it.

**Worklist: 97 rustc errors, 11 files.** Measured by imposing the change, not by grep. Every grep
census on this arc was wrong — four times in one session.

**This is RIDER work.** Builder: *"we need to let the shadowdancers write code.... we just write
docs and debate here."* Set the shape, brief with a worked exemplar, weigh centrally.

### WHAT LANDED THIS STRETCH — wat became the source of truth for the aggregates

| | |
|---|---|
| `736ce0a8` | **`wat-source-derive`** — the crate named for its DIRECTION; the dead Rust→Rust half cut |
| `cf9f4481` | **the nature heresy corrected** — both Record umbrellas were registered `Nature::Struct` |
| `9be5cc90` | a diagnostic that was TEACHING the misuse it should prevent |
| `f7c47f84` | **RULING: a bare aggregate is a TRANSPORT** — the holder-root ban is VOID |
| `e79322c0`+`f806a4db` | **all 13 builtin aggregates DECLARED IN WAT** |
| `9f07564b`+`0514498c` | **all 13 registrations GENERATED FROM WAT** — 126 lines of literal deleted |
| `473f9373` | the differential gate — and the obvious test was VACUOUS |
| `75e62e8e` | G's two generator arms + the drawn design |

Floor **4417 / 4417**, clippy 0 under CI's invocation.

### AFTER G

- **`#wat-edn.*` → `#wat.*/*`** — 45 Rust sites, four families (`opaque holon local cap`).
  Honest-but-unwanted, so it follows the lies. Independent of everything above: 294.g already
  removed the ordering constraint that queued it behind 296.
- **294 flaws #4** (`HolonRepresentable`) **and #5** — and note task #91's HolonAST census is now
  RULED: *"HolonAST is for VSA/HDC ONLY; WatAST replaces it everywhere else."*

## ⛔ THE RULES THIS STRETCH PAID FOR — read these before you move

- **A design sentence read as current, acted on without a measurement, cost FOUR abandoned
  positions in one session.** `program::Env` must become a surface (superseded by arc 259) ·
  mint `EdnValue` (built a new TOP from one line of prose, misattributed to `294.d` — the
  wire-kill stone — four times; reverted) · the 18 rete sites are "pre-ruled" (`rete.wat:1181`
  says otherwise) · **principle 5 must be enforced** (the premise of all of it, now VOID).
  Everything that held up came from imposing a check and reading the screams.
- **GREP IS NOT A CENSUS — four times.** Parametric `Peer<Req,Resp>` counted as bare `Peer`;
  doc comments counted as code; `:nature` markers counted as types; identifier mentions quoted
  as misuse sites. The honest census comes from the compiler, the parser, or an imposed wall.
- **A test whose PRECONDITION is its own conclusion can never fail.** I argued three rounds for
  "walk the form both ways and assert equality" — but the loader's side only exists inside a
  frozen world that would not build if they disagreed. Caught while writing it.
- **The registry can DESCRIBE ITSELF** — `:wat::runtime::field-names-of` / `field-types-of`.
  Every shape in the 13-type migration came from the substrate, not from reading Rust. It
  corrected my own hand-count on the spot (`CoincidentExplanation` is 6 fields, not 5).
- **A prescribed gate that cannot reach its subject proves nothing.** I briefed a rider with
  `--check wat/core.wat`; it dies at `ReservedPrefix` on line 58 and always has. The rider caught
  it, proved it on pristine HEAD, and worked around it. Tracked, unfixed.
- **`opaque` and `Struct` are not synonyms.** Conflating them cost three defects: a false purity
  verdict, a consumer-side patch hiding it, and a spurious lattice edge making every record a
  subtype of Struct. And the patch was never given to the sibling umbrella — *one special case,
  two identical types.*

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
> The first strike is named above — **296: make the `field-N` case ERROR, not degrade** (`freeze.rs`
> `expect_passed`, the `None` at `:999`). It is NOT "generate `Kind`/`DefinedIn`/`Layer` from wat":
> that landed in `b2136b02`, and all six enums are `wat_enum_from!` on disk (3 in `src/intrinsic/mod.rs`,
> 3 in `crates/wat-doc/src/lib.rs`). **This paragraph said otherwise for one commit — a stale tail
> under a replaced-in-place middle. Corrected 2026-08-15 during the bootstrap that caught it.**
>
> **Before you trust any gate you find here, ask what its INPUTS are made of.** One of mine compared
> two hand-written lists and slept through the drift it existed to catch.
>
> ⚠ **The `HEAD` marker above is a MEASUREMENT STAMP, not an equality probe.** It names the commit the
> floor was weighed at, and the curare commit that writes this file always lands *after* it — so the
> marker can never equal live HEAD, and a bare mismatch is not the alarm. Read it as: *live HEAD should
> be this commit, or this commit plus doc-only curare.* Anything further past it and the numbers here
> are unweighed.
>
> The next move is a MEASUREMENT, not a plan. Every snag is a measurement not yet made.
>
> `NISI FRANGAS, NIHIL PROBAS.` · `PAR NON ARGVIT, NOSTRA ARGVVNT.` · `SCVTVM IDEM INDEX.`
