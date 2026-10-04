# WEIGH — STONE 255.91: the name is the pair — ACCEPTED

**Executor: grok via pulsare, solo.** Commits `b928f45b4` … `4db92dc4e`, SCORE `53ae93a7c`. Weighed by the orchestrator
on 2026-10-04.

## Re-run by the orchestrator at `53ae93a7c`

| row | result |
|---|---|
| release floor | `.floor/2026-10-04T04-53-19Z`: **6415 passed / 24 skipped**, exit 0, doc-link exit 0, 382.6 s |
| test-name set | against `.floor/2026-10-03T13-09-11Z`: **MISSING 0**, ADDED 3 (the stone's own tests) |
| cost | the fuzz deftest **71.9 s** (71.2 s at 255.88) |
| clippy | exit 0 |
| ignores | 18 |

## What landed

- **`Name { namespace, name }`** in `wat-reader`, compared by content with a pointer fast path, no interner; `Display`
  is the one stringification. **`Identifier` equality is `(pair, scopes)`**; `flat` is a print cache.
- **`Name::from_keyword`**, the one transition translation: 11,105 distinct keywords from the parked conversion, **0
  disagreements**; 305 rows of every shape committed as a fixture.
- **The slash rule's local-first resolution is one function** (`local_spelling` / `same_local`), used by `substitute`,
  `env_key` and `scope_divergent_binder`; a slashed binder and its body reference share an env key (tested).
- **R-a made real in one place:** the macro registry's keyword/symbol cross arm compares `Name`s, so
  `:wat::core::Option/expect` and `wat.core.Option/expect` are one name, with no case rule.
- The census is an example; `syn` is back in `[dev-dependencies]`.

## Owed by stone 2 (255.92)

1. **A second copy of the translation.** `from_keyword` is a port of `wat_keyword_to_clojure_symbol`
   (`src/edn/render.rs`), the function the recorded converter calls. That is why the agreement is 0 (by construction,
   which is what the brief asked), and it is also two copies that can drift. One definition, in `wat-reader`;
   `render.rs` calls it.
2. **A bare `is_ok()`.** `duplicate_defmacro_symbol_spelling_is_the_same_macro` now asserts `member.is_ok()`. It passes
   equally if the second macro registered as a **separate** macro. It asserts the claim: one macro in the registry,
   and the second registration a no-op.
3. `local_spelling` borrows `flat` for a reference; when `flat` goes (stone 5) it is the pair's `Display`.

## Verdict

Accepted and pushed. Next: 255.92, the registries keyed by `Name`.
