# FINDING — can a user def change a stdlib verdict?

**Struck 2026-09-17**, branch `sns-sqs`, from HEAD `fd6688eec`. Spike only: **Tier B was not built and
nothing was fixed.**

---

## ⭑⭑ ROW 1 — ⭐ THE VERDICT, IN ONE WORD

# UNSOUND

`check(F)` for a stdlib `F` is **not** invariant under user definitions. A four-line user program
turns five green stdlib bodies red. The evidence is a **WITNESS**, not an argument — reproduced on
the pristine `fd6688eec` binary with no instrumentation compiled in, with the boot cache both ON and
OFF.

**Tier B as conceived is wrong.** Restricting the `ALL fns` sweeps to un-cached functions would have
made the reproduction below **compile clean** — five real type errors silently dropped.

---

## ⭑⭑ ROW 2 — THE REPRODUCTION

Two programs differing by **one top-level form**.

### BASELINE — `witness_base.wat`

```wat
;; BASELINE — byte-identical to the witness with the ONE user form removed.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "hi"))
```

### WITNESS — `witness_defclause.wat`

```wat
;; WITNESS — a user defclause on a name the STDLIB calls.
;; `:repl::` is not a reserved prefix, and `wat/repl.wat`'s own bodies call `:repl::turn`.
(:wat::core::defclause :repl::turn
  ([s <- :wat::core::String] -> :wat::core::String s))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "hi"))
```

### BOTH OUTCOMES, on the pristine `fd6688eec` release binary

```
$ WAT_BOOT_CACHE=off ./target/release/wat witness_base.wat
"hi"
exit=0

$ WAT_BOOT_CACHE=off ./target/release/wat witness_defclause.wat
exit=3
[#wat.kernel.LociDiedError/StartupError [#wat.check/CheckErrors {:message "5 type-check errors" …
  #wat.check/NoMatchingClauseAtCallSite {
    :message "no clause of `:repl::turn` matches arity 1 with types
              [(:wat::core::Vector :- [:wat::WatAST])]; clauses attempted: (1: [:wat::core::String])"
    :location #wat.core/Span {:file "wat/repl.wat" :line 91 :col 10 …}
    :name ":repl::turn" :called-arity 1
    :called-arg-types ["(:wat::core::Vector :- [:wat::WatAST])"]
    :attempted-clauses [{:arity 1 :param-types [":wat::core::String"]}]}
  … ×5 …
```

**All five errors are located in `wat/repl.wat` — lines 75, 91, 97, 104, 111 — inside the STDLIB
functions `:repl::eval-and-loop` (75) and `:repl::eval-form` (91, 97, 104, 111).** Not one is in the
user's file. These are precisely the bodies the `check:body-infer(ALL fns)` sweep walks and Tier B
would skip.

Cache-independent, measured both ways:

```
WAT_BOOT_CACHE=off      baseline exit=0 "hi"   ·  witness exit=3  5 errors  file "wat/repl.wat"
WAT_BOOT_CACHE=on       baseline exit=0 "hi"   ·  witness exit=3  5 errors  file "wat/repl.wat"
```

The same attack fires on all **three** reachable names, independently:

| user `defclause` on | outcome |
|---|---|
| `:repl::turn` | **5 errors**, all in `wat/repl.wat` |
| `:repl::eval-form` | **1 error** — `NoMatchingClauseAtCallSite`, arity 2, in `wat/repl.wat` |
| `:repl::eval-and-loop` | **1 error** — `NoMatchingClauseAtCallSite`, arity 2, in `wat/repl.wat` |

⛔ **Neither program is in `wat-scripts/scratch-pad/`, deliberately.** The
`every_wat_scripts_file_loads` gate type-checks every `.wat` under `wat-scripts/` on the current
runtime; a witness whose entire job is to FAIL type-check would turn that gate RED. Both are
reproduced verbatim above and live in the session scratchpad. Anyone can recreate them from this
file in ten seconds.

---

## ⭑⭑ ROW 6 — REACH vs CONSEQUENCE: this is CONSEQUENCE

Reach was already known from the DESIGN (`:817` writes before `:844` reads). What is shown here is a
**changed VERDICT**: `check(:repl::eval-form)` **passes** without the user form and **fails** with
it, with the diagnostic located in the stdlib file. Five stdlib call sites flip green→red on one
user line.

---

## ⭑⭑ ROW 3 — THE PATH, READ (and where the DESIGN's located mechanism was wrong)

⚠ **The mechanism the DESIGN located is NOT the hole.** `src/check.rs:817` (the
`collect_and_register_splice_defs` write into `env.defined_values`, pristine numbering: `:821`) is
**closed** in this corpus — see the door census below. The live hole is **earlier** and a different
map:

```
src/check.rs:800     for form in forms { preregister_defclause_in_env(form, &mut env); }   ← THE HOLE
src/check.rs:821     collect_and_register_splice_defs(...)      ← the DESIGN's suspect; closed here
src/check.rs:844     for (path, func) in sym.functions_iter() { check_function_body(...) } ← the read
```

The chain, each link named:

1. **`preregister_defclause_in_env`** (`src/check.rs:9277`) runs over the **user's** top-level forms
   at `:800` — *before* the sequential form loop and *well* before the body sweep at `:844`.
2. It calls `register_defclause_from_form(form, env, idempotent = true)` (`:9109`), which parses
   with `Privilege::User`. The reserved-prefix gate refuses `:wat::`, `:rust::`, `:$bound::` — and
   nothing else. **`:repl::` is not reserved.**
3. The idempotence guard is `if idempotent && env.get_defclause_clauses(&name).is_some()`. It
   consults **only `defclause_registrations`**. The stdlib's `:repl::turn` is a `defn`, so its name
   is absent from that map and **the user's registration lands**.
4. `CheckEnv::register_defclause` (`src/check/env.rs:423`) then inserts the clause table
   **unconditionally** — its own doc says so: *"The clause table is inserted unconditionally — a
   re-registration replaces the prior clause set."*
5. At `:844` the sweep infers `wat/repl.wat`'s bodies. `infer_list` reaches
   `if let Some(clauses) = env.get_defclause_clauses(canonical_k)` (`src/check.rs:6042`) for the
   call head `:repl::turn` — **and defclause dispatch takes precedence over the stdlib's own
   registered scheme.** The user's clause table is the only one tried.
6. No clause matches `(Vector :- [WatAST])` → `NoMatchingClauseAtCallSite`, five times, in
   `wat/repl.wat`.

### The door census — measured, not argued

A probe (`src/spike_probe.rs`, env-gated on `WAT_SPIKE_WITNESS`) records every **name** probed
against user-writable check state **while and only while** `check_function_body` runs for a function
whose body came from a stdlib file. Six doors instrumented: `schemes` (`CheckEnv::get`),
`defined_values` (`get_defined_value_type`, which also covers `corpus_values`), `defclause_regs`,
`unit_variant`, `registered_fns`, `extend_regs` (`get_extend_methods`), and `typeenv`
(`TypeEnv::get`).

Baseline run (`witness_base.wat`, `WAT_BOOT_CACHE=off`): **30,964** (body-file, fn, door, name)
records over stdlib bodies, **3,060 distinct names**.

| door | distinct names probed | of those, NOT under a reserved prefix |
|---|---:|---:|
| `schemes` | 2667 | 124 |
| `defclause_regs` | 1166 | **3** |
| `typeenv` | 462 | 27 |
| `unit_variant` | 330 | 119 |
| `defined_values` | 269 | 119 |
| `registered_fns` | 16 | 0 |
| `extend_regs` | **0** | 0 |

Prefix census of the 3,060 distinct names: **2,906** `:wat::` · **3** `:rust::` · **0** `:$bound::` ·
**146 unnamespaced** · **3** `:repl::` · 2 synthetic strings
(`(:wat::core::Seqable :- [:T])/seq`, `…[:Xt])/seq` — they carry parens and spaces and are not
expressible as a keyword, so no user can declare them).

⭐ **Of every name probed at every door, exactly SIX (door, name) pairs are names a user program
could legally declare** — namespaced, and outside the reserved prefixes:

```
defclause_regs  :repl::eval-and-loop
defclause_regs  :repl::eval-form
defclause_regs  :repl::turn
schemes         :repl::eval-and-loop
schemes         :repl::eval-form
schemes         :repl::turn
```

**Three names. The same three names Tier A's macro-expansion witness found.** Every other probed
name is either under a reserved prefix or **unnamespaced** (`:Ok`, `:Err`, `:Frame`, field keywords,
type variables `:T`/`:A`) — and an unnamespaced top-level name is refused by `UnnamespacedName`:

```
$ ./target/release/wat  -- (:wat::core::def :label "shadow")
#wat.check/UnnamespacedName {:message "top-level name ':label' is not namespaced — only fn
  arguments and let-bindings may be bare; give it a namespace, e.g. ':my::label'"}
```

Per-body attribution during the witness run, showing the exact edge:

```
:repl::eval-and-loop   defclause_regs   :repl::turn      ← THE EDGE
:repl::eval-form       defclause_regs   :repl::turn      ← THE EDGE
:repl::turn            defclause_regs   :repl::eval-and-loop
:repl::eval-and-loop   schemes          :repl::eval-form
…
```

**Two of the six pairs are user-writable maps.** `schemes` turns out to be closed by a different
mechanism (stdlib-wins, §Attack 1). `defclause_regs` is not closed at all. That is the whole hole.

⚠ **Bound on this reading.** The census is an over-approximation of *names* but only a
**one-program** sample of *control flow*: a door that a stdlib body reaches only when a lookup HITS
would not appear in a baseline where every lookup misses. Re-running the census with the
extend-type program present kept `extend_regs` at **0 probes**, which is the one case where that
mattered. The claim this census supports is therefore: *for the name-keyed doors, the user-reachable
surface is exactly those three `:repl::` names* — and that is enough, because the verdict is already
settled by the witness.

---

## ⭑ ROW 4 — THE FIVE ATTACKS, EACH TRIED

| # | attack | program | what it did |
|---|---|---|---|
| 1 | **Redefinition** | `(:wat::core::defn :repl::turn [x <- :wat::core::String] -> :wat::core::String x)` | **NOTHING — closed, by stdlib-wins.** Program stays green. The stdlib's entry survives in `schemes`; the user's `defn` is silently discarded. Proven, not assumed: calling `(:repl::turn "userversion")` from `:user::main` errors with `":repl::turn: parameter #1 expects (:wat::core::Vector :- [:wat::WatAST]); got :wat::core::String"` — the STDLIB signature is what the user's own call site is held to. ⚠ A user's `defn` vanishing without a diagnostic is an adjacent defect; **not touched** (trap-door 2). |
| 1b | Redefinition of a stdlib **value** const | `(:wat::core::def :wat::spawn::DEFAULT-MAX-MESSAGE-BYTES "clearly-not-an-i64")` — the `corpus_values`-shadow route (`defined_values` beats `corpus_values` in `get_defined_value_type`, and the redef check deliberately ignores `corpus_values`, so it would read as a FIRST binding) | **Refused by `ReservedPrefix`**: *"cannot define :wat::spawn::DEFAULT-MAX-MESSAGE-BYTES — reserved prefix (:wat::, :rust::, :$bound::)"*. The route is real; the gate closes it. Retried with a legal name — `(:wat::core::def :repl::turn 42)`, `(:wat::core::def :repl::eval-form "a string")`, `(:wat::core::def :user::bracket::work-fn 1)` — all green: no stdlib body reads a non-reserved name at the `defined_values` door (census above). |
| 2 | **Type-level shadowing** | `(:wat::core::defrecord :wat::WatAST …)` / `(… :T …)` / `(… :repl::WatAST …)` | **Closed, on both gates.** `:wat::WatAST` → `#wat.macro/ReservedPrefix`. `:T` → `#wat.macro/UnnamespacedName`. `:repl::WatAST` (legal, non-colliding) → green, no stdlib verdict moves. The census agrees: all 27 non-reserved names at the `typeenv` door are bare type variables (`:A A :Acc :D D E :G :I I :K K :Lu Lu :O O :R R :S S :Sh Sh :T T :U :V V :W`), every one of which `UnnamespacedName` forbids. |
| 3 | **`extend-type`** | user `defstruct` + `defsurface` + `extend-type`, well-formed and green | **NOTHING.** `extend_regs` is probed **zero** times during the stdlib body sweep — with *and* without a user `extend-type` in the program. No stdlib body consults `extend_registrations` in this corpus. |
| 4 | **Arity / among-many ambiguity** | `(:wat::core::defclause :repl::turn ([s <- :wat::core::String] -> :wat::core::String s))` | ⭐ **THE WITNESS.** 5 errors in `wat/repl.wat`. Variants: a clause that *does* match `(Vector :- [WatAST])` → green (verdict unchanged; ⚠ whether the stdlib's dispatch is thereby **re-pointed at user code at runtime** was not investigated — out of this stone's scope, and named in "what this does not settle"). Two overlapping matching clauses → also green (no ambiguity error raised). |
| 5 | **`defclause` / acronym / macro-adjacent** | the same defclause (attack 4) is a member of this class; plus `(:wat::core::declare-acronyms [:repl])` | `declare-acronyms` → green, nothing moves. `defclause` → **the witness**. The registration path is `preregister_defclause_in_env` at `src/check.rs:800`, and its write goes to `defclause_registrations`, **not** to `defined_values` — which is why the DESIGN's `:817` lead did not find it. |

---

## ⭑ ROW 5 — `redef_allowed`: DEFAULT, AND WHETHER THE ANSWER DEPENDS ON IT

**Default: `false`** — strict, every redef is an error. Three sites agree:

- `src/config.rs:97-101` — *"Arc 157 slice 1a-ii — compile-time redef opt-in. Default `false`
  (strict: every redef is an error)."*
- `src/config.rs:339` — `inherit.map(|c| c.redef_allowed).unwrap_or(false)`.
- `src/check/env.rs` `with_types` — `redef_allowed: false`; `from_symbols` then mirrors
  `sym.redef_allowed`. Opt-in is `(:wat::config::set-redef! true)`.

**The answer does NOT depend on it.** Measured both ways:

```
set-redef! true    exit=3   5 type-check errors   :file "wat/repl.wat"
set-redef! false   exit=3   5 type-check errors   :file "wat/repl.wat"
(flag absent, i.e. the default)  exit=3   5 type-check errors   :file "wat/repl.wat"
```

Mechanically expected: `redef_allowed` is read only at the `infer_def` redef-collision site. The
witness never goes through `infer_def` — `register_defclause` writes `defclause_registrations`
unconditionally and consults no flag. **The hole is open in the strict default configuration.**

---

## ⛔ ROW 7 — TIER B NOT BUILT, NOTHING FIXED

`git diff` against `fd6688eec` is **a probe and a finding, and nothing else**:

```
 M src/check.rs        ← probe: record the fn being inferred + its body file around the :844 sweep
 M src/check/env.rs    ← probe: one `probe(door, name)` line in each of 6 read accessors
 M src/lib.rs          ← probe: `pub mod spike_probe;`
 M src/types.rs        ← probe: one `probe("typeenv", name)` line in `TypeEnv::get`
?? src/spike_probe.rs  ← the probe (env-gated on WAT_SPIKE_WITNESS; a relaxed atomic load when off)
?? docs/excursus/…/FINDING.md
```

No check-sweep logic changed. No gate added. `.config/nextest.toml`, the boot cache, and
`register_defclause`'s unconditional insert are all **untouched** — the hole is reported, not
repaired. The probe is inert unless `WAT_SPIKE_WITNESS` is set; boot with it compiled in measures
**223 ms median of 9**, against Tier A's warm 220.86 ms.

---

## ⭑ ROW 8 — FLOOR, CONFIRMED NOT ASSUMED

`scripts/floor.sh`, release, with the probe in the tree:

```
    Summary [ 259.799s] 5291 tests run: 5291 passed, 22 skipped
[floor] exit=0. Log kept at .floor/2026-09-18T02-06-50Z/ regardless — a green run is evidence too.
```

**GREEN.** Read from the Summary line, not a piped exit code. Log kept at
`.floor/2026-09-18T02-06-50Z/`. Weighed **twice**, because the probe was touched between runs and
the tree that is graded must be the tree that was weighed: the earlier run was
`Summary [ 259.813s] 5291 tests run: 5291 passed, 22 skipped` at `.floor/2026-09-18T01-56-20Z/`. Note this is exactly what trap-door 3 predicted it would be worth:
**nothing**. The floor is green *because* no test in the corpus declares a `:repl::` name — which is
the same reason the hole survived to be found by construction rather than by regression.

---

## ⭑ ROW 9 — WHAT WOULD SETTLE AN UNKNOWN

Not applicable — the verdict is UNSOUND, settled by a witness. What is recorded here instead is
**what this stone does NOT settle**, each named precisely enough to run:

1. **Does the user's clause actually get CALLED at runtime, or only type-checked against?** The
   matching-clause variant (attack 4, variant) type-checks green. Whether `:repl::turn` inside
   `wat/repl.wat` then dispatches into user code is a behaviour question, and the experiment is:
   give the user clause the stdlib's exact signature and a side effect (`println`), then drive the
   REPL entry point and see whose body runs. **Not run here** — the DESIGN scoped this stone to
   verdicts.
2. **Is `:repl::` the whole surface, permanently?** No. It is the surface *today*, and the census
   above is how you re-measure it: `WAT_SPIKE_WITNESS=1 WAT_SPIKE_WITNESS_ALL=1 WAT_BOOT_CACHE=off`
   then filter `[spike-trace]` rows to namespaced, non-reserved names. Any future stdlib file under
   a non-`:wat::` namespace widens it silently. ⚠ **This finding expires the moment `wat/repl.wat`'s
   namespace changes** — and it would then be *narrower*, not wrong.
3. **Whether a Tier-B gate is buildable at all.** Not this stone's ruling. What the census hands the
   next one: the gate would have to key on the intersection of the user's declared `defclause` /
   `defn` / `def` names with the three-name probe witness — structurally the same shape as Tier A's
   `MacroRegistry::contains` gate, over `defclause_registrations` instead of the macro registry.
   **Whether that is worth 126 ms is a separate weighing.**

---

## ⭐ WHAT THIS COST TIER B

`check:body-infer(ALL fns)` is 111.10 ms of the 126 ms prize. Restricting it to un-cached functions
is **not** a safe elision: the reproduction above would have compiled clean, and five genuine type
errors in `wat/repl.wat` would have been dropped on the floor. The DESIGN's own prior was right —
⭐ *"expect this one to be false too, and be surprised if it is not."* It is false, and it is false
by **the same three names** that falsified the tidy argument in Tier A.

---

## ⭑ REGRADED BY THE ORCHESTRATOR ON HIS OWN RUNS, 2026-09-18 — the verdict stands: UNSOUND

```
floor    Summary [ 254.555s] 5291 tests run: 5291 passed, 22 skipped
         .floor/2026-09-18T02-14-48Z/ · exit=0 · NO ARM.txt   (Tier A's 2.2× intact)
clippy   0
scope    a probe + this FINDING; `register_defclause`'s unconditional insert UNTOUCHED
```

### ⛔ THE WITNESS REPRODUCES — on my own runs, pristine binary

```
base.wat      (:wat::core::defn :user::main …)                         → "hi"   exit 0
witness.wat   + (:wat::core::defclause :repl::turn ([s <- String] -> String s))
              → exit 3 · 5 × NoMatchingClauseAtCallSite · ALL in wat/repl.wat
                at lines 75 · 91 · 97 · 104 · 111
```

**One extra top-level form changes the verdict of five call sites inside stdlib bodies.** Tier B would
skip exactly those bodies as cached, so the program would compile **clean** and **five genuine type
errors would be silently dropped.** ⛔ That is not a performance regression — it is a compiler accepting
broken code. **Tier B, as conceived, is dead, and it died before anyone built it.**

⚠ **The counterfactual is reasoning, not measurement**, and is labelled as such: the errors are located
in stdlib bodies and Tier B's whole mechanism is to skip stdlib bodies. Nobody ran a Tier B that dropped
them, because nobody built one.

Independence from `redef_allowed` re-verified: **true / false / unset all identical.** `register_defclause`
consults no flag.

### ⭐ MY DESIGN POINTED AT THE WRONG HOLE, AND ADVERSARIAL CONSTRUCTION FOUND THE REAL ONE

I located `check.rs:817`/`:821` — user `def`s mutating `env.defined_values` before body-infer. **That
route is closed in this corpus.** The live hole is earlier and in a *different map*:

```
check.rs:800   for form in forms { preregister_defclause_in_env(form, &mut env) }   ← over USER forms
               its idempotence guard consults only `defclause_registrations`, so a stdlib name
               declared as a `defn` is ABSENT and the user registration lands
               `register_defclause` then inserts UNCONDITIONALLY
check.rs:6042  defclause dispatch TAKES PRECEDENCE over the stdlib's own scheme
```

★ A brief that names a plausible mechanism is worth having; it is **not** a substitute for trying to
build the counter-example. The DESIGN's instruction — *adversarial first* — is what found this, and the
mechanism I confidently wrote down would have sent a less adversarial executor looking in the wrong file.

### ⭐⭐ THE SAME THREE NAMES, A THIRD TIME — and now they are the whole attack surface, measured

The spike instrumented the sweep: **30,964 name-probes across seven doors** during stdlib body
inference. Of **3,060 distinct names**, exactly **three** are names a user could legally declare:

```
:repl::turn  ·  :repl::eval-form  ·  :repl::eval-and-loop
```

The identical three that falsified Tier A's tidy argument (*"the stdlib only calls `:wat::` heads"*).
Everything else is reserved-prefix or unnamespaced and therefore unreachable. ⭑ **That is not a
coincidence — it is the attack surface, and it is now a measured set of three rather than an assumption.**

### The five attacks, as reported and accepted

1. **redefinition via `defn`** — closed by stdlib-wins, proven by the *user's own* call site being held
   to the stdlib signature; (1b) redef of a stdlib value const — the `corpus_values` shadow route is
   real but refused by `ReservedPrefix`.
2. **type shadowing** — closed twice over (`ReservedPrefix` and `UnnamespacedName`).
3. **`extend-type`** — nothing: `extend_regs` is probed **zero** times during the stdlib sweep, with and
   without a user `extend-type`.
4. **arity / among-many** — ⭐ **the witness.**
5. **`defclause` / acronym** — `declare-acronyms` inert; `defclause` is the witness.

### It refused the tempting fix, correctly

`register_defclause`'s unconditional insert is **untouched**. A witness is a finding; changing dispatch
precedence is a separate ruling with its own blast radius — ⚠ **and it is arguably a real bug
independent of any cache**, since a user `defclause` silently re-pointing a stdlib call site is a
soundness question on its own.

### What is left of Tier B's 126 ms

Not these terms. A narrower question remains, and it now has a number attached: **can the four sweeps be
restricted to the functions reachable from a user-declarable name — a set measured at THREE?** That is a
different stone with a different soundness argument, and the FINDING's own unsettled item feeds it:
whether a user clause that *matches* the stdlib signature re-points `wat/repl.wat`'s dispatch into user
code **at runtime** while type-checking green was scoped out and never investigated.
