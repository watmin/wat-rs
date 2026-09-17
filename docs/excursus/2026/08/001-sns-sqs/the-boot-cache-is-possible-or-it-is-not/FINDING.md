# FINDING — the boot cache is possible, or it is not

**Struck 2026-09-17** against `DESIGN.md` / `EXPECTATIONS.md`, branch `sns-sqs`, from HEAD
`2e27b9454`. Instrument: `tests/diagnostics/boot_cache_roundtrip.rs` — **one new, untracked test
file. `git diff` is EMPTY. Not one line of `src/` changed.**

⛔ **THIS IS NOT THE CACHE.** No cache was built, no boot path was touched, no phase was
optimised. `3f159b0c5`'s closing line is the rule this obeyed: **a prescription is a claim.**
Row 5 is a recommendation with numbers attached, explicitly not a decision.

---

## ⭐ THE ANSWER, AND THE FIRST QUESTION HAD AN ANSWER NOBODY HAD GUESSED

## **GO.**

**The check does not need to be serialised, because `check_program` produces NOTHING TO
SERIALISE — it produces nothing at all.** Its signature is

```rust
pub fn check_program(forms: &[WatAST], sym: &SymbolTable, types: &TypeEnv)
    -> Result<(), CheckErrors>
```

— `sym` and `types` are **immutable borrows**, `src/check.rs` contains **zero** `RefCell` /
`Mutex` / `RwLock` / `Cell` / `unsafe` (grepped; empty), every intermediate it builds
(`CheckEnv`, `InferCtx`, `Subst`) is a local dropped at return, and the boot's call site
(`src/freeze.rs:1307`) matches the result for errors and **throws the `Ok(())` away**:

```rust
let check_result = check_program(&bundle.residue, &bundle.symbols, &bundle.types);
match (check_result, bundle.deferred_resolve.take()) { … (Ok(()), None) => {} }
```

The DESIGN feared `check:body-infer`'s output "may not be a thing that *has* a serialisable
form at all." It is worse than that and therefore better: **there is no output.** The check is a
pure validator over already-registered state. So caching it is not a serialisation problem — it
is an **elision** problem, and elision is free. A world restored from a cache was checked when
the cache was written; the 109 ms sweep re-derives a verdict it already has.

⛔ **The prize is therefore the 83 %, not the 51 % — with one named condition (row 1c) that is
the single largest open risk in this finding and is NOT established here.**

---

## Row 1 ⭑⭑ — CAN THE CHECK BE CACHED, OR ONLY THE EXPANSION?

**Cached: yes. Serialised: the question does not arise.** Three facts, each independently
checkable:

**(a) The check has no output.** Above. `check_program` returns `Result<(), CheckErrors>` and
the boot discards the `Ok`. Nothing downstream of step 8 reads anything the check produced,
because the check produced nothing. `FrozenWorld::freeze` (step 9) takes `types`, `macros`,
`symbols`, `program`, `loader`, `declared_rete_defns` — all of them built by steps 3–7.

**(b) The four expensive passes are STRUCTURALLY stdlib work wearing a user-program call
signature.** The SCORE established this; this stone read the loops. `check_program` is handed
`&bundle.residue` (the user's forms — **0 forms** in a bare boot, one in a one-line program) and
then does:

| census leaf | ms | what it actually iterates | `src/check.rs` |
|---|---:|---|---|
| `8b check:retired-syntax(ALL fns)` | 12.02 | `for func in sym.function_values()` | :691 |
| `8c check:restricted-call(ALL fns)` | 3.18 | `for (name, func) in sym.functions_iter()` | :742 |
| `8d check:def-position(ALL fns)` | 1.85 | `validate_def_positions_in_forms(forms, …)` **and** `for func in sym.function_values()` | :771, :774 |
| `8e check:form-loop(user residue)` | **0.02** | `for form in forms`, twice ← **the only leaf whose cost is ENTIRELY the user's** | :799, :817 |
| ⭐ `8f check:body-infer(ALL fns)` | **109.28** | `for (path, func) in sym.functions_iter()` | :844 |
| `8g check:impls-completeness` | 0.01 | `types.iter_subtype_edges()` | :855 |

**126.33 ms of the 129.72 ms (8b + 8c + 8d + 8f) is a sweep of the symbol table.** Only `8d`
mixes in a pass over `forms`, and `forms` is **0 forms** in the bare boot measured here. In a
boot whose user program is one line, 2,177 of the 2,177 functions in that table are stdlib.

(`8a check:env-from-symbols`, 3.35 ms, also builds itself from the whole symbol table and is
likewise a candidate; it is **excluded** from every figure below, conservatively.)

**(c) ⛔ THE CONDITION, AND IT IS NOT ESTABLISHED HERE.** Eliding the check means restricting
those four loops to the functions the cache did *not* supply. That is sound **iff no user
definition can change the verdict on a stdlib body.** The mechanism that could break it is
visible and specific: `check_program` builds `env` from `CheckEnv::from_symbols(sym, types)` and
then, in the `for form in forms` loop at `src/check.rs:817`, **mutates `env.defined_values` from
user top-level `def` forms BEFORE `check:body-infer` runs** — and `redef_allowed` exists, so
shadowing is a configured possibility rather than a hypothetical. Whether any reachable
combination makes a stdlib body's verdict depend on user state is **NOT DETERMINED BY THIS
STONE** (row 10). It is the spike the build stone must do first, because the whole difference
between 62 % and 93 % turns on it.

---

## Row 2 ⭑⭑ — THE STATE, SPLIT INTO DATA AND HANDLES

⭑ **The DESIGN's framing is the right one and the answer is better than it expected: after a
real freeze there is not a single un-re-registerable handle inside the cacheable state.**

### (a) PURE DATA — must be cached

| carrier | what it is | evidence |
|---|---|---|
| `SymbolTable.functions` | `HashMap<String, Arc<Function>>`, **2,177 entries, all `FunctionBody::Wat`** | measured, probe output below |
| `Function` | `name` / `params` (`Vec<Identifier>`) / `type_params` / `param_types` (`Vec<TypeExpr>`) / `ret_type` / `rest_param(_type)` / `body` / `rete` / `synthesized_for` | `src/value/environment.rs:48-129` |
| `WatAST` | 14 variants, **no `Arc<dyn>`, no closure, no pointer, no `Rc` cycle**; `Span{file: Arc<String>, line, col, end}` | `crates/wat-reader/src/ast.rs:60`, `span.rs:74` |
| `TypeEnv` | `types` / `builtin_names` / `subtype_edges` / `source_forms` — four owned std collections, **zero handles of any kind** | `src/types.rs:543-572` |
| `MacroRegistry` | `macros: HashMap<String, MacroDef>` + `surface_forms: HashMap<String, WatAST>`; **`MacroDef.body` is a `WatAST` template** | `src/macros/registry.rs:9,54` |
| `SymbolTable.unit_variants` | 122 prebuilt nullary enum values | measured |
| `binding_metadata`, `acronym_registry`, `redef_allowed`, `eval_redef_allowed` | plain maps / bools | `src/value/symbol_table.rs:114-150` |

**⛔ There is no `MacroBody::Native`, no native-expander table, no builtin-macro registry.**
Grepping `MacroBody|NativeMacro|native_macros|builtin_macros|expander` across `src/` and
`crates/` returns doc prose and nothing else. The expander's "native" behaviour is a literal
`match head { ":wat::core::do" => … }` in `src/macros/expand.rs:488` — **code, not state.**

### (b) RUST HANDLES — must NOT be cached; re-registered instead

| carrier | why it is a handle | how it comes back at load |
|---|---|---|
| `source_loader: Option<Arc<dyn SourceLoader>>` | trait object | **the caller already passes it** — `startup_from_source(src, base, loader)`. Nothing to re-derive. |
| `encoding_ctx: Option<Arc<EncodingCtx>>` | holds `Arc<EncoderRegistry>` → `RwLock<HashMap<usize, Arc<Encoders>>>` (`src/vm_registry.rs:95`) | `EncodingCtx::from_config(&config)` — **already constructed from scratch at every freeze**, `src/freeze.rs:526`. It is not derived from the stdlib at all. |
| `presence_sigma_fn` / `coincident_sigma_fn: Option<Arc<dyn SigmaFn>>` | trait objects | `DefaultPresenceSigma` / `DefaultCoincidentSigma` are **unit structs** installed unconditionally at `src/freeze.rs:551-552`; a user override is evaluated from `config.presence_sigma_ast`, which is cached config, not cached stdlib. |
| `primed_stdio: Option<Arc<PrimedStdio>>` | live `Address`es over crossbeam/unix sockets | **measured `false` after a real freeze** — it is not boot state at all. |
| `outer_symbols: Option<Arc<SymbolTable>>` | recursive | sandbox-only; `None` at boot. |
| `Function.closed_env: Option<Environment>` | `Arc<EnvCell>` of `Value`s, which *can* be handle-bearing | **measured 0 of 2,177.** `defn`-registered functions carry `None` by construction; only `fn` *values* close over an env, and those are runtime values. |
| `runtime_def_values: HashMap<String, Value>` | `Value` has handle variants (`Arc<dyn WatReader>`, senders, `RustOpaque`, …) | **measured 56 entries, all pure**: 32 `i64`, 23 `clauses`, 1 `String`. Zero handle-bearing variants. |

### (c) ⭐ NATIVE BUILTINS ARE ALREADY RE-REGISTERED BY NAME — the cache never sees a fn pointer

`FunctionBody::Native` is a **unit variant with no payload** (`src/value/environment.rs:22-29`).
Native dispatch is keyed on the head *string*, through a process-global `OnceLock`
`IntrinsicRegistry` populated by `inventory` (`src/intrinsic/mod.rs:434`) plus a literal
`match head { … }` (`src/runtime.rs:5444` and `:5560`) — **neither of which lives on `SymbolTable`**.
Every `FunctionBody::Native` arm in the tree is `unreachable!("native builtin fn-applied —
dispatched via the runtime match, not fn-apply")`. The measurement agrees: **0 of 2,177
functions carry `Native`.**

This is the DESIGN's ⭑ resolved in the strongest possible way: *the thing that cannot be cached
is already not stored.* The re-registration mechanism exists, is the live dispatch path, and
costs the cache nothing.

### (d) The one piece of `WatAST` that is genuinely process-local

`Identifier { name, scopes: BTreeSet<ScopeId> }`, where `ScopeId(u64)` is minted by a
process-global `static NEXT: AtomicU64` (`crates/wat-reader/src/identifier.rs:48,68`) and has
**no `from_u64` constructor**. A raw id restored by value would collide with the loading
process's own counter.

**This repo already ruled on that**, in the doc comment on the very field a cache must restore
(`src/value/environment.rs:65-69`):

> *"It stopped being an accident the moment a program had to cross a process boundary: an exec'd
> child restarts `fresh_scope()` at 1, so imported scopes must be REMAPPED."*

So the cache **re-mints** scopes at load through a dense remap table, preserving the sharing
structure and nothing else — which is all the semantics has. The prototype does exactly this and
proves it (row 3). **No `src/` change is needed for it.**

⚠ **And a measured surprise, stated because it cuts the other way**: after `startup_bare()`,
**0 of 21,925 `Symbol` nodes and 0 of 3,260 fn params carry any hygiene scope at all.** The
remap machinery is therefore, on today's manifest, machinery for a case that does not occur. It
is kept and tested anyway because a user program that invokes a symbol-introducing macro is not
covered by this measurement (row 10).

---

## Row 3 ⭑⭑ — A ROUND-TRIP, TIMED, AT REPRESENTATIVE SIZE, WARM AND COLD

### The payload, and why it is the right one

`tests/diagnostics/boot_cache_roundtrip.rs` boots a real `startup_bare()` and serialises
**every `FunctionBody::Wat` body in the frozen symbol table** — the post-expansion,
post-registration, post-check ASTs that *are* the 83 %.

```
functions (total)                  2177
  FunctionBody::Wat                2177
  FunctionBody::Native                0
  with closed_env (a handle)          0
MacroRegistry attached             true
TypeEnv entries                     431
unit_variants                       122
residue program forms                 0
primed_stdio present              false
runtime def_values by Value kind:
   wat::core::String                             1
   wat::core::clauses                           23
   wat::core::i64                               32
fn params (total)                  3260
  carrying hygiene scopes             0
--- reachable AST nodes, by component ---
function bodies                   95824
TypeEnv source forms (   0)           0
binding_metadata values              27
residue program                       0
(wat/**/*.wat as PARSED,  55 files)  65422
payload node-kind census:
   BoolLit             652
   FloatLit              1
   IntLit             1270
   Keyword           36352
   List              28430
   NilLit              590
   StringLit          2645
   Symbol            21925
   Vector             3959
  Symbols carrying scopes             0
AST nodes                         95824
distinct span files                  38
distinct hygiene scopes               0
payload bytes                   2042571
```

⭐ **The function bodies alone (95,824 nodes) are LARGER than the entire 55-file stdlib as
parsed (65,422 nodes).** Every other publicly reachable component is three orders of magnitude
smaller: `binding_metadata` 27 nodes, residue 0, `TypeEnv.source_forms` 0. **2.04 MB.** That is
the DESIGN's "largest serialisable component", measured rather than asserted.

⚠ **What the payload is NOT** (trap-door 5, honoured out loud): it omits `Function`'s
`param_types` / `ret_type` (`TypeExpr` trees), the 431 `TypeDef`s, and the whole
`MacroRegistry` — `MacroRegistry` has no public iterator, so its size could not be measured from
a test without changing `src/`, and it was not. **Every number below is a LOWER BOUND on the
real load cost.** It is the right lower bound to take, because those omitted parts are the same
kind of payload (owned trees of `String` + small data) at a visibly smaller scale.

### The format, and why crude is the point

Hand-rolled: LEB128 varints, zig-zag signed, length-prefixed UTF-8, one recursive walk, one
`Vec<u8>`, one 38-entry string table for span file labels. **No dev-dependency was added. No
dependency of any kind was added.** (The cold read uses `libc::posix_fadvise`, and `libc` is
already a first-class dependency at `Cargo.toml:117`.)

### The numbers — median of **10 separate process runs**, each a fresh `nextest` fork

| | median | min | max |
|---|---:|---:|---:|
| `startup_bare()` (the thing being replaced) | **415.46** | 410.21 | 443.40 |
| encode, first call in process (**cold**) | 7.18 | 6.70 | 8.30 |
| encode, steady state (**warm**) | 6.37 | 6.15 | 7.81 |
| decode from memory, first call (**cold**) | 9.43 | 9.35 | 10.18 |
| decode from memory, steady state (**warm**) | 7.04 | 7.02 | 7.15 |
| file read, **page cache EVICTED** | 5.07 | 4.87 | 6.36 |
| ⭐ **read + decode, COLD** — *the load path a real boot takes* | **15.67** | 14.50 | 25.24 |
| read + decode, **WARM** | 7.43 | 7.31 | 8.29 |

All in ms. **Cold is real, not asserted**: the file is `fsync`ed and `posix_fadvise(…,
POSIX_FADV_DONTNEED)`ed before the cold read, and the cold read costs 5.07 ms against a warm
read of ~0.4 ms (7.43 total − 7.04 decode). A 12× gap is the eviction working.

### The round-trip is PROVEN, not assumed — and the obvious proof would have been worthless

`Span`'s `PartialEq` is **hand-written and unconditionally `true`**, and its `Hash` is a **no-op**
(`crates/wat-reader/src/span.rs:137-149`) — deliberate, so structurally-identical ASTs from
different sources compare equal. **Therefore `assert_eq!(original, decoded)` passes on a decoder
that drops every span.** So the probe asserts a **byte-exact fixpoint** instead:
`encode(decode(encode(x))) == encode(x)`, over all 2.04 MB. The bytes carry the spans; byte
equality does not let them go missing.

And because the stdlib payload exercises only **9 of `WatAST`'s 14 variants** and **zero**
hygiene scopes, a second test
(`the_crude_format_round_trips_the_variants_the_stdlib_payload_never_exercises`) covers the
other five — `RationalLit` (−22/7), `BigIntLit` (2¹²⁷−1), `CharLit` (😀 and `\n`), `Map`, `Set`
— plus `-0.0`, `f64::MIN_POSITIVE`, `i64::MIN`/`MAX`, an empty `Vector`, a second span file, a
point-span, and **two scoped identifiers sharing one scope**. It asserts the fixpoint AND that
the restored scopes are *different* `ScopeId`s that *preserve the sharing*, which is the
row-2(d) design demonstrated rather than described.

---

## Row 4 ⭑⭑ — WHICH CONCLUSION THIS NUMBER SUPPORTS, AND WHICH IT DOES NOT

⭐ **It supports GO, and that is the direction the asymmetry runs.** A crude serializer is a
**pessimistic** estimate. 15.67 ms cold, from a naive byte-at-a-time varint decoder that
allocates a fresh `String` for every one of 36,352 keywords and 21,925 symbols, is **17× faster
than the 265.96 ms Tier A elides and 25× faster than Tier B's 392.29 ms** (row 5). A fast crude number proves GO because every real format —
zero-copy, `mmap`, arena, interned string table — is **faster than crude, never slower**.

⛔ **It does NOT support any claim about the best achievable load time.** 15.67 ms is not the
floor; it is a ceiling drawn by the dumbest implementation available. Nobody should quote it as
a target.

⛔ **It does NOT prove the full cache loads in 15.67 ms.** The payload is the largest component,
not the whole state (row 3's ⚠). Even at 3× for the omitted parts — 47 ms — the conclusion does
not move, which is why the omission is survivable; but the number that is *measured* is 15.67 ms
for 2.04 MB of function bodies, and only that.

---

## Row 5 ⭑⭑ — GO / NO-GO, AND THE FRACTION OF THE 430 ms

# **GO.**

Against the SCORE's accounted 404.94 ms (10 warm `=phases` runs), with a 15.67 ms load
substituted for the elided phases:

### Tier A — expansion + parse + registration only *(no check elision; row 1c's spike unneeded)*

| elided | ms |
|---|---:|
| `3a stdlib-parse` | 29.24 |
| `4 stdlib-defmacro-register` | 4.36 |
| `4 kwargs-companions` | 0.77 |
| `4 stdlib-expand` | **201.37** |
| `5 stdlib-types-register` | 2.70 |
| `5 aggregate-containment` | 0.23 |
| `6 stdlib-defines-register` | 11.68 |
| `6a defclause-stub-preregister` | 0.98 |
| `6b stdlib-runtime-def-filter` | 0.04 |
| `6a-9 auto-method-codegen` | 4.01 |
| `6.97 typeenv-clone-attach` | 0.47 |
| `7.6 stdlib-runtime-defs-register` | 10.11 |
| **total** | **265.96** (65.7 %) |

⇒ accounted boot **404.94 → 154.65 ms**. **Net saving 250.29 ms = 61.8 % of accounted,
55.9 % of the ~448 ms external wall.**

### Tier B — Tier A **plus** the four `ALL fns` check sweeps restricted to user functions

`8b` 12.02 + `8c` 3.18 + `8d` 1.85 + `8f` **109.28** = **126.33 ms** more.
Total elided **392.29 ms = 96.9 % of accounted.**

⇒ accounted boot **404.94 → 28.32 ms**. **Net saving 376.62 ms = 93.0 % of accounted,
84.2 % of the external wall.**

**The DESIGN's 83 % (expansion 206.52 + check 129.72 = 336.24 / 404.94) sits between the two
tiers, and both tiers clear it on the expansion side alone.** Tier B exceeds 83 % because a
cache that restores the *registered* world also elides registration (30.35 ms) and
`stdlib-parse` (29.24 ms), which the DESIGN's framing did not count.

**Recommendation, not a decision:** build Tier A first — it needs no soundness argument, it is
the larger single win (201 ms in one phase), and it is fully proven by this stone. **Do the row
1c spike before committing to Tier B**, because Tier B's extra 126 ms is worth exactly nothing
if a user `def` can change a stdlib body's verdict.

**The shape, and no more than the measurement supports:** cache the pure-data side
(`functions` / `TypeEnv` / `MacroRegistry` / `unit_variants` / `binding_metadata` /
`acronym_registry`), **re-register the handle side by name at load** (loader from the caller,
`EncodingCtx` from `Config`, sigma fns from their unit structs, natives from the existing
`IntrinsicRegistry`), **re-mint hygiene `ScopeId`s through a dense remap**, and key the cache on
the stdlib's content — the tree already has `hash_canonical_ast` / `verify_source_hash`
(`src/hash.rs`) for that, though this stone did not measure it.

---

## Row 6 ⭑ — BOOT UNCHANGED

⭐ **Proven by construction, and this is the strongest form the row can take: `git diff` is
empty.** The whole stone is two **untracked** files — this FINDING and one test.

```
$ git status --short
?? docs/excursus/2026/08/001-sns-sqs/the-boot-cache-is-possible-or-it-is-not/FINDING.md
?? tests/diagnostics/boot_cache_roundtrip.rs

$ git diff --stat
                                   (no output)
```

No `src/` file, no `Cargo.toml` entry (`tests/diagnostics/mod.rs` is a `build.rs`-generated
`include!` stub, so a new `.rs` registers itself). The prototype is **behind the test gate** —
it runs only under `cargo nextest`, `startup_from_source` does not know it exists.

**Six runs of the default path, final binary, env unset** (the row asks three):

```
run 1 wall=0.451s rc=0 stderr_bytes=0
run 2 wall=0.443s rc=0 stderr_bytes=0
run 3 wall=0.454s rc=0 stderr_bytes=0
run 4 wall=0.445s rc=0 stderr_bytes=0
run 5 wall=0.452s rc=0 stderr_bytes=0
run 6 wall=0.440s rc=0 stderr_bytes=0
```

**stderr is 0 bytes on every run.** 0.440–0.454 s sits inside the prior SCORE's measured
baseline band for this box (0.439–0.469 s). Nothing was A/B'd against a stashed binary because
there is nothing to A/B: the release binary's inputs are byte-identical to HEAD's.

---

## Row 7 ⛔ — NOTHING ELSE OPTIMISED; `.config/nextest.toml` UNTOUCHED

`git diff` is empty (row 6) — **no tracked file changed, so nothing could have been optimised.**
Not a `clone`, not an allocation, not a lookup, not a timeout, not a phase.

```
$ sha256sum .config/nextest.toml
706f59851bc01cca13ad37f2c0d07abe68a7133d362418ffeb77da21d22a5372  .config/nextest.toml

$ git show HEAD:.config/nextest.toml | sha256sum
706f59851bc01cca13ad37f2c0d07abe68a7133d362418ffeb77da21d22a5372  -
```

**Identical.** And `src/freeze.rs` is untouched, so the golden that pins `src/freeze.rs:1541`
(the fragility the census stone named) cannot have moved.

**No dependency was added — dev or otherwise.** The DESIGN anticipated a serde/bincode
dev-dependency; none was needed, because a crude format is hand-written by definition and
`libc` (already `Cargo.toml:117`) supplies `posix_fadvise` for the cold read. `Cargo.toml` and
`Cargo.lock` are both unmodified.

---

## Row 8 ⭑⭑ — FLOOR

**Summary line, verbatim:**

```
     Summary [ 557.959s] 5286 tests run: 5286 passed (9 slow), 22 skipped
```

**5286 / 5286 — the census stone's 5284 plus this stone's two new tests, all passing.** No red,
not one, and nothing was re-run.

- log: `.floor/2026-09-17T06-49-29Z/raw.log` · `clean.log` (untruncated, ANSI-stripped, kept
  before reading) — `exit=0. Log kept … regardless — a green run is evidence too.`
- **`ARM.txt`: NOT written**, and that is the correct outcome — `scripts/floor.sh` writes one
  only on a red. The directory holds exactly `raw.log` and `clean.log`.

---

## Row 9 — clippy + `--no-run`

```
$ cargo clippy --release --workspace --all-targets
   Compiling wat v0.1.0 (/home/john/work/holon/wat-rs)
    Checking with-loader-example v0.1.0 (/home/john/work/holon/wat-rs/examples/with-loader)
    Checking console-demo v0.1.0 (/home/john/work/holon/wat-rs/examples/console-demo)
    Finished `release` profile [optimized] target(s) in 16.70s
```

Zero warnings, zero errors — and the workspace denies `clippy::all` at the manifest, so a
finding would have been a build failure, not advice.

```
$ cargo nextest run --release --no-run
    Finished `release` profile [optimized] target(s) in 0.10s
```

---

## Row 10 ⭑ — WHAT COULD NOT BE DETERMINED

An honest ABSENT beats a plausible GO. Five of them, in descending order of how much they matter.

**1. ⛔ Whether the check sweeps can be safely restricted to user functions.** Row 1c. The
mechanism that could break it is located (`env.defined_values` mutated from user top-level
`def`s at `src/check.rs:817`, before `check:body-infer` at `:844`; `redef_allowed` makes
shadowing configurable) but **no attempt was made to construct a program where a user definition
changes a stdlib body's verdict, nor to prove none exists.** *The spike:* enumerate the routes by
which `CheckEnv` state reachable from a stdlib body's inference can be written by user code —
`defined_values`, `defclause_registrations`, `redef_allowed`, restriction whitelists — and for
each, either exhibit a program that flips a stdlib verdict or show the route is closed. **This
is 126 ms of the 376 ms and it is the only thing standing between Tier A and Tier B.**

**2. The size of the rest of the cache payload.** `MacroRegistry` has no public iterator and
`Function`'s `TypeExpr` fields were not serialised, so the load cost is a **lower bound**
(row 3's ⚠). *The spike:* a `pub fn len()` / iterator on `MacroRegistry` (or an in-`src` bench),
then extend the prototype to the full `FrozenWorld` and re-time. Cheap; it was out of scope here
because it needs an `src/` change and trap-door 1 forbade one.

**3. Whether a user program introduces hygiene scopes the bare boot does not.** Measured 0
scoped symbols and 0 scoped params after `startup_bare()`. `expand_template` adds the macro
scope to **template-origin `Symbol`s only** (`src/macros/expand.rs:1695-1699`), and today's
stdlib templates evidently emit keywords where they could emit bare symbols. **Why that is so
was not determined, and no world with user macro invocations was measured.** The round-trip
handles scopes either way (proved by the variant test), so this changes the payload size, not
the verdict.

**4. What the cache would be keyed on, and what a stale key costs.** `src/hash.rs` has
`hash_canonical_ast` / `canonical_edn_wat` / `verify_source_hash`, which look like the right
tools — **none of them was measured or tried.** A cache is only as safe as its invalidation and
this stone says nothing about it.

**5. Whether the 15.67 ms survives contact with a real load path.** The prototype decodes into a
`Vec<(String, Arc<WatAST>)>`. A real load must additionally build the `HashMap` spine, the
`Arc<Function>` records, and attach the capability carriers. **Not measured.** Expected small
against 15.67 ms, but "expected" is the word this campaign has been charged for before.

---

## What surprised me

**Three things, and the first one reframed the stone.**

⭐ **The hardest question had the easiest answer, for a reason nobody had stated.** The DESIGN
ordered the check first because it might have *no serialisable form*. It has no form because it
has no *output* — `check_program` is a pure validator whose `Ok(())` the boot throws away. The
fear was that the check would be uncacheable; the truth is that it is the one part of the 83 %
that needs no cache format at all, only permission to not run. That inverts the risk: the
expensive-to-build part (expansion) is the easy one, and the free part (the check) is the one
carrying the open soundness question.

⭐ **`FunctionBody::Native` is a unit variant, and 0 of 2,177 functions use it.** The DESIGN's
central worry — "native/builtin fns, `Arc<dyn …>`, loaders, fn pointers" — **does not exist
inside the symbol table.** Natives dispatch by *string* through a process-global `OnceLock` +
`inventory` registry that boot never touches. The repo had already built the
re-register-by-name mechanism the DESIGN asked for, years before anyone asked whether a cache
needed one. Likewise `runtime_def_values`, the one field that *could* have held a live socket:
56 entries, all `i64` / `clauses` / `String`.

⭐ **The obvious correctness test would have been worthless.** `Span::eq` returns `true`
unconditionally and `Span::hash` is a no-op, so `assert_eq!(original, decoded)` — the assertion
anyone would write first — passes on a decoder that silently drops every span. The byte-fixpoint
was not extra rigour; it was the minimum that proves anything. A round-trip prototype that had
been "verified" the obvious way would have reported GO on a format that loses source locations,
and the first bad diagnostic after the cache shipped would have been the discovery.

One smaller one: **the function bodies out-mass the entire parsed stdlib** — 95,824 nodes
against 65,422 for all 55 files. Expansion does not merely take 201 ms; it makes the thing
half again bigger, which is why the payload is 2 MB and why caching it is worth 26× its own
load cost.

---

## ⭑ REGRADED BY THE ORCHESTRATOR ON HIS OWN RUNS, 2026-09-17

```
floor    Summary [ 549.165s] 5286 tests run: 5286 passed (9 slow), 22 skipped
         .floor/2026-09-17T07-00-56Z/ · exit=0 · NO ARM.txt · inside the 540.4–561.5 s band
clippy   0 · git diff EMPTY — the whole probe is two untracked files, zero production code moved
```

| claim | the orchestrator's own check |
|---|---|
| ⭐ **the check has no output** | ✅ `pub fn check_program(forms: &[WatAST], sym: &SymbolTable, types: &TypeEnv) -> Result<(), CheckErrors>` — immutable borrows, nothing returned on success. `grep -cE 'RefCell\|Mutex\|RwLock\|Cell<\|unsafe' src/check.rs` → **0**. Caching it is **elision, not serialisation.** |
| ⭐⭐ **`Span::eq` is unconditional** | ✅ verbatim at `crates/wat-reader/src/span.rs:137`: `fn eq(&self, _: &Self) -> bool { true }`, and `Hash` is a no-op at `:146`. |
| round-trip | ✅ re-run: boot **422.30 ms**, **read+decode COLD 15.553 ms**, warm 7.463 ms, 95,824 AST nodes. Both probe tests PASS. |
| measure-only | ✅ `git status`: two untracked files. `git diff` empty. `.config/nextest.toml` untouched. |

### ⭐ THE RESULT INVERTS THE RISK THE DESIGN ASSUMED

The DESIGN ordered its questions so the *check* would kill the idea fastest — *"its output may have no
serialisable form at all."* **It has no output.** `check_program` is a pure validator whose `Ok(())`
boot discards. So the expensive-and-scary half is the *easy* half, and the free half — elision — is
where the only open question lives.

⛔ **And that question is real: 126 ms rides on it.** Eliding the four `ALL fns` sweeps is sound only if
no user definition can change a **stdlib** body's verdict. The executor located the mechanism that could
break it (`env.defined_values` mutated from user top-level `def`s at `src/check.rs:817`, *before*
body-infer at `:844`; `redef_allowed` exists) and **proved neither direction**. Naming the spike instead
of assuming the answer is the correct call and is why Tier A is recommended first.

### ⭐⭐ THE TRAP IT CAUGHT IS WORTH MORE THAN THE MEASUREMENT

`Span::eq` returns `true` unconditionally. **Therefore `assert_eq!(original, decoded)` — the obvious
round-trip test — PASSES ON A DECODER THAT DROPS EVERY SPAN.** The probe asserts a byte-exact fixpoint
`encode(decode(encode(x))) == encode(x)` instead, plus a second test covering the five `WatAST` variants
the stdlib payload never exercises and scoped identifiers (proving scopes come back re-minted *and*
sharing-preserving).

★ Any future stone touching AST serialisation inherits this: **equality on this AST is not equality.**

### The handle problem evaporated on measurement

`FunctionBody::Native` is a **unit variant with no payload**, and **0 of 2,177** functions carry it —
natives dispatch by **string** through a process-global `OnceLock` + `inventory` registry that boot never
touches. So the "re-register by name" mechanism the DESIGN asked for **already exists and is the live
dispatch path**. `closed_env`: **0 of 2,177**. `runtime_def_values`: 56, all `i64`/`clauses`/`String`.
`MacroRegistry` holds `WatAST` templates — **no `MacroBody::Native`, and no native-expander table exists
anywhere in the tree**. The only process-local value is `ScopeId`, and the repo had already written the
remedy in `Function::params`' own doc: *"an exec'd child restarts `fresh_scope()` at 1, so imported
scopes must be REMAPPED."*

### The prize, and the honesty about its bounds

```
Tier A  expansion + parse + registration      elides 265.96 ms   404.9 → 154.7 ms   61.8 % of accounted
Tier B  + the four ALL-fns check sweeps       elides 392.29 ms   404.9 →  28.3 ms   93.0 % of accounted
```

⚠ **15.67 ms is a LOWER BOUND, not a load-time claim** — `MacroRegistry` has no public iterator and
`TypeExpr` fields went unserialised, and the real spine (`HashMap` + `Arc<Function>`) was not built. The
executor says so itself, and states the asymmetry correctly: **fast crude proves GO; slow crude would
have proved nothing.**

### Five ABSENTs, all honest, one strange

Check-elision soundness (the spike, 126 ms) · the rest of the payload's size · **0 of 21,925 symbols and
0 of 3,260 fn params carry hygiene scopes after a bare boot — surprising and unexplained** · cache
keying/invalidation (untried; `src/hash.rs` looks right) · whether 15.67 ms survives building the real
spine.


---

## ⛔⛔ RULED 2026-09-17 — TIER A, THEN TIER B

**Builder:** *"let's do tier A, ten B -- make sure we've got these noted in our docs and we roll"*.

This FINDING's recommendation is now a **decision**, and it is recorded here so no later reader mistakes
it for an open question:

1. **Tier A is BUILT** — cache the expanded + registered state, eliding parse + expand + registration.
   **265.96 ms**, no soundness argument required. Stone:
   `the-boot-cache-elides-what-it-already-knows/`.
2. **Tier B FOLLOWS** — elide the four `ALL fns` check sweeps for a further **126 ms**, and it is gated
   on the spike this FINDING named: *can a user definition change a **stdlib** body's verdict?*
   (`env.defined_values` mutated from user top-level `def`s at `src/check.rs:817`, before body-infer at
   `:844`; `redef_allowed` exists.) **The spike runs before Tier B is drawn**, not during it.

⭑ The order is the builder's and it matches the measurement: Tier A is the larger phase *and* the one
with no open question. Tier B is smaller *and* carries the only soundness risk in the whole finding.
