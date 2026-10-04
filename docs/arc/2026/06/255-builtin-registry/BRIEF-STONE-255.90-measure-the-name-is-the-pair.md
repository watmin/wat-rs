# BRIEF — STONE 255.90: measure — the name is the pair

**Drawn 2026-10-04 against `main` @ `ddb1261b5`.** **Executor: grok via pulsare, working solo.** A **measuring** stone:
no cure, no edit to `src/`, `crates/`, `wat/`, `tests/` or the corpus. Its output is a committed instrument and a SCORE.
Commit locally on `main` (`git add -- <paths>`, never `-A`); **do not push**. 255.89's commits stay local and untouched.

## The ruling (builder, 2026-10-04)

> *"the end state must be — `wat.core//` :=> `{:namespace wat.core :name /}`,
> `wat.core.Option/expect` :=> `{:namespace wat.core.Option :name expect}` — there are no strings. if we need to
> string-ify a symbol, its a format call to use `"{namespace}/{name}"`. the first slash is the partition … the double
> colon syntax is being annihilated."* — and: *"flat is being deleted as its pointless … its just ns / name."*

A symbol is `{:namespace <base symbol> :name <base symbol> :scope <hygiene scopes>}`. `(wat.core/defn u/some-name …)`
is `{u, some-name, {}}`; `(wat.core/let [x 42] …)` is `{$bound, x, {}}`; a macro-introduced symbol carries scopes.
The end state, as three gates:

1. **The reader refuses `::`** (a keyword holding `::` cannot be read).
2. **Identity is a type:** an interned `Name {namespace, name}`, an `Identifier` = `Name` + scopes, with `Eq`/`Hash`
   over those fields only; every registry keyed by it. No name is ever a map key as a string. A string exists only from
   the printer: `"{namespace}/{name}"`, or the bare name for `$bound`.
3. **No `::` survives** in `src/`, `crates/`, `tests/`, the corpus or the goldens.

This stone measures the distance to gate 2 (and the literal half of gate 3), so the work can be cut into stones.

## What is known (orchestrator, measured at `ddb1261b5`, by text: the instrument's job is to replace these)

| | |
|---|---|
| `Identifier` | `crates/wat-reader/src/identifier.rs:111`: `ns`, `name`, `flat`, `scopes`. **`PartialEq` (`:128`) and `Hash` (`:135`) use `flat` + `scopes`: the spelling decides equality.** `as_str` returns `flat` (`:238`); `leaf`/`path` split `flat` on `::` (`:261`, `:266`, `:346`, `:352`) |
| registries | `src/value/symbol_table.rs:33` `functions: HashMap<String, Arc<Function>>`, `:42` `unit_variants: HashMap<String, EnumValue>` |
| maps keyed by text | ~384 `HashMap/BTreeMap/HashSet/BTreeSet<String \| Arc<str> \| &'static str …>` in `src/` (not all hold names) |
| `::` literals | `src/` 6,473 in 215 files; `crates/` 238 in 18; `tests/` 3,459 in 647. Rough split of `src/`: ~622 in `#[wat_*(…)]` attributes, ~874 on match-arm lines, ~253 in comparisons, ~3,100 heads inside embedded wat |
| the door | `canonical_identity` (now `crates/wat-reader`), 133 call sites; `fold_member_twin` (255.89 AMEND-3) decides type-ness by capitalization: **ruled out**, recorded here, reverted by a later stone |
| 255.88 | 504,923 `as_str` calls on one probe run |

## The work

Build **`src/bin/name_census.rs`** (a bin, not a test: it reports and does not gate; the heresy ledger's `syn` walk is
the shape to copy, `tests/lint/keyword_heresy_ledger.rs`). It parses every `.rs` under `src/` and `crates/` (and,
separately reported, `tests/`), and writes `docs/arc/2026/06/255-builtin-registry/name-census.tsv` plus a summary. Each
table below is one section of the SCORE, every row from the instrument, every count with its denominator:

1. **The `::` literals.** Every string literal whose text holds a `::` name, classified by **what the code does with
   it**, from the syntax tree, not the line:
   - `REG` a registration (an attribute argument, an insert into a registry, a declared name)
   - `DISPATCH` a match arm or `if` chain that selects behaviour by a name
   - `CMP` an equality, `starts_with`/`ends_with`/`strip_prefix`/`contains`, or a set-membership test
   - `BUILD` a `format!`/`concat` that **assembles** a name (the class that mangled generics in arc 278)
   - `MSG` text inside a diagnostic or panic whose content is not compared anywhere
   - `WAT` inside embedded wat source (5c-iv's)
   - `OTHER`, each listed

   Counts per class, per file (top 20), and per namespace prefix.
2. **The maps.** Every map or set whose key type is text: does it hold **names** (keys reach it from a registration, a
   symbol, or the door) or other text (paths, labels)? Per map: `file:line`, key type, NAME/OTHER, and the evidence.
3. **`Identifier`'s string surface.** Every caller of `as_str`, `leaf`, `path`, `receiver`, `flat` (and the free fns
   `leaf`/`path`), classified **IDENTITY** (the string becomes a key, is compared, or is passed to the door) or
   **PRINT** (it only reaches output). Counts per class and the top 20 files.
4. **Where `Identifier` equality is relied on.** Every `==` on `Identifier`, every map/set keyed by it, every
   `HashMap<Identifier,…>`: what would change if equality became `(namespace, name, scopes)` instead of `(flat,
   scopes)`. Find the inputs where the two disagree (two spellings, one pair, e.g. `:a::b/c` vs `a.b/c` in the
   transition), and say whether any site depends on them being **unequal**.
5. **The door's callers.** The 133 `canonical_identity` call sites (and `fact_class_key`, `ns_to_wat_path`,
   `canonical_type_key`): what each does with the string it gets (IDENTITY/PRINT, as in 3).
6. **The hot paths.** On the fuzz deftest (`wat-tests/rete/differential-fuzz.wat`, 89.3 s on `.floor/2026-10-04T02-26-49Z`
   against 71.2 s at 255.88): count calls of `canonical_identity`, `as_str`, and `String` allocations made by name
   handling (a counter behind a feature, in a scratch build, removed before commit). Name the top call sites. This is the
   cost the interned pair would remove.

**Prove the instrument:** plant one literal of each class in a scratch file and show each is classified correctly; show
one known `IDENTITY` caller and one known `PRINT` caller land right. A classifier that has never been seen to tell two
classes apart is not an instrument.

## Then

**A proposed split** of gate 2 into stones from the numbers (for example: the `Name` type and `Identifier` equality; the
registries; the `REG`/`DISPATCH` literals by a recorded Rust-literal codemod; the `CMP`/`BUILD` sites; `flat` deleted
last). Each stone is named with its size and its gate. The split is a proposal for the orchestrator to weigh, not a
plan to start.

## Gates

| what | how | expected |
|---|---|---|
| no edits | `git diff ddb1261b5 --stat` | only the bin, the TSV, the SCORE |
| the instrument | its planted-literal proof | every planted class classified right |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | rc 0 |

No floor is needed (nothing under test changed). **STOP-1:** a count needs a ruling to classify (a literal or map that
is a name in one reading and not in another). List it and keep counting the rest. A STOP means STOP.

## Doctrine

`holon/CLAUDE.md` binds you. Capture `rc=$?` on the next statement. Never wait with `pgrep -f`. Never write a number,
file:line or example you did not measure; the counts above are text-matched and are to be replaced, not confirmed. If
this brief contradicts the code, the code wins: say so. Write `SCORE-STONE-255.90-measure-the-name-is-the-pair.md`
beside this brief, commit it, **do not push**.
