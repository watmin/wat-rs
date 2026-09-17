# SCORE — the boot cache elides what it already knows (Tier A)

**Struck 2026-09-17** against `DESIGN.md` / `EXPECTATIONS.md`, branch `sns-sqs`, from HEAD
`ee8205e57`. Nine tracked files changed (six `src/`+`build.rs`, three test-side goldens/gates the
red floor named), two new files. **`.config/nextest.toml` untouched; the check sweeps untouched;
no dependency added.**

---

## ⭐ THE RESULT

```
                        accounted   since-boot    wall     boot-cache-load
BASELINE  cache off        400.55      407.97    446.18          —
WARM      cache hit        179.14      185.31    220.86        36.26
COLD      cache hit        188.40      194.67    230.91        43.05      (page cache evicted)
FIRST     derive + store   432.71      464.37    501.62           —       (store 33.33)
```

*(ms, median of 9 separate processes each, `WAT_BOOT_CENSUS=phases`, one-line `hello.wat`.
Wall without the census: 446.55 baseline → 219.72 warm — the census's own overhead is not the
story.)*

**Accounted boot 400.55 → 179.14 ms warm — 221.41 ms saved, 55.3 % of accounted; 188.40 ms cold,
212.15 ms saved. External wall 446.18 → 220.86, 50.5 %.**

⚠ **The DESIGN's shape was 404.9 → ~154.7. This landed at 179.1, and the 24 ms gap is stated
rather than rounded away.** Three named causes, all deliberate:

| gap | ms | why |
|---|---:|---|
| the load costs more than the probe's lower bound | +20.6 | 36.26 warm, not 15.67 — the probe serialised function bodies only, reached no `TypeExpr`, no `MacroRegistry`, and **built no `HashMap` + `Arc<Function>` spine.** This one builds all of it. ⛔ **The DESIGN said not to inherit 15.67 ms as a target, and it was right to.** |
| `7.6 stdlib-runtime-defs-register` NOT elided | 9.0 | that pass EVALUATES stdlib `def` forms into `Value`s, and `Value` has handle-bearing variants. The forms are cached; the evaluation is re-run. See "what was deliberately left". |
| `6a-9` + `5 containment` + `6.97` NOT elided | 4.5 | all three walk the **combined** (stdlib + user) `TypeEnv`. Restricting them to stdlib types is the same class of soundness question as Tier B. |

⛔ **Tier B is untouched and visible in the warm table: `8b` 12.00 + `8c` 3.41 + `8d` 1.80 +
`8f` 108.94 = 126.15 ms still runs, every boot.** One floor, one claim.

### The warm phase table — every elided phase has ZERO hits

```
     3a-6b boot-cache-load                 37.05   20.21      1   ← the replacement
     3a   stdlib-parse                      0.00    0.00      0
     4    stdlib-defmacro-register          0.00    0.00      0
     4    kwargs-companions                 0.00    0.00      0
     4    stdlib-expand                     0.00    0.00      0   ← the 201 ms
     5    typeenv-with-builtins             0.00    0.00      0
     5    stdlib-types-register             0.00    0.00      0
     6    stdlib-defines-register           0.00    0.00      0
     6a   defclause-stub-preregister        0.00    0.00      0
     6b   stdlib-runtime-def-filter         0.00    0.00      0
     …
     8f   check:body-infer(ALL fns)       108.94   59.42      1   ← ⛔ TIER B, untouched
     ACCOUNTED (sum of leaves)            183.36
```

A phase with **0 hits did not run**. That is what elision looks like in the instrument the
previous stone built, and it is why this stone hand-timed nothing.

---

## THE FOUR DECISIONS (rows 4, 5, 10)

### 1 ⭑ — WHEN IS THE CACHE BUILT? **On first boot.** *(row 10)*

`build_env` writes the payload the first time it derives one for a given build fingerprint
(`7.9 boot-cache-store`, guarded by `should_store()`: enabled, fingerprint present, no payload
already on disk).

**Measured cost of the choice: `store` 33.33 ms, and a first-boot wall of 501.62 ms against a
446.18 ms baseline — a +55.4 ms one-time penalty, paid once per rebuild.** (33.3 ms of that is
the encode+write; the rest is the three snapshot clones, which show up as in-pipeline
unaccounted.) It is repaid by the second boot and 200× over by a floor.

**What the other would have cost.** A bake step at `cargo build` (a build script or a separate
bin) would have moved that 55 ms into the build and produced a deterministic artefact — but:

- it needs the **same fingerprint machinery anyway** (a baked payload must still be invalidated
  when the Rust that derives it changes), so the "deterministic" advantage is the fingerprint's,
  not the bake step's;
- a build script cannot call the crate it is building, so the bake would need a separate binary
  and a cargo ordering rule between it and 250 test binaries — build plumbing this stone was
  told not to grow;
- **and the payload is not a build product in the first place**: it depends on
  `installed_dep_sources()`, which a *host* may install at RUNTIME (`src/load/source.rs:71`).
  A bake step would produce a cache that is correct only for the no-dep-sources case and would
  have to fall through to deriving for every other — which is what first-boot already does, with
  no plumbing.

**Measured: the bake step's only real prize is the 33 ms first-run penalty, and it costs a second
binary plus a cargo ordering rule to win it.** Not taken.

### 2 ⭑⭑ — HOW IS STALENESS DETECTED? **A compile-time content fingerprint. DRIVEN.** *(row 4)*

Two axes, because there are two ways the answer can go out of date:

1. **The stdlib source changed** (`wat/**/*.wat`, baked by `include_str!`).
2. **The Rust that derives from it changed** — the expander, the registrars, the AST, the format
   itself. ⛔ **A source-only hash misses this, and missing it is the dangerous direction: a stale
   cache that is USED.** `src/hash.rs` (which the probe judged "looks right") hashes *canonical
   ASTs and source bytes*; it has no opinion about `expand.rs`, so it answers axis 1 only. It is
   not used here, and this is the reason.

So `build.rs` content-hashes **399 files / 11,611,964 bytes** (`wat/`, `src/`, `crates/`,
`Cargo.lock`, `Cargo.toml`, `build.rs`) into a 128-bit `WAT_BUILD_FINGERPRINT`, emitting one
`cargo:rerun-if-changed` **per file** (a directory-level line only catches add/delete on Linux,
never an edit). The runtime key is `fingerprint + FORMAT_VERSION + hash(installed dep sources)`,
and it is written **inside** the payload as well as into the filename, so a filename collision
cannot pass a foreign world off as this build's.

**DRIVEN, not asserted** — one comment appended to `wat/seq.wat`, rebuild:

```
before:  key 20d3f696a877aaea6dbfc9fd3beacdc1-v1-…   warm hit, stdlib-expand 0 hits, acc 187.12
after:   key cefc5d088a9ffd01c237de6a90b48f00-v1-…   ← the old payload REJECTED
         3a-6b boot-cache-load    0.05 ms    1 hit
         3a   stdlib-parse       32.87 ms    1 hit   ← derived
         4    stdlib-expand     201.07 ms    1 hit   ← derived
         7.9  boot-cache-store   34.71 ms    1 hit   ← new payload written
         program output: "hi"                        ← still correct
next:    warm hit on the NEW key, acc 182.61
```

**Detection cost: 0.05 ms on a miss** (a key format + one `open` that returns ENOENT) — the
fingerprint itself is `option_env!`, a compile-time constant, so it costs nothing at runtime, and
`installed_dep_sources()` is empty in every boot measured here. **Four orders of magnitude cheaper
than the 221 ms it protects.** On a hit, verification is the header + key compare + a 128-bit
checksum over the payload, inside the 36.26 ms.

### 3 ⭑⭑ — ABSENT / STALE / CORRUPT ⇒ DERIVE, SILENTLY. **All three DRIVEN.** *(row 5)*

`tests/diagnostics/boot_cache_fixpoint.rs::absent_truncated_and_corrupt_payloads_all_fall_back_to_deriving`
drives four damage modes against a real `startup_bare()`, and asserts both that the decoder
REFUSES and that the boot still produces the identical world (function count vs a cache-off boot):

| drive | result |
|---|---|
| **delete** the payload | `load()` → `None`; boot derives; world identical |
| **truncate** to 0, 1, 7, 8, 32, ⅓, ½, len−17, len−1 bytes | every one → `None`; boot at ½ derives; world identical |
| **corrupt one byte** at ¼, ½, len−40 | every one → `None` (the 128-bit checksum); boot derives; world identical |
| **foreign key** under the right filename | → `None`; boot derives; world identical |

⛔ **There is no `unwrap`, no `panic`, no `expect` on cache content anywhere in `boot_cache.rs`,
and every decoder read is bounds-checked** — a count larger than the bytes remaining is refused
before it can drive a `Vec::with_capacity`, an unknown tag is `None`, a non-UTF-8 string is
`None`, an `Identifier` name carrying U+0001 is `None` (that one would otherwise trip
`Identifier::bare`'s `debug_assert!`). The body must also end exactly where the string table
begins: a payload that decodes but leaves bytes over is one this decoder did not understand.

**A boot that fails because a cache file is bad is a worse product than a slow boot, so the only
way this module can affect a boot is by making it faster.**

### 4 ⭑ — FORMAT: **hand-rolled. NO DEPENDENCY ADDED.**

`Cargo.toml` and `Cargo.lock` are unmodified. LEB128 varints, zig-zag signed, one recursive walk,
one `Vec<u8>`, two string tables in the trailer, a 128-bit checksum.

The one departure from the probe's shape is **string interning**, and it is here on a
measurement, not on taste: a stdlib world is ~200 k AST nodes over ~15 k distinct names
(`:wat::core::defn` alone appears thousands of times).

| | payload | warm load |
|---|---:|---:|
| probe-shaped (length-prefixed UTF-8 at every node) | 5,075,554 B | 46.23 ms |
| interned (one index per string) | **2,853,591 B** | **36.26 ms** |

**−43.8 % bytes, −21.6 % load.** A serde/bincode dependency was never reached for, and the
measurement above is why it would have had to justify itself against: the remaining 36 ms is
building 200 k `WatAST` nodes and a 652-entry `Arc<Function>` spine, which no wire format makes
cheaper.

---

## ⛔⛔ ROW 2 ⭑⭑ — SPANS SURVIVE, AND THE OBVIOUS TEST WOULD HAVE BEEN WORTHLESS

`crates/wat-reader/src/span.rs:137` — verbatim:

```rust
impl PartialEq for Span {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}
```

— and `Hash` at `:146` is a no-op. **`assert_eq!(original, decoded)` therefore PASSES ON A DECODER
THAT DROPS EVERY SPAN.** A cache that silently discarded source locations would have shipped green
and destroyed every diagnostic in the language.

**The test used: bytes.** `the_real_payload_is_a_byte_exact_fixpoint_so_no_span_can_go_missing`
asserts the strong form the real payload admits —

```
file_bytes == encode(decode(file_bytes))          2,853,591 bytes, byte for byte
encode(decode(encode(x))) == encode(x)            the fixpoint the DESIGN named
```

— over the whole 2.85 MB payload of a real `startup_bare()`. A decoder that dropped a span, or
interned two distinct files to one, or lost a scope's sharing, cannot land back on those exact
bytes. The payload is also asserted non-trivial (>100 macros, >100 types, >500 functions,
>1 MB) — a fixpoint over nothing proves nothing.

**And the variants the stdlib never exercises** are covered separately, because the stdlib payload
reaches only 9 of `WatAST`'s 14 variants and zero hygiene scopes:
`the_variants_the_stdlib_never_exercises_round_trip_and_scopes_come_back_shared` carries
`RationalLit(−22/7)`, `BigIntLit(2¹²⁷−1)`, `CharLit('😀')`, `CharLit('\n')`, `Map`, `Set`, an empty
`Vector`, `i64::MIN`/`MAX`, `-0.0`, `f64::MIN_POSITIVE`, `NaN`, two span files, a point span
(`end: None`) beside a range span, and three identifiers — one scoped, one sharing that scope plus
another, one bare. It asserts the byte fixpoint AND, by hand, that `-0.0` kept its sign, that
`NaN` survived, that the point span did not grow an `end`, and that the range span's `end` is
exactly `(9, 9)` — the field a lazy decoder drops.

## ROW 7 ⭑ — `ScopeId` IS RE-MINTED, AND THE SHARING SURVIVES

Scopes are written as **dense first-encounter indices** and re-minted at decode through
`fresh_scope()`. The same test asserts all three halves:

```rust
assert_eq!(a.scopes().len(), 1);  assert_eq!(b.scopes().len(), 2);  assert_eq!(c.scopes().len(), 0);
assert!(b.scopes().contains(&a0), "the SHARED scope did not come back shared");
assert_ne!(a0, shared, "a raw ScopeId was restored by value — it must be RE-MINTED");
```

This is `Function::params`' own doc executed rather than quoted: *"an exec'd child restarts
`fresh_scope()` at 1, so imported scopes must be REMAPPED."*

## ROW 6 ⭑ — NATIVES ARE RE-REGISTERED, NOT SERIALISED

⭐ **They are not serialised because there is nothing to serialise, and the payload is checked
rather than trusted.** `store()` runs `is_representable()` first — ⛔ **ENCODE-OR-REFUSE**: the
payload is written ONLY if every function is `FunctionBody::Wat` with `closed_env: None`, and the
`SymbolTable` carries no `unit_variants`, no `runtime_def_values`, and none of the eight capability
carriers (`encoding_ctx`, `source_loader`, `macro_registry`, `types`, `primed_stdio`, the two sigma
fns, `outer_symbols`). Anything else and **nothing is written and every boot derives** — which is
exactly today's behaviour. A lossy cache is worse than no cache, and this is the rung above hoping.

`no_native_body_is_ever_written_into_the_payload` asserts it on the real payload.

## ROW 3 ⭑⭑ — BEHAVIOUR IDENTICAL: THE CIRCUIT, BYTE-IDENTICAL

Run with the cache **ON** (the default path), from `~/work/BREADCRUMB.md`'s one-liner:

```
"timeout=yes;discarded=yes;redial=Connected;retry-on=fresh"
"n=2000;m=4;j=3;total=8000;distinct=8000;dup=0;workers=12;empty=1;seen-recorded=8000;
 seen-skipped=0;check-exhausted=0;mark-exhausted=0;ack-retries=0;ack-exhausted=0"
rc=0
```

Both lines are the ones the EXPECTATIONS pins, unchanged.

And `the_cache_can_be_switched_off_and_the_world_is_the_same` drives the A/B door directly:
`WAT_BOOT_CACHE=off` neither reads nor writes, and produces a world with the same function count.

---

## ⛔ THE SOUNDNESS QUESTION NOBODY HAD NAMED, AND HOW IT IS ANSWERED

**The DESIGN did not anticipate this and it is the most important thing this stone found.**

`build_env` registers **USER defmacros and USER acronyms BEFORE it expands the stdlib**
(`src/freeze/env.rs`: `register_defmacros(user)` → `preregister_acronyms(user)` →
`expand_all_with(stdlib)`). So the 201 ms stdlib expansion is, on its face, a function of user
state — the *same shape* of question Tier B is gated on, sitting inside Tier A.

⛔ **It is not answered with an argument. It is answered with a witness.**

- `MacroRegistry::contains` / `::get` are the ONE door every macro-head probe goes through. While
  the snapshot's stdlib expansion runs — and only then — every probed name that is **not a
  reserved prefix** is recorded into the payload. A user macro can carry no other kind of name
  (the reserved-prefix gate refuses the rest), so **a user macro whose name is absent from that
  witness cannot have been consulted**, and the cache is refused when any user macro name is in
  it. The check is a `HashSet` intersection over the names a user actually declared.
- The acronym registry consulted at expand time was **empty** when the snapshot was taken, so the
  cache is refused outright for any program containing `declare-acronyms`. Conservative rather
  than precise, deliberately: it needs no argument at all, and it costs a boot only for programs
  that use the form.

The gate's inputs are derived by an **over-approximating** walk of the user's raw forms (every
`defmacro` name anywhere in the tree, including inside quoted templates that cannot actually
register). Over-approximation is the safe direction: it can refuse a cache that would have been
fine; it cannot accept one that would not.

**The names SUBTRACTED from the payload, by contrast, are exact** — a before/after difference of
`MacroRegistry::name_set()` across user registration. ⚠ This is the bug that nearly shipped: the
first version subtracted the *pre*-registration set (the stdlib's own names), which cached a
registry holding only the USER's macros. It was caught in the first smoke test, by the program
printing a **type-check error instead of `"hi"`** — which is the whole reason a behaviour drive
exists and is why one was run before anything else.

## ROW 12 ⭑ — WHAT GOT SLOWER. THE BILL.

| cost | measured | notes |
|---|---:|---|
| **first boot after any rebuild** | **+55.4 ms wall** (446.18 → 501.62) | `store` 33.33 ms + three snapshot clones. Paid once per fingerprint. |
| **binary size** | **+1,658,160 B, +8.4 %** (19,657,512 → 21,315,672) | `git stash -u` / build HEAD / `stash pop`, then rebuilt and verified byte-identical to the pre-stash binary. Larger than its 1,690 source lines suggest — the recursive encoder/decoder inline heavily. **This is the single biggest bill and it is not small.** |
| **build-script time** | **+24–54 ms** per build-script run | hashing 399 files / 11.6 MB. Against a 36.8 s incremental release rebuild: **0.07–0.15 %.** |
| **build-script re-runs** | one `rerun-if-changed` per file, 399 lines | build.rs now re-runs on any `src/`/`wat/`/`crates/` edit. Such an edit already recompiles the crate, so the marginal cost is the 24–54 ms above. |
| **disk** | **2,853,591 B per fingerprint** in `$XDG_CACHE_HOME`/`~/.cache/wat-boot-cache` | ONE file per build, shared by all 250 test binaries (the fingerprint is a compile-time constant, not the executable's identity — that was the alternative, and it would have cost ~250 × 2.85 MB). Nothing prunes old fingerprints today — see "could not drive". |
| **peak memory at store** | not measured | the snapshot clones `types` + `symbols` + the macro registry before the user's definitions land. See "could not drive". |

**Nothing got slower on the warm path.** `WAT_BOOT_CACHE=off` restores the old behaviour exactly
and is the A/B door every number above was taken through.

## ROW 9 — clippy + `--no-run`

```
$ cargo clippy --release --workspace --all-targets
    Finished `release` profile [optimized] target(s) in 16.87s
$ cargo nextest run --release --no-run
    Finished `release` profile [optimized] target(s) in 0.10s
```

Zero warnings, zero errors. The workspace denies `clippy::all` at the manifest, so a finding would
have been a build failure, not advice.

## ROW 11 ⛔ — TIER B UNTOUCHED, `.config/nextest.toml` UNTOUCHED

```
$ sha256sum .config/nextest.toml
706f59851bc01cca13ad37f2c0d07abe68a7133d362418ffeb77da21d22a5372  .config/nextest.toml
$ git show HEAD:.config/nextest.toml | sha256sum
706f59851bc01cca13ad37f2c0d07abe68a7133d362418ffeb77da21d22a5372  -
```

Identical. `src/check.rs` is **not in the diff** — `git diff --stat` is `build.rs`,
`src/freeze.rs`, `src/freeze/census.rs`, `src/freeze/env.rs`, `src/macros/registry.rs`,
`src/types.rs`, plus three test-side files the red floor named and two new files — 348
insertions, 31 deletions in total. The four `ALL fns` sweeps still
cost 126.15 ms in the warm table above, which is the positive proof that they still run.

The parked queue promotion (`queue-promotion-blocked-on-startup-cost`, `6136d144f`) was **not
un-parked and not touched**. It got 221 ms cheaper per startup; whether that is enough is its own
stone's question.

---

## WHAT IS ACTUALLY IN THE PAYLOAD

```
2,853,591 bytes · macros + surface-forms + types + builtin-names + 652 functions
                + the stdlib runtime-def residue + the probe witness
```

⭐ **652 functions, not 2,177 — and the difference IS the design.** The snapshot is taken after
`register_stdlib_defines` and **before `6a-9 auto-method-codegen`**, which mints the other ~1,525
(struct ctors, aggregate accessors, enum variant ctors, newtype methods, type predicates) by
walking the **combined** `TypeEnv`. A user `defstruct` changes what that pass emits, so a snapshot
taken before user types exist cannot stand in for it. That is 3.81 ms left deliberately on the
floor, stated rather than claimed.

**What the cache holds:** `MacroRegistry` (post stdlib-expansion, minus the user's own
registrations) · `TypeEnv` (`with_builtins()` + stdlib types) · `SymbolTable` (stdlib defines +
defclause stubs, plus `binding_metadata` and `acronym_registry`) · the `defclause`/`extend-type`/
`def` residue step 7.6 re-registers · the probe witness.

**What it does NOT hold, and why:** no `Value` (that is step 7.6's 9 ms, and `Value` has
handle-bearing variants — `Sender`, `RustOpaque`, `Arc<dyn WatReader>`; the 56 entries measured are
all pure, but a serialiser over that type is a trap-door waiting for the 57th) · no
`unit_variants` · no capability carrier (the loader comes from the caller, `EncodingCtx` from
`Config`, the sigma fns from their unit structs, natives from the existing `IntrinsicRegistry`).

## WHAT SURPRISED ME

⭐ **The soundness question Tier B was supposed to own has a sibling inside Tier A, and nobody had
named it.** The FINDING's whole framing was "Tier A needs no soundness argument". It does:
`register_defmacros(user)` runs BEFORE `expand_all(stdlib)`, so the 201 ms phase formally depends
on user state. The DESIGN, the FINDING and the EXPECTATIONS are all silent on it. What saves it is
that the dependency is *witnessable* — `MacroRegistry::contains` is a single door — so the answer
is a recorded fact rather than an argument. ⛔ **Tier B has no equivalent single door** (four
sweeps, `env.defined_values` mutated mid-pass), which is a real asymmetry the Tier B spike should
inherit: *Tier A got to measure its dependency; Tier B will have to prove its one.*

⭐ **The first version was WRONG, and the byte-level tests would not have caught it.** Subtracting
the pre-registration name set instead of the difference cached a registry holding only the *user's*
macros. Every round-trip property still held — the payload was a perfect fixpoint of the wrong
state. **What caught it was running a one-line program and looking at its output**: `"hi"` on the
cold boot, a `MalformedForm` about `:wat::core::nil` on the warm one. A cache is a *state* bug
class, and state bugs are invisible to format tests.

⭐ **String interning was worth 2.2 MB and 10 ms, and the probe's own note predicted it** ("a naive
byte-at-a-time decoder that allocates a fresh `String` for every one of 36,352 keywords"). 15 k
distinct names carry 200 k nodes. It is the one place where the format mattered at all; everything
else in the 36 ms is building the tree, which no format changes.

⭐ **`cargo` gives you no build identity for free.** The obvious keys are all wrong: the stdlib
source hash misses expander changes; `CARGO_PKG_VERSION` misses everything; the executable's
mtime+size would have minted ~250 separate 2.85 MB payloads for one floor. A content fingerprint
computed in `build.rs` with a per-file `rerun-if-changed` is the only one of the four that is both
correct and shared, and it costs 24–54 ms.

## WHAT I COULD NOT DRIVE

An honest ABSENT beats a plausible claim.

1. **Cache eviction / pruning.** Nothing deletes an old fingerprint's payload. A developer who
   rebuilds fifty times accumulates fifty 2.85 MB files (~143 MB) in `~/.cache/wat-boot-cache`.
   **Not built, not measured.** The fix is small (an LRU sweep, or a `.watbc` count cap at store
   time) and it is deliberately not in this stone.
2. **Peak RSS at store time.** `store` clones `types`, `symbols` and the macro registry. The clones
   are shallow where it matters (`Arc<Function>`) but `TypeEnv` and `MacroDef` are deep. **Not
   measured.**
3. **The concurrent first-boot storm.** With a cold cache, N nextest processes all derive and all
   encode before one wins the `rename`. It is correct (unique temp file + atomic rename, and
   `path.exists()` short-circuits most of them) but the aggregate cost of the first floor after a
   rebuild was **not isolated** — the floor below ran against a pre-warmed cache, which is the
   steady state, and that is stated rather than hidden.
4. **Whether the gate ever actually fires in this corpus.** The probe witness is recorded and the
   intersection is checked, but no program was constructed that *trips* it (a user macro whose name
   the stdlib expansion probes). **The refusal path is exercised only by the acronym arm and by the
   staleness/corruption drives.**
5. **A user program that introduces hygiene scopes.** The FINDING measured 0 of 21,925 symbols
   carrying scopes after a bare boot and could not explain it; that is unchanged here. The re-mint
   is proven by the synthetic test, not by a world that needed it.

---

## ⛔ THE FIRST FLOOR WENT RED — 6 FAILURES, EVERY ARM NAMED, NOTHING RE-RUN BEFORE IT WAS FIXED

```
     Summary [ 257.713s] 5291 tests run: 5285 passed, 6 failed, 22 skipped
     .floor/2026-09-17T07-46-54Z/  ·  exit=100  ·  ARM.txt WRITTEN (4,840 lines)
```

⛔ **Reported, not dismissed.** The log was captured whole before it was read, the failing run was
not re-run, and each arm is named below with the line it fired on. None of the six is a flake and
none was pre-blessed; all six are **goldens and gates that encode a fact this change legitimately
moved**, plus one that is a real finding about the instrument.

| # | arm | why it fired | disposition |
|---|---|---|---|
| 1–3 | `freeze::census::tests::report_text_{ranks_manifest_entries…, shouts_about_a_phase…, sums_leaves_and_reconciles…}` at `src/freeze/census.rs:766/775/795` | the three verbatim report goldens; `PHASE_ORDER` gained `3a-6b boot-cache-load` and `7.9 boot-cache-store`, so the rendered table gained two rows | goldens updated (two rows inserted at their pipeline positions) |
| 4 | `wat::diagnostics boot_census::an_armed_boot_census_names_the_manifest_entries_it_spent_time_in` — `files >= 50` got **1** | ⭐ **a real finding, below** | the test disables the cache; the report now SAYS when it was blinded |
| 5 | `wat::macros probe_hygiene_scopes_reader_gate::only_sanctioned_files_read_identifier_scopes` at `tests/macros/probe_hygiene_scopes_reader_gate.rs:119` — *"these src/ files read `Identifier::scopes()` but are NOT sanctioned scope-readers: `["freeze/boot_cache.rs"]`"* | the hygiene-class gate, working exactly as designed | `freeze/boot_cache.rs` added to `SANCTIONED_SCOPES_READERS` **with its reason and its backing probe**, as the gate's own message demands |
| 6 | `wat::process probe_supervisor_select_lost::select_prime_yields_lost_when_process_child_crashes` at `:202` — golden frame `{:file "src/freeze.rs" :line 1541}`, actual `1547` | ⭐ **the fragility the census stone named**: a golden pins a `rust_caller_span!()` line number in `src/freeze.rs`, and this change added 6 lines above it | golden updated 1541 → 1547 |

### ⭐ ARM 4 IS A FINDING, NOT A TEST FIX: **the cache BLINDS `WAT_BOOT_CENSUS=files`**

`file_guard` opens inside `stdlib_forms()` — *"the last place in the whole boot that knows which
manifest entry a form came from"*, in `census.rs`'s own words. **The boot cache elides
`stdlib_forms()` entirely**, so a warm boot has no stdlib manifest entry to attribute at all: the
armed run in the ARM attributed exactly **1** file (`wat-scripts/probes/arc-170/probe-trivial.wat`,
0.44 ms) where the test requires ≥ 50.

A near-empty per-entry table with no explanation is the shape that gets read as *"the stdlib is
free"*. It is not free; it was **elided**. So two things changed, and neither is a loosened
assertion:

1. **`report_text` now says so.** When `3a-6b boot-cache-load` has hits, the `files` footer prints
   `⛔ THE BOOT CACHE ANSWERED, so stdlib-parse never ran and NO stdlib manifest entry could be
   attributed … Re-run with WAT_BOOT_CACHE=off`.
2. **The test sets `WAT_BOOT_CACHE=off`**, because its claim is about the *derivation*. Measuring a
   cached boot and calling it a derivation census would be measuring A and claiming B.

⚠ **This is the cache's second-order cost and it belongs in row 12 as much as the binary size:
one of this campaign's own instruments stops working on a warm cache unless it is told not to.**

---

## ROW 8 ⭑⭑ — THE FLOOR, AFTER THE FIXES

**Summary line, verbatim** — from the tree EXACTLY as it stands (the earlier green was one comment
behind, so it was re-run rather than quoted):

```
     Summary [ 257.722s] 5291 tests run: 5291 passed, 22 skipped
```

- log: `.floor/2026-09-17T08-10-53Z/raw.log` · `clean.log` (untruncated, ANSI-stripped, kept
  before reading) — `exit=0. Log kept at .floor/2026-09-17T08-10-53Z/ regardless — a green run is
  evidence too.`
- **`ARM.txt`: NOT written** — `scripts/floor.sh` writes one only on a red. The directory holds
  exactly `raw.log` and `clean.log`.
- 5291 = the 5286 of `9722bea39` plus this stone's five new tests.
- corroborated by `.floor/2026-09-17T07-59-38Z/` — `Summary [ 256.051s] 5291 tests run: 5291
  passed, 22 skipped`, also no `ARM.txt`.
- ⛔ The RED run before both is `.floor/2026-09-17T07-46-54Z/` and it DOES hold an `ARM.txt`
  (4,840 lines). It is reported in full above. It was not re-run until every arm was fixed.

### ⭐⭐ AND THE FLOOR ITSELF IS 2.1× FASTER — THE UNASKED-FOR RESULT

Five green floors on this box today, before this stone, against the two after it:

```
06-03-42Z   558.656s   5284 tests   (9 slow)
06-17-23Z   546.721s   5284 tests   (9 slow)
06-49-29Z   557.959s   5286 tests   (9 slow)      ← the feasibility probe's floor
07-00-56Z   549.165s   5286 tests   (9 slow)      ← the orchestrator's regrade
───────────────────────────────────────────
07-46-54Z   257.713s   5291 tests   (RED, 6)      ← this stone, cache warm
07-59-38Z   256.051s   5291 tests   GREEN         ← this stone, cache warm
08-10-53Z   257.722s   5291 tests   GREEN         ← the final tree
```

**546.7–558.7 s → 256.1 s: the whole release floor runs in 46 % of the time, with five MORE tests
— and `(9 slow)` is gone.** ⛔ Stated with its caveat: the floor is ~250 binaries each paying one
boot, so it is the most cache-favourable workload there is, and the comparison is the same box on
the same day but not an interleaved A/B. It is reported because 290 seconds off every floor is the
kind of number that changes what a day costs, and because trap-door 3 asked exactly this — the
loader gate's 687 startups did get cheaper.

⛔ **The parked queue promotion (`queue-promotion-blocked-on-startup-cost`) was NOT un-parked.**
Its blocker moved by 221 ms per startup; whether that is enough is its stone's question, not this
one's.

---

## ⭐ THE GATE IS NOT VACUOUS — THREE NAMES, MEASURED

The probe witness recorded for this stdlib is **not empty**:

```
payload 2,853,591 bytes · macros 245 · surface-forms 16 · types 431 · builtin-names 37
                        · functions 652 · runtime-def forms 717
                        · probe witness 3  [":repl::eval-and-loop", ":repl::eval-form", ":repl::turn"]
```

**Three non-reserved macro-head names are probed while the stdlib expands** — all in `:repl::`,
which is *not* a reserved prefix, so a user program declaring `(defmacro :repl::turn …)` **would**
have changed the cached expansion. The cache refuses itself for exactly those three names and no
others.

⭐ **This is why the gate is a witness and not an argument.** The tidy argument — *"stdlib forms
only ever call `:wat::` heads, so user macros cannot reach them"* — is FALSE for this corpus, and
it is false by three names nobody would have guessed. Had this shipped on the argument, the bug
would have been a user macro silently changing what the stdlib expanded to — or, with the cache,
silently *not* changing it.

---

## THE VERDICT

| # | row | |
|---|---|---|
| 1 | ⭑⭑ boot drops, census, cold AND warm | ✅ 400.55 → **179.14 warm / 188.40 cold** accounted; wall 446.18 → 220.86. 24 ms above the DESIGN's shape, with the 24 ms itemised. |
| 2 | ⭑⭑ spans survive | ✅ byte-exact fixpoint **and** `file_bytes == encode(decode(file_bytes))` over 2.85 MB; `assert_eq!` never used. |
| 3 | ⭑⭑ behaviour identical | ✅ `distinct=8000;dup=0` · `timeout=yes;discarded=yes;redial=Connected;retry-on=fresh`, cache ON. |
| 4 | ⭑⭑ staleness DETECTED | ✅ driven by mutating `wat/seq.wat`; key changed, payload rejected in **0.05 ms**, boot correct. |
| 5 | ⭑⭑ absent / corrupt falls back | ✅ delete · 9 truncations · 3 byte-flips · a foreign key — all refused, all boot correctly. |
| 6 | ⭑ natives re-registered | ✅ encode-or-refuse; 0 native bodies; asserted on the real payload. |
| 7 | ⭑ `ScopeId` re-minted, sharing preserved | ✅ asserted three ways, plus the hygiene-class gate now sanctions the reader with its reason. |
| 8 | ⭑⭑ floor | ✅ `Summary [ 257.722s] 5291 tests run: 5291 passed, 22 skipped` · `.floor/2026-09-17T08-10-53Z/` · no `ARM.txt`. ⛔ The prior RED (6 failures) is reported in full, with every arm named. |
| 9 | clippy + `--no-run` | ✅ both clean. |
| 10 | ⭑ build-time vs first-run | ✅ first-run, measured at +55.4 ms, with what the bake step would have cost and why it is not merely a trade. |
| 11 | ⛔ Tier B / nextest.toml untouched | ✅ sha256 identical; `src/check.rs` not in the diff; the four sweeps still cost 126.15 ms. |
| 12 | ⭑ what got slower | ✅ +1.66 MB binary, +55 ms first boot, +24–54 ms build script, 2.85 MB per fingerprint with **no pruning**, and — the one nobody would have listed — **`WAT_BOOT_CENSUS=files` is blinded by a warm cache** and now says so. |

---

## THE CIRCUIT, RE-RUN ON THE FINAL BINARY — AND ONE HONEST WOBBLE

```
"timeout=yes;discarded=yes;redial=Connected;retry-on=fresh"
"n=2000;m=4;j=3;total=8000;distinct=8000;dup=0;workers=12;empty=1;seen-recorded=8000;
 seen-skipped=0;check-exhausted=0;mark-exhausted=0;ack-retries=2;ack-exhausted=0"
```

Both **pinned** facts are identical to the first run and to the EXPECTATIONS: the whole
`timeout=…;retry-on=fresh` line, and `distinct=8000;dup=0`.

⚠ **One field moved between my own two runs and it is reported rather than rounded off:
`ack-retries` was `0` on the first, `2` on the second** — both with the cache ON, both
`ack-exhausted=0`, both `dup=0`. It is a retry COUNT on the ack path, not pinned by any
EXPECTATIONS row in this campaign, and it is the field that moves with box load. **I did not
bisect it, so I am not asserting it is load** — it is stated so the next reader has it.

---

## THE DIFF

```
 build.rs                                           |  76 ++++++++++
 src/freeze.rs                                      |   6 +
 src/freeze/census.rs                               |  27 ++++
 src/freeze/env.rs                                  | 162 +++++++++++++++++----
 src/macros/registry.rs                             |  51 +++++++
 src/types.rs                                       |  27 ++++
 tests/diagnostics/boot_census.rs                   |  10 ++
 tests/macros/probe_hygiene_scopes_reader_gate.rs   |  18 +++
 ...robe_supervisor_select_lost__process_panics.edn |   2 +-
 9 files changed, 348 insertions(+), 31 deletions(-)

 src/freeze/boot_cache.rs                  NEW  1,690 lines (≈ 45 % of it comment)
 tests/diagnostics/boot_cache_fixpoint.rs  NEW    248 lines
```

⛔ `src/check.rs` is not in it. `.config/nextest.toml` is not in it. `Cargo.toml` / `Cargo.lock`
are not in it — **no dependency was added.**

---

## ⭑ REGRADED BY THE ORCHESTRATOR ON HIS OWN RUNS, 2026-09-17 — plus one DEFECT FOUND AND FIXED

```
floor (healthy cache)   Summary [ 252.076s] 5291 tests run: 5291 passed, 22 skipped
floor (after the fix)   Summary [ 257.087s] 5291 tests run: 5291 passed, 22 skipped
                        .floor/2026-09-17T08-36-48Z/ · exit=0 · NO ARM.txt · "(9 slow)" GONE
baseline, same box/day  546.7 – 558.7 s across five green runs                ⇒ 2.2×
boot, mine              cache off 0.448 / 0.487 / 0.448 s → on 0.220 / 0.220 / 0.223 s
circuit ×3              distinct=8000;dup=0 · retry-on=fresh · 23–24 s
scope                   src/check.rs, .config/nextest.toml, Cargo.toml, Cargo.lock ALL unchanged
```

## ⛔⛔ THE DEFECT GRADING FOUND: A REFUSED CACHE IS NEVER HEALED

The stone drove delete · 9 truncations · 3 byte-flips · a foreign key, and every one **refuses and boots
to an identical world** — which is the *correctness* question, and it passes. **What no drive asked is
whether the cache RECOVERS.** It did not: a refused payload was re-read and re-refused on every
subsequent boot, so **one bad byte meant full-price startup FOREVER — silently, correctly, invisibly.**

★ **It was measured by accident, and that is the point.** My own first floor came back **555.936 s** and
I nearly reported *"the 2.1× does not reproduce."* It did not reproduce because **I had poisoned the
cache two commands earlier with my own corruption test**. Removing that one file: **0.509 s → 0.223 s**,
and the next floor was **252.076 s**. ⛔ A boot that is correct and silently 2× slow forever is exactly
the failure shape this entire thread exists to end — and it would have shipped.

**Fixed** in `boot_cache::load()`: a refused payload is **removed**, and the derive path that follows
re-stores it. Removal rather than rewrite, because `store` already runs next and a second write would
race 200 nextest processes. Failure to remove is ignored, for the same reason a failure to write is: a
diagnostic must never turn a slow boot into a failed one.

**Driven, correctly, after two invalid attempts:** corrupt the live entry → **0.518 s** (reject → delete
→ derive → store) → **0.230 / 0.222 s**. Pre-fix: slow, slow, slow.

⚠ **And a killed process makes this reachable in normal use.** `store` writes to a unique temp then
renames, so a half-written payload is never visible — but a file truncated by a full disk, or written by
an older binary whose key collides, lands in the same state.

### ⭑ TWO OF MY OWN DRIVES WERE INVALID BEFORE THEY WERE VALID

1. `ls -S` picked the **largest** cache file, not the one the program keys to — so three "fast" boots
   proved nothing.
2. After rebuilding with the fix, `build.rs` produced a **new fingerprint**, so the entries I was
   corrupting belonged to the **previous binary** and were simply unused.

Both times the boots came back fast and both times the honest reading was *"my fixture is wrong"*, not
*"the fix works"*. ★ **Three times today a green result meant my instrument had missed, not that the
thing under test was sound.**

### The no-pruning bill, now observed rather than disclosed

The third fingerprint appeared the moment the binary was rebuilt: **3 entries ≈ 8.6 MB** after a handful
of builds, none ever removed. On a dev box that rebuilds often this grows without bound. The stone
disclosed it (row 12); grading measured it.

### Accepted as reported, and worth more than the speedup

1. ⭐ **A soundness question nobody had named, answered with a WITNESS rather than an argument.**
   `register_defmacros(user)` runs *before* `expand_all(stdlib)`. The tidy argument — *"the stdlib only
   calls `:wat::` heads"* — is **false for this corpus**, and the witness proves it:
   `[":repl::eval-and-loop", ":repl::eval-form", ":repl::turn"]`. Three names nobody would have guessed.
2. ⭐ **The first implementation was wrong and NO format test could have caught it.** It cached a
   registry holding only the *user's* macros — **a perfect fixpoint of the wrong state**. What caught it
   was running a one-line program: `"hi"` cold, `MalformedForm` warm. The same lesson as the
   one-ring-per-thread spin: only an end-to-end run sees it.
3. ⭐ **The cache blinds one of our own instruments.** `WAT_BOOT_CENSUS=files` hooks a function the cache
   elides, so an armed run attributed **1** file instead of ≥50. It did not paper over it — it made the
   report *say so* and pointed the test at the derivation path.
4. **The `ack-retries` 0 → 2 drift it honestly refused to claim did NOT reproduce** in three of my runs
   (all `ack-retries=0`, `ack-exhausted=0`). Consistent with an unpinned counter under load; neither of
   us bisected it, and it stays on the record.
