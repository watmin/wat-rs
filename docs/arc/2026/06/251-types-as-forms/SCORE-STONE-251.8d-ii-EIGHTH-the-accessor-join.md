# SCORE — STONE 251.8d-ii (EIGHTH DRAW): the accessor join

Branch `main`, drawn against `2fed10f14`. Parent:
`BRIEF-STONE-251.8d-ii-EIGHTH-the-accessor-join.md`. **Not pushed.** Every number below was
produced this session on a binary I built; `cargo build --release` ran after **every** `src/`
edit and after every `wat/` change, and each row names which binary produced it.

## VERDICT — **LANDED (one cure, one gate). STOP (the conversion).**

⭐⭐ **THE BRIEF'S ADDRESS REPRODUCES — the first one to, after four consecutive wrong
addresses.** Converting `wat/spawn.wat` alone (rc 0, one file, 174+/174−) and rebuilding
(21.89 s, the converted spelling verified embedded) turns
`probe_arc209_c0b3bc_post_spawn::accessor_typechecks_at_parse_time` from PASS to FAIL, with
exactly the two unresolved references the brief printed and in that order. ⛔ **The brief's
CONTROL row is the one thing that does not hold** — see §8.1.

⭐⭐ **NARROWED TO ONE TOKEN, and it is not an accessor at all — it is a `defn` NAME.**
Putting back **only** the name at `wat/spawn.wat:130` — `wat.spawn.process/post-spawn` →
`:wat::spawn::process/post-spawn`, everything else in the file left converted — flips the
target test and `process_post_spawn_hook_receives_child_pid` to PASS on their own. The third
test in the file stays red on its own sibling, `wat.spawn.thread/post-spawn` at line 110.

⭐⭐ **WHICH SIDE IS WRONG, ANSWERED ON A TEN-ROW PAIRED PROBE (§2): the CORPUS is.** A
faithful-Clojure symbol carries exactly ONE `/` and it is the namespace/name split, so
`wat.spawn.process/post-spawn` is the image of **both** `:wat::spawn::process/post-spawn`
and `:wat::spawn::process::post-spawn`. The conversion is not injective and cannot be
undone. ⭐ **THE CURE IS THEREFORE RESTRICTIVE, NOT PERMISSIVE** — four consecutive
permissive stones, and this one breaks the run. ⛔ **And the restrictive cure is a
documented public-API respelling across **27 live files**, which is the builder's ruling and
255.8's already-scheduled stone. I did not take it. I built the gate it needs instead.**

⭐ **`Fault/of` IS A DIFFERENT DEFECT WITH THE OPPOSITE DIRECTION, and that one I cured.**
Its parent `Fault` **is** a type, so the join is legitimate and recoverable — but
`freeze::env::rekey_type_member_functions`, the pass that recovers it, walks
`sym.functions_iter()` only, and a `defmacro` is registered (step 4) and expanded (step 5)
before the `TypeEnv` is attached (step 6.97). ⭐ **One door, permissive, one direction:** the
Keyword arm of `macros::expand`'s macro-call dispatch now asks `other_join_spelling` when
the primary key misses — the same second question its **Symbol** arm already asked since arc
300.1. ⛔ `/` → `::` ONLY; `::` → `/` stays refused, because 255.8's WEIGH ruled the
wrong-join acceptance is the thing to close.

⭐ **THE GATE: `every_stdlib_declaration_name_survives_the_faithful_surface`** — a DERIVED
census (799 declaration names measured, 17 of them `/`-joined) of every stdlib declaration
name whose faithful spelling does not read back as itself. It freezes at **8**: the seven
`wat/spawn.wat` builder constructors and `:wat::core::Fault/of`. Not a hand-list, not a
count — names, with two non-vacuity floors.

⭐ **THE PROBE HAS FAILED ONCE, AND ON THE RIGHT ROW.** Pre-cure (`src/macros/expand.rs` stashed
to HEAD, rebuilt 39.4 s): **exactly 1 row wrong, `:user::cure`**, naming
`:path ":user::Box/of"`; the control and all three walls are GREEN on that same binary.

Landable floor **6013 / 6013 GREEN** · clippy **0** (not cached, 12.92 s) · census
**`no STOP-8`**, 213 = 213 · delta **NEW 3 = baseline 3, RECOVERY 0** · ⭐ ledger **220,
A 94 / B 1 / C 0 / E 125 — UNMOVED, no re-anchoring**. ⛔ **TWO floors went RED before that one, both on a LINT, both MINE** — `no_error_flattening_helper`
(§6.1) and, on the POST-COMMIT run, 255.4's `one_member_join` (§6.2). Both captured
verbatim and cured; the second is a corroboration of this stone's own direction ruling.

⛔ **THE CONVERTED FLOOR IS **107**, still RED, so the conversion does NOT land.**
`wat/` restored (`git checkout -- wat/` → 0 modified paths + rebuild — the eighth proof of
that recovery).

⛔ **THREE CORRECTIONS TO THE BRIEF (§8).**

---

## 0. ⭐ THE PROBE, PROVEN BEFORE USE — the gate the seventh draw's absence created

The brief makes this a gate: *"show it returning PASS **and** FAIL, and carry a control in
each run."* The matcher is
`sed 's/\x1b\[[0-9;]*m//g' | grep -E '^[[:space:]]*(PASS|FAIL|TRY [0-9]+ FAIL)' | …`.

| proof | input | result |
|---|---|---|
| ⭐ **A — it can say FAIL** | recorded RED log `.floor/2026-09-23T02-38-37Z/clean.log` | **224 FAIL / 5899 PASS** |
| ⭐ **B — it can say PASS** | recorded GREEN log `.floor/2026-09-23T02-54-37Z/clean.log` | **6011 PASS / 0 FAIL** |
| ⭐ **C — it survives LIVE ANSI** | live `cargo nextest` over the three post-spawn tests | **3 PASS**, and the Summary line agrees: `3 tests run: 3 passed` |

⭐ **Both words demonstrated, on recorded logs AND on live ANSI-bearing output, before the
first conversion.** Every probe run below also checks the build's exit code and reads the
`Summary` line, never a piped exit code.

---

## 1. ⭐⭐ THE NARROWING — the address holds, and it is a `defn` NAME

### 1.1 Baseline and address, both measured

Baseline, `wat/` untouched, `cargo build --release` up to date:

```
PASS probe_arc209_c0b3bc_post_spawn::accessor_typechecks_at_parse_time
PASS probe_arc209_c0b3bc_post_spawn::process_post_spawn_hook_receives_child_pid
PASS probe_arc209_c0b3bc_post_spawn::thread_post_spawn_hook_fires_with_empty_launch
     Summary [   0.973s] 3 tests run: 3 passed, 6030 skipped
```

`printf '["…/wat/spawn.wat"]\n' | ./target/release/wat ./wat-scripts/fixes/to-faithful-clojure.wat`
(rc 0; `git status --porcelain` = exactly ` M wat/spawn.wat`; `git diff --stat` = 174+/174−),
`cargo build --release` **exit 0, 21.89 s**, converted spelling verified embedded
(`strings target/release/wat | grep -c 'wat\.spawn\.process/post-spawn'` → **1**, and the
keyword spelling → **0**):

```
     Summary [   0.959s] 3 tests run: 0 passed, 3 failed, 6030 skipped
```

⭐ The target's arm, verbatim — **the deliberate bogus field is still there; a second,
legitimate name joined it**:

```
    --- actual (raw) ---
    #wat.resolve/UnresolvedReferences {:message "2 unresolved references" :location nil :causes [] :unresolved [#wat.resolve/UnresolvedReference {:path ":wat::spawn::process/post-spawn" :context "call head — not a builtin, not a registered function" :span #wat.core/Span {:file "tests/services/probe_arc209_c0b3bc_post_spawn_bogus_accessor.wat" :line 16 :col 15 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 16 :col 46}}}} #wat.resolve/UnresolvedReference {:path ":wat::spawn::ProcessLaunch/bogus-field" :context "call head — not a builtin, not a registered function" :span #wat.core/Span {:file "tests/services/probe_arc209_c0b3bc_post_spawn_bogus_accessor.wat" :line 18 :col 81 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 18 :col 119}}}}]}
```

### 1.2 ⭐⭐ ONE TOKEN — and the brief's word "accessor" is the wrong word

`sed -i '130s|wat\.spawn\.process/post-spawn|:wat::spawn::process/post-spawn|'` — the
**declaration name alone**, nothing else in the 174 changed lines put back — then
`cargo build --release` (22.35 s):

| `wat/spawn.wat`, converted except… | the three tests |
|---|---|
| fully converted | ⛔ **0 / 3** |
| ⭐ **only the `defn` NAME at :130 put back** | ⭐ **2 passed / 1 failed** |

The one still failing is `thread_post_spawn_hook_fires_with_empty_launch`, whose own
declaration `wat.spawn.thread/post-spawn` at line 110 is still converted — the same class,
one file, seven sites.

⛔ **The discriminating form is a `defn` NAME, not an accessor:**

```wat
(:wat::core::defn :wat::spawn::process/post-spawn [f <- [:wat::spawn::ProcessLaunch :-> :wat::core::nil]] -> :wat::spawn::ProcessOpts …)
;; converts to
(wat.core/defn   wat.spawn.process/post-spawn   [f :- [wat.spawn/ProcessLaunch :-> wat.core/nil]] :- wat.spawn/ProcessOpts …)
```

`process` is not a type. It is the lower-case first half of `wat/spawn.wat`'s documented
per-env builder-constructor convention (that file's own header, lines 90–98:
`(thread/init f)`, `(process/post-spawn f)`, `(process/runner-count n)`, …).

---

## 2. ⭐⭐ WHICH SIDE IS WRONG — a ten-row paired probe, and the answer is the CORPUS

Each row is a two-line program checked with `./target/release/wat --check`. Rows q1–q8 use a
NON-type parent (`process`), r1–r7 a TYPE parent (`Lru`), so a disagreement between the two
blocks isolates the registry's arbitration rather than the spelling.

| row | declaration | call | result | reading |
|---|---|---|---|---|
| q1 | kw `:user::process/post-spawn` | kw `/` | ✅ CLEAN | today's stdlib shape |
| ⛔ **q2** | **SYM `user.process/post-spawn`** | **kw `/`** | ⛔ **UNRESOLVED `:user::process/post-spawn`** | ⭐ **THE DEFECT** |
| q3 | SYM | SYM | ✅ CLEAN | whole-corpus conversion is self-consistent |
| q4 | kw `/` | SYM | ✅ CLEAN | the SYMBOL call head already flips |
| ⛔ q5 | kw `::` | kw `/` | ⛔ UNRESOLVED | a KEYWORD call head does **not** flip |
| q7 | SYM `user.process/post-spawn` | kw **`::`** | ✅ **CLEAN** | ⭐ **the symbol declaration registered under `::`** |
| q8 | BOTH spellings declared | kw `/` | ✅ CLEAN | ⛔ **two spellings, two identities — no duplicate error** |
| r1 | kw `:user::Lru/new` | kw `/` | ✅ CLEAN | |
| ⭐ r2 | **SYM `user.Lru/new`** | **kw `/`** | ⭐ **CLEAN** | **a TYPE parent does not break** |
| ⭐ s1 | kw `:user::Lru::new` | kw `::` | ⛔ **UNRESOLVED** | ⭐ the `::` name was REKEYED away |
| ⭐ s2 | kw `:user::Lru::new` | kw `/` | ✅ resolves | ⭐ …to the `/` member join |
| ⛔ r5 | (none) | kw `:user::Lru/nope` | ⛔ UNRESOLVED | the wall on a type parent |
| ⭐ r7 | SYM, parent **made a record** | kw `/` | ✅ **CLEAN** | ⭐ **making `process` a TYPE cures q2** |

⭐ **r7 and s1/s2 together name the mechanism without a grep:**
`freeze::env::rekey_type_member_functions` (`src/freeze/env.rs:304`) walks every registered
function, and when the last `::` segment of its name is a **known type** it rekeys
`parent::method` → `parent/method` (255.4: *a member join is `/`, always*). That is the pass
that puts back the `/` that `ns_to_wat_path` destroyed. It cannot fire when the parent is not
a type.

### 2.1 ⭐ The two functions, and why they are not inverses

- forward — `edn::render::wat_keyword_to_clojure_symbol`: *"the last segment is the NAME
  unless it is `Type/method`, in which case `Type` folds into the namespace"*. So
  `:wat::spawn::process/post-spawn` → `wat.spawn.process/post-spawn`.
- back — `declare::parse`'s three name slots (`:409`, `:589`, `:716`) and
  `macros::parse`'s (`:158`), all `ns_to_wat_path(id.receiver(), id.method())`, which is
  `format!(":{}::{}", ns.replace('.', "::"), name)` — **`::` always**.

⛔ **The forward map is many-to-one and the back map is total, so the composition is not the
identity.** `reconstruct_call_path` is the registry-arbitrated inverse for CALL heads and
`rekey_type_member_functions` for stored FUNCTION names; both can only answer when the
parent names a type.

### 2.2 ⛔ THE DIRECTION, STATED PLAINLY

**RESTRICTIVE.** A `/` join whose parent is not a type is **unspellable in the faithful
surface** — not "spelled differently", *unspellable*: `wat.spawn.process/post-spawn` is the
only faithful string for it and that string already denotes
`:wat::spawn::process::post-spawn`. No door can recover the difference, because the
information is gone before any door is reached.

⛔ **A permissive cure was available and is WRONG.** Teaching the keyword call head to try
`other_join_spelling` on a miss would make q2 resolve — and would make **q5** resolve too: a
keyword author writing the wrong join against a correctly-stored name. That is exactly what
255.8's WEIGH ruled must be CLOSED before 8d-iii (*"the wrong join is now ACCEPTED for a
SYMBOL author … 8d-iii is what un-contains it"*), and it admits two spellings for one name,
against the campaign's whole premise. **I did not take it.**

### 2.3 ⛔ WHY I DID NOT LAND THE RESTRICTIVE CURE

The restrictive cure is: respell the seven `:wat::spawn::{thread,process}/X` declarations and
every call site as `::`. Measured, excluding `bootstrap/`:

| where | files | note |
|---|---|---|
| `wat/spawn.wat` | 1 | the 7 declarations |
| `tests/**` | 15 | `.wat` fixtures |
| `wat-scripts/**` | 10 | probes, `.intueri` notes, **2 recorded replay `.pre`/`.post`** |
| `wat-tests/**` | 1 | |
| **live total** | ⭐ **27** | |
| `docs/arc/**` | 8 | prose — history, not to be rewritten |

(`git grep -l -E ':wat::spawn::(process|thread)/(post-spawn|env|max-message-bytes|runner-count|init)\b'`,
minus `bootstrap/`, minus this stone's own gate in `src/freeze/env.rs`.)

⛔ **This is a respelling of a DOCUMENTED PUBLIC API** (`wat/spawn.wat` lines 90–98 spell the
convention out as a feature), it must go through a recorded `wat-scripts/fixes/*.wat` codemod
with a replay fixture and an ORACLE (`every_recorded_migration_replays` is a gate), and two of
the files that carry the name are **another migration's recorded byte-exact `.post`**, which
must not move. ⭐ **255.8's WEIGH already scheduled it as its own stone** — *"A stone for the
wrong-join acceptance BEFORE 8d-iii"* — and opening it is the builder's ruling, not a
rider's. **What I owed this stone was the DIRECTION and an exact, derived, non-growing list.
Both are below.**

---

## 3. ⭐ `Fault/of` — CHECKED SEPARATELY, AND IT IS THE OTHER DIRECTION

The brief: *"`Fault/of` is one of the THREE families 255.8 named under 'what a door cannot
express' … Check `Fault/of` separately — a different file, and 255.8 says a different
registry family."* ⭐ **Correct on all three counts, and the difference is bigger than the
file:** `Fault` **is** a type (`(:wat::core::defrecord :wat::core::Fault …)`, `wat/core.wat:2199`),
so this join is legitimate and recoverable — but `:wat::core::Fault/of` is a **`defmacro`**
(`wat/core.wat:2204`), and `rekey_type_member_functions` walks `sym.functions_iter()`.

### 3.1 Measured in situ, by the single-file loop

`wat/core.wat` converted ALONE (rc 0), rebuilt (22.13 s) — the declaration is now
`(wat.core/defmacro wat.core.Fault/of …)`:

| probe | result on the PRE-CURE binary |
|---|---|
| `(:wat::core::Fault/of "boom")` — keyword call | ⛔ `#wat.resolve/UnresolvedReferences :path ":wat::core::Fault/of"` |
| `(wat.core.Fault/of "boom")` — symbol call | ⛔ `unknown callee: :wat::core::Fault/of` |
| a program naming `Fault` nowhere at all | ⛔ **still** `unknown callee: :wat::core::Fault/of` |

⭐ **The third row is the one that matters: the stdlib's OWN call** (`wat/spawn.wat:475`,
`:error (:wat::core::Fault/of msg)`) **fails, so every program that loads the stdlib
inherits it.**

### 3.2 The unit-probe pair that isolates it to the MACRO registry

| row | declaration | call | PRE-CURE | POST |
|---|---|---|---|---|
| m1 | kw `defmacro :user::Fault/of` | kw `/` | ✅ CLEAN | ✅ CLEAN |
| ⭐ **m2** | **SYM `defmacro user.Fault/of`** | **kw `/`** | ⛔ **UNRESOLVED** | ⭐ **CLEAN** |
| m3 | SYM | SYM | ✅ CLEAN | ✅ CLEAN | |
| ⛔ m4 | SYM | kw `:user::Fault/nope` | ✅ refused | ✅ refused |
| ⛔ m5 | kw `/` | kw `::` | ✅ refused | ✅ refused |
| ⛔ m6 | SYM, NON-type parent | kw `:user::helper/nope` | ✅ refused | ✅ refused |

⭐ **m3 is why the defect hid**: `macros::expand.rs`'s **Symbol** arm has asked the second
question since arc 300.1 — its own comment says it outright, *"A defmacro name is stored with
`ns_to_wat_path` (`::`). A call whose parent is a type reconstructs to `/`. Ask the registry
which spelling it holds."* The **Keyword** arm, forty lines above it, did not.

### 3.3 ⭐ THE CURE — one door, one direction, the `Keyword` arm's hot path unchanged

`src/macros/expand.rs`, the keyword arm of the macro-call dispatch:

```rust
let joined_alt = if registry.contains(head) || !head.contains('/') {
    None
} else {
    crate::types::other_join_spelling(head).filter(|alt| registry.contains(alt))
};
let macro_key: Option<&str> = if registry.contains(head) { Some(head.as_str()) } else { joined_alt.as_deref() };
```

- ⛔ **`/` → `::` ONLY.** `other_join_spelling` flips whichever join it is handed; the
  `head.contains('/')` guard makes this one-directional, so a keyword author's WRONG join
  (`::` where the registry holds `/`) stays refused — **m5 is that control, green on both
  binaries.**
- ⭐ **The guard is also the hot path.** Every keyword-headed list in every program reaches
  this line; without it each miss would allocate a flipped `String`. With it, only a head
  that actually carries a `/` and already missed pays anything.
- ⭐ **The `Keyword` arm's success path is byte-identical** (255.13's DUAL-ARM RULE): when
  `registry.contains(head)`, `macro_key` is `head` and nothing else changes.

---

## 4. ⭐ THE NON-VACUITY ROWS — five rows, ONE test, and two of them are green on both binaries

`tests/macros/probe_arc251_8d_macro_member_join.rs` + four fixtures
(`…join.wat`, `…join_control.wat`, and three `.wat.bad` negatives).

| row | what it pins | PRE-CURE | POST |
|---|---|---|---|
| ⭐ 1 `:user::cure` | a FAITHFUL-spelled `defmacro` name called in the KEYWORD spelling | ⛔ **UnresolvedReference `:user::Box/of`** | ✅ **i64(7)** |
| ⛔ 2 `:user::control` | the identical shape declared in the keyword surface | ✅ i64(9) | ✅ i64(9) |
| ⛔ 3 unknown member, TYPE parent | the spelling widened, the population did not | ✅ refused, names `:user::Box/nope` | ✅ refused |
| ⛔ 4 **the DIRECTION control** | kw `/` declared, kw `::` called — must stay refused | ✅ refused, names `:user::helper::of` | ✅ refused |
| ⛔ 5 unknown member, NON-type parent | the wall is on the NAME, not the parent's kind | ✅ refused, names `:user::helper/nope` | ✅ refused |

⛔ **FOUR of the five rows are green on BOTH binaries** — which is what makes them a test of
the WALL rather than of the cure (255.12's discipline). Pre-cure, verbatim
(`git checkout 2fed10f14 -- src/macros/expand.rs`, rebuilt 38.78 s):

```
thread 'probe_arc251_8d_macro_member_join::a_faithful_macro_name_answers_to_its_keyword_spelling' (3395599) panicked at /home/john/work/holon/wat-rs/tests/macros/probe_arc251_8d_macro_member_join.rs:121:5:
the macro member join answered wrongly on 1 row(s):
  :user::cure -> Err(#wat.resolve/UnresolvedReferences {:message "1 unresolved reference" :location nil :causes [] :unresolved [#wat.resolve/UnresolvedReference {:path ":user::Box/of" :context "call head — not a builtin, not a registered function" :span #wat.core/Span {:file "tests/macros/probe_arc251_8d_macro_member_join.wat" :line 20 :col 18 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 20 :col 31}}}}]}), want i64(7) — THE CURE: a FAITHFUL-spelled defmacro name called in the KEYWORD spelling
```

⭐ **The control lives in its OWN fixture, on purpose.** Row 1's fixture does not FREEZE
pre-cure, so a control sharing that file would be unreadable on exactly the binary it exists
to measure — and the helper returns a `Result` rather than using `call_beside_value`, which
PANICS on a freeze failure and would have taken rows 2–5 with it. The first draft did both
wrong and the pre-cure run showed only a panic; this is the second measurement, not the
first.

### 4.1 ⛔ THE DELIBERATE BOGUS FIELD IS UNTOUCHED — three independent checks

The brief: *"the negative test must go red for its OWN reason, and this stone must not cure
it by accident."*

1. ⭐ **This stone's cure cannot reach it.** The cure is in the MACRO registry consult.
   `:wat::spawn::ProcessLaunch/bogus-field` is not a macro in any spelling, and
   `other_join_spelling` would ask for `:wat::spawn::ProcessLaunch::bogus-field`, which no
   registry holds either.
2. ⭐ **`accessor_typechecks_at_parse_time` is GREEN on the landable floor** (6013/6013),
   which means the fixture still fails startup with **exactly one** unresolved reference and
   that reference is `:wat::spawn::ProcessLaunch/bogus-field` — the `.edn` oracle asserts the
   whole error, not a substring.
3. ⭐ **It is still refused under conversion.** In the converted floor's log the path
   `:wat::spawn::ProcessLaunch/bogus-field` is still reported; the test is red only because a
   SECOND, legitimate name joined it.

⭐ Rows 3 and 5 of §4 are the general form of the same wall: a member no macro declares is
refused whether or not the parent names a type, and the refusal is LOCATED and NAMES the head.

---

## 5. ⭐ THE GATE — a derived census, frozen at 8, with two non-vacuity floors

`src/freeze/env.rs`, `mod faithful_surface_round_trip`:
**`every_stdlib_declaration_name_survives_the_faithful_surface`**.

It parses every baked stdlib source (`load::stdlib::stdlib_files()`), takes every top-level
declaration's NAME, and asks one question of each: does
`wat_keyword_to_clojure_symbol` → `ns_to_wat_path` → (for a function) `reconstruct_call_path`
against the **live** `TypeEnv` give the name back?

| | |
|---|---|
| declaration names measured | ⭐ **799** |
| of those, `/`-joined | ⭐ **17** |
| ⛔ **do not survive** | **8** |

The 8, frozen by NAME:

```
:wat::core::Fault/of
:wat::spawn::process/env          :wat::spawn::thread/init
:wat::spawn::process/max-message-bytes   :wat::spawn::thread/post-spawn
:wat::spawn::process/post-spawn   :wat::spawn::thread/runner-count
:wat::spawn::process/runner-count
```

- ⭐ **DERIVED, not a hand-list** (`[[a gate over two hand-lists is a hand-list]]`): the
  question is asked of all 799, and the answer is frozen. The other nine `/`-joined
  declarations (`wat/cache.wat`'s `Lru/*` and `HolographicLru/*`) survive because their
  parents are types and the rekey pass restores them.
- ⛔ **TWO non-vacuity floors, not one.** `measured > 500` catches a stdlib that did not
  load; `slash_joined >= 16` catches the population the gate *discriminates on* vanishing —
  a green over zero `/`-joined names would prove nothing.
- ⛔ **IT DELIBERATELY DOES NOT MODEL THE CURE.** An earlier draft gave macros a modelled
  "rescued by the other-join consult" arm; I reverted `expand.rs` to HEAD and the census
  **still passed** — a gate that models a cure passes whether or not the cure is there. The
  census now asks only the one question the rekey pass asks, and records the code fact that
  a macro is *never* rekeyed; the rescue is proven behaviourally by §4 instead. That is why
  `Fault/of` is ON this list even though it is cured.

---

## 6. EVERY GATE ON THE LANDABLE STATE, WITH ITS EVIDENCE

| gate | landable state (`wat/` untouched) |
|---|---|
| `cargo build --release` after every `src/` edit | ✅ exit 0, 21.9–22.4 s each (39–42 s with `--tests`) |
| ⭐ **`scripts/floor.sh`** | ✅ **GREEN — 6013 / 6013**, 9 slow, 22 skipped, **320.865 s**, exit 0 · `.floor/2026-09-23T04-41-54Z/` (the run the docs-reading lints could see this SCORE in) · ⭐ **AND AGAIN ON THE CORRECTED, FULLY-TRACKED TREE — 6013 / 6013, 317.431 s, exit 0 · `.floor/2026-09-23T04-58-05Z/`**, the run in which `one_member_join` (§6.2) could finally see every fixture · ⭐ **AND A THIRD TIME ON THE COMMITTED TREE — 6013 / 6013, 320.819 s, exit 0 · `.floor/2026-09-23T05-06-02Z/`**, the run the docs-reading lints saw this SCORE's final text in |
| floor denominator | 6011 (seventh draw) **+ 2 of mine** (1 probe + 1 unit row) = **6013** ✓ |
| ⚠ `harvest_wrap_split` | **PASS** [0.192 s] and [0.147 s] (1574/6013) on the two green floors, **PASS** [0.207 s] (1572/6013) on the converted one, **PASS** on the two red ones — named either way, per the standing bookkeeping exception (`278-rules-engine/FINDING-harvest-cost-is-not-a-gate.md`) |
| clippy `-D warnings --all-targets --workspace` | ✅ **0 warnings, rc 0**, twice — **NOT cached** either time (`touch src/macros/expand.rs src/freeze/env.rs` first): **12.92 s**, and **12.80 s** on the final tree |
| census | ✅ **`census-diff: no STOP-8`, rc 0** · final `.census/2026-09-23T05-03-43Z.txt` (**2216** paths — my 2 new tracked `.wat`) vs the seventh draw's landable `.census/2026-09-23T02-53-46Z.txt` (2214), **213 failing = 213 failing** |
| ⭐ `scripts/replay/delta.sh` | ✅ **NEW 3** = baseline 3 (`probe-c1-clean-surface`, `probe-kwargs-peer`, `wat/holon/Ngram.wat`), ⭐ **RECOVERY 0**, exit 0, twice · ORIG-CLEAN 160/179, CONV-CLEAN 157/179 · list-sha `33ede76c…f1cbc3e5`, paths=179 missing=0 · `.delta/2026-09-23T04-04-53Z/` and `.delta/2026-09-23T05-04-37Z/` |
| ⭐ ledger | ✅ **4 / 4 green at 220** — `keyword_heresy_ledger`, unchanged |
| `wat/` clean at commit | ✅ `git status --porcelain | grep -c '^ M wat/'` → **0**, twice (after the single-file loops and after the whole-stdlib restore) |

### 6.1 ⛔ THE FIRST FLOOR WENT RED — ONE LINT, AND IT WAS MINE

⛔ **Not re-run.** Captured whole from `.floor/2026-09-23T03-57-42Z/clean.log`.

```
     Summary [ 319.582s] 6013 tests run: 6012 passed (9 slow), 1 failed, 22 skipped
        FAIL [   0.036s] ( 107/6013) wat::lint no_error_flattening_helper::tests_carry_no_error_flattening_helper
```

The arm, verbatim:

```
🔥🔥🔥 ERROR-FLATTENING HELPERS — 1 site(s) map_err a TYPED error into a String,
destroying the discriminant before any assertion (including Stone L's
`assert_startup_error!`/`matches!`) can see it. This is arc 296 Stone M's whole thesis
(git log -1 de49c56b1) — Stone M drove this census from 71 to 0 with `git diff src/`
empty, and the obvious fix is wrong about HALF the time. Do not reach for it blind:

1. RETURN THE TRUE ERROR TYPE. `StartupError` is the right union only when the helper
genuinely chains SEVERAL of Parse/Macro/Type/Resolve/Check/Runtime. A helper that only
chains `call_beside_value` (which already returns `RuntimeError`) should return
`RuntimeError` directly — wrapping it in `StartupError::Runtime(Box::new(e))` is
gratuitous envelope that hides the real type just to satisfy this rule.

2. IF NO CALLER EVER INSPECTS THE `Err`, the honest shape is NO `Result` AT ALL: panic on
the broken precondition instead. Worked example: `crosses` in
tests/program/probe_arc170_edn_bridge_unspellable.rs — its header records why the Result
was removed rather than unified, so read it before 'restoring' one here.

Population is 0 on a clean tree, so THERE IS NO ALLOWLIST — wanting one here is itself the
signal that the true type hasn't been found yet. Exempt only a genuinely irreducible case
with a per-site `// rune:lint(error-flattening) — <reason>`.

Drive it to ZERO. Offenders:

tests/macros/probe_arc251_8d_macro_member_join.rs:63  fn entry_value -> Result<Value, String>
```

⭐ **The gate was right and both of its rubric points applied to the same helper.** My
`entry_value` chained `startup_from_file` (`StartupError`) and `apply_function`
(`RuntimeError`) and flattened both to `String`, and it had a third arm — "the fixture has no
such entry" — that no row discriminates. Cured per the rubric: it returns
`Result<Value, StartupError>` (the genuine union, the shape
`tests/rete/probe_arc278_then_user_forms.rs:47` already uses), and the missing-entry arm is a
`panic!` on a broken precondition, not a `Result` variant. The floor in §6 is a **different
tree**, not a re-run.

### 6.2 ⛔⛔ THE SECOND FLOOR WENT RED TOO — the POST-COMMIT one, and the gate that caught it is this stone's own argument

⛔ **Not re-run.** `.floor/2026-09-23T04-47-48Z/`. This is exactly why the post-commit floor
is a separate run: `one_member_join` walks `git ls-files`, so it could not see my fixtures
until they were tracked.

```
     Summary [ 316.862s] 6013 tests run: 6012 passed (8 slow), 1 failed, 22 skipped
        FAIL [   0.099s] ( 216/6013) wat::lint one_member_join::no_colon_joined_type_member_in_tracked_wat
```

The arm, verbatim:

```
thread 'one_member_join::no_colon_joined_type_member_in_tracked_wat' (3288684) panicked at /home/john/work/holon/wat-rs/tests/lint/one_member_join.rs:115:5:
colon-joined Type::member still in tracked .wat (1):
tests/macros/probe_arc251_8d_macro_member_join_wrong_join.wat.bad:10: :user::Box::of
```

⭐⭐ **The gate is 255.4's — *a member join is `/`, always* — and it is a CORROBORATION of
this stone, not an obstacle to it.** My DIRECTION control wrote `(:user::Box::of 7)` to prove
that a keyword author's `::`-for-`/` stays refused; that text is the colon-joined
`Type::member` spelling 255.4 RETIRED, and the corpus wall refuses it in any tracked `.wat`
(`.wat.bad` included — the lint's own filter says so). ⛔ **The direction I declined to
implement in §2.2 is already corpus-illegal, and a standing gate said so before I did.**

⛔ **I did NOT add my fixture to that lint's `RECORD_DEF_EXEMPT` hand-list** — an allowlist
entry for a retired spelling is the wrong move twice over
(`[[a gate over two hand-lists is a hand-list]]`). The control is **re-aimed**: a LOWER-CASE
parent (`:user::helper/of` declared, `(:user::helper::of 7)` called) tests the identical
direction, is not a `Type::member` in the lint's sense, and — measured — is a better control,
because it separates the DIRECTION from the RETIREMENT.

⚠ **And the first re-aim was wrong, which the measurement caught before the floor did.** I
first wrote the declaration faithful-spelled (`wat.core/defmacro user.helper/of`), which
stores under `::` — so `(:user::helper::of 7)` was the STORED key and the fixture checked
**rc 0**. A negative fixture that does not fail is not a control. The landed shape declares
the macro in the KEYWORD surface so the stored key really is `/`, and the refusal names
`:user::helper::of`.

Pre-cure was re-measured on the re-aimed fixture (`git checkout 2fed10f14 --
src/macros/expand.rs`, rebuilt 38.78 s): still **exactly 1 row wrong, `:user::cure`** — §4's
block is that run.

---

## 7. ⭐ THE CONVERSION — 112 → 107, the `Fault/of` class GONE, and ONE NEW that is my own instrument

⛔ **Operational discipline, stated because the arc has paid for its absence:** the 64 paths
came from `git ls-files | grep -E '^wat/.*\.wat$'` (⚠ **not** `git ls-files 'wat/**/*.wat'`,
which returns 31 of 64), were passed as ONE explicit EDN vector, and **no `cargo` command and
no `git add` ran while the codemod was writing `wat/`** — the codemod ran detached under
`setsid` and every later step waited on the `DONE` line it writes itself, never on a process
probe (`[[feedback_never_decide_on_pgrep_f]]`).

### 7.1 The numbers

| | seventh draw | **this draw** |
|---|---|---|
| conversion | 64/64, rc 0 | ✅ **64/64, rc 0** (`CONVERSION_RC=0`), 22 m 20 s; `git status --porcelain` = exactly 64 ` M wat/` paths and nothing else outside my own `src/`+`tests/`+SCORE |
| `cargo build --release` on the converted tree | 22.89 s | ✅ **exit 0, 22.83 s** |
| binary starts | ✅ | ✅ (`--check` of my own fixture: rc 0) |
| ⭐ **floor** | ❌ **112 / 6011**, 459.531 s | ❌ ⭐ **107 / 6013**, **453.252 s**, exit 100 · `.floor/2026-09-23T04-30-33Z/` |
| ⚠ `harvest_wrap_split` | PASS | **PASS** [0.207 s] (1572/6013) — named either way |

```
     Summary [ 453.252s] 6013 tests run: 5906 passed (18 slow), 107 failed, 22 skipped
```

⚠ 453.3 s is the *converted-but-red* band (the seventh draw's was 459.5 s), not the 24 s
"stdlib-did-not-load" symptom.

### 7.2 ⭐ FIXED / NEW — **6 fixed, 1 NEW, and the NEW is my own gate refusing a vacuous green**

`comm` over the two sorted failing-test-NAME sets, extracted with a parser that strips the
**entire** `FAIL [   0.836s] (1523/6002) ` prefix including a padded index and then the
binary-name column. ⭐ **The control that says the parser is right: it reproduces the seventh
draw's 112 exactly from its own `clean.log`.**

```
only in the SEVENTH draw's 112  (i.e. FIXED here) : 6
only in mine                    (i.e. NEW)        : 1
```

**FIXED — all six, and they name their own mechanism:**

```
probe_arc278_failure_carries_structured_error::failure_carries_the_raised_error_as_a_structured_record
probe_arc278_loci_died_error_round_trip::recv_lost_cause_is_a_matchable_loci_died_error
probe_arc296_raise_gate::fault_of_type_checks_and_raise_is_caught
wat_arc113_raise_round_trip::raise_data_round_trips_through_failure_message
wat_run_sandboxed::body_raise_surfaces_as_lost_panic
wat_run_sandboxed::partial_stdout_arrives_before_panic
```

⭐⭐ **Every one is on the RAISE path, and `:wat::core::Fault/of` is the smart constructor
that path uses.** The attribution is not an argument — it is in the logs:

| `:path` reported in the converted `clean.log` | seventh | **eighth** |
|---|---|---|
| ⭐ `:wat::core::Fault/of` | **4** | ⭐ **0** |
| `:wat::spawn::process/post-spawn` | 23 | 23 |
| `:wat::spawn::thread/runner-count` | 5 | 5 |
| `:wat::spawn::thread/init` | 3 | 3 |
| `:wat::spawn::process/runner-count` | 3 | 3 |
| `:wat::spawn::process/max-message-bytes` | 2 | 2 |
| ⛔ `:wat::spawn::ProcessLaunch/bogus-field` | **2** | ⛔ **2** |
| `:wat::spawn::thread/post-spawn` | 1 | 1 |

⭐ **THE CLASS THE CURE WAS AIMED AT IS GONE, NOT REDUCED — and nothing else moved by one.**
⛔ **And the deliberate bogus field is still reported, twice, exactly as before.** This stone
did not cure it by accident; the negative test is still red for the spawn class alone.

**NEW — one, and it is `every_stdlib_declaration_name_survives_the_faithful_surface`**, my
own gate. Its arm, verbatim from `.floor/2026-09-23T04-30-33Z/ARM.txt`:

```
thread 'freeze::env::faithful_surface_round_trip::every_stdlib_declaration_name_survives_the_faithful_surface' (3118208) panicked at src/freeze/env.rs:1128:9:
only 1 stdlib declaration names carry a `/` member join — the population this gate discriminates ON has vanished, so a pass proves nothing
```

⭐⭐ **That is the NON-VACUITY floor firing, and it is a SECOND, whole-corpus confirmation of
§2's unit finding.** On a converted stdlib every declaration name arrives as a `Symbol`,
`ns_to_wat_path` writes `::`, and the `/`-joined population collapses from **17 to 1** — the
join is destroyed, corpus-wide, exactly as the ten-row probe said. ⛔ **I am counting it as a
NEW failure and not tuning it away.** A conditional skip could not tell "clean" from "never
ran" (`[[feedback_a_conditional_probe_cannot_tell_clean_from_never_ran]]`). The gate's doc
now records that it is RETIRED or RE-AIMED when 8d-iii lands, never loosened.

### 7.3 The 107 that remain — CLASSIFIED, NOT DIAGNOSED

⛔ **Stated plainly, as the fourth through seventh draws did: I bucketed every failing test's
block by the first typed wat error its stderr carries. I diagnosed the `Fault/of` class and
the spawn class and read their blocks; I did not diagnose the rest, and the classes may
interact.** The same classifier is run over BOTH logs, so the columns are like against like
(⚠ it is NOT the seventh draw's classifier — its buckets differ, which is why the seventh
column below is re-derived here rather than quoted).

| seventh (112) | **eighth (107)** | class |
|---|---|---|
| 31 | **29** | `#wat.resolve/UnresolvedReferences` |
| 25 | **24** | a bare assertion / panic with no typed wat error |
| 19 | **19** | `#wat.check/CheckErrors` |
| 15 | **15** | `#wat.runtime/DeclarationInExpressionPosition` |
| 11 | **9** | `#wat.kernel/LociDiedError` |
| 7 | **7** | `#wat.macro/ProgramBodyEvalFailed` |
| 2 | **2** | `#wat.kernel/AssertionFailure` |
| 1 | **1** | `#wat.check/MalformedForm` |
| 1 | **1** | `#wat.doc/Row` |
| — | **1** | ⭐ my own gate's non-vacuity floor (§7.2) |

⭐⭐ **THE WHOLE `UnresolvedReferences` CLASS IS ONE DEFECT.** Across the entire converted
floor there are only EIGHT distinct unresolved paths (the table in §7.2), and every one of
them is a `Namespace::…/member` surface join. Seven of the eight are `wat/spawn.wat`'s
builder constructors; the eighth is the fixture's deliberate bogus field. **After this stone
the resolve class is, to the last occurrence, the respelling stone's.**

### 7.4 ⛔ THE CONVERSION IS RESTORED

```
git checkout -- wat/    →  git status --porcelain | grep -c '^ M wat/'  →  0
cargo build --release   →  exit 0, 22.80 s
```

The eighth proof of that recovery.

---

## 8. ⛔ WHAT THE BRIEF GOT WRONG — three

### 8.1 ⛔ THE CONTROL ROW DOES NOT HOLD — the class is wider than the named test

The brief's table claims, for `wat/spawn.wat` converted alone: *"build 0 · target FAIL ·
control ⭐ PASS"*. It never names the control. **The two sibling tests in the same file —
`process_post_spawn_hook_receives_child_pid` and
`thread_post_spawn_hook_fires_with_empty_launch` — BOTH FAIL** on that binary (`3 tests run:
0 passed, 3 failed`). Whatever unrelated test the brief carried, a control chosen from the
address's own neighbourhood refutes the implication that one test is affected: the file's
entire post-spawn surface goes down, because all seven builder constructors move at once.

### 8.2 ⛔ "ACCESSOR" IS THE WRONG WORD, AND IT AIMED THE STONE AT THE WRONG SITE

The brief is titled *the accessor join* and says *"⭐⭐ THE LEGITIMATE ACCESSOR stops
resolving."* `:wat::spawn::process/post-spawn` is not an accessor — it is a top-level
`(:wat::core::defn …)` whose NAME carries a `/`, with a lower-case, non-type parent. The
actual accessor in that fixture, `:wat::spawn::ProcessLaunch/pid`, has a TYPE parent and
**survives conversion fine** (r1/r2/s2 measure the class directly). ⭐ The word sent the
question to `canonical_identity` and `reconstruct_call_path` — the brief's §1 names exactly
those two — when the deciding code is `declare::parse`'s name slot and
`freeze::env::rekey_type_member_functions`, neither of which the brief mentions.

### 8.3 ⚠ THE CLASS TABLE IS RIGHT, AND ITS 33 IS 31

Verified against the seventh draw's own converted `clean.log`: the eight `:path` values and
their counts are **exactly** as the brief printed them (55 / 6 / 5 / 4 / 4 / 2 / 2 / 1 = 79
occurrences). ⭐ **And better than the brief claims: those eight are the ONLY unresolved
paths in the entire converted floor — every `UnresolvedReference` in it is a
`Namespace::…/member` surface join.** But the brief's "33 of the 112" is a count of
occurrences mapped onto tests; attributed per test, **31** of the 112 carry any unresolved
reference, of which **3** carry `Fault/of` and **21** carry `process/post-spawn`.

⭐ **And the brief got the hard things right.** Its ADDRESS reproduces, exactly, with the
probe proven first — the first in five draws where that is true. Its instruction to check
`Fault/of` separately found a genuinely different defect with the opposite direction. Its
warning not to assume a permissive cure was the correct warning: the spawn class is
restrictive, and I would have shipped the wrong door without it. And its shape-B note was
correctly aimed — I did not cure `walk_rete_defn_callees`, so the calibration anchor stands
untouched.

---

## 9. ⛔ WHAT MY GREEN CANNOT SEE

1. ⛔ **THE SEVEN ARE NOT CURED, ONLY FENCED.** Every keyword-spelled call to
   `:wat::spawn::{thread,process}/X` still breaks the moment `wat/spawn.wat` is converted.
   The gate stops the class GROWING; it does not shrink it. 8d-iii cannot run until the
   respelling stone does.
2. ⛔ **THE GATE READS DECLARATIONS, NOT CALL SITES.** It censuses the stdlib's declaration
   NAMES. A call site writing a `/` join against a `::`-registered name (row q5) is a
   different defect and nothing here measures it.
3. ⛔ **THE GATE READS THE BAKED STDLIB ONLY.** Not `tests/`, not `wat-scripts/`, not
   `wat-tests/`, not user code. A `.wat` fixture that declares a non-type `/` name is
   invisible to it.
4. ⛔ **THE SIBLING ASYMMETRY IS STILL THERE, AND I MEASURED IT RATHER THAN GUESSING.**
   `macros::eval.rs`'s purity gate (`:173-183`) builds `head_candidates` with the SAME
   shape: the `Symbol` arm pushes `canonical_identity` **and** `other_join_spelling`, the
   `Keyword` arm pushes the raw literal and nothing else. It is a default-deny allow-list,
   so its failure direction is REFUSAL, not acceptance — but a keyword-spelled
   `Type/member` head whose allow-list row is stored under the other join is refused there
   exactly as it was refused in `expand.rs`. I cured ONE arm in ONE file and left this one,
   and I did not construct the program that reaches it.
5. ⛔ **`macros::parse`'s NAME SLOT IS UNTOUCHED.** A `defmacro` still registers under
   `ns_to_wat_path`, so the registry still holds the non-canonical key; I taught the CONSULT
   to ask, not the REGISTRATION to be right. A second consumer of the macro registry that
   does not ask will see the same hole.
6. ⛔ **THE ONE-DIRECTION GUARD IS A STRING TEST.** `head.contains('/')`. If a future name
   grammar puts a `/` somewhere else in a head, the guard admits it. I constructed no such
   name; that is a statement about my search.
7. ⛔ **THE ROUND-TRIP CENSUS'S FIRST DRAFT CONVICTED 727 REGISTERED NAMES** — 416
   `Enum.Variant/field` accessors, 171 nested type names, 140 `defservice`-minted members
   (`stdin-svc/start`, `lru-svc/get`, `journal/…` — 255.8's other two families). I narrowed it
   to DECLARATION names because a minted name never passes through the codemod. ⛔ **That
   narrowing is an argument, not a measurement**: I did not prove that every minted name is
   re-minted correctly from its converted declaration. The variant-accessor class in
   particular round-trips through `ns.replace('.', "::")`, which destroys the enum/variant
   separator, and I left it unexamined.
8. ⛔ **THE LEDGER DID NOT MOVE AND COULD NOT.** My cure's deciding expressions are
   `registry.contains(head)` and `head.contains('/')` — no keyword literal, so the
   discriminator cannot see this cure and a revert of it will not be caught by the ledger,
   only by §4's probe. Same blind spot the seventh draw's `match_qq_head` cure hit.
9. ⛔ **THE 799 IS A DECLARATION COUNT, NOT A NAME COUNT.** A `defservice` is one declaration
   and mints dozens of names; the census asks the declaration.
10. ⛔ **THE CONVERTED FAILURES ARE CLASSIFIED, NOT DIAGNOSED.** I diagnosed the `Fault/of`
    class and the spawn class and read their blocks; the rest are bucketed by the typed error
    their stderr carries, and the classes may interact.
11. ⛔ **`delta.sh`'s RECOVERY COLUMN IS ONLY MEANINGFUL AGAINST AN UNCONVERTED EMBEDDED
    STDLIB** (the fourth draw proved that). My delta row is the landable one.
12. ⛔ **THE CENSUS GATE COULD NOT SEE MY NEW FIXTURES UNTIL THEY WERE COMMITTED** —
    `census.sh` reads tracked files, so the 2214-path run predates my two new tracked `.wat`.

---

## 10. WHAT THE NINTH DRAW NEEDS

1. ⭐⭐ **THE RESPELLING STONE — the builder's ruling.** `:wat::spawn::{thread,process}/X` →
   `::X`, seven declarations across **27 live files**, through a recorded
   `wat-scripts/fixes/*.wat` codemod with a replay fixture and an ORACLE. ⛔ Two of the
   files carrying the name are another migration's byte-exact `.post` and must not move.
   §5's frozen list is the exact input.
2. ⭐ **THE OTHER TWO 255.8 FAMILIES, NOW COUNTED.** `defservice` mints **140** `/`-joined
   members over 11 lower-case parents (`stdin-svc`, `stdout-svc`, `stderr-svc`, `lru-svc`,
   `hologram-svc`, `mem-store`, `sqlite-store`, `journal`, `span`, plus spawn's two). They
   are minted, not written, so the codemod does not touch them — **prove that, or they are
   the next 33.**
3. ⭐ **THE VARIANT-ACCESSOR ROUND TRIP** (§9.7): 416 registered `Enum.Variant/field` names
   whose faithful spelling routes through `ns.replace('.', "::")`. Unexamined.
4. `macros::parse`'s name slot (§9.5) — teach the REGISTRATION, retire the consult.
5. `rete/purity.rs::walk_rete_defn_callees` — still the **last** shape-B site and still the
   calibration anchor. Read 255.13 §4.3's note before touching the assertion.
6. Shape **E**, 125 sites, 78 in `check.rs` — untouched across nine draws, 57% of the ledger.
   The terminal cut needs A+E at zero.
