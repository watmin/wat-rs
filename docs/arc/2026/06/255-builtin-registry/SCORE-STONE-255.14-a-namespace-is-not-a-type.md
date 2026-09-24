# SCORE — STONE 255.14: a namespace is not a type

Branch `main`, drawn against **`f7bf07fac`** ("255.14: draw the join ruling — a namespace is not a
type"). Parent: `BRIEF-STONE-255.14-a-namespace-is-not-a-type.md`. **Not pushed.** Every number
below was produced this session on a binary I built; `cargo build --release` ran after **every**
`src/` edit and after **every** `wat/` change, and each row names which binary produced it.

## VERDICT — **PART 1 LANDED. PART 2: STOP — its premise is refuted by measurement.**

⭐⭐ **PART 1 LANDED, WHOLE.** The seven `wat/spawn.wat` builder constructors are respelled
`/` → `::` in their DECLARATIONS and at every call site, through a **recorded, replayed codemod**
(`wat-scripts/fixes/spawn-builder-namespace-join.wat`) over **24 tracked files**, and the frozen
`UNSPELLABLE_IN_THE_FAITHFUL_SURFACE` list is **8 → 1**. The names did not change: the faithful
image `wat.spawn.process/post-spawn` is byte-identical either way, and a **new test proves both
surfaces reach the same function** while the retired `/` spelling is refused by name.

⛔⛔ **PART 2 IS NOT THE STONE THE BRIEF PRICED, AND I DID NOT TAKE IT.** The brief says *"ONE
interpolation template in `wat/service.wat:2021-2023`"*, *"that single interpolation mints the whole
`<svc>/<op>` family"*, blast radius *"svc 24 / 6 files"*. **All three are false, measured:** there
are **21 non-type-parent mint sites** in `wat/service.wat`, the named one at `:2022` mints **2 of
the 96** distinct minted `/`-joined names in the corpus (`start` alone is **62 occurrences** and is
minted at `:2561`), and the true radius is **340 occurrences across 130 files**. Changing the mint
also **collides** — measured on two probes. §3 has the numbers. This is the **twenty-seventh
correction**.

⛔ **Q2 ANSWERED — YES, THE VARIANT ACCESSORS ARE IN THIS CLASS, SO I STOPPED**, per the brief.

Landable floor **6014 / 6014 GREEN** (318.587 s, exit 0) · clippy **0**, not cached (11.73 s) ·
census **`no STOP-8`**, 213 = 213 · delta **NEW 3 = baseline 3, RECOVERY 0** · ledger **4/4 at 220,
UNMOVED** · 255.4's `one_member_join` **PASS, run not re-read**.

⚠ `harvest_wrap_split` — **PASS** on every run: [0.169 s] (1573/6014) and [0.104 s] (1571/6014) on
the two green floors, [0.103 s] (1570/6014) on the converted one. Named either way, per the standing
bookkeeping exception (`278-rules-engine/FINDING-harvest-cost-is-not-a-gate.md`).

---

## 0. ⭐ THE PROBE, PROVEN BEFORE USE

The matcher is `sed 's/\x1b\[[0-9;]*m//g' | grep -E '^[[:space:]]*(PASS|FAIL)'`, and the
failing-NAME parser (the helper used in §7.2) strips the **entire** `FAIL [   0.836s] (1523/6002) `
prefix — padded index included — and then the binary-name column.

| proof | input | result |
|---|---|---|
| ⭐ **A — it can say FAIL** | my own recorded RED log (§4.1, the pre-migration run) | **2 FAIL / 0 PASS** |
| ⭐ **B — it can say PASS** | this stone's GREEN floor `.floor/2026-09-23T23-10-10Z/clean.log` | **0 FAIL / 6014 PASS** |
| ⭐ **C — the NAME parser reproduces a known answer** | the eighth draw's converted `.floor/2026-09-23T04-30-33Z/clean.log` | **107** — exactly its reported number |
| ⭐ **D — and yields nothing on a green** | this stone's GREEN floor | **0** |

⭐ Both words demonstrated, and the derived instrument reproduces a number it did not produce.

---

## 1. ⭐ THE ADDRESSES — THE BRIEF'S PART 1 REPRODUCES EXACTLY

`grep -n 'defn :wat::spawn::\(thread\|process\)' wat/spawn.wat`, on the binary's own source:

```
 99:(:wat::core::defn :wat::spawn::thread [] …)                      ← NOT in the class (no member)
105:(:wat::core::defn :wat::spawn::thread/init …)                    ✓ brief says :105
110:(:wat::core::defn :wat::spawn::thread/post-spawn …)              ✓ brief says :110
116:(:wat::core::defn :wat::spawn::thread/runner-count …)            ✓ brief says :116
122:(:wat::core::defn :wat::spawn::process [] …)                     ← NOT in the class
130:(:wat::core::defn :wat::spawn::process/post-spawn …)             ✓ brief says :130
133:(:wat::core::defn :wat::spawn::process/env …)                    ✓ brief says :133
141:(:wat::core::defn :wat::spawn::process/max-message-bytes …)      ✓ brief says :141
149:(:wat::core::defn :wat::spawn::process/runner-count …)           ✓ brief says :149
```

⭐ **Seven for seven, to the line.** The brief's PART 1 is the first address in this arc that needed
no correction at all.

---

## 2. ⭐ THE RECORDED CODEMOD — `wat-scripts/fixes/spawn-builder-namespace-join.wat`

⭐ **`rename-keyword-exact`, NOT `rename-keyword-prefix`.** Seven whole-token renames. Exact
whole-name equality is what keeps the rewrite off the prefix siblings (`:wat::spawn::process`,
`:wat::spawn::thread`) and off the RETIRED heads (`:wat::spawn::process/grants`,
`…/uses` — `src/intrinsic/mod.rs:1959,2111`), and it is **idempotent by construction**: after the
rewrite no token equals the old name.

### 2.1 ⛔ THE PATH LIST, AND THE TWO FILES DELIBERATELY LEFT OUT

`git grep -l … -- ':!docs/' ':!bootstrap/' ':!src/' ':!wat-scripts/fixes/replay/' ':!*.intueri'`
→ **24 paths** (⚠ built with `git grep`, never `git ls-files 'wat/**/*.wat'`, which returns 31 of 64).

- `wat/spawn.wat` (the 7 declarations) · `tests/**` 15 · `wat-scripts/probes/**` 7 · `wat-tests/**` 1.
- ⛔ **NOT in the list:** `wat-scripts/fixes/replay/caller-to-emitted-from/{before.pre,after.post}` —
  another migration's recorded **byte-exact** replay fixture, the oracle for
  `every_recorded_migration_replays`. They are text fixtures for a text rewrite and name nothing at
  runtime; moving either falsifies that gate's record. **Both still carry the old spelling, by
  design, and the replay gate is green (§2.4).**
- Hand-edited, and said so plainly because they are **comments, not a structural rewrite** (the
  codemod rides `fix-text-apply` over keyword LEAF spans and cannot see prose):
  `wat/spawn.wat`'s own header (the documented public-API convention, now rewritten as a
  rust-scheme ↔ faithful-Clojure table), `wat-scripts/probes/arc-170/probe-s2-runner-count.wat:4`,
  `wat-scripts/intueri/process-signal-vocabulary.wat.intueri:58`.
- `docs/arc/**` (36 occurrences) is **history and is not rewritten.**

### 2.2 ⭐ THE DRY RUN — 24 CHANGED, 3 CONTROLS UNCHANGED

Copies under `/tmp/s25514/dry/…`, codemod rc **0**, then `diff` per file:

| | |
|---|---|
| files changed | ⭐ **24 / 24**, 7+1+1+1+1+1+1+1+1+1+3+2+4+4+1+1+3+2+1+2+1+2+1+2 line-edits |
| ⛔ **CONTROL — `wat/cache.wat`** | ⭐ **UNCHANGED** — the nine `Lru/*` + `HolographicLru/*` members are a **legitimate `Type/member` join** and this stone must not move them |
| ⛔ **CONTROL — `wat-scripts/probes/arc-170/probe-cap2-process-grantpath.wat`** | ⭐ **UNCHANGED** — the retired `:wat::spawn::process/grants` head is not in this class |
| ⛔ **CONTROL — `tests/services/probe_arc170_c1_kwargs_bracket.wat`** | ⭐ **UNCHANGED** — its `:probe::echo/start`, `:probe::Echo/echo`, `:probe::echo::Handle/addr` are the **minted** family (§3) and a TYPE-parent join |
| idempotence | ⭐ **re-run on the already-migrated copies → `diff -r` = 0 lines** |

The diff is exactly the intended structural change and nothing else — in
`tests/services/probe_arc278_sift_rules.wat`, four `:wat::spawn::process/post-spawn` heads move
while `:wat::query::mem-store/grant`, `:wat::telemetry::journal/start`, `:usr::my-sift'/start` and
`:wat::query::mem-store::Handle/addr` on the *same lines* do not:

```
-             :locus (:wat::spawn::process/post-spawn
+             :locus (:wat::spawn::process::post-spawn
                       (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                         (:wat::query::mem-store/grant msh
```

### 2.3 THE APPLIED RESULT

`printf '[<24 absolute paths>]' | ./target/release/wat ./wat-scripts/fixes/spawn-builder-namespace-join.wat`
→ rc **0**, 24 `[namespace-join]` lines, and `git status --porcelain` showed **exactly** those 24
` M` paths and nothing else. ⛔ **No `cargo` command and no `git add` ran while the codemod was
writing**, and every later step waited on a file the job writes itself, never on a process probe
(`[[feedback_never_decide_on_pgrep_f]]`).

Rebuild after: **`cargo build --release` exit 0, 22.38 s**, and the new spelling is **embedded** —
`strings target/release/wat | grep -c 'wat::spawn::process::post-spawn'` → **2**. ⚠ The old
spelling still returns **1**, and that is not a miss: it is `wat/spawn.wat`'s own header comment
(line 94), which explains the many-to-one map and must quote the retired spelling to do so. The
stdlib is baked with its comments.

### 2.4 ⭐ THE REPLAY FIXTURE — the gate the new codemod owes

`wat-scripts/fixes/replay/spawn-builder-namespace-join/{before.pre,after.post,ORACLE}`.
`before.pre` carries the seven declarations **and four negative controls** that must survive
byte-identically: a TYPE-parent join (`:wat::cache::Lru/get`), the retired head
(`:wat::spawn::process/grants`), a defservice-MINTED join (`:probe::echo/start`), and the two bare
builders. The diff `before.pre` → `after.post` is **exactly 7 lines**; the four controls do not move.

The ORACLE is 7 `spec` + 7 `spec-before` rows against the codemod's own **header spec** (the seven
mappings). ⛔ **There is no `history` row and that is stated in the file**: this migration lands in
the same commit as its fixture, so no prior commit carries the respelled text to cite.

```
cargo nextest run --release -E 'test(every_recorded_migration)'
     Summary [   7.080s] 18 tests run: 18 passed, 6017 skipped
```

⚠ **And it caught me once, before the floor did.** The first draft put `SCOPE: corpus` mid-sentence
in the header; `every_recorded_migration_is_fixtured_or_runed` went RED with
`spawn-builder-namespace-join: expected exactly one ';; SCOPE:' line, found 0`. Cured by giving it
its own line.

---

## 3. ⛔⛔ THE THREE UNESTABLISHED QUESTIONS — ANSWERED BY MEASUREMENT

### 3.1 Q1 — "do the ~140 runtime-minted members all come from THAT template?" → **NO. Measured.**

⭐ **Read, not argued: I macroexpanded a real `defservice` at the FORM level** (a `defservice`
cannot be runtime-macroexpanded — a `:wat::core::Record` evaluates to its constructor fn) and
printed the head and name of every emitted top-level form. The probe is kept, per convention, at
**`wat-scripts/scratch-pad/255-14-defservice-minted-names.wat`** (it loads, so it conforms).

For ONE service (`:probe::kv`) with TWO ops (`get`, `put`), the macro emits:

| join | minted names |
|---|---|
| ⭐ **`::` already** | `kv::Record` `kv::State` `kv::Op` `kv::Admin` `kv::Status` `kv::Handle` `kv::serve` `kv::init` `kv::stop-project` `kv::hibernate-project` `kv::dispatch-admin` `kv::extract-addr` `kv::service-forms` |
| ⛔ **`/` — the class** | `kv/get` `kv/put` **`kv/stop` `kv/hibernate` `kv/grant` `kv/revoke` `kv/start$impl` `kv/resume$impl`** (+ `/start`, `/resume`, `/start$impl-thread`, `/start$impl-process` and the `resume` twins inside the emitted `do`s) |

⛔ **Only `get` and `put` come from `wat/service.wat:2022`.** The rest come from **twenty other
interpolation sites**. `grep -n 'interpolate "[^"]*/' wat/service.wat` gives 30 slash-bearing
templates, of which **21 mint a NON-TYPE-parent join**:

```
2022 {b}/{op-str}     2179 {b}/stop        2218 {b}/hibernate   2254 {b}/grant   2258 {b}/grant
2300 {b}/revoke       2302 {b}/revoke      2549 {b}/start$impl  2551 …           2553 {b}/start$impl-thread
2555 …                2557 {b}/start$impl-process               2559 …           2561 {b}/start
2563 {b}/resume$impl  2565 …               2567 {b}/resume$impl-thread           2569 …
2571 {b}/resume$impl-process               2573 …               2575 {b}/resume
```

and **9 mint a legitimate TYPE-parent join and must NOT move** (`{b}::State/durable`,
`{b}::Handle/handle`, `{b}::Handle/addr`, `{s-str}/surface-forms` and `{proto-base}/surface-forms`
— the surface base is Pascal, so `Kv/surface-forms` is a type parent — and the four
`…/Request` / `…/Response` alias mints).

⭐ **And the corpus population, derived rather than quoted.** 72 `defservice` names extracted from
the corpus, matched back against every tracked `*.wat`/`*.rs`/`*.pre`/`*.post`/`*.intueri` outside
`docs/` and `bootstrap/`:

| | brief | ⭐ **measured** |
|---|---|---|
| mint sites to change | 1 | ⛔ **21** |
| distinct minted `/`-joined names | — | ⛔ **96** |
| occurrences | 24 | ⛔ **340** |
| files | 6 | ⛔ **130** (219 in `tests/`, 110 in `wat-tests/`, 81 in `wat-scripts/`, 6 in `wat/`, 4 in `src/`) |
| of which minted at `:2022` | "the whole family" | ⭐ **≈8 names**; `start` alone is **62 occurrences**, minted at `:2561` |

### 3.2 Q2 — "are the 416 `Enum.Variant/field` accessors in this class?" → ⛔ **YES. I STOPPED.**

Measured with the **authoritative forward map** (`wat-scripts/fixes/to-faithful-clojure.wat`), not
by hand-spelling the faithful form. Input, rc **0**:

```wat
(:wat::core::defenum :u::Demo :- [T] :wat::enum::Pure :Has [has <- :T] :HasNot [])
… (:u::Demo.Has/has d) …
```

The converter writes `(u.Demo.Has/has d)`, and that program is **rc 1**:

```
#wat.resolve/UnresolvedReference {:path ":u::Demo::Has::has"
  :context "namespaced symbol ref — not a builtin, not a registered function (arc 251)"}
```

⛔ **Same defect, one turn worse**: `ns_to_wat_path` destroys the `/` **and** the `.` that separates
the enum from its variant, so `Demo.Has/has` → `Demo::Has::has`. Per the brief — *"if they are, say
so and STOP — that is a separate ruling"* — **I did not touch them.** (The variant CONSTRUCTOR
survives: `u/Demo.Has` round-trips; only the accessor does not.)

### 3.3 Q3 — "does changing the mint collide with the macro's other minted names?" → ⛔ **YES. Measured on two probes.**

The collision is not with the Pascal names (`::Record`, `::Handle`, `::State`, `::Op`, `::Admin`,
`::Status`) — an op is lower-case kebab. It is with the macro's own **lower-case** `::` mints:
`::init`, `::serve`, `::stop-project`, `::hibernate-project`, `::dispatch-admin`, `::extract-addr`,
`::service-forms`.

⭐ **And `init` and `serve` are legal op names TODAY** — both probes `--check` **rc 0** on this
session's binary:

| probe | op | today |
|---|---|---|
| a `defservice` whose surface feature is `init` | mints `:probe::kv2/init` **beside** `:probe::kv2::init` | ✅ **rc 0** |
| a `defservice` whose surface feature is `serve` | mints `:probe::kv3/serve` **beside** `:probe::kv3::serve` | ✅ **rc 0** |

⛔ Under a `::` mint both pairs become **one name**. The `/` join is currently doing real work: it
is what keeps the user's op namespace disjoint from the macro's own. Any part-2 stone must rule on
that first — it is a **name-space design question**, not a respelling.

### 3.4 ⭐ AND A FOURTH THING THE BRIEF DID NOT ASK, WHICH DECIDES THE SCOPE

⛔ **A converted CALL SITE of a minted name resolves CLEAN today.** `tests/services/probe_arc278_mem_store_on_process.wat`,
converted whole by `to-faithful-clojure.wat` — `(wat.query.mem-store/start …)`, a faithful SYMBOL
call head against a registry that holds `/start` — is **rc 0**. That is the eighth draw's row q4
(*"the SYMBOL call head already flips"*). So the minted family is **not** what blocks 8d-iii; it is
alive on the permissive symbol-flip door that 255.8's WEIGH wants closed, and **closing that door
is the stone that must precede any mint change** — not this one. That is why §7's converted floor
reports zero unresolved `<svc>/<op>` paths.

---

## 4. ⭐ THE NON-VACUITY ROWS — SEVEN, ONE TEST, AND IT HAS FAILED ONCE

`tests/services/probe_arc255_14_namespace_join.rs` + four fixtures (one `.wat`, three `.wat.bad`).

| row | what it pins | PRE-MIGRATION | POST |
|---|---|---|---|
| ⭐ 1 `:user::kw-spelled` | `(:wat::spawn::process::runner-count 3)` — the KEYWORD surface | ⛔ **UnresolvedReference** | ✅ **i64(3)** |
| ⭐ 2 `:user::faithful-spelled` | `(wat.spawn.process/runner-count 3)` — **byte-identical to its pre-migration faithful spelling** | ⛔ UnresolvedReference | ✅ **i64(3)** |
| ⛔ 3 `:user::type-parent-join` | `:wat::spawn::ProcessOpts/max-message-bytes` — **a legitimate `Type/member` join is UNTOUCHED** | ⛔ (same fixture) | ✅ **i64(77)** |
| ⭐ 4 registry identity | `:wat::spawn::process::runner-count` present **and** `…/runner-count` ABSENT — ONE key, not two | ⛔ (fixture did not freeze) | ✅ |
| ⛔ 5 the retired join | `(:wat::spawn::process/runner-count 3)` must be REFUSED **and named** | ⛔ **STARTED** | ✅ refused, names it |
| ⛔ 6 unknown member, new join | `(:wat::spawn::process::nope 3)` | ✅ refused | ✅ refused |
| ⛔ 7 unknown member, TYPE parent | `(:wat::spawn::ProcessOpts/nope …)` — the `/` wall still reports the `/` join | ✅ refused | ✅ refused |

⭐⭐ **THE GUARD HAS FAILED ONCE, AND ON THE RIGHT ROWS.** `git stash push -- wat/spawn.wat` (the
pre-migration declarations, everything else migrated), rebuilt **21.50 s**: **5 of 7 rows RED**,
verbatim:

```
thread 'probe_arc255_14_namespace_join::a_namespace_member_answers_to_both_surfaces_and_only_the_new_join' (687123) panicked at /home/john/work/holon/wat-rs/tests/services/probe_arc255_14_namespace_join.rs:137:5:
the namespace join answered wrongly on 5 row(s):
  :user::kw-spelled -> Err(#wat.resolve/UnresolvedReferences {:message "2 unresolved references" … :path ":wat::spawn::process::runner-count" … :path ":wat::spawn::process::max-message-bytes" …}), want i64(3) — the KEYWORD spelling `:wat::spawn::process::runner-count`
  :user::faithful-spelled -> Err(…same…), want i64(3) — the FAITHFUL spelling `wat.spawn.process/runner-count` must reach the same function
  :user::type-parent-join -> Err(…same…), want i64(77) — a TYPE-parent `/` join (`:wat::spawn::ProcessOpts/max-message-bytes`) is UNTOUCHED
  tests/services/probe_arc255_14_namespace_join.wat -> did not freeze: …
  tests/services/probe_arc255_14_namespace_join_old_join.wat.bad -> STARTED, want REFUSED — THE SIDE THAT MOVED — the retired `/` join at a non-type parent must be refused, by name
```

⛔ **Rows 6 and 7 are GREEN on BOTH binaries** — which is what makes them a test of the WALL rather
than of the cure. ⭐ **And the last line is the whole ruling in one row**: pre-migration the retired
spelling STARTED; post-migration it is refused and names itself. The side that moved is the
DECLARATION's.

`git stash pop`, rebuilt **22.22 s**, 3/3 green (this test, the gate, and 255.4's lint).

### 4.1 ⭐ 255.4's `one_member_join` — CONFIRMED BY RUNNING IT, NOT BY RE-READING THE BRIEF

```
PASS [   0.053s] (1/3) wat::lint one_member_join::no_colon_joined_type_member_in_tracked_wat
```

Its discriminator is CASE-based (`tc.is_ascii_uppercase() && (mc.is_ascii_lowercase() || mc == '_')`),
so `process::post-spawn` — lower-case parent — does not trip it. ⚠ **That lint walks `git ls-files`**,
so it could not see my new fixtures until they were in the index; §6's floor is the run in which it
could. That is the eighth draw's §6.2 lesson applied in advance rather than after a red.

---

## 5. ⭐ THE GATE — SHRUNK, NOT LOOSENED

`src/freeze/env.rs`, `mod faithful_surface_round_trip`,
`every_stdlib_declaration_name_survives_the_faithful_surface`.

| | eighth draw | **this stone** |
|---|---|---|
| declaration names measured | 799 | ⭐ **799** — unchanged (the respelled names still carry a `/` in their FAITHFUL image, so they are still asked the question) |
| of those, `/`-joined | 17 | ⭐ **10** |
| ⛔ **do not survive** | **8** | ⭐ **1** |

**The frozen list after this change — one name:**

```rust
const UNSPELLABLE_IN_THE_FAITHFUL_SURFACE: &[&str] = &[
    ":wat::core::Fault/of",
];
```

⭐ **Which side moved: the DECLARATION's.** `:wat::spawn::process::post-spawn` →
`wat.spawn.process/post-spawn` → `ns_to_wat_path` → `:wat::spawn::process::post-spawn`. The round
trip is now the identity, with no rekey pass needed, because the declaration finally says what the
faithful surface can spell. `:wat::core::Fault/of` stays for the OTHER reason — it is a `defmacro`,
and `rekey_type_member_functions` walks `sym.functions_iter()` only.

### 5.1 ⛔ THE NON-VACUITY FLOOR `slash_joined >= 16` → `>= 9`, AND WHY IT STILL DISCRIMINATES

⭐ **The measured value is 10, and I read it rather than inferred it** — I temporarily raised the
floor to an impossible value and read the gate's own message on this session's binary:

```
only 10 stdlib declaration names carry a `/` member join — the population this gate discriminates ON has vanished, so a pass proves nothing
```

(and, by the same method on the first floor, `the declaration census measured only 799 names`).
Both floors were then restored — `measured > 500`, `slash_joined >= 9`.

17 → 10 is **exactly** the seven that moved. The 10 that remain are `wat/cache.wat`'s nine
`Lru/*` + `HolographicLru/*` members — **TYPE parents, restored by the rekey pass, the very rows
that make a green here mean something** — plus `Fault/of`. ⛔ The floor is set **one below the
measured 10**, for the same reason it was set one below 17: it catches the population **vanishing**
(an 8d-converted stdlib collapses it to **1** — measured again in §7), not a single row moving. It
is **not re-derived from the answer** each run; the constant is frozen and the SCORE says why it
moved. ⛔ The gate's doc still says: when 8d-iii lands it is **RETIRED or RE-AIMED, never loosened.**

---

## 6. EVERY GATE ON THE LANDABLE STATE, WITH ITS EVIDENCE

| gate | result |
|---|---|
| `cargo build --release` after every `src/`/`wat/` edit | ✅ exit 0 — 21.50 s (baseline) · 22.38 s (post-codemod+gate) · 22.11 s · 21.50 s (pre-migration probe) · 22.22 s (restored) |
| ⭐ **`scripts/floor.sh`** | ✅ **GREEN — 6014 / 6014**, 9 slow, 22 skipped, **318.587 s**, exit 0 · `.floor/2026-09-23T23-10-10Z/` · doctests `0 failed` · ⭐ **AND AGAIN ON THE FULLY-TRACKED TREE — 6014 / 6014, 315.212 s, exit 0 · `.floor/2026-09-23T23-54-32Z/`**, the run in which the `git ls-files` lints (255.4's `one_member_join`, the tracked-wat gates, the doc-link lint) could see every new fixture AND this SCORE's final text |
| floor denominator | eighth draw **6013 + 1 of mine** (the §4 probe) = **6014** ✓ |
| ⚠ `harvest_wrap_split` | **PASS** [0.169 s] (1573/6014) and [0.104 s] (1571/6014) — named either way |
| clippy `-D warnings --all-targets --workspace` | ✅ **0 warnings, rc 0**, **11.73 s** — ⚠ **NOT cached** (`touch src/freeze/env.rs src/lib.rs` first; the log shows `Compiling wat`, and 11.73 s is the same band as the eighth draw's un-cached 12.92 s, not the 0.07 s cache hit) |
| census | ✅ **`census-diff: no STOP-8`, rc 0** · `.census/2026-09-23T23-16-28Z.txt` (**2219** paths — the eighth draw's 2216 **+ my 3** new tracked `.wat`) vs `.census/2026-09-23T05-03-43Z.txt` (2216), **213 failing = 213 failing** |
| ⭐ `scripts/replay/delta.sh` | ✅ **NEW 3 = baseline 3** — the same three files (`probe-c1-clean-surface`, `probe-kwargs-peer`, `wat/holon/Ngram.wat`) — ⭐ **RECOVERY 0**, exit 0 · ORIG-CLEAN 160/179, CONV-CLEAN 157/179 · list-sha `33ede76c…f1cbc3e5`, paths=179 missing=0 · `.delta/2026-09-23T23-17-25Z/` |
| ⭐ ledger | ✅ **4 / 4 green at 220 — UNMOVED.** ⚠ **It could not move**: `keyword_heresy_ledger`'s discriminator counts SHAPES in `src/`; my change is a `.wat` respelling plus one frozen `&[&str]`, which it cannot see. A revert of this stone would be caught by §4's probe and by §5's gate, not by the ledger. |
| `every_recorded_migration_replays` | ✅ **18 / 18** (after the SCOPE cure, §2.4) |
| 255.4 `one_member_join` | ✅ **PASS**, and the whole-tree run in §6 is the one where it could see my fixtures |
| `wat/` clean at commit | ✅ `git status --porcelain | grep -c '^ M wat/'` → **0** after the converted-floor restore (§7.3) |

---

## 7. ⭐⭐ THE CONVERSION — **107 → 78**, 29 FIXED, **0 NEW**, AND THE RESOLVE CLASS IS GONE

⛔ **Operational discipline, stated because the arc has paid for its absence:** the 64 paths came
from `git ls-files | grep -E '^wat/.*\.wat$'` (⚠ **not** `git ls-files 'wat/**/*.wat'`, which
returns 31 of 64), were passed as ONE explicit EDN vector, and **no `cargo` command and no
`git add` ran while the codemod was writing `wat/`** — it ran detached under `setsid` and every
later step waited on a `CONVERSION_RC=` file the job writes itself, never on a process probe.

### 7.1 The numbers

| | eighth draw | **this draw** |
|---|---|---|
| conversion | 64/64, rc 0, 22 m 20 s | ✅ **64/64, `CONVERSION_RC=0`**, 16:19:49 → 16:42, **~23 min**; `git status --porcelain` = exactly 64 ` M wat/` paths |
| `cargo build --release` on the converted tree | 22.83 s | ✅ **exit 0, 22.16 s** |
| binary starts | ✅ | ✅ (`--check` of my own fixture: rc 0) |
| ⭐ **floor** | ❌ **107 / 6013**, 453.252 s | ❌ ⭐ **78 / 6014**, **444.212 s**, exit 100 · `.floor/2026-09-23T23-42-43Z/` |
| ⚠ `harvest_wrap_split` | PASS | **PASS** [0.103 s] (1570/6014) — named either way |

```
     Summary [ 444.212s] 6014 tests run: 5936 passed (20 slow), 78 failed, 22 skipped
```

⚠ 444.2 s is the *converted-but-red* band (the eighth draw's was 453.3 s), not the ~24 s
"stdlib-did-not-load" symptom.

### 7.2 ⭐⭐ FIXED / NEW — **29 FIXED, 0 NEW**

`comm` over the two sorted failing-test-NAME sets, from each run's own `clean.log`, with the
parser proven in §0 (control C reproduces the eighth draw's 107 from its own log; control D yields
0 on a green one).

```
only in the EIGHTH draw's 107  (i.e. FIXED here) : 29
only in mine                   (i.e. NEW)        :  0
```

⭐ **All 29 are the spawn class, and they name their own mechanism** — the three
`probe_arc209_c0b3bc_post_spawn` tests (the eighth draw's address), the three
`probe_arc259_program_init_fn` tests, five `probe_arc278_journal_*`/`s2s_peer`/`recv_over_budget`,
eleven `probe_arc278_sift_*`, two `probe_mapv_side_effect_once`, two `deftest_wat_tests_*`, and
⭐ `wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime` — the
`wat-scripts/` loader gate itself, which went green because the probes under it now name a
declaration that survives conversion.

### 7.3 ⭐⭐ THE ATTRIBUTION IS NOT AN ARGUMENT — IT IS IN THE LOGS, AND IT IS ZERO

Every `:path` reported anywhere in each converted `clean.log`:

| `:path` | eighth | **this draw** |
|---|---|---|
| `:wat::spawn::process/post-spawn` | 55 | ⭐ **0** |
| `:wat::spawn::thread/init` | 6 | ⭐ **0** |
| `:wat::spawn::thread/runner-count` | 5 | ⭐ **0** |
| `:wat::spawn::process/runner-count` | 4 | ⭐ **0** |
| `:wat::spawn::process/max-message-bytes` | 2 | ⭐ **0** |
| `:wat::spawn::ProcessLaunch/bogus-field` | 2 | ⭐ **0** |
| `:wat::spawn::thread/post-spawn` | 1 | ⭐ **0** |
| **any other path** | **0** | **0** |

⛔ **The bogus field's two occurrences are gone because its TEST now PASSES, not because the
control was cured.** `probe_arc209_c0b3bc_post_spawn::accessor_typechecks_at_parse_time` is in the
FIXED list, and that test's `.edn` oracle asserts the **whole** error — so it passing on a
converted tree means the fixture still fails startup with **exactly one** unresolved reference and
that reference is still `:wat::spawn::ProcessLaunch/bogus-field`. A passing test does not print its
stderr into the log. The deliberate negative control is intact; it is the second, legitimate name
that left.

⭐ And the typed-error census over the WHOLE of each converted log, like against like:

| marker | eighth | **mine** |
|---|---|---|
| ⭐ `#wat.resolve/UnresolvedReference` | **75** | ⭐ **0** |
| ⭐ `#wat.resolve/UnresolvedReferences` | **36** | ⭐ **0** |
| `#wat.check/MalformedForm` | 21 | **21** |
| `#wat.runtime/MalformedForm` | 20 | **20** |
| `#wat.check/CheckErrors` | 19 | **19** |
| `#wat.kernel/LociDiedError` | 16 | **16** |
| `#wat.runtime/DeclarationInExpressionPosition` | 15 | **15** |
| `#wat.macro/ProgramBodyEvalFailed` | 14 | **14** |
| `#wat.ast/Keyword` | 14 | **14** |

⭐⭐ **THE CLASS THE STONE WAS AIMED AT IS GONE, NOT REDUCED — and nothing else moved by one.**
The eighth draw wrote *"after this stone the resolve class is, to the last occurrence, the
respelling stone's."* Measured: it was.

⚠ **My own gate is still RED on the converted tree, by design, and it is NOT a NEW failure** — it
is in the eighth draw's 107 too. Its arm, verbatim:

```
only 1 stdlib declaration names carry a `/` member join — the population this gate discriminates ON has vanished, so a pass proves nothing
```

That is a third whole-corpus confirmation that the conversion destroys the join, and the gate's doc
says what to do about it: when 8d-iii lands it is RETIRED or RE-AIMED, **never loosened**.

⛔ **The 78 that remain are classified only on the RESOLVE axis.** I measured that axis to zero and
read the spawn class's blocks; I did **not** diagnose the other 78, and I did not reproduce the
eighth draw's full per-test bucket table (my block splitter did not capture the stderr sections —
stated so the absence is not read as a finding).

### 7.4 ⛔ THE CONVERSION IS RESTORED

```
git checkout -- wat/    →  git status --porcelain | grep -c '^.M wat/'  →  0
cargo build --release   →  exit 0, 22.34 s
```

The ninth proof of that recovery.

---

## 8. ⛔ WHAT THE BRIEF GOT WRONG — FIVE, AND THE BIG ONE IS PART 2

⭐ **Its PART 1 is flawless** — seven addresses, seven lines, exact (§1); the gate shrink is exactly
as predicted; 255.4's lint does not conflict and its case-based reason is the right reason; and
`§"what is not established"` was an honest boundary that pointed at the three things that mattered.

### 8.1 ⛔⛔ "ONE interpolation template … mints the whole `<svc>/<op>` family" — **NO. 21 SITES, AND :2022 MINTS 2 OF 96 NAMES.**

Measured twice, independently: by **macroexpanding a real `defservice`** (§3.1 — the minted roster,
read not argued) and by **grepping every slash-bearing template** in `wat/service.wat`. `:2022`
mints the user-declared ops only. `/start` — **62 of the 340 occurrences** — is minted at `:2561`.
`/stop`, `/grant`, `/revoke`, `/hibernate`, `/resume` and the six `$impl` variants each have their
own site. A stone that changed `:2022` alone would respell two ops per service and leave the entire
lifecycle surface behind.

### 8.2 ⛔⛔ "svc 24 / 6 files" — **NO. 340 OCCURRENCES / 130 FILES / 96 DISTINCT NAMES.**

Derived from the corpus's own 72 `defservice` names (§3.1), not from a pattern. The brief's 24/6 is
roughly the **stdlib** subset; the class lives overwhelmingly in `tests/` (219) and `wat-tests/`
(110). A 17× understatement in both columns.

### 8.3 ⛔ THE NON-VACUITY ROW INVERTS ITS OWN CONTROL

The brief: *"`ProcessLaunch/bogus-field` (a type parent) is UNTOUCHED and **still resolves**."* It
does **not** resolve and must not: it is the deliberate unresolved reference in
`probe_arc209_c0b3bc_post_spawn_bogus_accessor.wat`, and the eighth draw's own §4.1 calls it "the
deliberate bogus field." What must hold is that it is **still REFUSED, and refused for its own
reason**. §4 row 7 pins exactly that (an unknown member on a TYPE parent, still reported under the
`/` join), and §7.3 shows the fixture's test passing on the converted tree with the whole-error
oracle intact.

### 8.4 ⚠ THE PART-1 BLAST RADIUS IS RIGHT IN SPIRIT, DIFFERENT IN COUNT

"spawn 62 occurrences / 28 files": my census of live code is **24 tracked `.wat` files** the codemod
touches, plus `src/freeze/env.rs` (the frozen list), two recorded replay fixtures that must NOT
move, and one `.intueri` — the eighth draw's "27 live files" reconciles. The brief's 62 counts
`src/` and the replay fixtures; the number that matters for a codemod is the path list, and it is 24.

### 8.5 ⚠ PROVENANCE

The brief says *"drawn against `main` @ `170e39834`"*; that commit exists and is the WEIGH, and the
brief itself landed as `f7bf07fac`, which is the HEAD I worked from. Not a defect — recorded so the
two numbers are not read as a conflict. Its landable floor `6013` + my one new test = the **6014**
in §6.

---

## 9. ⛔ WHAT MY GREEN CANNOT SEE

1. ⛔⛔ **PART 2 IS UNTOUCHED — 96 minted names, 340 occurrences, 130 files.** They are `/`-joined at
   a non-type parent exactly as the seven were, and they resolve today only because a converted
   call site is a **SYMBOL** head, which already flips (§3.4, measured on a whole converted
   fixture). 255.8's WEIGH wants that permissive door closed; **the day it closes, all 340 break.**
   My green says nothing about that day.
2. ⛔ **THE 416 `Enum.Variant/field` ACCESSORS ARE IN THIS CLASS AND I DID NOT TOUCH THEM** (§3.2,
   measured). Their `.` separator is destroyed as well as their `/`, so they are strictly worse than
   what this stone cured, and nothing in this tree gates them.
3. ⛔ **`<Surface>::<op>/Request` / `/Response` LOOK LIKE THE SAME CLASS AND I ONLY LOOKED AT THE
   NAME SHAPE.** `wat/service.wat:1381, 2036, 2045` and `src`'s alias mint build
   `:<b>::<op>/Request` — parent `<op>`, lower-case, not a type. I did not construct the program
   that reaches it; that is a statement about my search.
4. ⛔ **THE GATE READS DECLARATION NAMES IN THE BAKED STDLIB ONLY** — not `tests/`, not
   `wat-scripts/`, not `wat-tests/`, not user code, and not call sites. A `.wat` fixture that
   declares a non-type `/` name is invisible to it.
5. ⛔ **NOTHING STOPS THE CLASS COMING BACK OUTSIDE THE STDLIB.** 255.4's `one_member_join` is
   case-based and by construction cannot fire on a lower-case parent — which is what made this
   stone legal, and is also why there is **no** wall against writing `:my::ns::thing/member` again
   tomorrow in a test fixture. I did not build one; that would be a new ruling.
6. ⛔ **THE TWO RECORDED REPLAY FIXTURES STILL CARRY THE RETIRED SPELLING**
   (`wat-scripts/fixes/replay/caller-to-emitted-from/{before.pre,after.post}`), deliberately. They
   are inert text, but a reader mining them for an example gets a dead name, and no gate says so.
7. ⛔ **`docs/arc/**` STILL NAMES THE RETIRED SPELLING — 36 occurrences.** History is not rewritten;
   the cost is that the arc's own prose now documents a name that does not resolve.
8. ⛔ **THE LEDGER DID NOT MOVE AND COULD NOT.** `keyword_heresy_ledger`'s discriminator counts
   shapes in `src/`; a `.wat` respelling plus one frozen `&[&str]` is invisible to it. A revert of
   this stone is caught by §4's probe and §5's gate, and by nothing else.
9. ⛔ **THE PRE-MIGRATION DEMONSTRATION STASHED ONLY `wat/spawn.wat`.** That is the declaration
   half, which is the half the ruling is about — but I did not measure the test against a fully
   reverted tree, so §4's PRE column is "declarations reverted", not "stone reverted".
10. ⛔ **`wat/spawn.wat`'s NEW HEADER TABLE IS PROSE.** It now states the rust-scheme ↔ faithful
    correspondence for nine constructors and nothing checks it against the declarations below it.
11. ⛔ **`delta.sh`'s RECOVERY COLUMN IS ONLY MEANINGFUL AGAINST AN UNCONVERTED EMBEDDED STDLIB**
    (the fourth draw proved that). My delta row is the landable one.
12. ⛔ **THE CONVERTED FLOOR'S REMAINING 78 ARE MEASURED ON ONE AXIS.** I drove the resolve class to
    zero and proved every other typed-error count unchanged to the unit; I did not diagnose any of
    the 78, and the classes may interact.

---

## 10. WHAT THE NEXT STONE NEEDS

1. ⭐⭐ **THE MINT IS NOT ONE TEMPLATE AND NOT A RESPELLING — IT IS A NAMESPACE RULING.** Before any
   `<svc>/<op>` → `<svc>::<op>` stone: **`init` and `serve` are legal op names today** and would
   collide with the macro's own `::init` / `::serve` (§3.3, two probes). The `/` join is currently
   what keeps the user's op namespace disjoint from the macro's. 21 mint sites, 96 names, 340
   occurrences, 130 files.
2. ⭐ **CLOSE THE SYMBOL-CALL-HEAD FLIP FIRST, OR MEASURE WHAT IT HOLDS UP.** §3.4 shows the minted
   family alive on it. That door is 255.8's WEIGH's target and it is now the only thing standing
   between 8d-iii and 340 broken call sites.
3. ⭐ **THE VARIANT-ACCESSOR RULING** (§3.2): 416 registered `Enum.Variant/field` names whose
   faithful spelling routes through `ns.replace('.', "::")`, destroying both separators. **In the
   class, measured, untouched.**
4. **8d-iii can now run on the resolve axis** — the converted floor carries **zero** unresolved
   references (§7.3). What remains there is 78 failures in other classes, undiagnosed.
5. The gate in `src/freeze/env.rs` is RETIRED or RE-AIMED when 8d-iii lands, never loosened, and its
   `slash_joined >= 9` floor moves with it.
