# BRIEF — STONE 255.92: the registries are keyed by `Name`

**Drawn 2026-10-04 against `main` @ `4d4087f03`.** **Executor: grok via pulsare, working solo** (it runs the floor).
Commit locally on `main` (`git add -- <paths>`, never `-A`; `git status` clean before the floor you report);
**do not push**.

## Where this sits

The builder's ruling (255.90's brief): a symbol **is** `{namespace, name, scope}`; no string is an identity. 255.91
built `Name` and pair equality (`WEIGH-STONE-255.91-…`). This is stone 2 of 5: **no registry is keyed by a name
string.** Next: (3) the `REG`/`DISPATCH` literals by a recorded Rust-literal codemod, (4) `CMP`/`BUILD`, (5) `flat`
deleted; then 5c-iii re-run.

## The registries (measured)

| registry | today |
|---|---|
| `SymbolTable` (`src/value/symbol_table.rs`) | `functions: HashMap<String, Arc<Function>>` (`:33`), `unit_variants` (`:42`), `runtime_def_values` (`:108`), `BindingMetadata` (`:16`, the outer map); `get(&self, path: &str)` (`:272`) |
| `TypeEnv` (`src/types.rs:1077`) | `types: HashMap<String, TypeDef>`; `get(&self, name: &str)` (`:1320`) |
| `MacroRegistry` (`src/macros/registry.rs:54`) | `macros: HashMap<String, MacroDef>`, `symbol_alias: HashMap<String, String>` |
| `CheckEnv` (`src/check/env.rs:95`) | `unit_variant_types: HashMap<String, TypeExpr>` |
| `UseDeclarations` (`src/rust_deps/mod.rs:278`) | `declared: HashSet<String>` (ruled a name registry in 255.90's WEIGH) |
| closure capture (`src/closure_extract.rs:682-693`) | `captured_deps`, `captured_types`, `captured_macros` |

The hot paths (255.90 §5, one fuzz deftest): `canonical_type_key` (`src/types.rs:374`) **71.1M** calls, `TypeEnv::get`
(`:1346`) 6.0M. Today's fuzz deftest: **71.9 s** (`.floor/2026-10-04T04-53-19Z`).

## The work

1. **Owed by 255.91, first:** (a) **one translation**: `wat_keyword_to_clojure_symbol` (`src/edn/render.rs`) and
   `Name::from_keyword` are two copies of one function. Keep one definition in `wat-reader`; `render.rs` calls it.
   (b) `duplicate_defmacro_symbol_spelling_is_the_same_macro` asserts its claim (one macro in the registry; the second
   registration a no-op), not a bare `is_ok()`.
2. **Every registry above is keyed by `Name`.** Its lookup takes `&Name`. A name arriving as text during the transition
   (a keyword literal in Rust, a keyword in source) becomes a `Name` **once, where it enters**: `Name::from_keyword`
   for a keyword, the `Identifier`'s pair for a symbol. **The compiler is the census:** change the key type and follow
   every error; each map that a `Name` now flows into is in scope, and each one is named in the SCORE with what it
   holds. A text-keyed map that turns out not to hold names (a file path, a label) stays and is named too.
3. **The door's role shrinks to the transition.** `canonical_identity` / `canonical_type_key` / `fact_class_key` stop
   producing registry keys. Where a caller still needs one of them after this stone (a printed message, a stone-3
   literal not yet converted), say which and why; it is the residue stone 3 sizes.
4. **The hot paths:** with the key a `Name`, `canonical_type_key`'s 71.1M string builds have no reason to exist on a
   lookup. Report the fuzz deftest's time; it is not to rise above today's 71.9 s, and the expectation is that it falls.

**A type key is a name, not a rendered type.** If any registry today is keyed by a rendered parametric form (text
holding `(`, `<`, `:-`), that key is not a `Name`: list it (STOP-2) rather than force it into one.

## Gates

| what | how | expected |
|---|---|---|
| no string-keyed name registry | the registries above and every map the compiler led to | each keyed by `Name`, or named as non-name text with its evidence |
| release floor | `scripts/floor.sh`, foreground, nothing else running, `git status` clean | all passed. **Test-name set against `.floor/2026-10-04T04-53-19Z`: MISSING 0** |
| cost | the fuzz deftest on your floor | ≤ 71.9 s, reported |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | rc 0 |
| ignores | the SEAM's ledger command | 18 |
| census | `cargo run --release --example name_census` | the MAP section's NAME rows fall; report before and after |

## Reds and STOPs

- A red caused by this stone's own change is the work: capture it **verbatim**, cure it, run a **new** floor. Any
  other red is a STOP. Never re-run unchanged code for a green.
- **STOP-1:** keying by `Name` changes what an existing program resolves (two keys that were distinct strings and are
  one pair, or the reverse). Quote it and STOP on it.
- **STOP-2:** a registry key that is not a name (a rendered parametric type, a composite). List it and STOP on it;
  finish the rest.
- **STOP-3:** the cascade reaches beyond the registries into the `REG`/`DISPATCH` literal sites in bulk (hundreds of
  call sites needing a `Name` built from a literal). Convert none by hand; report the count and STOP. That is stone 3's
  recorded codemod. A transitional constructor at the registry boundary is acceptable; a hand sweep is not.
- A STOP means STOP.

## Doctrine

`holon/CLAUDE.md` binds you. No string compare stands in for identity; no name is built by `format!` except `Name`'s
`Display`. No time limit is raised. Capture `rc=$?` on the next statement. Never wait with `pgrep -f`. Never write a
number, file:line or example you did not measure. If this brief contradicts the code, the code wins: say so. Write
`SCORE-STONE-255.92-the-registries-are-keyed-by-name.md` beside this brief, commit it, **do not push**.
