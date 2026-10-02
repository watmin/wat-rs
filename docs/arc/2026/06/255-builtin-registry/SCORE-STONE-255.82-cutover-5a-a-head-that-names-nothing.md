# SCORE — STONE 255.82: cutover 5a — STOP-1

**HEAD** `71a577901` (parent `5555c55f2`, the brief's draw ancestor). The refusal is not in the tree. No floor, no clippy, no `census.sh --diff`.

## 1. The dispatch (code agrees with the brief)

`infer_list` is `src/check.rs:2875`. It matches a keyword head at `src/check.rs:2899` and does not match a symbol head there.

A symbol with no `/` is not a reference (`Identifier::is_reference`, `crates/wat-reader/src/identifier.rs:231`: true only when `namespace() != $bound`). In `infer` (`src/check.rs:2329`) a miss in `locals` that is not a hygiene divergence returns `CheckResult::ok(fresh.fresh())` with no error (`src/check.rs:2344-2347`).

That fresh type is not `TypeExpr::Fn`, so the call-head fallthrough (`src/check.rs:6521-6550`) returns another `fresh.fresh()` and no error. The comment at `6549` calls that silent-by-intent.

The keyword `UnresolvedReference` is not emitted from `infer_list`. It is `check_form` in `src/resolve/walk.rs:87-98`, and only when the head is `WatAST::Keyword`, via `is_resolvable_call_head` (`src/resolve/walk.rs:264`). `src/resolve/mod.rs:53-58` states this pass does not check bare-symbol references.

`UnresolvedReference` (`src/resolve/error.rs:13`) is `path`, a `&'static` `context`, and `span`. No remedy field.

## 2. Census (before any refusal)

Print-only classification inside `infer_list`, on the locals map that arm already consults, then deleted. Not committed. `WAT_CENSUS_25582=1 ./target/release/wat --check` on `git ls-files '*.wat'`: **2280 files**, rc histogram **2070×0, 208×1, 2×101**. Unique by class + name + `file:line:col`. Heads `infer_quote` does not walk are absent: `infer_quote` (`src/intrinsic/special/quote.rs:88-109`) returns without entering the argument.

| class | sites | what the checker did |
|---|---|---|
| local | 140 | `locals` hit. 32 names, 58 files |
| poisoned-ctor (`Some` / `Ok` / `Err` as the whole head) | 0 | none reached `infer_list` |
| unbound | 28 | not a local |

Local names (site counts): `f` 37, `overlay` 16, `mk` 13, `fire` 11, `at` 10, `work-fn` 6, `body-fn` 5, `pred` 5, `rows` 4, `ga` 3, `ob-at` 3, `fa` 2, `facts` 2, `fb` 2, `head` 2, `uses-result?` 2, `v` 2, and one each of `bat`, `cat`, `fc`, `g`, `ga-at`, `grant-fn`, `inner`, `mat`, `plus`, `probe`, `prop`, `revoke-fn`, `seed`, `still-fails?`, `worker-init`.

No separate "macro-introduced" class is visible here. This census runs after expansion, and the only binding question `infer` asks a no-slash symbol is the locals map.

The one door, measured by `is_resolvable_call_head` on the symbol table and macro registry of `wat-scripts/scratch-pad/probe-arc278-57-round1a-nine-aliases.wat` after `startup_from_source` (the test that printed this was deleted; not committed):

| head string | door | registry.contains | lookup_special_form | sym.get |
|---|---|---|---|---|
| `def` | false | false | false | false |
| `:wat::core::def` | true | true | false | false |
| `Some` | true | false | false | false |
| `Ok` | true | false | false | false |
| `Err` | true | false | false | false |
| `foozle` | false | false | false | false |

`src/remedy/retirement.rs` has rows `Some` → `:wat::core::Some` (line 153), `Ok` (159), `Err` (162). The door's `is_retired` rung sits after the two false rungs above. Which of the remaining rungs made `Some`/`Ok`/`Err` true was not printed separately.

Embedded wat literals were not walked. The tracked `.wat` set already crosses the cap below.

## 3. STOP-1

A refusal of "symbol head, not a local, door false" hits **28** sites. The cap is 20.

24 are the bare head `def`, and each of those three files `--check` exits 0 today:

- `wat-scripts/scratch-pad/probe-arc278-57-persistentmap-contains-key.wat` — 39:2, 43:2, 52:2, 56:2, 63:2
- `wat-scripts/scratch-pad/probe-arc278-57-round1a-nine-aliases.wat` — 5:2, 6:2, 7:2, 8:2, 9:2, 10:2, 11:2, 12:2, 13:2
- `wat-scripts/scratch-pad/probe-arc278-57-round1b-parametric-and-hof.wat` — 9:2, 10:2, 15:2, 32:2, 34:2, 36:2, 39:2, 77:2, 82:2, 91:2

Each is `(def :name expr)` at column 2. The checker does not see `def` as a local. The door does not accept the string `def`, and it does accept `:wat::core::def`. That is the design question: how a bare declaration name becomes bound, without a second lookup beside `is_resolvable_call_head`.

The other 4 are the probe `wat-scripts/scratch-pad/arc-255/probe-255.82-a-head-that-names-nothing.wat`, also rc 0: `foozle` 4:66, `my.made.up.thing` 5:68, `wat.core.Option.zzznope` 6:72, and `wat.core.Option.Some` 6:97 (a fourth head in that form, beside the three the brief names).

Baseline named by the brief, read, not re-run: `.floor/2026-10-02T09-46-26Z` Summary `[ 393.583s] 6368 tests run: 6368 passed (24 slow), 24 skipped`. No `RC=` line in that `clean.log`.

## 4. Amend — the refusal is in

Sections 1–3 stand. The 24 bare `def` heads were respell-then-deleted. The refusal is coded. No STOP.

### The three scratch files

One-file edits, not a codemod. Each `(def` became `:wat::core::def`. With `def` real, `wat --check` was RC=1 on each file, `#wat.check/UnnamespacedName`, counts 9, 10, and 5. They no longer prove the rete-spelling claim they were written for. A failing `.wat` under `wat-scripts/` is loaded by `every_wat_scripts_file_loads`, so the three files were deleted:

- `wat-scripts/scratch-pad/probe-arc278-57-persistentmap-contains-key.wat`
- `wat-scripts/scratch-pad/probe-arc278-57-round1a-nine-aliases.wat`
- `wat-scripts/scratch-pad/probe-arc278-57-round1b-parametric-and-hof.wat`

### The probe's fourth head

`wat.core/Some` normalizes to `:wat::core::Some`. The checker refuses that keyword as a retired bare variant (`MalformedForm`, "write `:wat::core::Option.Some`") before resolve, and a check error that is not `UnknownCallee` hides the resolve record. The fourth head is now `(:wat::core::Option.Some {:value 1})`. The file is `wat-scripts/scratch-pad/arc-255/probe-255.82-a-head-that-names-nothing.wat.bad`. `./target/release/wat --check` on it is RC=1 and names exactly `foozle`, `my.made.up.thing`, and `wat.core.Option.zzznope` ("3 unresolved references"). The map constructor is not among them.

### The rung, and the remedy

Measured on `startup_bare`, then the probe was deleted. `Some` / `Ok` / `Err`: door true, registry false, special false, sym false, retired true, def-value false, unit false, macro false. `def` and `foozle`: every rung false. `:wat::core::def`: door true, registry true. The rung that admits the three constructors is `is_retired`. The refusal still fires for them, and the remedy is the table's replacement (`:wat::core::Some`, `:wat::core::Ok`, `:wat::core::Err`).

`UnresolvedReference` gained `remedy: Option<String>`. It is omitted from the EDN when `None`, so a keyword-head refusal stays byte-identical. A dotted wrong-join whose last segment the door accepts carries the slash spelling: `wat.core.Option.expect` → remedy `wat.core.Option/expect`. That is not `ns_to_wat_path` (`:wat::core::Option::expect`).

### What the user sees for bare Some / Ok / Err

The checker's poisoned arms emit `TypeMismatch` "retired bare-symbol exception" naming the replacement. That is not `UnknownCallee`, so freeze reports the check error and the resolve record is shadowed. The retired-ctor probe asserts those three replacement names. A local named `Some` skips the three arms (`bare_symbol_is_local`) and runs: `user/shadow-some` returns 4.

Scope for a bare head mirrors the checker: sequential `let`, `fn` / `lambda` (via `parse_fn_signature_for_check`, with a top-level symbol fallback when that parse fails), `defclause`, and `match`. Quote is data. A keyword `(:wat::core::unquote …)` is walked; a symbol `wat.core/unquote` inside a template is data, because normalize does not rewrite template symbols and the escape detector is keyword-only.

### extend-type method names

The first census of the refusal, `.census/2026-10-02T18-39-59Z.txt`, flipped 141 files from rc 0 to 1. The heads were method names inside expanded `(:wat::core::extend-type …)` — `grant`, `revoke`, `coordinate`, `coord` — which defservice emits for Capability and Dialable. Those names are the method being defined. `walk_extend_type` binds the param vector and walks the body. That census is not the gate.

### Embedded literals

`extract_literal_spans` over `src`, `tests`, and `crates` `*.rs`, one shared `startup_bare`, then `expand_all` and `resolve_references`. A literal counts only when it parses as a sequence of lists and at least one head is a keyword or a namespaced symbol: 1289 such literals. Bare `UnresolvedReference` paths (no leading `:`): 39.

| names | sites | what the literal is |
|---|---|---|
| `foo` 7, `echo` 7, `write-logs` 5, `ping` 2, `get` 2, `stats` 1 | 24 | a defsurface feature or message name in a Rust string |
| `tmp` 2, `t` 2, `load-a`, `load-b`, `body-expr` | 7 | a defmacro template, or the list-shaped let it expands to |
| `pool` 2, `_` 2, `x` | 5 | a list-shaped let binder `((name expr))` walked as a call |
| `>` | 2 | an ensure-snapshot string compared as text |
| `some-body-form` | 1 | the residue of a config-parse unit |

None of the 39 is in the three scratch files or the probe. None is a loaded `.wat` program. `startup_bare` succeeds with the refusal in place. This is not a new STOP-1.

### Census

Gate: `.census/2026-10-02T18-57-51Z.txt`, files=2278, histogram 2068×0, 208×1, 2×101. Against `.census/2026-10-02T18-03-11Z.txt` (2070×0, 208×1, 2×101): `census.sh --diff` exit 0, no STOP-8. The only path changes are the four deletions (each was rc 0) and two new files at rc 0:

- gone: the three scratches above, and `wat-scripts/scratch-pad/arc-255/probe-255.82-a-head-that-names-nothing.wat`
- new rc 0: `tests/resolve/probe_arc255_82_bound_bare_head.wat`, `tests/resolve/probe_arc255_82_quasiquote_bare_head.wat`

`.wat.bad` files are outside `git ls-files '*.wat'`.

### Floor and clippy

Red, not re-run: `.floor/2026-10-02T18-44-30Z`

```
Summary [ 401.535s] 6374 tests run: 6371 passed (28 slow), 3 failed, 24 skipped
```

exit 100. The three arms, from `.floor/2026-10-02T18-44-30Z/ARM.txt`:

- `no_loose_string_assert::tests_carry_no_loose_string_assert` at `tests/lint/no_loose_string_assert.rs:135`. Offenders `tests/resolve/probe_arc255_82_bound_bare_head.rs:18` and `:81` (`contains` on a multi-reference EDN value).
- `one_variant_separator::only_identifier_rs_spells_the_variant_separator` at `tests/lint/one_variant_separator.rs:260`. Offender `src/resolve/walk.rs:338`, `stem.replace('.', "::")`, kind DATA. That join turns a dotted namespace into the door's `::` key. It is not an enum/variant split. Runed `namespace`.
- `keyword_heresy_ledger::the_heresy_ledger_matches_its_frozen_census` at `tests/lint/keyword_heresy_ledger.rs:1332`. `src/resolve/walk.rs` `is_resolvable_call_head` 1 site `[Ax1]` → `[Bx1]`. A raw symbol spelling had reached the keyword rungs. The bare name now passes through `canonical_identity` first (a slash-less name is unchanged, so `is_retired("Some")` still hits). The row is gone. Ledger 147 → 146.

Green, after those three cures: `.floor/2026-10-02T18-58-54Z`

```
Summary [ 400.883s] 6374 tests run: 6374 passed (28 slow), 24 skipped
```

RC=0. The brief's baseline was 6368 passed. The six new probes are the difference. 24 skipped, unchanged.

`cargo clippy --release --all-targets -- -D warnings` RC=0.

STOP-2 did not fire. Quoted `(foozle 1)` checks and runs. The keyword unquote escape refuses `foozle`.
