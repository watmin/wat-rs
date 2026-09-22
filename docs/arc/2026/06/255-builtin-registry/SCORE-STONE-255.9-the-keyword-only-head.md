# SCORE — STONE 255.9: the keyword-only head, and the greens it forges

**Executed against `b1b6c3c23`** (brief drawn at `342ddd051`; `3c32ac281` drew the stone, `b1b6c3c23`
is the SEAM on top of it). **Every number below was produced this session.** No `.wat` in the live
tree changed; the diff is one file, `src/load/loader.rs`.

## VERDICT — **LANDED**

Both defects in the load family are closed through the one door; all three non-vacuity rows hold in
one test; a real corpus file's loads work after conversion (shown, run, byte-identical stdout);
clippy 0, census `no STOP-8`, delta **18** (= baseline).

⛔ **THE FIRST FLOOR WENT RED, AND THE OFFENDER WAS THIS STONE'S OWN TEST.** `no_loose_string_assert`
named `src/load/loader.rs:1404` — my row-1 `contains(…)`. **Arm captured whole in §3.7, verbatim,
before anything was re-run.** Fixed to an exact `assert_eq!` (which makes the row stronger), then
**run 2: `5962 tests run: 5962 passed, 22 skipped`.** ⚠ I also walked into
`[[NEVER READ A PIPED EXIT CODE]]` — the harness reported exit 0 for `floor.sh | tail -30`; that was
`tail`'s. **The Summary line is what caught it.**

⭐ **Two corrections to the brief, both measured** — see *"What the brief got wrong"*.
⛔ **One finding the brief never looks at, cured in this stone:** `scan_for_setter` in the *same
file* is a **second forged green**, and a worse one — `(wat.config/set-redef! true)` inside a loaded
file went **rc 1 → rc 0** with *nothing at all* downstream catching it.

---

## 1. THE CENSUS — whole

### 1.1 What the conversion actually does (corrects the brief's framing)

⚠ The brief describes the class as "*a form's **head***". **The codemod is wider than that.**
`wat/fix.wat`'s `fix-text-leaf-edits` visits **every leaf**, and `head-keyword?` is
`kind == "keyword" AND name contains "::"` — **position-independent**. So *every* `::`-namespaced
keyword becomes a symbol, head or not, except three carve-outs:

| carve-out | rule | consequence for the census |
|---|---|---|
| post-arrow keyword | → a type **form**, not a symbol | a `Some(WatAST::Keyword)` **type-slot** test also stops matching |
| type-shaped keyword (`Head<…>`, `(…)`) | → type form | same |
| trailing-`::` marker | → namespace symbol | `:restricted-to` whitelists |
| **reader-synthesized leaf** (`source-matches-name?` class B) | **skipped** | ⭐ `~x`, `` ` ``, `'` survive as sugar — see §1.4 |

Bare data keywords (`:else`, `:messages`, `:value`) have no `::` and are never touched.

### 1.2 The raw grep, corrected

The brief's crude probe: **83** `Some(WatAST::Keyword` under `src/`, **~14** near a silent `_ => None`.

Measured today at `b1b6c3c23`: **82** matches in **22 files**. `[[GREP IS NOT A CENSUS]]` — I read
all 82. The brief's "~14 within three lines of a silent decline" is **not** the answer either: it
over-counts (five of its `runtime.rs`/`expand.rs` hits are *slot-1 name* reads or test asserts, not
head dispatch) and it **under-counts** (`scan_for_setter` — the second real defect — is a `starts_with`
guard four lines from its decline and the three-line window missed it; so did `config.rs`'s
`setter_head_of`, which is the *already-cured half* of the very same wall).

Of the 82: **3 are test code** (`macros/tests.rs:146`, `runtime.rs:15850`, `freeze/env.rs:850`);
**11 already read both spellings** (`types.rs` ×5, `rete/purity.rs` ×2, `rete/kernel/stratify.rs` ×1,
`rete/kernel/arm.rs` ×3 — each with a `Some(WatAST::Symbol(id,_))` sibling arm, landed by
255.1/255.2/251.9); the rest are head or slot-1 reads, classified below.

⚠ **My own proximity probe is a worked example of the brief's warning.** "A `Some(WatAST::Symbol`
within ±3 lines" returns **14**, not 11: `check.rs:1692`, `check.rs:1770` and `runtime.rs:13834`
match because a `Symbol` appears nearby for an unrelated purpose (extracting the joined variable
name; a prose comment). **Three false positives in fourteen** — read each, do not count them.

### 1.3 The reachability rule — where a symbol head can legally arrive

This is the load-bearing fact, and it is a **pipeline-order** fact, not a per-site one
(`freeze.rs::startup_from_source`'s steps 1–9 / `freeze/env.rs::build_env`):

```
2. collect_entry_file (config)   ← raw symbols
3. resolve_loads                 ← raw symbols   ⛔ THE STONE
3b. extract_rete_defn_names      ← raw symbols
4. register_defmacros → expand_all
5. register_types
6. register_defines (+ methods)
7. normalize_symbol_refs   ←←← SYMBOL → KEYWORD, HERE
   normalize_stored_function_bodies
   resolve_references
8. check_program
9. freeze → run
```

> **Anything at or after step 7 in a CODE position cannot see a symbol head** — normalize already
> rewrote it. Anything **before** step 7 can. And **data positions are never normalized at all**
> (`resolve/boundary.rs`: `quote` / `forms` / `holon::literal` / quasiquote template /
> `matches?` pattern / `match` arm pattern / `make-rule`'s quoted `:when`/`:then`), so a symbol head
> inside quoted data reaches *runtime* unrewritten.

### 1.4 The census table

Disposition key: **SILENT** = declines with no diagnostic (the dangerous one) · **LOUD** = errors
· **BACKSTOPPED** = this site declines silently but a second wall refuses the program anyway
· **UNREACHABLE** = a symbol head cannot arrive.

| site | stage | keyword-only? | symbol head arrives? | disposition | evidence |
|---|---|---|---|---|---|
| **`load/loader.rs::match_load_form`** (head) | 3 | was yes | **YES** | ⛔ **SILENT** → **CURED** | §2, §3 |
| **`load/loader.rs::scan_for_setter`** (head) | 3 | was yes | **YES** | ⛔ **SILENT** → **CURED** | §2, §3 |
| `load/loader.rs::parse_verify_algo` (arg) | 3 | **yes** | YES | **LOUD** (`MalformedLoadForm`) | reported, §4 |
| `load/loader.rs::parse_payload_interface` (arg) | 3 | **yes** | YES | **LOUD** (`MalformedLoadForm`) | reported, §4 |
| `config.rs::setter_head_of` (head) | 2 | **no** (already both) | YES | fine | `config.rs:759-760` |
| `freeze/env.rs::extract_rete_defn_names` / `rewrite_rete_defn_heads` | 3b | **yes** | YES | **LOUD** | §1.5 probe C |
| `macros/expand.rs::expand_once` / `expand_form` (macro-call head) | 4 | **yes** | YES | **LOUD** | §1.5 probe D |
| `macros/expand.rs` quasiquote helpers (1687/1699/1718/2050) | 4 | **yes** | only for the **explicit** `(:wat::core::quasiquote …)`/`(…unquote …)` spelling | **LOUD** | §1.5 probe B |
| `macros/expand.rs::expand_make_rule_*` (877/995/1080) | 4 | **yes** | YES (quoted `:when`/`:then` data) | **LOUD** | §1.5 probe C |
| `macros/eval.rs::validate_pure_total` / `validate_quasiquote_template` | 4 | **yes** | explicit spelling only | **LOUD** | same as probe B |
| `types.rs` (4204/4219/4254/4290/4552) | 5 | **no** (already both) | YES | fine | read |
| `types/surface.rs:680` (`:messages`) | 5 | yes | **NO** — bare keyword, no `::`, never converted | **UNREACHABLE** | §1.1 |
| `types/surface.rs:729`, `declare/preregister.rs:123/278` | 5–6 | yes | slot-1 **name**, converted to a symbol | **LOUD** | falls out as malformed/unregistered |
| `types/surface.rs:1003` (post-arrow type keyword) | 5 | yes | **NO** — the codemod emits a type **form**, not a keyword | **UNREACHABLE for a Keyword test**; the `Some(WatAST::Keyword)` simply stops matching → a *silent* narrowing of `collect_user_type_paths` | ⚠ **see §4 (a)** |
| `resolve/normalize.rs:537/577` (make-rule `quote`/`where`) | 7 | **yes** | YES (data) | **LOUD** | probe C |
| `resolve/normalize.rs:605` (qq escape) | 7 | **yes** | explicit spelling only | **LOUD** | probe B |
| `resolve/walk.rs:87` (call-head resolve) | 7 | yes | **NO** — normalize ran first | **UNREACHABLE** | §1.3 |
| `resolve/walk.rs:236/243`, `resolve/quote.rs:30` | 7 | **yes** | YES (data) | **LOUD** | probes B, C |
| `freeze.rs::refuse_mutation_forms` (eval wall) | 9 | **yes** | YES (eval'd quoted data) | ⚠ **BACKSTOPPED** | §1.5 probe A |
| `runtime.rs::refuse_mutation_forms_in` (eval-ast wall) | 9 | **yes** | YES (eval'd quoted data) | ⚠ **BACKSTOPPED** | §1.5 probe A |
| `runtime.rs:8682/13834` (match-pattern canonicality) | 9 | yes | YES (arm patterns are data) | **LOUD** | the arm simply does not match → `MalformedForm`/no-match |
| `runtime.rs:12674` (`head_of`, session) / `14723` / `19970` (`def` skip) | 9 | **yes** | YES only for a non-freeze `run()`/session path | ⚠ **see §4 (b)** | not probed |
| `check.rs` 527/550/1692/1770/5191/6598/8711/9156 | 8 | yes | **NO** in code position (post-normalize); YES inside `match`-arm/quoted data | **UNREACHABLE** (code) / LOUD (data) | §1.3 |
| `closure_extract.rs:2580/2622/894/2723` | 9 (runtime) | yes | **NO** — bodies pass `normalize_stored_function_bodies` | **UNREACHABLE** | `normalize.rs:71-90` |
| `closure_extract.rs:1308` (qq escape, 3rd descent) | 9 | **yes** | explicit spelling only | **LOUD** | probe B |
| `holon/ast.rs:1034` | 8/9 | yes | **NO** (code position) | **UNREACHABLE** | §1.3 |
| `kernel/serve.rs:74/76/96/98` | 9 | yes | **NO** — the List arg is a *type* form, normalize's type-slot path rewrote its head | **UNREACHABLE** | `normalize.rs::normalize_type_slot` |
| `rete/purity.rs` (667/671/1103/1225/1233/1241/1254/1282/1327) | 8 | **yes** | YES inside a quoted `:when`/`where` DATA region | **LOUD** | probe C |
| `rete/purity.rs:1349/1843` | 8 | **no** (already both) | — | fine | read |
| `rete/kernel/stratify.rs:723/738` (`fences`, LHS head) | 9 | **yes** | YES (LHS conditions are data) | **LOUD** | probe C |
| `rete/kernel/stratify.rs:41`, `rete/kernel/arm.rs:251/267/425` | 9 | **no** (already both) | — | fine | read |
| `rete/expr_ir/mod.rs:512/553` | 8/9 | **yes** | YES | **LOUD** — `553` errors *by name*: `"call head must be a keyword"` | read |
| `rete/matcher.rs:836/855` | 9 | n/a — `Value → WatAST` construction, not head dispatch | — | **UNREACHABLE** | read |
| `freeze.rs:1894` / `macros/tests.rs:146` / `runtime.rs:15850` / `freeze/env.rs:850` | — | test code (3 of them) | — | **UNREACHABLE** | read |

⭐ **The two SILENT sites in the entire census are both in `src/load/loader.rs`, and both are now
cured.** Everything else that a converted form can reach is LOUD, BACKSTOPPED, or unreachable.

### 1.5 The probes behind the dispositions

Every probe: write the rust-scheme file, run it, run
`printf '[…]' | ./target/release/wat ./wat-scripts/fixes/to-faithful-clojure.wat` on a **copy**,
run again, diff.

**A — the two mutation walls are BYPASSED but BACKSTOPPED.** `(:wat::eval-ast! '(<mutation form>))`:

| form | keyword spelling | symbol spelling |
|---|---|---|
| `load-file!` | `eval refused mutation form: :wat::load-file!` | `DeclarationInExpressionPosition` |
| `core::defmacro` | `eval refused mutation form: :wat::core::defmacro` | `DeclarationInExpressionPosition` |
| `config::set-global-seed!` | `eval refused mutation form: :wat::config::set-global-seed!` | `unknown function: :wat::config::set-global-seed!` |

The guard declines in all three; a *different* wall refuses the program anyway. ⚠ **The refusal
survives; the diagnostic does not.** I found no mutation head that escapes both walls — but see §5.

**B — the explicit quasiquote/unquote spelling breaks LOUDLY.** `~x` is reader sugar and the codemod
**skips reader-synthesized leaves**, so the sugar survives (macro probe: `"6"` → `"6"`, rc 0 → 0).
The explicit `(:wat::core::quasiquote (:wat::i64::+ (:wat::core::unquote lit) 1))` converts, the
escape is not recognised, and the program dies `rc 0 → 1` with `unbound symbol: lit`. ⚠ **The
explicit spelling is real corpus: `:wat::core::unquote` appears on 103 lines across 34 `.wat` files
(18 of them `unquote-splicing`)** — an 8d-iii blocker, but a loud one.

**C — the rete path breaks LOUDLY.** Three real corpus files
(`wat-scripts/scratch-pad/probe-arc278-rules-ship-as-declared-payload.wat`,
`probe-reland10-fire-direct.wat`, `probe-reland10-journal-then-fire.wat`), converted:
`:wat::rete::core::defn` surfaces as `UnresolvedReference` (env.rs:742 declined), and the fact-type
keyword `'usr/Hot` converts into something `:wat::core::keyword-node` refuses
(`keyword-node requires a ':'-prefixed string; got "usr/Hot'"`). All three change output visibly.

**D — a macro call head.** Post-conversion `(user/twice …)` still expands (normalize rewrites the
head before `walk` asks), so the macro path is fine for the sugar-free case; when it is not, it dies
at `resolve` as `macro call survived expansion` / unresolved. LOUD either way.

---

## 2. WHAT CHANGED, AND WHY THAT DOOR

`src/load/loader.rs`, **one file, +164/−10** (the bulk is doc-comment and the two new tests).

**(a) `match_load_form`** — the head test now goes through
`crate::form_match::canonical_identity_of` (Keyword **or** Symbol → the one identity string). The six
`match` arms are **untouched**: still keyed on `":wat::load-file!"` … `":wat::signed-load-string!"`.

**Why this door and not another.** `canonical_identity_of` → `canonical_identity` is the exact
inverse of `ns_to_wat_path`, which is what `keyword/to-symbol` (and therefore the codemod) used to
produce `wat/load-file!` in the first place. It is the identity door 255.1/255.6/255.7 already routed
three subsystems through. **The alternatives were rejected:** a second `WatAST::Symbol` arm per form
is the shape that cost this arc four stones and cost 255.4 thirty-two reds; a `head_fqdn` call would
work but is `declare/parse`'s *declaration-name* door with its own STOP-3 caveat ("do not use the
result as a lookup key") — and this **is** a lookup key; a per-site string normaliser is the exact
"one side normalized and the other not" class the injected CLAUDE.md names.

**(b) `scan_for_setter`** — same door, `items.first().and_then(canonical_identity_of)`, feeding the
existing `starts_with(":wat::config::set-") && ends_with('!')` predicate and the error's
`setter_head` field. **Why here:** it is the *loaded-file* half of the entry-file-discipline wall
whose *entry-file* half (`config.rs::setter_head_of`) was already routed through
`canonical_identity`. One wall, two encodings, one of them stale — the drift class exactly.

**Two new tests** in `src/load/loader.rs`'s `mod tests`:
`symbol_head_is_the_same_load_form_and_nothing_else_becomes_one` (the three rows) and
`every_load_form_accepts_the_symbol_head` (the five arms the corpus does not exercise);
plus `setter_in_loaded_file_is_refused_in_either_spelling_and_nothing_else_is`.

---

## 3. THE GATES

### 3.1 The three non-vacuity rows — in one test

`cargo test --release --lib load::loader::tests`:

```
test load::loader::tests::symbol_head_is_the_same_load_form_and_nothing_else_becomes_one ... ok
test load::loader::tests::every_load_form_accepts_the_symbol_head ... ok
test load::loader::tests::setter_in_loaded_file_is_refused_in_either_spelling_and_nothing_else_is ... ok
test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 1291 filtered out
```

And at the CLI, on the converter's own output shape:

| row | file | before the cure | after the cure |
|---|---|---|---|
| **1** `(wat/load-file! "missing.wat")` | `probe/sym.wat` | **rc=0** ⛔ | **rc=1** `#wat.load/Fetch {:message "load: file not found: missing.wat" …}` |
| **2** `(:wat::load-file! "missing.wat")` | `probe/kw.wat` | rc=1, same message | **rc=1, byte-identical** (the test asserts `kw_err.to_string() == sym_msg`) |
| **3** `(some.other/thing "x")` | `probe/row3.wat` | rc=1 `UnresolvedReference :some::other::thing` | **rc=1, identical** — still not a load form |

Row 3 at unit level additionally asserts the form is returned **unchanged, still a `Symbol` head**,
and that `(:some::other::thing "x")` is likewise untouched — so the cure did not turn every
symbol-headed list into a load candidate.

### 3.2 ⭐ A converted REAL corpus file's loads WORK — shown, not asserted

`examples/with-loader/wat/{main,helper,deeper}.wat` (a **3-deep** load chain, the smallest of the 26)
copied to scratch and put through the codemod. Converted `main.wat` reads
`(wat/load-file! "helper.wat")`, `helper.wat` reads `(wat/load-file! "deeper.wat")`.

```
# before the cure
$ wat --check <conv>/main.wat
#wat.resolve/UnresolvedReferences … :path ":user::with_loader::helper::greeting"   rc=1
        ↑ the ONLY surviving symptom: the load vanished, the content it provided is "missing"

# after the cure
$ wat --check <conv>/main.wat      rc=0
$ wat        <conv>/main.wat       "hello, wat-loaded"          rc=0
$ wat  examples/with-loader/wat/main.wat   (unconverted)  "hello, wat-loaded"   rc=0
```

**Byte-identical stdout, converted vs original, through two nested loads.**

### 3.3 ⭐⭐ The forged green, caught on a file in the committed delta sample

`wat-scripts/fmt/run-all.wat` is **the one load-form file in `delta-sample-179.txt`** (see §5 —
the brief says four). Checked from a copy tree its load target is absent, so it is legitimately red.
Both binaries built this session from the same commit, differing only in my diff:

| | original (keyword) | **converted (symbol)** |
|---|---|---|
| **pre-cure binary** | rc=1 `load: file not found: rules/defn.wat` | ⛔ **rc=0 — THE FORGED GREEN** |
| **cured binary** | rc=1 (identical) | ✅ **rc=1, identical diagnostic** |

### 3.3b A SECOND real corpus file — 12 loads through a subdirectory

`wat-scripts/fmt/` copied whole to scratch (74 `.wat`), all 74 converted. `run-all.wat` is
12 × `(wat/load-file! "rules/<x>.wat")`.

```
$ wat --check <copy>/fmt/run-all.wat   (unconverted, keyword)                 rc=0
$ wat --check <copy>/fmt/run-all.wat   (converted, CURED binary)              rc=0
$ wat-baseline --check <copy>/fmt/run-all.wat (converted, PRE-CURE binary)    rc=0
```

⚠ **State plainly what this does and does not show.** It shows the cure does not break a
12-load, subdirectory-relative chain. It does **not** discriminate the two binaries — because
`run-all.wat` type-checks clean whether or not its loads happened. **That is the defect's whole
shape**, and it is exactly why §3.3's *missing-target* case is the discriminating evidence and this
one is not.

### 3.4 Delta — **18**, on the committed list file

`docs/arc/2026/06/251-types-as-forms/delta-sample-179.txt`, 179 paths, sha256
`33ede76cbb9b8809ecebc7656db3d4733769df54c2641b7e6d798773f1cbc3e5` — **used as-is, never rebuilt.**
All 179 present in the tree (`missing=0`). Copies staged under scratch, codemod run over all 179
(`179` conversion lines, rc 0).

| | orig clean | conv clean | **NEW (orig clean → conv broken)** | **recoveries (orig broken → conv clean)** |
|---|---|---|---|---|
| **pre-cure binary** | 160/179 | 143/179 | **18** | **1 — `wat-scripts/fmt/run-all.wat`** |
| **cured binary** | 160/179 | 142/179 | **18** | **0** |

Gate ✅ **18 ≤ 18**, and `160/179` reproduces 8d-ii-THIRD's pre-conversion baseline exactly.

⚠ **Disclosed:** the delta was measured with the build taken *before* §3.7's test fix. That fix is
entirely inside `#[cfg(test)]`, so the release binary's behaviour is unchanged — and the whole-tree
census **was** re-run on the post-fix binary (`.census/2026-09-22T05-21-06Z.txt`, 212 nonzero,
**0 rc changes** against the same baseline). I did not re-run the 179-file delta on the post-fix
binary; that is an argument from the change's scope, not a fourth measurement.

⭐⭐ **THE DELTA NUMBER IS IDENTICAL ACROSS THE CURE — and that is the brief's point, measured.**
The gate counts `clean → broken`. A forged green is `broken → clean`. **The number that would have
caught this class for eight stones is the RECOVERY count, and no stone has ever printed it.**
It is one line of the same join. Recommend it become a standing column.

### 3.5 Census — `no STOP-8`

⚠ **`[[DIFF LIKE AGAINST LIKE]]` — I could not use the census on disk.** The most recent artifact,
`.census/2026-09-22T03-34-24Z.txt`, has **343** failures against today's **212**: it was captured
under the 8d-ii **converted-stdlib** tree, and diffing my run against it would have shown **131
spurious "improvements"**. I redrew the baseline instead: stashed the diff, rebuilt, censused, popped.

```
baseline (src unmodified, b1b6c3c23): .census/2026-09-22T05-04-46Z.txt  files=2202   nonzero 212
this stone (final binary):            .census/2026-09-22T05-21-06Z.txt  files=2202   nonzero 212
$ ./scripts/replay/census.sh --diff .census/2026-09-22T05-04-46Z.txt .census/2026-09-22T05-21-06Z.txt
census-diff: no STOP-8                                                       (rc 0)
```

**Zero rc changes in either direction across all 2202 files** (`join … | awk '$2!=$3' | wc -l` = 0).
(An earlier census of the same tree before the §3.7 test fix, `.census/2026-09-22T05-03-14Z.txt`,
is identical — 212 nonzero, 0 changes.)

### 3.6 Clippy

`cargo clippy --all-targets --workspace -- -D warnings` → **exit 0**, no warnings
(re-run after the §3.7 test fix; exit 0 both times).

### 3.7 Floor — ⛔ **THE FIRST RUN WENT RED, AND IT WAS MINE. DISCLOSED WHOLE.**

#### Run 1 — `.floor/2026-09-22T05-10-59Z/` — **RED**

```
     Summary [ 319.320s] 5962 tests run: 5961 passed (9 slow), 1 failed, 22 skipped
exit=100
```

```
        FAIL [   0.111s] ( 161/5962) wat::lint no_loose_string_assert::tests_carry_no_loose_string_assert
```

**THE ARM, verbatim** (`.floor/2026-09-22T05-10-59Z/ARM.txt`, the failing test's whole stdout+stderr):

```
        FAIL [   0.111s] ( 161/5962) wat::lint no_loose_string_assert::tests_carry_no_loose_string_assert
  stdout ───

    running 1 test
    test no_loose_string_assert::tests_carry_no_loose_string_assert ... FAILED

    failures:

    failures:
        no_loose_string_assert::tests_carry_no_loose_string_assert

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 359 filtered out; finished in 0.11s

  stderr ───

    thread 'no_loose_string_assert::tests_carry_no_loose_string_assert' (2493639) panicked at /home/john/work/holon/wat-rs/tests/lint/no_loose_string_assert.rs:135:5:


    🔥🔥🔥 LOOSE STRING ASSERTIONS — 1 site(s) assert a value with contains/starts_with/
    ends_with where an exact `assert_eq!` belongs. A loose check passes on reordered fields,
    malformed maps, and appended garbage.

    THE FIX (RUBRIC: docs/CONVENTIONS.md § 'Test idioms' -> 'The .edn golden'): a deterministic
    STRUCTURED value goes in a co-located `<probe>__<label>.edn` golden, compared via
    `wat::assert_edn_eq!(actual, include_str!("...edn"))` (parses both sides, structure-exact) —
    capture the whole value, never guess. A scalar -> byte-identical `assert_eq!`. EXEMPT a
    legitimately-loose one (a value that varies per run: path/pid/hash/timestamp, or a targeted
    absence over a large output) with a per-site `// rune:lint(loose-assert) — <reason>`.

    Drive it to ZERO. Offenders:

    src/load/loader.rs:1404

    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**The arm that fired:** `tests/lint/no_loose_string_assert.rs:135` — the terminal
`assert!(violations.is_empty(), …)`. **The offender is one site, and it is mine:**
`src/load/loader.rs:1404`, **row 1 of this stone's own non-vacuity test**, which asserted
`sym_msg.contains("missing.wat")`.

⚠ **`[[NEVER READ A PIPED EXIT CODE]]` — I nearly missed this.** I launched the floor as
`./scripts/floor.sh 2>&1 | tail -30`; the harness reported **"exit code 0"**, which was `tail`'s.
The Summary line is what said `1 failed`. Exactly the trap `wat-rs/CLAUDE.md` names, walked into
by me, in this stone.

**The fix**, in the offending assertion only — an exact `assert_eq!` on the scalar per the rubric,
no exemption rune (the message is fully deterministic):

```rust
match &*sym_err.kind {
    LoadErrorKind::Fetch(fetch) => assert_eq!(
        fetch.to_string(),
        "load: file not found: missing.wat",
        "the fetch diagnostic must name the missing file, byte for byte"
    ),
    other => panic!(
        "symbol head must reach the FETCH, not be declined as 'not a load form'; got {other:?}"
    ),
}
```

⭐ **This made row 1 STRONGER, not weaker** — it now pins the whole message, not a substring.
`cargo nextest run --release -E 'test(tests_carry_no_loose_string_assert)'` → `1 passed`.

#### Run 2 — `.floor/2026-09-22T05-19-29Z/` — **GREEN**

```
     Summary [ 372.024s] 5962 tests run: 5962 passed (13 slow), 22 skipped
[floor] exit=0. Log kept at .floor/2026-09-22T05-19-29Z/ regardless — a green run is evidence too.
```

No `ARM.txt` written (the script only writes one on a red). Doctests: `exit=0` in both runs.
**The re-run followed the order the doctrine sets** — the log was captured before it was read, the
arm was copied whole, the exact assertion was named, and the cause was a located defect in my own
new test, not a mystery. **Nothing was re-run to see whether it would go green.**

### 3.8 ⛔ NOT ONE `.wat` CONVERTED

```
$ git status --porcelain
 M src/load/loader.rs
$ git status --porcelain -- '*.wat'      (empty)
```

Every conversion in this SCORE ran on copies under `/home/john/.claude/jobs/edaabf97/tmp/`.

---

## 4. SITES I DID NOT CURE — each with its reason

**(a) `load/loader.rs::parse_verify_algo` and `parse_payload_interface`** (the `:wat::verify::*`
markers inside `digest-load!` / `signed-load!`). The codemod converts these keywords too, and both
slots are keyword-only, so a converted verified-load raises `MalformedLoadForm`. **Not cured because
the failure is LOUD** — the stone's class is the silent one — **and because zero corpus files use any
verified-load form** (measured: 26 `.wat` files use a load form, **all 26 `load-file!`**, 0
`load-string!`/`digest-*`/`signed-*`). It is a one-line-each change through the same door when 8d-iii
needs it. ⚠ Note the *runtime* siblings at `runtime.rs:14272`/`14343` (`eval-file!` etc.) have the
same shape — cure them together or not at all.

**(b) `types/surface.rs:1003`** — `collect_user_type_paths` reads the post-`<-` child with
`Some(WatAST::Keyword(k,_))`. After conversion that child is a type **form**, not a keyword, so the
arm silently stops matching and the pass collects fewer user type paths. **I could not determine
whether that narrowing is observable** without a ruling on what `collect_user_type_paths` is
load-bearing for. **Named, not guessed.**

**(c) `runtime.rs:12674` (`head_of`, the session/REPL path) and `14723`/`19970` (the `def`-skip
guards in `run()`)** — these sit on paths that do **not** go through `build_env`'s normalize, so a
symbol head plausibly arrives. I did not build a session-level probe (the REPL/`run()` entry is a
different harness). **Reported as unmeasured, not as clean.**

**(d) The two mutation walls** (`freeze.rs:1894`, `runtime.rs:14442`). They are silently declined
but BACKSTOPPED (§1.5 A). Curing them is the same one-line door and would restore the *correct*
diagnostic. **Not cured because a security wall's population is the builder's call**, and widening
`is_mutation_form`'s input is a behaviour change I will not make unasked. ⛔ **Recommend it.**

---

## 5. WHAT MY GREEN CANNOT SEE

1. ⛔ **The delta gate is still blind to this class, and my green does not fix that.** 18 before,
   18 after. Only the recovery column moved (1 → 0). Any future forged green of this shape will pass
   the delta gate exactly as this one did for eight stones. **§3.4's recommendation is the cure and
   it is not in this stone.**
2. ⛔ **The corpus census is also blind.** `no STOP-8` only looks at `0 → non-zero`. A file that goes
   `non-zero → 0` because a wall stopped firing reads as an improvement. My own census diff shows
   0 changes either way *because nothing in the live tree is converted* — it would not have caught
   the defect if something had been.
3. **My unit tests use `InMemoryLoader` and `resolve_loads` directly.** They prove the form is
   recognised and the diagnostic is identical; they do **not** exercise `FsLoader`, `ScopedLoader`
   path-escape rules, or the digest/signed verification chain under a symbol head beyond "the head
   parsed".
4. **I proved the real-file case on TWO of the 26** (`examples/with-loader`, a 3-deep chain, as the
   brief specified; and `wat-scripts/fmt/` — 12 sibling loads through a `rules/` subdirectory, §3.3b).
   **The other 24 are unconverted and unprobed.** `wat-scripts/lib/wat-grep.wat` in particular is
   loaded by `../lib/` relative paths from several directories; the cure is path-agnostic, but that
   is an argument, not a measurement.
5. **The census's "UNREACHABLE" rows are pipeline-order arguments, not probes.** I read
   `freeze.rs`'s step list and `normalize.rs`'s contract and reasoned from them. If any pass is ever
   re-ordered relative to `normalize_symbol_refs` (step 7), **every row in §1.4 marked UNREACHABLE
   becomes live again, silently.** There is no gate on that ordering beyond
   `expand_runs_before_register_defines_phase_order`.
6. **"LOUD" means *a* diagnostic fired, not the *right* one.** Probe B's real cause is "the unquote
   escape was not recognised"; what the user sees is `unbound symbol: lit`. Probe A's real cause is
   "the mutation wall declined"; what the user sees is `unknown function`. Every LOUD row in §1.4
   will cost an 8d-iii debugging session pointed at the wrong subsystem.
7. **I did not probe the `:wat::eval-file!` / `:wat::eval-edn!` / `:wat::eval-ast!` *form* family**
   in `runtime.rs` (the runtime siblings of the load family, `runtime.rs:14028`+). They are the same
   shape and the same risk. **Unmeasured.**
8. Two new tests here are **`InMemoryLoader` green tests**. Per R59, a green number nothing depends
   on is a claim — but §3.3's pre-cure/post-cure binary pair *is* the failing-once proof
   (`[[A GUARD IS NOT A GUARD UNTIL IT HAS FAILED ONCE]]`): the same converted corpus file returns
   rc 0 on the baseline binary and rc 1 on the cured one.

---

## 6. WHAT THE BRIEF GOT WRONG

⭐ **(1) "All 4 load-form files in the 179 sample are CLEAN → CLEAN."** Measured: there is exactly
**ONE** load-form file in `delta-sample-179.txt` — `wat-scripts/fmt/run-all.wat` — and it is
**BROKEN → CLEAN** (pre-cure), i.e. the forged green itself, sitting inside the sample the whole time.
Both the count and the direction are wrong. The brief's conclusion ("no count run in eight stones
could have caught this") is **right**, but for a sharper reason than it gives: the file *was* in the
sample and *did* flip — the delta metric simply does not count flips in that direction.

⭐ **(2) The class is not "a form's HEAD."** `fix.wat`'s rewrite is position-independent over every
`::`-keyword leaf (§1.1). The census the brief asked for is therefore a *subset* of the exposure;
slot-1 names, `match`-arm patterns, rete fact-type keywords and `:wat::verify::` markers all convert
too. Three of the four highest-impact 8d-iii blockers I found (§1.5 B, C) are **not** head sites.

⚠ **(3) "~14 sit within three lines of a silent decline."** The three-line window is both too wide
(five of the hits are slot-1 reads or test asserts) and too narrow — it **missed `scan_for_setter`**,
the second real defect, which sits four lines from its decline **in the file the brief names**.

⚠ **(4) The census artifact the brief implies is current is not.** `.census/2026-09-22T03-34-24Z.txt`
was captured under the converted-stdlib tree (343 failures vs 212). A stone that diffed against it
would have reported 131 phantom improvements.

---

## 7. REPRODUCTION

```bash
cargo build --release
cargo test --release --lib load::loader::tests
cargo clippy --all-targets --workspace -- -D warnings
./scripts/replay/census.sh
./scripts/replay/census.sh --diff <BASELINE>.txt <CURR>.txt
./scripts/floor.sh
```

Scratch (not committed): `/home/john/.claude/jobs/edaabf97/tmp/` —
`wat-baseline` / `wat-cured` (the two binaries), `delta/{orig,conv,pre.tsv,post.tsv,post-baseline.tsv}`,
`scratch/{wl,probe,c,r,set}`. Committed: nothing but `src/load/loader.rs` and this SCORE.
