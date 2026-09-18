# SCORE — Tier B: the check skips what it already proved

**Struck 2026-09-18**, branch `sns-sqs`, from HEAD `fee90ec9d`. Nothing committed; the tree is left
dirty for the orchestrator.

---

## ⭑⭑ ROW 1 — ⭐ PHASE 1, IN ONE WORD

# REFUTED

> For every stdlib function `F`, the verdict of `check(F)` is determined solely by state fixed at
> bake time — no user-supplied state can reach it.

**False, and the counter-example is one line long.** `{:restricted-to [:my::]}` on the user's own
`:user::main` turns **nine** green bodies inside **six** stdlib files red. A second, independent
one on `:user::spawn::service-locus` turns **ten** red, across ten stdlib functions.

⛔ **Phase 2 was NOT built.** `src/` is untouched — `git status` is three new test files and this
document. The elision machinery was written, measured, and then **reverted** when phase 1 came back
refuted: the DESIGN's own instruction is *"if any door can consult user-supplied state, THAT IS THE
FINDING — report it and stop"*, and a refutation was declared worth more than a shaky elision.

### ⭐⭐ THE ARGUMENT — THE DOOR LIST WAS SHORT BY ONE, AND THE MISSING DOOR IS THE HOLE

The DESIGN says *"prove or refute it structurally, by reading the seven doors `src/spike_probe.rs`
instruments."* Those seven doors are **all read by one sweep** — `check:body-infer`. The spike's
window is literally `begin_body_sweep` / `end_body_sweep` around `P_CHECK_BODIES` (`check.rs:868`),
so the census is a census of ONE of the four sweeps. The other three read their own state, and one
of them reads a user-writable map:

```
sweep                         reads                                          instrumented?
8b  check:retired-syntax      the body AST. Nothing else.                    n/a (nothing to probe)
8d  check:def-position        the body AST. Nothing else.                    n/a
8c  check:restricted-call     the body AST  +  CheckEnv::binding_metadata    ⛔ NEVER — the 8th door
8f  check:body-infer          the body AST  +  the seven doors               yes (7/7)
```

**`CheckEnv::get_binding_metadata` is the eighth door** (`check.rs:1679`, the only live read). Its
map is `SymbolTable::binding_metadata`, keyed by the binding's own FQDN, and **a user program writes
it with an ordinary metadata-map** — `(:wat::core::defn :name {:restricted-to [:my::]} …)`. There is
no reserved-prefix problem to solve: the user writes under **their own** name and the stdlib reads
**that same name**.

It reads it because two user-declarable names appear inside stdlib bodies, and because the walker
was deliberately widened to fire on mention rather than head position:

1. ⭐ **`walk_for_restricted_call` fires on EVERY `WatAST::Keyword` leaf it walks.** Its own doc
   (arc 198, `DESIGN-STONE-a-restriction-governs-mention-not-head-position`) says so: *"To call a
   thing you must first name it, so this walker fires on every `WatAST::Keyword` node the walk
   encounters, not only ones sitting in call-head position."* It does **not** know that a leaf inside
   a quoted `(:wat::core::forms …)` template is data for a CHILD program rather than a call.
2. ⭐ **Nine stdlib `…::service-forms` bodies quote `:user::main`**, and those nine plus
   `:wat::spawn::ProcessOpts/launch` quote `:user::spawn::service-locus` — both inside
   `(:wat::core::forms …)` child-program templates (`wat/spawn.wat:613` is the visible one).
3. **Both names are namespaced and outside every reserved prefix**, so `resolve::gate` lets a user
   declare them. `:user::main` is not an exotic name — **it is the name every wat program declares.**

That is the whole chain, and every link of it was read in the source before it was driven.

### The surface, measured exhaustively rather than sampled — and it is TWO

A walk over the frozen bare world (`startup_bare()`), covering every function's own path, every type
its signature mentions, and **every `Keyword`/`Symbol` leaf in its body**, asking of each name: is it
namespaced AND outside `:wat::` / `:rust::` / `:$bound::` — i.e. a name `resolve::gate` would let a
user program insert?

```
functions walked                    every fn in the frozen bare world (gate asserts >1000, non-vacuity)
user-declarable names reachable from a body          2
    :user::main                    reached from 9 fns  (…::service-forms ×9)
    :user::spawn::service-locus    reached from 10 fns (…::service-forms ×9 + :wat::spawn::ProcessOpts/launch)
```

This is a closure over the whole key space, not a sample of one program's control flow — which is
exactly what the DESIGN asked for and what the spike's census could not give. It is gated:
`probe_tier_b_stdlib_verdict_is_not_bake_fixed::the_user_declarable_surface_inside_stdlib_bodies_is_exactly_two`.

### ⭑ AND THE OTHER HALF: THE SEVEN DOORS *ARE* CLOSED, AND THE SPIKE'S BOUND **IS** LIFTABLE

Worth recording because it is what any narrowed Tier B would rest on, and because the DESIGN doubted
it could be done at all.

`wat-scripts/fixes/vend-only-wat-repl-names.wat` took `:repl::turn` / `:repl::eval-form` /
`:repl::eval-and-loop` to `:wat::repl::*`, so re-running the spike census today gives:

```
$ WAT_SPIKE_WITNESS=1 WAT_BOOT_CACHE=off ./target/release/wat hello.wat
[spike-witness] 4911 (door, name) pairs across 6 doors
  door defclause_regs   probed=1166  user-reachable=0
  door defined_values   probed=269   user-reachable=119   ← all UNNAMESPACED (:Ok :Err :Frame …)
  door registered_fns   probed=16    user-reachable=0
  door schemes          probed=2668  user-reachable=122   ← 119 unnamespaced + 2 synthetic + :user::main
  door typeenv          probed=462   user-reachable=27    ← all bare type variables (:T :K :V …)
  door unit_variant     probed=330   user-reachable=119   ← all unnamespaced
  door extend_regs      probed=0     user-reachable=0
```

Filtering the full `[spike-trace]` by **body file**, the only namespaced non-reserved name probed
from any body is `:user::main`, **from the user's own `hello.wat` body** — never from a stdlib file.
Re-run with a user `defstruct` + `extend-type` present: `extend_regs` still **0**, and the only new
rows are `:my::Point'` / `:my::Point/x` / `:my::is-Point?` attributed to `src/runtime.rs` — the
user's OWN synthesised companions, not stdlib bodies.

⭐ **The empirical bound lifts by induction, and this is the part the spike said a census could not
do.** The probe records every lookup, **hit or miss**. Let `W` be the set of names probed while
checking stdlib bodies on the recorded boot. Two sweeps under two different user programs can only
diverge at a probe whose *result* differs; a result differs only where a user wrote; a user write
lands only under a namespaced non-reserved name (`resolve::gate`: `Unnamespaced` → refused,
`Reserved` under `Privilege::User` → refused, `Equivalent` → `NoOp`, `Divergent` → `Duplicate`).
`W` contains no such name, so there is no first divergence, so `W` and the verdicts are the same for
**every** user program. The base case is a measurement; the step is the gate.

⛔ **And that induction is exactly why the refutation matters**: it is valid, it is what a narrowed
Tier B would cite — and it says nothing whatever about the eighth door, because the eighth door is
in a sweep the probe never watched. **A proof whose premise is a door census is only as good as the
door list.**

---

## ⭑⭑ ROW 2 — THE SIX ROUTES, EACH CLOSED OR NOT, EACH DRIVEN

| # | route | verdict | driven |
|---|---|---|---|
| 1 | user `defn` / `def` on a stdlib name | **CLOSED by construction** | `(:wat::core::defn :wat::repl::turn …)` → `#wat.check/ReservedPrefix`. `(:wat::core::def :wat::spawn::DEFAULT-MAX-MESSAGE-BYTES "…")` → `#wat.check/ReservedPrefix`. |
| 2 | user `defclause`, incl. **eval-time / REPL** | **CLOSED by construction, at BOTH doors** | see row 7 |
| 3 | user macros | **CLOSED twice over** | `(:wat::core::defmacro :wat::core::when …)` → `#wat.macro/ReservedPrefix`. And Tier A refuses the cache outright for any program whose macro name is in the expansion witness, so a user macro that *could* have been consulted disarms the snapshot before it is read. |
| 4 | user types / `extend-type` | **CLOSED** | `(:wat::core::defrecord :wat::mine::Thing …)` (a FRESH reserved name) → `#wat.macro/ReservedPrefix`; `(… :wat::core::Span …)` → `#wat.type/DuplicateType`; `(… :T …)` → `#wat.macro/UnnamespacedName`. A legal user `defrecord` + `extend-type` → green, and `extend_regs` is probed **0** times with it present. |
| 5 | acronyms / `defclause` stubs / generated companions | **CLOSED** | `declare-acronyms` disarms the Tier A snapshot outright, so the elision could never apply to such a program. Generated companions derive from type names: a stdlib type is `:wat::…`, so `:wat::T/field` and `:wat::T'` are reserved too — and the `a-defclause-outranks-a-defn` wall now refuses a clause on either. |
| 6 | `installed_dep_sources()` | **CLOSED, but read the bound** | Dep sources are `&'static [WatSource]` installed by the **host Rust binary** (`src/load/source.rs:59`), never by a `.wat` program, and `stdlib_forms()` concatenates them into the STDLIB corpus (`src/load/stdlib.rs:565`) — so their names are stdlib state, not user state. `cache_key()` hashes every dep path and byte (`boot_cache.rs:167`), so a different dep set is a different key and a fresh derive. ⚠ **The bound:** a dep that vends a non-`:wat::` name would widen the surface above, and `load::stdlib::tests::stdlib_vends_only_wat_prefixed_names` runs with **no** deps installed, so it cannot see one. Any Tier B must compute its closure at RUNTIME, over the world it is about to store — not in a test. |
| ⛔ | **the seventh route nobody listed — `binding_metadata`** | **OPEN. This is the finding.** | row 1 and row 4 |

⚠ **Route 1's mechanism is now different from what the spike reported, and better.** The spike found
`defn` redefinition "closed by stdlib-wins" — a silent discard. Today it is closed by
`ReservedPrefix`, **because every stdlib name is `:wat::`-prefixed** (`stdlib_vends_only_wat_prefixed_names`
gates it from the frozen symbol table). Closed *by construction*, with a located diagnostic, which is
what the DESIGN asked for.

---

## ⭑⭑ ROW 4 — ⭐ THE CONTROL THAT MATTERS, DRIVEN — AND THE REFUTATION IS ITS TWIN

Two programs, differing by one metadata-map. Both reproduced verbatim as co-located fixtures.

### BASELINE — `…_base.wat`
```wat
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "hi"))
```

### WITNESS — `…_neg_restricted.wat.bad`
```wat
(:wat::core::defn :user::main {:restricted-to [:my::]} [] -> :wat::core::nil
  (:wat::kernel::println "hi"))
```

```
baseline   →  "hi"                                     exit 0
witness    →  9 type-check errors, exit 3
              #wat.check/DefRestrictedCallerNotAllowed  ×9
              :enclosing-fn  :wat::telemetry::span::service-forms
                             :wat::kernel::stderr-svc::service-forms
                             :wat::telemetry::journal::service-forms
                             :wat::query::mem-store::service-forms   … ×9
              :file  wat/cache.wat ×2 · wat/kernel/services/stdio.wat ×3 ·
                     wat/query/mem.wat · wat/query/sqlite-store.wat ·
                     wat/telemetry/journal.wat · wat/telemetry/span.wat
```

**Not one error is in the user's file.** The message tells the author to *"move the caller into one
of the allowed namespaces"* — the caller being `:wat::telemetry::span::service-forms`, a stdlib
function the author has never seen and cannot edit.

### The second name, independently
```wat
(:wat::core::defn :user::spawn::service-locus {:restricted-to [:my::]} [] -> :wat::core::i64 1)
```
→ **10 errors**, the nine `…::service-forms` plus `:wat::spawn::ProcessOpts/launch`.

⚠ **Measured asymmetry, not repaired:** the same restriction via a scalar
`(:wat::core::def :user::spawn::service-locus {:restricted-to [:my::]} 1)` is **green** — a
metadata-map on a non-fn-shaped `def` does not reach `binding_metadata` by the time the sweep runs.
Named, not investigated.

### ⭐ The user-type-error control, driven across every cache state

The row-4 control proper — *"a user program with a type error must still fail, with the cache warm"* —
plus the stdlib-verdict witness, across all four payload states.

```wat
;; u_typeerror.wat
(:wat::core::defn :user::add [a <- :wat::core::i64] -> :wat::core::i64 a)
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:user::add "not-an-i64")))
```

| payload state | `hello.wat` | user type error | the stdlib-verdict witness |
|---|---|---|---|
| WARM (present, valid) | `"hi"` exit 0 | **1 error · TypeMismatch** | **9 errors · DefRestrictedCallerNotAllowed** |
| ABSENT (deleted) | derives | **1 error · TypeMismatch** | **9 errors** |
| TRUNCATED (½ of 2,853,604 B) | derives | **1 error · TypeMismatch** | **9 errors** |
| CORRUPT (one byte flipped at ½) | derives | **1 error · TypeMismatch** | **9 errors** |
| STALE (key byte flipped — the payload claims another build) | `"hi"` exit 0, derives | **1 error · TypeMismatch** | **9 errors** |
| `WAT_BOOT_CACHE=off` | `"hi"` exit 0 | **1 error · TypeMismatch** | **9 errors** |

⚠ **This control is currently trivial, and saying so is the point.** Nothing elides, so of course
nothing is dropped. What the table is actually for is the NEXT stone: it is the negative control a
Tier B has to survive, and the witness row is pinned as a test so that a Tier B which silences it
goes **red** instead of going quiet.

---

## ⭑⭑ ROW 3 — THE SWEEPS ARE PURE. RE-CONFIRMED, NOT INHERITED.

```
$ grep -cE 'RefCell|Mutex|RwLock|Cell<|unsafe|static mut|AtomicU|AtomicBool' src/check.rs
0
$ grep -n 'pub fn check_program' -A 4 src/check.rs
648:pub fn check_program(forms: &[WatAST], sym: &SymbolTable, types: &TypeEnv) -> Result<(), CheckErrors>
```

`sym` and `types` are immutable borrows; the only `&mut` are `env` / `fresh` / `errors`, all local to
the call. Each of the four sweeps writes exactly one thing: `errors`.

⚠ **Two honest exceptions, both instrumentation and neither verdict-bearing.** (1)
`crate::freeze::census::phase` mutates process-global counters — that is the boot census itself, the
instrument every number in this document comes from. (2) `crate::spike_probe::probe` takes a `Mutex`,
but only when `WAT_SPIKE_WITNESS` is set; unset it is one relaxed atomic load. Purity here means *no
verdict-bearing side effect*, and that is what was checked.

★ Purity is necessary and turned out not to be sufficient: the sweeps are pure functions of
`(body, env)`, and the refutation is entirely about **what is in `env`**.

---

## ⭑ ROW 7 — EVAL-TIME / REPL `defclause` ON A STDLIB NAME: **REFUSED.** DRIVEN, NOT GUESSED.

`38b43a6ac` disclosed that a `defclause` registered at eval time does not pass through
`check_program`, so its wall (`ClauseOverExistingDeclaration`) does not cover it. **The reservation
does, and it covers it upstream of the phase split.**

```
$ printf '(:wat::core::defclause :wat::repl::turn ([s <- :wat::core::String] -> :wat::core::String s))\n' \
    | ./target/release/wat --repl
#wat.core/Fault {:message "cannot define :wat::repl::turn — reserved prefix (:wat::, :rust::, :$bound::);
  user defines must use their own prefix"
  :causes [#wat.runtime/ReservedPrefix {… :prefix ":wat::repl::turn" :file "<read-string>"}]}
```

**The mechanism, read:** `register_defclause`'s own reserved guard is on the `Stub` arm only
(`runtime.rs:1056`) — the `Runtime` arm has none, which is what made this worth driving. The refusal
comes one level up, from **`parse_defclause_form`** (`runtime.rs:7869`), which calls
`crate::resolve::register(&name, privilege, Existing::Absent, …)` and is called by **both** doors —
the check-time pre-pass (`Privilege::User`) and the eval-time `dispatch_keyword_head` arm
(`runtime.rs:3023`, `Privilege::User`). One gate, both phases.

**Two more facts from the same drive, neither of them assumed:**

- Non-vacuity: `(:wat::core::defclause :my::f ([s <- String] -> String s))` then `(:my::f "ok")` at the
  REPL → `"ok"`. The eval-time door does register.
- ⭐ **`wat --repl` DOES reach `check_program`** — a `defclause` over a *user's own* `defn` at the
  REPL fires `ClauseOverExistingDeclaration` with `:file "<read-string>"`. `wat/repl.wat`'s
  `eval-and-loop` accumulates definitions and re-checks the whole accumulated form set each turn, so
  the `--repl` path is *not* the uncovered one. ⚠ The gap `38b43a6ac` named is therefore **narrower
  than it reads**: it is about eval-time registration paths that are not the REPL loop
  (`eval-ast!`, a host embedding), and this stone did not enumerate those.

---

## ⭑⭑ ROW 5 — BOOT NUMBERS: BEFORE AND AFTER ARE THE SAME NUMBER

Phase 2 was not built, so there is no "after". The "before" is measured on my own runs so the prize
is on the record at today's HEAD, and so the next stone does not inherit Tier A's figures stale.

`WAT_BOOT_CENSUS=phases`, one-line `hello.wat`, **median of 9 separate processes** each.

```
                          accounted   since-boot    wall (median of 9)
WARM   (cache hit)          185.71      190.17        0.224 s
COLD   (WAT_BOOT_CACHE=off) 405.46         —          0.451 s
FIRST  (derive + store)     434.68      469.70          —        (store 34.58)

the four sweeps, warm:
  8b check:retired-syntax(ALL fns)     11.13
  8c check:restricted-call(ALL fns)     3.07   ← ⛔ the refuting sweep
  8d check:def-position(ALL fns)        1.72
  8f check:body-infer(ALL fns)        108.73
  ───────────────────────────────────────────
                                      124.65 ms  =  67.1 % of accounted, EVERY BOOT
  3a-6b boot-cache-load                37.38   ← Tier A's replacement, for scale
```

⚠ **Target shape not met, and it was never attempted: 185.7 → 185.7.** The DESIGN's ~65 ms /
~0.09 s stands unclaimed. ⭐ **124.65 ms is still on the table** — and 108.73 of it is
`check:body-infer`, whose seven doors row 1 shows ARE closed. What is not closed is the 3.07 ms
sweep. See row 9.

---

## ⭑⭑ ROW 6 — STALE / ABSENT / CORRUPT ⇒ DERIVE **AND** CHECK. ALL THREE DRIVEN.

The table in row 4. All four damage modes (absent · truncated to ½ · one byte flipped at ½ · key byte
flipped so the payload claims another build) produce a correct boot, and in every one of them the
user's type error is still caught and the stdlib-verdict witness still fires. Tier A's standing test
`boot_cache_fixpoint::absent_truncated_and_corrupt_payloads_all_fall_back_to_deriving` is untouched
and still weighs the decoder-level half.

⚠ **"AND check" is currently free**, for the reason row 4 gives: nothing elides. The drive is
recorded so the next stone has the baseline rather than the claim.

---

## ⭑⭑ ROW 8 — FLOOR

```
     Summary [ 258.472s] 5302 tests run: 5302 passed, 22 skipped
[floor] exit=0. Log kept at .floor/2026-09-18T22-39-35Z/ regardless — a green run is evidence too.
```

**GREEN.** Read from the Summary line, never a piped exit code. Log at
`.floor/2026-09-18T22-39-35Z/` (`raw.log` + `clean.log`, byte-identical here — no ANSI to strip).
**NO `ARM.txt`.** `grep -cE '^ +(FAIL|TIMEOUT|SIGSEGV|ABORT)' clean.log` → **0**.

**5302, up from the prior stone's 5299 — the +3 is this stone's three tests**, all present in the
log by name under `wat::function probe_tier_b_stdlib_verdict_is_not_bake_fixed::*`. Tier A's band
holds: 258.472 s against the prior runs' 255.830 / 258.434 / 259.799 / 254.555 s.

⚠ **And a green floor proves almost nothing here, exactly as trap-door 5 predicted.** `src/` is not
in the diff, so the whole corpus was always going to be green. What the green *does* prove is that
the three new tests hold on the tree that is being handed over — including the one that asserts a
user program still reddens nine stdlib bodies.

---

## ⭑ ROW 9 — clippy + `--no-run`, READ FROM THE OUTPUT

```
$ cargo clippy --release --workspace --all-targets     exit 0
    grep -c '^error'   on the captured log   →  0
    grep -c '^warning' on the captured log   →  0
$ cargo nextest run --release --no-run                 exit 0   (0 error, 0 warning)
$ cargo build --release                                exit 0
```

Counted from `tee`'d logs, not inferred from a tail. The lint group was also weighed on its own
before the floor: `Summary [ 139.595s] 111 tests run: 111 passed, 0 skipped`.

---

## ⭑ ROW 10 — CIRCUIT, BYTE-IDENTICAL

One line, from `~/work/BREADCRUMB.md` § FRESHNESS PROBE:

```
"timeout=yes;discarded=yes;redial=Connected;retry-on=fresh"
"n=2000;m=4;j=3;total=8000;distinct=8000;dup=0;workers=12;empty=1;seen-recorded=8000;
 seen-skipped=0;check-exhausted=0;mark-exhausted=0;ack-retries=0;ack-exhausted=0"
pub-exh-last=none   ·   bp-delay=0   ·   total=13850   ·   rc=0
```

Both pinned lines unchanged.

---

## ⛔ ROW 11 — OPEN ITEMS UNTOUCHED

```
$ sha256sum .config/nextest.toml
706f59851bc01cca13ad37f2c0d07abe68a7133d362418ffeb77da21d22a5372  .config/nextest.toml
$ git show HEAD:.config/nextest.toml | sha256sum
706f59851bc01cca13ad37f2c0d07abe68a7133d362418ffeb77da21d22a5372  -
```

Identical. `check.rs:6068`'s dispatch precedence · `check/env.rs:456`'s unconditional insert ·
`runtime.rs:2112`'s `Existing::Equivalent` · the boot cache · `src/spike_probe.rs`: **all untouched**.
`git status --porcelain`:

```
?? tests/function/probe_tier_b_stdlib_verdict_is_not_bake_fixed.rs
?? tests/function/probe_tier_b_stdlib_verdict_is_not_bake_fixed_base.wat
?? tests/function/probe_tier_b_stdlib_verdict_is_not_bake_fixed_neg_restricted.wat.bad
```

**`src/` is not in the diff.** No `.wat` corpus file was edited, by hand or otherwise; the two new
`.wat` are new test fixtures, and the failing one carries `.wat.bad` so no loader gate walks it.

### What was BUILT and then REVERTED, named so nobody re-walks it

Phase 2 was implemented end-to-end before phase 1 came back refuted, and reverted with
`git checkout`. For the next stone, the shape that worked:

- `SymbolTable::{arm_bake_checked, was_checked_at_bake}` — the elision set captured at
  `freeze/env.rs:304`, the exact line where `symbols` IS the snapshot's world and holds nothing else.
  Keyed by path but compared by **`Arc::ptr_eq`**, so a later pass that re-registers a path is checked
  in full rather than trusted by name.
- `StdlibSnapshot::bodies_check_closed` + `FORMAT_VERSION` 1 → 2, the permission computed at store
  time over the world about to be stored (which is what covers installed deps; see route 6).
- `if sym.was_checked_at_bake(path, func) { continue; }` in each of the four sweeps — the whole
  elision is four branches.
- `boot_cache::is_representable` refuses a snapshot whose symbols are armed, so an elided world can
  never become the source of a snapshot.

It compiled, it was weighed, and its own store-time closure **refused to grant the permission** —
which is how the two names above were found. ⭐ **The gate I built to protect the elision is what
killed it.**

---

## ⭑ ROW 12 — WHAT GOT SLOWER OR RISKIER. THE BILL.

| cost | measured |
|---|---|
| boot | **nothing.** 185.71 ms warm, unchanged; `src/` is not in the diff. |
| floor | **+3 tests** (5299 → 5302). Their combined cost is 0.68 s of a ~256 s run — two of the three freeze a bare world, which every neighbour already does. |
| ⚠ a pinned number | `a_user_restriction_on_user_main_reddens_stdlib_bodies` asserts **exactly 9**. If someone adds a tenth `defservice` to the stdlib it goes red for an innocent reason. That is deliberate — the count IS the surface — and the failure message says to re-read the surface gate before touching the number. |
| ⚠ a pinned list | `the_user_declarable_surface_inside_stdlib_bodies_is_exactly_two` asserts the two names EXACTLY, so **progress reddens it too** (removing one is a red). Also deliberate: this list moving is a decision, not a detail. |
| risk taken | none in `src/`. The risk this stone actually retired is the one it refused to take. |

---

## ⭐ WHAT THIS LEAVES FOR THE NEXT STONE — AND IT IS NOT NOTHING

⛔ **Do not read this as "Tier B is dead."** It is `check:restricted-call` that is open. Read the
numbers again:

```
8c check:restricted-call     3.07 ms   ← the sweep that refutes the claim
8b + 8d + 8f               121.58 ms   ← the three whose state row 1 shows is closed
```

⭐ **A Tier B that elides `8b` / `8d` / `8f` and leaves `8c` running in full would keep 121.58 of the
124.65 ms and be immune to the witness above.** `8b` and `8d` read nothing but the body AST — for
them "the same bytes, the same verdict" is the entire argument. `8f`'s seven doors are closed, by the
census plus the induction in row 1.

⛔ **I did not do it, and the reason is the finding itself.** The eighth door was found by reading
`check.rs`, not by the instrument that was supposed to find it — so the honest next step is an
**exhaustive enumeration of what each sweep reads**, gated, before anything is elided. The grep that
starts it (all 16 distinct `env.*` reads in `check.rs`) is:

```
361 env.register(     158 env.types(        21 env.get(       4 env.get_defined_value_type(
  3 env.unit_variant_type(   3 env.get_defclause_clauses(     2 env.get_binding_metadata(  ← the 8th
                                                              (1 of those 2 is a doc comment; the
                                                               live read is check.rs:1679)
  1 env.has_registered_function(   1 env.get_defined_value_span(   1 env.get_defined_value_ast(
  1 env.iter(   …
```

and the two sites the seven-door census never covered are `env.get_binding_metadata` (`:1679`,
sweep 8c) and `env.iter()` (`:15647`, `validate_aggregate_containment` — **not** one of the four
sweeps, and the only UNKEYED iteration anywhere near them; `check_impls_completeness`'s
`types.iter_subtype_edges()` at `:894` is the other, also outside the four).

---

## ⭐ WHAT SURPRISED ME

1. ⭐⭐ **The hole is not in the expensive sweep. It is in the 3 ms one.** Every line of the DESIGN,
   the spike and the prize table points at `check:body-infer` — 108.73 of the 124.65 ms — and its
   doors turn out to be closed. `check:restricted-call` costs 2.5 % of the prize, was instrumented by
   nobody, and is the whole refutation.
2. ⭐⭐ **`:user::main` is the attack surface.** Every previous stone in this arc hunted for an exotic
   name a user might collide with — `:repl::turn`, a squatted `:T/field`. The name that actually
   reaches into stdlib bodies is the one **every single wat program declares**. It is not a collision
   at all: the user writes their own name and a stdlib body reads it, because the stdlib quotes it.
3. ⭐ **A walker that was deliberately widened is what opened the door.** Arc 198 widened
   `walk_for_restricted_call` from head-position to every mention — correctly, for aliasing. Nobody
   asked what "mention" means inside a quoted child-program template, and a `defservice`'s
   `service-forms` body is nothing but quoted child-program template.
4. ⭐ **The gate I wrote to protect the elision is what killed it.** I expected the store-time closure
   to come back empty and grant the permission. It came back with two names, and chasing them to
   their reader is the whole stone. A gate that only ever says yes proves nothing.
5. **The DESIGN's namespace ruling has actually held everywhere it was tested.** Four routes, eight
   drives, all refused by `ReservedPrefix` / `UnnamespacedName` / `DuplicateType` with located
   diagnostics. The ruling is sound; the claim built on it was under-specified — "every name a
   stdlib body can RESOLVE" and "every name a stdlib body CONTAINS" are different sets, and
   `walk_for_restricted_call` reads the second one.

---

## ⭑ WHAT I COULD NOT SETTLE

1. ⛔ **Is `DefRestrictedCallerNotAllowed` firing inside a quoted `(:wat::core::forms …)` a BUG?**
   I believe so — a restriction is about who may *call*, and a stdlib body that names `:user::main`
   in a template for a child program is not calling it. But repairing it means teaching
   `walk_for_restricted_call` about quoting, which is a semantics decision with its own blast radius
   (arc 198 widened this walker on purpose), and the DESIGN scoped this stone to establishing reach.
   **Reported, not repaired.** If it *is* repaired, the surface above goes to zero and Tier B's
   claim becomes true — which makes it the cheapest route to the 124 ms.
2. **Which other eval-time paths bypass `check_program`.** Row 7 shows `--repl` does not. `eval-ast!`
   and host embeddings were not enumerated.
3. **Whether a scalar `def`'s metadata-map ever reaches `binding_metadata`.** Measured green where the
   `defn` form is red; the mechanism was not traced.
4. **Whether `:user::spawn::service-locus` should be `:wat::`-namespaced at all.** It is a name the
   stdlib *writes into a child program*, so the `vend-only-wat` ruling does not obviously reach it —
   but it is also the second half of this stone's attack surface. A ruling, not a measurement.
5. **A cold (page-cache-evicted) boot number.** Tier A measured 188.40 ms cold against 179.14 warm;
   I measured warm only, and eviction needs privileges I did not take. The delta is Tier A's, not
   re-verified.

---

## ⭑ REGRADED BY THE ORCHESTRATOR ON HIS OWN RUNS, 2026-09-18 — REFUTED, and correctly so

```
floor    Summary [ 255.541s] 5302 tests run: 5302 passed, 22 skipped
         .floor/2026-09-18T22-45-45Z/ · exit=0 · NO ARM.txt
clippy   0  ← read
src/     UNTOUCHED — Phase 2 was built, measured, and REVERTED
```

### ⛔ THE WITNESS REPRODUCES: one metadata map on the USER'S OWN function

```
tierb-base.wat      (defn :user::main [] -> nil (println "hi"))                → "hi"
tierb-witness.wat   (defn :user::main {:restricted-to [:my::]} [] -> nil …)
                    → 9 × DefRestrictedCallerNotAllowed across SIX stdlib files:
                      kernel/services/stdio.wat ×3 · cache.wat ×2 · telemetry/span.wat ·
                      telemetry/journal.wat · query/mem.wat
```

**The user never names a stdlib name.** They restrict *their own*. So the claim Tier B rests on — *no
user-supplied state can reach a stdlib verdict* — is **false**, and the 126 ms is not taken.

### ⭐⭐ THE REFUTATION DEFEATS THE PROOF STRATEGY I DESIGNED

My DESIGN said: *prove it by reading the seven doors the spike instrumented.* Those seven are **all read
by ONE sweep** — the probe window literally wraps `P_CHECK_BODIES` (`check.rs:868`).
**`check:restricted-call` reads an EIGHTH door nobody instrumented**: `CheckEnv::get_binding_metadata`
(`check.rs:1679`), a map a user writes with an ordinary metadata map, **keyed by their own name** — so
there is no reserved-prefix problem to solve. And `walk_for_restricted_call` fires on **every
`WatAST::Keyword` leaf** (arc 198: *"a restriction governs MENTION, not head position"*), including
leaves inside quoted `(:wat::core::forms …)` child-program templates. **Nine stdlib `service-forms`
bodies quote `:user::main`.** That is the path.

★★ The executor's own line is the lesson, and it generalises past this stone:

> **"A proof whose premise is a door census is only as good as the door list."**

⛔ **The eighth door was found by READING `check.rs` — not by the instrument built to find doors.** My
brief pointed the executor at an enumeration and called it a proof strategy; the enumeration was
incomplete, and only reading found that out.

### What it DID prove, and it is worth keeping

The **seven body-infer doors are closed**, and it **lifted the spike's empirical bound by induction**:
the probe records hits *and misses*, so a divergence requires a user write, which requires a
user-declarable key, which is never probed ⇒ there is no first divergence. That is a real proof — of a
smaller claim than Tier B needed. `extend_regs` remains **0 probes** even with a user `extend-type`.

All six routes closed individually, now **by construction** rather than by corpus: `defn`/`def`,
`defclause` (both doors), macros, types/`extend-type`, acronyms/companions, and
`installed_dep_sources()` (host-Rust, folded into the stdlib corpus, hashed into the cache key).

### Two bonus results

1. ⭐ **REPL `defclause` is REFUSED, driven** — `parse_defclause_form` (`runtime.rs:7869`) calls
   `resolve::register`, shared by both phases, so the reservation fires **upstream** of the
   `check_program` bypass. And **`wat --repl` DOES reach `check_program`** —
   `ClauseOverExistingDeclaration` fires there — so **`38b43a6ac`'s named gap is narrower than it
   reads.** A disclosed open item, closed by measurement.
2. **The user-error control was driven across six cache states** (warm / absent / truncated / corrupt /
   stale / off): user `TypeMismatch` caught in all six. Trivial today because nothing elides — recorded
   as the baseline the next stone needs.

### It built Phase 2 and threw it away

Boot **185.71 → 185.71 ms** (warm accounted, median of 9; wall 0.224 s). Sweeps unchanged at
**124.65 ms, 67.1 %** of accounted. ⭑ **Phase 2 was implemented end-to-end, measured, and reverted**
rather than shipped on a premise it had just falsified. `src/` carries nothing; the shape is recorded in
this SCORE for whoever takes the next stone.

### ⭐ THE PATH IT LEAVES, with a number

`check:restricted-call` is only **3.07 ms** of the 124.65. **A Tier B eliding `8b` + `8d` + `8f` alone
keeps 121.58 ms and is immune to this witness.** ⛔ But the honest prerequisite is an **exhaustive,
gated enumeration of what each sweep reads** — precisely because the eighth door was not found by the
door-finding instrument.

It also believes `DefRestrictedCallerNotAllowed` firing inside **quoted** `forms` is itself a bug, and
that fixing *that* is the **cheapest route to the whole 124 ms**. Unsettled, named: which non-REPL eval
paths bypass `check_program`; why a scalar `def`'s metadata map does not reach `binding_metadata`;
whether `:user::spawn::service-locus` should be `:wat::`-namespaced (a ruling); and a cold,
page-cache-evicted number.
