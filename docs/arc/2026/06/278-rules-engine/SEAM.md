# SEAM — the ONE live breadcrumb for arc 278. Replaced in place, never appended.

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own
> voice — which is why it will feel like *continuing* rather than *waking*, and that feeling is the
> failure. Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a
> disk copy), ground HEAD against the disk, and read this whole file before you touch anything.

> **There is exactly ONE seam. If you find a second, one of them is lying — prune it.** History
> lives in `REALIZATIONS.md`.

## Where the code is

```
HEAD 00de6d18   pushed   floor 4391 passed / 0 failed / 262 skipped   clippy 0
```

Tree clean. ⚠ **One commit of drift at wake is EXPECTED** (this file commits on top).

**⛔ `stash@{0}` STILL HOLDS THE LIFECYCLE STRIKE — do not `git stash drop`.** Made with `-u`, so
`git stash show --stat` **cannot see the untracked payload**; read it with `git show 'stash@{0}^3:<path>'`.
Its `.wat` is STALE (`--check` exit 1, five errors, four sharing one root: a `DisconnectReason`
scrutinee resolving to an unbound type var). Restoring it turns the floor red.

⚠ **`--check <f> | tail` returns TAIL's exit code.** Read exits unpiped, always.

## ★ THE LIVE FORK — the builder's to rule, and everything is blocked on it

`defrule`'s macro cannot type the parameters of a lifted `where` body, because **macro expansion runs
before the type registry is populated.** Verified independently, by a *calling* probe:

```
field-names-of form: unknown type ':usr::Temp'      ← even for a stdlib type
```

Three escapes are dead by run (see the ⛔ box atop `DESIGN-STONE-defrule-splits-at-expansion-time.md`
for the verbatim evidence): a bare type var, `:wat::type::Infer`, `:wat::core::Value`; Clara's
pass-the-fact shape (wat's rete binds a **field**, not a fact — `matcher.rs:295`); and deferring via
a type expression (`TypeExpr` has no field accessor — `types.rs:72-92`).

★ **THE GAP IS NOT RETE-SPECIFIC.** Any DSL lifting a user body into a typed defn hits it. Same
shape as the privilege we are deleting: the language is missing something every DSL needs.

**THE THREE ROUTES, UNRULED:** (1) a macro-phase reflection door · (2) a type-level field accessor
the macro emits and the checker resolves · (3) a whole-fact binding in the rete surface (Clara's
shape — changes the user surface).

## What this arc is actually for (re-anchored by the builder this session)

Rete's remaining cost is **interpreted wat — ~95% of the work at stress-test scale**. The
purity/determinism/totality campaign existed to make rete's forms **compilable**. The jump table
(#49) implements function calls. Two of three compilers are built (`src/rete/compiled_cond.rs`,
`compiled_rhs.rs`); this is the third, and the other two get failed over once it lands.
**rete first, wat second — this is the proving ground for compiling wat itself.**

The standing frame, verbatim: *"i fucking hate seeing dsl machinery hard coded in our rust — this
means /every fucking dsl we envision/ requires rust changes."*

## ★ WHAT LANDED — four commits, docs + probes only, ZERO src since `228b68fa`

| commit | |
|---|---|
| `228b68fa` | **`closure_extract` honours `MatchesSubject`** — a `matches?` in a fn body blocked closure extraction entirely; proven, fixed, gated |
| `cf732fce` | the stone + **three probes** committed *before* the brief (examinare) |
| `9ffbf9c7` | the **intueri cast** — `:usr::ok-rule$where0` / `$where0` |
| `00de6d18` | **BRIEF + EXPECTATIONS**, and every loose measurement given a durable home |

## PROVEN this session — by run, with non-vacuity controls. Do not re-derive.

- **The delivery defect is real.** A fn called only from inside a quoted `where` is **never shipped**
  — `PC 6 · BASE 5 · SUBJECT 5`. This is "we fail to deliver rules to install-rules", mechanised.
- **One mention ships the whole chain, transitively** — `PC 7 · BASE 5 · MENTION-1 7`.
- **A macro CAN mint a computed-name top-level defn plus its consumer** (STOP-3 cleared). The
  template may not introduce a *literal* binder (hygiene gate E, arc 249) — splice it via
  `~(:wat::core::symbol-node "…")`, as `core.wat:1163` does.
- All three probes live in `wat-scripts/scratch-pad/` under `every_wat_scripts_file_loads`.

## ⛔ FOUR SHAPES REFUTED BY RUN — each died to evidence, not argument. Do not re-propose.

1. **quasiquote's `~` as the code hole** — `runtime.rs:10891` *evaluates* the escape; the form dies.
2. **Making `forms` resolve-transparent** — `forms` is the **CHILD-PROGRAM constructor**; it must not
   resolve locally. Imposed the check: floor **4367/24**, failures clustering on the *services* path.
   `probe_resolver_quote_awareness.rs:19` states the contract; its fixture names `ghost-inner`
   deliberately.
3. **A `fn` on the `Rule`** — 293.W containment refuses it. Rules are records **because they cross
   the wire** (R5's `{facts,rules}` snapshot). The "precedents" I cited were `defstruct`s and a defn
   *parameter*; `Rule` is a `defrecord`.
4. **The `Condition` ADT** — ruled **4/4** and then **void**: it existed only to hold the fn that (3)
   proves cannot be held.

## ⛔ ALSO OPEN

- **#90 — `walk` skips the FIRST TWO ARMS of every `match`.** Proven; 817 forms affected;
  `NOTE-walk-skips-the-first-two-arms-of-every-match.md` carries the repro inline. Untouched.
- **`MakeRule` refused in `closure_extract` costs rule delivery TODAY** (`6/5/5`). **Holding that
  line is deliberate** — honouring it adds a fourth Rust consumer of rete's name, which is what the
  stone deletes. Cost stated, not hidden.
- **The index is UNMEASURED.** Every proof used exactly ONE `where` per rule.
- `NOTE-a-rule-may-reference-an-unbound-variable-and-compile-clean.md` **Tier 0** still owed.
- **Filed, not scheduled:** `109/NOTE-two-resolvers-over-the-five-registries.md`. Owed intueri casts:
  the admission type; the correlation surface. Older: #87 · #49 · #7 · #17 · #19 · #20 · #50 · #58 ·
  #60 · #64 · #67 · #81.

## The rules this stretch paid for

- **The four questions cannot see a premise BOTH options share.** I ruled the `Condition` ADT 4/4
  against a side-table and never asked whether a fn could live on a `Rule` at all. Neither could.
  The questions discriminate *between* options; they do not validate what the options rest on.
- **A probe that never invokes the thing proves nothing.** My first verification of the rider's
  blocker returned EXIT=0 — because it *defined* the macro and never *called* it, so the body never
  ran. I nearly overturned a correct STOP on it.
- **An empty match arm is a discard wearing diligence's clothes.** `MatchesSubject | MakeRule => {}`
  sat beside `Ordinary => {}` under twenty lines of principle, behaviourally identical to it, hiding
  a live bug.
- **A refusal that removes no privilege is a gesture.** Refusing `MakeRule` in one consumer left the
  door, and three other consumers, exactly as they were.
- **The builder names the subject; answer THAT.** Asked whether two *enum variants* were dead, I
  measured the *wat functions* of similar name and proposed deleting live machinery.

---

> **SEAM.** You are NEW. The disk is the truth; this note is a lossy cache.
>
> This stretch the record's own discipline did the work: four designs of mine died to runs, a rider
> STOPPED correctly and killed a fifth, and every one of those deaths made the answer better. The
> shape that survives — *the macro does the split, the way Clara's does* — came from the builder's
> cut, **"why doesn't clojure need some new form?"**, and the answer was three lines above
> `defrule`'s own template the whole time: *"The macro is kept TRIVIAL."*
>
> Do not trust confidence here. Trust the probes; they are committed and they run.
>
> `NISI FRANGAS, NIHIL PROBAS.` · `IN TENEBRIS VISVS CORRIGOR.`
