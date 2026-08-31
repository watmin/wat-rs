# DESIGN SEAM — the registry must know what wat ships

> **Builder, 2026-08-30**, on discovering `sort$native` cannot `@see` its own public wrapper:
> *"and sort is a wat defclause — hrm.... this feels like... the rust side should know what wat
> ships... the tooling should be in the registry too?..."*
>
> **A SEAM, not a stone. Nothing is drawn.** The measurements are here; the shape was the builder's
> to rule, because it is architecture, not cleanup.
>
> ✅ **THE SHAPE IS NOW RULED** — properties declared as wat DATA in the metadata map, lifted at
> build time by a `wat_enum_from!`-shaped macro. Two questions remain open (lifecycle, and whether
> it reaches the Rust half); the middle one is answered.

## The measurement

```
the registry KNOWS        431   Rust intrinsics (429 #[wat_intrinsic] + 2 #[wat_special_form])

the registry is BLIND to  409   wat-defined callables
                                  defn 340 · defclause 23 · defmacro 41 · defalias 5
                          156   wat-defined types
                                  defrecord 103 · defenum 42 · defstruct 11
```

**Nearly a 1:1 split.** "The registry is the sole source of truth for the substrate" is true of
**half** the substrate.

Measured directly through the reflection surface:

```clojure
(:wat::runtime::metadata-of :wat::core::sort$native)  ;; => Some {full record}
(:wat::runtime::metadata-of :wat::core::sort)         ;; => None      a wat defclause
(:wat::runtime::metadata-of :probe::double)           ;; => None      a wat defn
```

## ⛔ AND IT IS WORSE THAN OMISSION — the surface ASSERTS provenance it never measured

`src/runtime.rs:13617-13624`, unconditional:

```rust
put(":kind",       …to_enum_value(&entry.kind));                    // a real FIELD
put(":defined-in", …to_enum_value(&DefinedIn::Rust));               // a CONSTANT
put(":layer",      …to_enum_value(&Layer::Substrate));              // a CONSTANT
```

`:kind` is derived from the entry. `:defined-in` and `:layer` are **spliced literals sitting right
beside it**, so a reader cannot tell which fields are data and which are decoration. Today the
constant is accidentally true — everything the registry knows *is* Rust. **The moment one wat verb
registers, `metadata-of` starts lying**, and it will lie in the one field whose entire purpose is to
say which half of the substrate a verb came from.

★ This also corrects `WORKLIST-the-registry-properties.md`, which marks `defined_in`/`layer`
⛔ *DO NOT BUILD — it would be a CONSTANT*. That was right about the entry field and **missed that
the reflection surface already publishes them**. Not built, and already shipped.

## The convergence nobody planned

A wat `defn` declares no `@Purity`/`@Determinism`/`@Totality`/`@ExpandTime`. Six stones ago that
would have made registration impossible without a syntax change.

**It is now possible without one.** `classify_expr` + `ClassifyCtx` + `find_axis_violation_ctx`
(A-2-i, A-2-ii-a) derive exactly those axes **from a body AST** — which is what a wat `defn` is.
The machinery built to gate `sort$native`'s comparator is the machinery that could give 409 wat
verbs honest axes without anyone declaring anything.

⚠ **And its limits are already measured, so nobody should be surprised by them:** the classifier
default-denies an unmeasured head, so a wat verb calling any of the 403 `Unreviewed` intrinsics
derives as *not proven* — not as impure. Derivation would produce a large, honest, `Unreviewed`-
shaped residue, exactly as `@Totality` does today.

## ★ THE SHAPE — builder, 2026-08-30, and it answers question 2 outright

> *"we made all the defs take a metadata map, yes?.... we can declare all of these properties as
> actual wat data... not 'magic comments' ... the actual def exprs are consulted by some rust macro
> who lifts them and installs?... just like how we do the runtime meta via wat files"*

**DECLARE, as wat data, lifted at BUILD time.** Not derive, and not doc-comment prose. Every piece
already exists and is proven:

| piece | status | evidence |
|---|---|---|
| defs take a metadata map | ✅ shipped | `(defn :name {:restricted-to […]} [args] -> :Ret body)`; 4 live corpus uses (`wat/spawn.wat:338`, `wat/kernel/services/stdio.wat`) |
| a wat form lifted into Rust at build time | ✅ shipped | `wat_enum_from!` — `CARGO_MANIFEST_DIR` → `read_to_string` → generate |
| the anti-drift property | ✅ solved | it emits `const _: &str = include_str!(…)` so **rustc tracks the file**; edit the wat, the Rust rebuilds |
| the doctrine | ✅ ruled 2026-08-15 | *"wat is the source of truth; this crate generates the Rust… there is no second copy to go stale"* |

★ **So this is not a new idea — it is the completion of one already ruled.** `wat_enum_from!` made
wat the source of truth for the runtime *enums*. This makes it the source of truth for the verb
*properties*, through the same door, in the same direction.

★★ **And it kills the magic comments.** `/// @Purity Pure` is prose in a Rust doc comment that a
scraper parses. `{:purity :Pure}` in a metadata map is **data the reader already parses** — the same
reader, the same forms, no second grammar. Every "the directive parses but nothing consumes it" and
"the doc says X while the code does Y" defect this arc has found lives in the gap between those two.

### ⛔ ~~The one question the ruling does NOT settle~~ — THE QUESTION WAS MALFORMED

~~Does this reach the 431 Rust intrinsics too, or only the 409 wat-defined verbs?~~ **Struck
2026-08-30.** The builder: *"the rust ones are already configured .... the wat-doc does this for
us?"* — correct. There is no two-sided migration to weigh, because **`wat-doc` is already ONE shared
crate serving both**, and its own header says so:

> *"the prose+`@tag` parser + the mutual-checks live in ONE shared leaf crate, depended on by BOTH
> `wat-macros` … AND `wat` … An intrinsic's `///` block and a wat form's docstring parse through the
> same code … parity by construction, not by discipline."*

### ⚠ BUT THE WAT HALF OF THE PARSER HAS NO CALLER — measured

That header describes a design the wat side was never connected to:

```
the runtime side uses wat_doc for TYPES only:
  Purity 14 · Totality 11 · Determinism 11 · ExpandTime 7 · Category 4 · DocExample 1

wat_doc::parse callers — ALL of them:
  crates/wat-macros/src/wat_intrinsic.rs        the Rust attribute path
  crates/wat-macros/src/wat_special_form.rs     the Rust attribute path
```

**No wat `defn` docstring is parsed by anything.** "Parity by construction" is true of the *types*
and aspirational for the *parser* — another door whose paperwork is ahead of it.

★ **And this makes the builder's shape SMALLER, not larger.** The parser is already shared and
already shaped for both halves; the work is wiring the wat side to a crate built for it, and moving
the grammar from line-based `@tag` text to metadata-map DATA is then **one change that lands on both
halves at once** — because there is only one parser to change.

### What the smallest proving step looks like under this shape

One wat `defn` carrying `{:purity :Pure :determinism :Deterministic :total :Total :expand-time :Legal}`,
lifted by a `wat_enum_from!`-shaped sibling, appearing in `metadata-of` with **`:defined-in Wat`** —
which is precisely the value that makes the hard-coded constant above start discriminating.

## The remaining questions — the builder's, not mine

1. ~~**What does "registered" MEAN for a wat verb?**~~ ✅ **ANSWERED 2026-08-30 — and the answer is
   "it already is."**

   > **Builder:** *"how do we get user def in the registry?... is this a thing we can do when we load
   > a file?... or... do we do it now... the act of loading the code modifies the runtime?.. hrm......"*

   That hesitation was correct, and measuring it dissolves the question:

   ```rust
   pub(crate) fn registry() -> &'static IntrinsicRegistry {
       static REGISTRY: std::sync::OnceLock<IntrinsicRegistry> = …    // built ONCE from inventory
   ```

   **`registry()` is a `&'static OnceLock`** — `get_or_init`, built once at first touch, and
   **shared by every program**.

   ⛔ **CORRECTED — I first wrote that putting user defs there "runs into the ZERO-MUTEX doctrine".
   That was the wrong mechanism, and the builder was right to reject it:** *"source code is loaded
   single file... its frozen before any code runs... what contention could there be?"* **None.** Load
   is single-threaded and completes before execution; `OnceLock` is built for precisely that shape.

   The real constraint is **isolation + write-once**, and neither is a lock:

   - **`get_or_init` writes ONCE PER PROCESS.** A second program's load would not contend — it would
     be **silently ignored**. The first program's defs frozen in forever, everyone else's dropped.
     Worse than a race: a race is loud, this is a wrong answer with no symptom.
   - **The scope is the process; a program's defs belong to a program.** FM 7-ter's canonical fact,
     which the substrate already enforces one layer up for config: *"Threads share the parent's
     address space, RUNTIME, and fd 0/1/2."* `run-thread` is `deftest`'s default, so a process-global
     def store makes test A's defs visible to test B — across 5109 tests in one process.

   ★ **So two stores is the right shape, not a compromise.** `FrozenWorld` is per-program *because
   programs need isolation*; `registry()` is process-global *because intrinsics genuinely are* —
   compile-time facts, identical for every program. The query layer unifies them without merging the
   lifetimes. Put another way: a process-global def store would need every read to answer **"which
   program's defs?"**, and at process scope that question has no answer.

   ★ **But there are already TWO stores with two correct lifetimes, and the second one already holds
   every wat def:**

   ```
   registry()                            &'static, Rust intrinsics, compile-time, SHARED
   FrozenWorld.symbols + binding_metadata  per-program, built by LOADING, holds every wat def
   ```

   ★★ **And they are already unified at the QUERY layer.** `eval_metadata_of` consults both, in
   order — `registry().lookup_entry(&name)` first, then `sym.binding_metadata.get(&name)`. That is
   why it answers `Some` for **both** `sort$native` (an intrinsic) and `:wat::string::capitalize`
   (a wat `defn`). The unification happened; the two stores stayed separate, which is correct.

   So *"the act of loading modifies the runtime"* is **already true and always was** — loading builds
   a `FrozenWorld`. What it does not do, and must not, is mutate the shared static. The ordering also
   fixes a shadowing rule worth naming: **the registry answers first, so a user def cannot shadow a
   Rust intrinsic.**

   ⇒ **Nothing to build here, and nothing to defer.** What remains is making the query surface
   *honest* — the `:defined-in` constant is the named defect, and it is the one thing this seam says
   should not wait.
2. ~~**Does a wat verb DECLARE its axes or DERIVE them?**~~ ✅ **RULED — DECLARE, as wat data in the
   metadata map, lifted at build time.** See THE SHAPE above. Kept struck rather than deleted so the
   fork stays visible: derivation was live, and the classifier that made it possible is what made
   the question worth asking at all.
3. **What is the registry FOR, once it holds both?** Today it answers four axes and feeds the
   completeness gate. If it holds 409 more verbs, is it also the checker's scheme source? the
   doc surface? `@see`'s resolution domain? **Each answer pulls a different design.**

## What this seam would fix, concretely

- **`@see` could cross the boundary.** `sort$native` cannot cite `sort` today — the single most
  useful cross-reference it has — because `all_see_fqdns_resolve_to_registered_intrinsics` requires
  a registered intrinsic. That gate is right; its domain is half the language.
- **`defined_in`/`layer` become real** — the worklist's own stated unblocking condition: *"build
  these when a SECOND KIND can enter the registry, so `DefinedIn` has a `Wat` to discriminate."*
- **The completeness gate would see the whole substrate**, not the Rust half. Every "N verbs
  unhomed" number this arc has produced is a number about one half.

## Out of scope for this SEAM

It draws no stone and rules nothing. In particular it does **not** propose registering all 409 at
once: whatever the shape, the first move is one wat verb registering and `metadata-of` returning
`Some` with `:defined-in Wat` — the smallest thing that makes the constant above discriminate, and
therefore the smallest thing that proves the design.

## ⛔ The one thing that should NOT wait for the ruling

`metadata-of` publishes a constant as data. That is a defect today under any shape the fork takes,
and its fix does not depend on the ruling: either the value is derived, or it is not published.
Worth a stone on its own before the seam is drawn.
