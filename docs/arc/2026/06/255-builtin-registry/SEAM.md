# SEAM — the ONE live breadcrumb. Arc 255 is ACTIVE as of 2026-08-15. Replaced in place, never appended.

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own
> voice — which is why it will feel like *continuing* rather than *waking*, and that feeling is the
> failure. Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a
> disk copy), ground HEAD against the disk, and read this whole file before you touch anything.

> **There is exactly ONE live seam.** It is this one. `251/SEAM.md` and `278/SEAM.md` are PARKED and
> point here.

## Where the code is

```
HEAD fd2aa4d8   floor 4413 / 4413 passed / 263 skipped   clippy 0 (CI invocation: --workspace)
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

## ⛔ FIRST STRIKE ON THE FAR SIDE — 296: make the `field-N` case ERROR, not degrade

**The road ends at 294/296, not 255.** This seam stays the ONE live breadcrumb, but the WORK has
moved. `255/CHAIN-rendering-before-the-string-home.md` is a TAIL, and its A/B stones are void or
absorbed — read `294/RULING-holonast-and-hologram-are-both-correctly-named.md` before anything.

### What landed 2026-08-14 (each floor-green at push)

| | |
|---|---|
| `b2136b02` | **all six enums generated from wat** — no Rust enum mirrors a `defenum` by hand |
| `25d9d015` | **`str` is TOTAL** — routed through the EDN encoder. `show` REVERTED: it is a bounded SUMMARIZER (`<Vector dim=1024>`, `<Duration …ns>`), not a renderer. 27 floor reds said so |
| `21b7079f` | **294.g — the holon record's wire is PLAIN EDN.** `#t/Holo {:x 1 :y 2}`, not a 250-byte hologram. Flaw #3 CLOSED |
| `fd2aa4d8` | **the `None` door DELETED** — `value_to_edn_string` is gone; every caller names its own case |

Floor **4413 / 4413**, clippy 0 **under CI's invocation** (`--release --workspace --all-targets`).

### The next strike, and its prerequisite is already done

296's `NOTE-value-to-edn-renders-fields-positionally.md` demanded grounding before any fix. **That
grounding is now on the disk** (see `fd2aa4d8`): the `_` arm swallowed THREE causes; `EvalError` IS
registered with its fields; **the names were never missing — the lookup was never wired.**

What remains is the honest half: **`expect_passed` (`freeze.rs`) still passes `None`** because it has
no `SymbolTable`, so 296's `field-N` diagnostics blob is still real. It is now a **visible** gap with
a comment at the site rather than a hidden default. The stone: thread a registry into
`DeftestOutcome`, and make a genuinely-unresolvable name an ERROR — *"a failure to surface, not a
silent degradation to indices"* (296's own words). **294.g's rider already set the precedent one file
over**: a decode reaching a HolonRecord with no ctx now errors loudly.

### Then, in order

- **`#wat-edn.*` → `#wat.*/*`** — ~118 sites, five families (`opaque holon cap float local`). The
  crate name is in the wire format. AFTER 296, since 294.g deleted some of what would be renamed.
- **294 flaws #4 (`HolonRepresentable`, 11 impls) and #5** (HolonAST doing CODE-AST duty —
  `special_forms.rs` 17 HolonAST / 2 WatAST is the sharpest tell; task #91).
- **`Seqable` + `join`** then the **`wat.string/*`** rename (1,617 sites, codemod) — the old chain's tail.
- **`CLOSE-SEQUENCE-293-294.md` is STALE** — marks a half-landed item `▶ NEXT`, omits `294.f`, and its
  PHASE-1 block was overridden by decree seven weeks ago. It self-describes as canonical. Fix before
  working from it.

## THE RULES THIS DAY PAID FOR

- **A TOTALITY claim is only as good as its SAMPLING.** Twice: `str` was certified total by a probe
  that sampled a map, a float, a keyword, nil and a nested string — every shape EXCEPT the one that
  consults the type registry — so records rendered `{:field-0 1}` for twelve hours. And `show` was
  called a "Rust Debug leak" from four sample outputs without asking what it was FOR (a bounded
  SUMMARIZER; 27 floor reds said so). **List the shapes your claim ranges over, then check you sampled
  the awkward one.**
- **A default you cannot see at the CALL SITE is a default nobody audits.** `value_to_edn_string`
  hardcoded `None` for the registry; 7 callers silently rendered positionally. Deleting the door beat
  fixing the callers — now a caller with no registry passes `None` in the open, where the next reader
  can ask why. The names were never missing; the lookup was never wired.
- **A STOP its own gate cannot SEE is a STOP that cannot fire.** 279.2's brief carried STOP-2/STOP-3
  and an eight-test gate that could not observe either; the rider truthfully reported "none fired
  within scope" while the floor was 28 red. 294.g's gate was the whole floor, and both STOPs fired.
- **⛔ MY LOCAL CLIPPY WAS NARROWER THAN CI'S.** Mine: `--release --all-targets`. CI's:
  `--release --workspace --all-targets`. The wide one found 4 diagnostics mine reported as 0.
  **Use CI's invocation.** Same class one rung out: `scripts/floor.sh` matches CI's nextest fine
  (CI's `--profile ci` is strictly MORE lenient — 60s kill vs 30s), so the test gate was never the gap.
- **"Commit the disconfirming probe RED" and "CI on every push" are incompatible by construction.**
  The probe MUST be red at that commit — it is the evidence — and CI cannot tell red-by-design from
  red-by-regression. Every properly-drawn stone reds CI for the window between the probe commit and
  the stone commit. Not a defect in either practice; a collision to be aware of and to close fast.
- **A number you did not RUN is not a baseline.** I wrote "floor 4408 / 4407 / 1 failed" into an
  EXPECTATIONS from arithmetic — the last actual run was 28 minutes before the commit it described.
  Truth was 4412 and 2. The rider caught it.


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
