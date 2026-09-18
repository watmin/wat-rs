# SCORE — the stdlib vends only `:wat::`

**Struck 2026-09-18**, branch `sns-sqs`, from HEAD `043259d3c` (HEAD moved to `3ca0d0f41` mid-strike
— a docs-only commit for the NEXT stone, `a-defclause-outranks-a-defn/DESIGN.md`; no code, no effect
on this tree). Builder-ruled, verbatim: *"it must be `:wat::repl::*` — we must only vend `:wat::*`"*.

Nothing was committed. The tree is left dirty for the orchestrator.

---

## ⭑⭑ ROW 1 — ZERO non-`:wat::` NAMES VENDED, FROM THE SYMBOL TABLE

**Instrument:** `freeze::startup_bare()` — a world frozen with NO user source — read across the five
name-keyed tables a user program inherits. **Not grep.** The gate lives at
`src/load/stdlib.rs:938` (`load::stdlib::tests::stdlib_vends_only_wat_prefixed_names`), in-crate
because `MacroRegistry::name_set` is `pub(crate)` (the same reason `stdlib_error_stays_narrow` sits
there).

| table | API | count |
|---|---|---:|
| functions | `SymbolTable::functions_iter` | 2177 |
| types | `TypeEnv::iter` | 431 |
| macros | `MacroRegistry::name_set` | 245 |
| unit variants | `SymbolTable::unit_variants_iter` | 122 |
| `def` values | `SymbolTable::def_values_iter` | 56 |
| | **total vended** | **3031** |

```
BEFORE   :wat:: = 3028     :repl:: = 3        outside :wat:: = 3
AFTER    :wat:: = 3031     :repl:: = 0        outside :wat:: = 0
```

Both lines are direct measurements, taken with a temporary census block in the gate (patched in,
read, and reverted byte-identically — `sha256sum -c` clean both times), not arithmetic.

⭐ **And the vended surface is ENTIRELY `:wat::` — there is no `:rust::` and no unnamespaced name in
it.** That was not assumed; it is why the gate carries **no exemptions at all**. Two would have been
defensible on unforgeability grounds (`:rust::` is reserved; a bare name is refused by
`UnnamespacedName`) and I wrote them first — then measured them at **zero each** and deleted them.
An exemption nothing uses is an unmeasured licence for the next stdlib file. The gate is the ruling
literally: `!name.starts_with(":wat::")` is an offender.

The three renamed names, named by the same instrument after the rename:

```
function :wat::repl::turn   ·   function :wat::repl::eval-form   ·   function :wat::repl::eval-and-loop
```

### The vending claim itself, driven both ways

Not "the name is registered" but "a user program reaches it", before and after, on my own runs:

```
BEFORE   (:repl::turn "x")        -> #wat.check/TypeMismatch  ":repl::turn: parameter #1 expects
                                     (:wat::core::Vector :- [:wat::WatAST]); got :wat::core::String"
         (:wat::repl::turn "x")   -> #wat.runtime/UnknownFunction {:path ":wat::repl::turn"}

AFTER    (:repl::turn "x")        -> #wat.resolve/UnresolvedReference {:path ":repl::turn"
                                       :context "call head — not a builtin, not a registered function"}
         (:wat::repl::turn "x")   -> #wat.check/TypeMismatch  ":wat::repl::turn: parameter #1 …"
```

The pair swaps exactly. A `TypeMismatch` is the proof the name RESOLVED to the stdlib's signature;
an `UnresolvedReference` is the proof it no longer does.

---

## ⭑⭑ ROW 2 — A GATE ENFORCES IT, DRIVEN RED TWICE

**Driven red before the fix** — the strongest available drive, because the red is the defect itself
and the instrument found it independently of the spike:

```
thread 'load::stdlib::tests::stdlib_vends_only_wat_prefixed_names' panicked at src/load/stdlib.rs:974:9:
the frozen stdlib vends 3 name(s) OUTSIDE `:wat::` (of 3031 vended names across five tables). …
  function     :repl::eval-form
  function     :repl::turn
  function     :repl::eval-and-loop
```

**Driven red again AFTER the fix**, as the row asks: two offending forms appended to `wat/repl.wat`
(`(:wat::core::defn :squat::probe …)` and `(:wat::core::defrecord :squat::Probe …)`), rebuilt:

```
the frozen stdlib vends 6 name(s) OUTSIDE `:wat::` (of 3037 vended names across five tables). …
  function     :squat::is-Probe?
  function     :squat::probe
  function     :squat::Probe/a
  function     :squat::Probe'
  type         :squat::Probe
  macro        :squat::Probe
```

⭐ **Two forms produced SIX names across THREE tables** — the accessor, the predicate, the positional
constructor, the type and the macro, not just the `defn`. That is the reason the gate reads five
tables rather than `functions_iter` alone: one careless stdlib `defrecord` vends a whole family, and
a functions-only gate would have caught one sixth of it. Then removed, rebuilt, **green**:

```
running 1 test
test load::stdlib::tests::stdlib_vends_only_wat_prefixed_names ... ok
```

Verified removed: `grep -rn ':squat::' wat/ src/ wat-scripts/` → none. (`grep -c squat wat/repl.wat`
returns 1 — the word *"squats"* in the new header prose, not a name.)

**Non-vacuity is asserted, not assumed:** the gate fails if it walks fewer than 1000 vended names, so
a broken instrument or a stdlib that stopped loading goes red instead of passing on an empty set.

---

## ⭑⭑ ROW 3 — THE WITNESS IS DEAD

The FINDING's witness, reproduced verbatim from its own text, plus the two siblings it names. Both
boot-cache modes, on my own runs, pristine release binaries built either side of the rename.

### BEFORE (HEAD `043259d3c`)

```
witness_base.wat           (:wat::core::defn :user::main …)                    -> exit 0   "hi"
witness_defclause.wat      + (:wat::core::defclause :repl::turn …)             -> exit 3   5 errors
                             #wat.check/NoMatchingClauseAtCallSite ×5, ALL :file "wat/repl.wat"
                             lines 75 · 91 · 97 · 104 · 111
witness_eval_form.wat      + (:wat::core::defclause :repl::eval-form …)        -> exit 3   1 error
                             :file "wat/repl.wat" :line 77, called-arity 2
witness_eval_and_loop.wat  + (:wat::core::defclause :repl::eval-and-loop …)    -> exit 3   1 error
                             :file "wat/repl.wat" :line 129, called-arity 2
```

### AFTER

```
                                        WAT_BOOT_CACHE=off      WAT_BOOT_CACHE=on
witness_base.wat                        exit 0  "hi"            exit 0  "hi"
witness_defclause.wat                   exit 0  "hi"            exit 0  "hi"
witness_eval_form.wat                   exit 0  "hi"            exit 0  "hi"
witness_eval_and_loop.wat               exit 0  "hi"            exit 0  "hi"
```

⭐ **All three attack programs are now byte-for-byte indistinguishable from the baseline.** Not
"the error moved" — the user's `defclause` on `:repl::turn` is now a definition of a name nothing in
the stdlib calls, so it changes no stdlib verdict at all. Cache-independent, measured both ways.

### And the door is WALLED, not merely vacated

```
$ ./target/release/wat user_tries_wat_repl.wat      # (:wat::core::defclause :wat::repl::turn …)
#wat.runtime/ReservedPrefix {:message "cannot define :wat::repl::turn — reserved prefix
  (:wat::, :rust::, :$bound::); user defines must use their own prefix"
  :prefix ":wat::repl::turn"}
```

A user chasing the same attack at the new name is refused at registration. The witness cannot be
re-aimed; that is what takes the surface to zero rather than merely moving it.

---

## ⭑⭑ ROW 4 — A DISTRIBUTED BINARY STILL GETS A REPL (DRIVEN)

`src/distribution/mod.rs:158`'s `REPL_SOURCE` is a **generated program in a Rust string literal** —
the only `:wat::repl::turn` call site outside `wat/repl.wat`, invisible to every `.wat` census and to
the codemod. It was renamed by hand.

Driven through **both published distribution entries** with a real four-turn session
(`defn` / call it / a line that fails / arithmetic / EOF):

```
$ ./target/release/wat --repl < session.txt              # src/bin/wat.rs = distribution::run(&[])
42
#wat.core/Fault {… :path ":usr::nope" :context "call head — not a builtin, not a registered function" …}
7
exit=0

$ ./target/release/cargo-wat wat --repl < session.txt     # distribution::run_with_args
42
#wat.core/Fault {… :path ":usr::nope" …}
7
exit=0
```

`42` proves a definition from an EARLIER turn was callable later (the load-bearing property); the
`Fault` then `7` proves a bad line was reported non-fatally and the session continued; `exit=0`
proves EOF returned rather than raising.

### ⭐ THE DRIVE IS NON-VACUOUS, AND I PROVED THAT TOO

I put the old name back in `REPL_SOURCE` alone, rebuilt, and re-ran:

```
$ ./target/release/wat --repl < session.txt
exit=3
[#wat.kernel.LociDiedError/StartupError [#wat.resolve/UnresolvedReferences {:message "1 unresolved
  reference" … :unresolved [#wat.resolve/UnresolvedReference {:path ":repl::turn" :context "call head
  — not a builtin, not a registered function" :span #wat.core/Span {:file "<repl-entry>" :line 2 …}}]}]]
```

⛔ **Exactly the trap-door as predicted: the REPL dies at freeze while the entire `.wat` corpus is
green and the codemod's own finder reports zero remaining hits.** Then restored and re-driven green.
The standing wall is `tests/cli/wat_repl.rs` (6 tests, real binary over a pipe) — all 6 PASS in the
floor log, and `definitions_persist_across_turns` is the one that would have gone red.

⚠ **Bound, stated:** what I drove is the REPL through `distribution::run` / `run_with_args` with the
canonical (empty) battery slice — the same functions any third-party distribution calls. I did **not**
build a battery-carrying binary; batteries add dep sources and do not touch `REPL_SOURCE`'s freeze
path, and `tests/cli/synthetic_battery.rs` covers battery composition separately. That composition is
reasoning, not measurement, and is labelled as such.

---

## ⭑ ROW 5 — THE CODEMOD IS RECORDED AND IDEMPOTENT

**`wat-scripts/fixes/vend-only-wat-repl-names.wat`** — self-hosted, two entry points
(`--grep` finder / applier), three `:wat::fix::rename-keyword-exact` calls. No python, no sed, no
hand-edit of any `.wat` *form*.

**EXACT, not prefix, and that is the load-bearing choice.** A `:repl::` prefix rename would have been
wrong: two USER programs in this corpus own that namespace — `stdio-service.wat`'s
`:repl::serve`/`:repl::Cmd`/`:repl::Reply` and `repl-daemon.wat`'s `:repl::serve`. Renaming those into
`:wat::repl::` would be refused by `ReservedPrefix` and would break both demos.

### Census FIRST — over all 1872 tracked `.wat`, before anything was written

```
$ git ls-files '*.wat' | … | ./target/release/wat --grep ./wat-scripts/fixes/vend-only-wat-repl-names.wat
10 × #wat.grep/Match  —  ALL :file "wat/repl.wat"
  repl-turn            lines  75 · 91 · 97 · 104 · 111 · 124
  repl-eval-form       lines  77 · 79
  repl-eval-and-loop   lines  63 · 129
```

⭐ **The DESIGN named four `.wat` files; the census says ONE.** `stdio-service.wat`,
`repl-daemon.wat` and `probe-repl-declaration-refusal.wat` hold **zero** occurrences of the three
names — the first two own unrelated `:repl::serve`/`:repl::Cmd` names, and the probe mentions
`:repl::turn` only in prose. The census is the authority, and it disagreed with the brief.

### Dry-run on `/tmp` copies, diffed

All four DESIGN-named files copied and run through the applier:

```
DIFF repl.wat          10 lines changed, exactly the 10 census hits, comments untouched
DIFF stdio-service     (byte-identical)
DIFF repl-daemon       (byte-identical)
DIFF probe             (byte-identical)
```

### Applied to the corpus, then IDEMPOTENCE PROVEN TWO WAYS

```
$ printf '["wat/repl.wat" "…stdio-service.wat" "…repl-daemon.wat" "…probe…wat"]\n' \
    | ./target/release/wat ./wat-scripts/fixes/vend-only-wat-repl-names.wat
$ git status --short        ->  M wat/repl.wat          (the other three untouched)
$ git diff --stat -- wat/   ->  10 insertions(+), 10 deletions(-)

# re-run, same paths
$ sha256sum -c repl.sha     ->  wat/repl.wat: OK        (byte-identical after the second run)
$ wat --grep <fix> < all-1872-paths   ->  0 matches
```

Idempotent **by construction** (`rename-keyword-exact` keys on full-name equality, and
`:wat::repl::turn` ≠ `:repl::turn`), and idempotent **by measurement** (identical bytes, zero
matches).

### The manual tail, named because the codemod cannot reach it

Comment-faithful means incomplete by design. Hand-edited, all outside the form tree:
`src/distribution/mod.rs` (the program STRING + its doc), `src/load/stdlib.rs`'s registration
comment, `src/check.rs`'s note (row 6), `wat/repl.wat`'s header prose, and
`probe-repl-declaration-refusal.wat`'s two prose mentions.

**Residual census, anchored** (`grep -oP '(?<!wat:):repl::(turn|eval-form|eval-and-loop)'` over
`*.wat`/`*.rs`): every remaining occurrence is a deliberate historical reference — the codemod's own
old→new mapping, the recorded witness form, or a "this was X until 2026-09-18" note. Not one is a
live reference; the symbol table and the finder both say zero. One arc record,
`docs/arc/2026/06/278-rules-engine/DESIGN-wat-mcp.md`, still names the old spellings; it is record,
not code, and was left alone.

---

## ⭑ ROW 6 — `check.rs`'s COMMENT NO LONGER ARGUES FROM A REMOVED FACT

`src/check.rs:845-863` (was `:848`). The old text — *"the stdlib is NOT all `:wat::`-prefixed
(`wat/repl.wat` defines `:repl::turn`), so a prefix test would have hidden exactly the three names
that answer this spike"* — is **quoted inside the new note rather than deleted**, because it was true
when written and is exactly how the spike found the three names; a later reader who cannot see it
re-derives it. The note now says a `:wat::` prefix test IS valid today, names the codemod and the gate
that made it so, and keeps file attribution for a reason that does **not** expire: the probe's
question is "which names did a body from a stdlib FILE reach for", and a name prefix answers a
different question that merely coincides. **The coincidence is now gated; the question is still not
the same one.**

---

## ⭑⭑ ROW 7 — FLOOR

`./scripts/floor.sh`, release. **Summary verbatim:**

```
     Summary [ 259.390s] 5292 tests run: 5292 passed, 22 skipped
[floor] exit=0. Log kept at .floor/2026-09-18T02-54-02Z/ regardless — a green run is evidence too.
```

**GREEN.** Read from the Summary line, never a piped exit code. Log kept at
`.floor/2026-09-18T02-54-02Z/` (`raw.log` + `clean.log`). **NO `ARM.txt`.** Zero
`FAIL`/`TIMEOUT`/`SIGSEGV`/`ABORT` lines in the log.

**5292, up from the prior 5291 — the +1 is this stone's gate**, and it is in the log by name:

```
PASS [   0.519s] (1063/5292) wat load::stdlib::tests::stdlib_vends_only_wat_prefixed_names
```

Tier A's band holds: 259.390 s against the prior three runs' 259.799 / 259.813 / 254.555 s — the same
top-of-band. `wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime` PASSED at
259.382 s, which is what type-checks the new `wat-scripts/fixes/` file on the current runtime. All six
`wat::cli wat_repl::*` tests PASSED.

The tree weighed is the tree left behind: the only post-floor edit was the temporary census patch to
`src/load/stdlib.rs`, reverted and confirmed byte-identical by `sha256sum -c`.

---

## ⭑ ROW 8 — `:wat::` RESERVATION DID NOT REFUSE THE STDLIB'S OWN RENAME

**Trap-door 3 does not bite, and the guard CAN tell stdlib from user.** Driven, both directions:

- The renamed stdlib registers cleanly — `cargo build --release` exit 0, and every program (including
  the bare `witness_base.wat`) freezes and runs.
- A USER attempting the identical `defclause` at the new name is refused:
  `#wat.runtime/ReservedPrefix {:prefix ":wat::repl::turn"}` (full text in row 3).

The mechanism, read: `resolve::registration::gate` (`src/resolve/registration.rs:120`) reaches
`Registration::Reserved` only on `privilege == Privilege::User && is_reserved_prefix(name)`; the
baked-stdlib expand pass threads `Privilege::Stdlib`, for which a reserved name is a plain `Insert`.
One gate, one explicit privilege bit, no ambient flag — so there was nothing here to find beyond
confirming it.

---

## ⭑ ROW 9 — CLIPPY + `--no-run`

```
cargo clippy --release --workspace --all-targets   ->  exit 0, zero warnings
cargo nextest run --release --no-run               ->  exit 0
```

### ⛔ BUT CLIPPY WAS RED AT HEAD, IN CODE THIS STONE DID NOT WRITE — AND THAT IS A FINDING

The first `--all-targets` clippy run failed:

```
error: very complex type used. Consider factoring parts into `type` definitions
  --> src/spike_probe.rs:23:15
   |
23 | static TRACE: Mutex<Option<BTreeSet<(String, String, &'static str, String)>>> = Mutex::new(None);
   = note: `-D clippy::type-complexity` implied by `-D clippy::all`
error: could not compile `wat` (lib) due to 1 previous error
error: could not compile `wat` (lib test) due to 1 previous error
clippy exit=101
```

⭑ **PRE-EXISTING at `043259d3c`, and "pre-existing" is not a disposition — it describes my search,
so here is the evidence instead.** `git diff --name-only` did not list `src/spike_probe.rs` when the
red fired; `git show HEAD:src/spike_probe.rs | sed -n '23p'` is byte-identical to the failing line;
`git log -- src/spike_probe.rs` names exactly one commit, **`52b5574ac`** — the FINDING's own commit,
whose regrade reports *"clippy 0"*. It is the **lib** target that fails, so `cargo clippy --release
--workspace` alone is red too: this was not an `--all-targets`-only condition. The most likely reason
it was missed is a clippy pass that was fresh-cached behind a preceding `cargo build`.

**Repaired**, minimally and in the lint's own suggested shape — a named `type TraceRecord` alias,
with a comment recording that it was a red at HEAD. 8 lines in `src/spike_probe.rs`, no behaviour
change. Left unfixed, row 9 could not be met and the orchestrator would have inherited the same red.

---

## ⛔ ROW 10 — TIER B NOT BUILT, `defclause` NOT "FIXED"

`git diff --stat` is the whole scope:

```
 src/check.rs                                       |  20 +++-   comment only (row 6)
 src/distribution/mod.rs                            |  21 +++-   REPL_SOURCE + its doc (row 4)
 src/load/stdlib.rs                                 | 108 ++++-   the GATE + registration comment
 src/spike_probe.rs                                 |   8 +-     the clippy red at HEAD (row 9)
 wat-scripts/scratch-pad/probe-repl-declaration-refusal.wat |  9 +-  prose only
 wat/repl.wat                                       |  55 +++--   the codemod's 10 edits + header prose
?? wat-scripts/fixes/vend-only-wat-repl-names.wat              the recorded migration
```

**No sweep logic changed.** `check.rs:800`'s `preregister_defclause_in_env`, `CheckEnv::register_defclause`'s
unconditional insert (`check/env.rs:423`), and defclause's dispatch precedence over a registered
scheme (`check.rs:6042`) are **untouched**. No cache change, no `.config/nextest.toml` change, no
Tier B. ⚠ The `defclause` precedence flaw is real and independent of namespaces — it is a separate
open ruling, now drawn as `a-defclause-outranks-a-defn` — and this stone deliberately did not go near
it. What changed is that no STDLIB name is reachable by it any more; a user `defclause` over another
USER's name is exactly as unsound as it was yesterday.

---

## ⭑ ROW 11 — THE NEW ATTACK-SURFACE COUNT: **3 → 0**

The spike's own census, re-run against the renamed tree with its own invocation
(`WAT_SPIKE_WITNESS=1 WAT_SPIKE_WITNESS_ALL=1 WAT_BOOT_CACHE=off`), filtered to stdlib body files
exactly as the FINDING filtered it:

```
rows            = 30964      (FINDING: 30,964 — identical)
distinct names  = 3060       (FINDING:  3,060 — identical)
```

| prefix | FINDING (before) | this run (after) |
|---|---:|---:|
| `:wat::` | 2906 | **2909** |
| `:rust::` | 3 | 3 |
| `:$bound::` | 0 | 0 |
| unnamespaced | 146 | 146 |
| `:repl::` | **3** | **0** |
| synthetic (`(:wat::core::Seqable :- [:T])/seq`, `…[:Xt])/seq`) | 2 | 2 |

⭐ **Exactly the three names moved from `:repl::` to `:wat::`, and nothing else changed — the totals
are identical to the digit.** Applying the FINDING's own criterion (namespaced, outside the reserved
prefixes, and expressible as a keyword so a user could actually write it):

```
USER-DECLARABLE names reachable during the stdlib body sweep = 0
door/name pairs                                              = (NONE)
```

Against the FINDING's six pairs over three names (`defclause_regs` and `schemes`, each × `:repl::turn`
/ `:repl::eval-form` / `:repl::eval-and-loop`). The probe's own per-door report agrees:
`defclause_regs probed=1166 user-reachable=0` — it was 3 before. Every other probed name is
reserved-prefix (unforgeable — `ReservedPrefix`), unnamespaced (unforgeable — `UnnamespacedName`), or
one of the two synthetic strings that carry parens and spaces and are not expressible as a keyword.

⚠ **The FINDING's bound carries over verbatim and has NOT been lifted:** this is an
over-approximation of *names* but a one-program sample of *control flow*, so a door a stdlib body
reaches only when a lookup HITS would not appear. It is also why row 11 says Tier B's soundness became
**reachable**, not proven. ⛔ **Tier B was not built and must not be built on this number alone.**

⚠ **And this finding, like its parent, has an expiry condition:** it is true only while the stdlib
vends nothing outside `:wat::`. What is different from the FINDING — which had to say *"any future
stdlib file under a non-`:wat::` namespace widens it silently"* — is that the widening is now **loud**:
`stdlib_vends_only_wat_prefixed_names` goes red at build time. The expiry condition is gated rather
than merely written down.

---

## ⭐ WHAT SURPRISED ME

1. **Grep got it wrong again, in my own hands, in this session.** My residual census —
   `grep -rno ':repl::\(turn\|eval-form\|eval-and-loop\)'` over `*.wat`/`*.rs`, excluding the
   codemod file — reported **38 hits**. The anchored form
   (`grep -rnoP '(?<!wat:):repl::(turn|eval-form|eval-and-loop)'`, same scope) reports **15**, all of
   them deliberate historical prose. ⛔ **23 of the 38 were the pattern matching its own fix** — the
   substring `:repl::turn` *inside* `:wat::repl::turn`:

   ```
   false positives by file:   wat/repl.wat 14 · src/distribution/mod.rs 5
                              src/load/stdlib.rs 2 · probe-repl-declaration-refusal.wat 2
   ```

   That is trap-door 2's exact failure mode — not as a quoted warning, but as a wrong count I printed
   and had to catch. ⭑ **This is the whole argument for the gate being a symbol-table test: I could
   not use grep correctly on this question while holding a written warning that grep gets this
   question wrong.** The authoritative answers came from the two instruments that cannot make this
   mistake — the frozen symbol table (0 names outside `:wat::`) and the codemod's own keyword-leaf
   finder (0 matches over 1872 files).

2. **I made the same class of mistake once more, on the row-11 census, and the numbers caught it.**
   My first pass reported 30,968 rows / 3,061 distinct names against the FINDING's 30,964 / 3,060 —
   and I nearly wrote the +4/+1 up as corpus drift between `fd6688eec` and HEAD. It was my filter:
   the probe records EVERY function's body sweep including the user's own, and the FINDING filtered to
   stdlib body files. Filtering the same way reproduced 30,964 / 3,060 exactly. The discrepancy was
   the measurer, not the system.

3. **The DESIGN's file list was over-broad and the census said so.** Three of its four `.wat` targets
   hold zero occurrences. Had I trusted the brief and prefix-renamed `:repl::`, I would have tried to
   move two USER programs' `:repl::serve` into a reserved namespace and broken both demos.

4. **Two stdlib forms vend six names.** The `:squat::` drive showed the accessor, predicate,
   positional constructor, type and macro all arrive with a `defrecord`. A functions-only gate would
   have passed five sixths of that.

5. **The vended surface is 100 % `:wat::` — no `:rust::`, no unnamespaced.** I expected the census's
   three `:rust::` names to appear in the vended set (they appear in the *probed* set) and wrote
   exemptions for them. Measuring them at zero is what let the gate be the ruling verbatim.

6. **Clippy was red at HEAD** with the previous stone's regrade reporting `clippy 0` (row 9).

---

## WHAT I COULD NOT DRIVE

1. **A battery-carrying distributed binary's REPL.** I drove `distribution::run` (via `wat`) and
   `distribution::run_with_args` (via `cargo-wat`), both with the canonical empty battery slice. A
   third-party binary with real batteries would need a new example crate; that batteries do not touch
   `REPL_SOURCE`'s freeze path is **reasoning**, and is labelled so in row 4.

2. **Whether a user clause that MATCHES the stdlib signature re-points dispatch at RUNTIME.** The
   FINDING scoped this out and it stays out: with the stdlib's names now reserved it is unreachable
   *for stdlib names*, but it remains open for user-over-user names and belongs to the
   `a-defclause-outranks-a-defn` ruling.

3. **Tier B's 126 ms.** Reachable, not proven. See row 11's bound.

---

## ⭑ REGRADED BY THE ORCHESTRATOR ON HIS OWN RUNS, 2026-09-18

```
floor    Summary [ 257.151s] 5292 tests run: 5292 passed, 22 skipped
         .floor/2026-09-18T03-08-41Z/ · exit=0 · NO ARM.txt · Tier A's band holds
clippy   0  ← READ this time, not assumed (see the correction below)
```

| row | the orchestrator's own result |
|---|---|
| 1 | ✅ symbol-table count: **3031 vended, all `:wat::`, ZERO outside**. The gate carries **no exemptions** — the executor wrote two on unforgeability grounds, measured each at zero, and deleted them. |
| 2 | ✅ gate exists and names itself: `load::stdlib::tests::stdlib_vends_only_wat_prefixed_names` PASS. Driven red **twice** by the executor — once pre-rename, naming exactly the spike's three names (an independent reproduction), once post-fix where two added forms produced **six** names across three tables, because one stdlib `defrecord` vends a whole family. |
| 3 | ✅ **witness DEAD** on my run: the program that gave exit 3 + five `NoMatchingClauseAtCallSite` now prints `"hi"`, exit 0. |
| — | ✅ **and the door is WALLED, not vacated**: `(defclause :wat::repl::turn …)` → **`ReservedPrefix`**. |
| 11 | ✅ attack surface **3 → 0**, spike census reproduced **to the digit** (30,964 rows / 3,060 distinct; `:wat::` 2906→2909, `:repl::` 3→0). |

## ⛔⛔ A CORRECTION TO THE ORCHESTRATOR'S OWN RECORD: I CLAIMED A CLIPPY NUMBER I NEVER READ

The executor found **clippy RED at HEAD**, in code this stone did not write: `src/spike_probe.rs:23`,
`clippy::type_complexity` on `static TRACE: Mutex<Option<BTreeSet<(String, String, &'static str,
String)>>>`, failing the **lib** target. Introduced by **`52b5574ac`** — **whose commit message I wrote,
and which says "Clippy 0."**

**Why I got it wrong:** my grading command was
`head -8 "$O" | cut -c1-90; grep -E 'Summary|FAIL|…' "$O" | tail -3`. `head -8` printed `git status` and
the diff stat and **truncated the clippy count off the bottom**, and I then asserted the number from
habit. ⛔ That is `[[feedback_a_truncating_pager_makes_absence_unfalsifiable]]` and *"a line you printed
is not a line you read"* — in the same session in which I wrote both warnings into other people's briefs.

Repaired by the executor with the lint's own suggestion (a named `type TraceRecord` alias, 8 lines), and
**it reported the red rather than silently fixing something it could have blamed on HEAD.** Clippy now
**0**, read.

## ⭑ AND MY CODEMOD FILE LIST WOULD HAVE BROKEN TWO DEMOS

The DESIGN named four `.wat` targets. **Three hold ZERO occurrences of the three names**, and worse:
`wat-scripts/demos/stdio-service/stdio-service.wat` and `crates/wat-edn/demo/repl-daemon.wat` own
**unrelated USER names** — `:repl::serve`, `:repl::Cmd`. ⛔ **A prefix rename of `:repl::` on my brief's
authority would have moved two user programs into a reserved namespace and broken both.** The executor
used exact-name renames instead of trusting the list.

★ I built that list from `grep -rl` without checking which names the files held. **A file list is not a
census.**

## ⭐ GREP GOT THIS QUESTION WRONG A THIRD TIME — in the executor's hands, holding a written warning

Its residual census reported **38** hits; anchored, **15**. **23 of the 38 were the pattern matching its
own fix** — `:repl::turn` inside `:wat::repl::turn`, 14 in `wat/repl.wat` alone. It printed the wrong
count and caught it while holding the DESIGN's warning that grep gets this wrong.

★★ **That is the argument for the gate being a symbol-table test.** A grep-based gate would have
enshrined the error it was written to prevent.

It made the same class of error on row 11 and **nearly wrote it up as corpus drift**: 30,968/3,061
against the FINDING's 30,964/3,060. The cause was its own filter (the probe records every function's
sweep, including the user's). Filtered as the FINDING did, the numbers matched exactly. **The
discrepancy was the measurer.**

## Accepted as reported

- **Trap-door 3 does not bite**: the reservation gate (`resolve/registration.rs:120`) reaches `Reserved`
  only under `Privilege::User`; the baked-stdlib pass threads `Privilege::Stdlib`. Driven both ways.
- **Distributed REPL driven through both published entries** (`wat --repl`, `cargo-wat wat --repl`), four
  turns, exit 0 — ⭐ **with non-vacuity proven**: putting the old name back in `REPL_SOURCE` alone gives
  exit 3 and `UnresolvedReference {:path ":repl::turn"}` *while the whole `.wat` corpus is green*. That
  is the trap-door-1 drive done properly.
- **Codemod recorded and idempotent**: census over 1872 tracked `.wat` → 10 matches, all in
  `wat/repl.wat`; dry-run diffed on `/tmp` copies; re-run byte-identical, finder → 0.
- **Tier B: reachable, NOT proven** — stated plainly. The spike's one-program control-flow bound carries
  over unlifted; what changed is that a future widening is now **loud rather than silent**.
