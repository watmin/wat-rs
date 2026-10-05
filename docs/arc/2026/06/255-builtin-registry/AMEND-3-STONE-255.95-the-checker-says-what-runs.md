# AMEND-3 — STONE 255.95: the checker says what runs

**Drawn 2026-10-05.** **Executor: grok via pulsare, working solo** (it runs the floor). Continues at `13db3a39a`. Commit
locally on `main`; **do not push**.

Amendment 2 is accepted in substance. My re-run at `13db3a39a`: floor `.floor/2026-10-05T07-34-12Z` **6421 passed / 24
skipped**, doc-link 0, test-name set against `.floor/2026-10-04T07-52-21Z` **MISSING 0**, ADDED 4; clippy 0; ignores
18; fuzz six-and-six 31.95 s against 31.99 s (grok). `Value::Symbol`, keywords holding pairs, `$bare`, `<` as a name
character, the sentinel respelled, `defservice` passing names as symbols resolved by `apply` across a process boundary:
all landed.

**One defect, cured before acceptance: the checker states a type the runtime does not return.** `compose-variant`
returns `Value::symbol` (`src/reflect/verbs.rs:1980`) and is typed `:wat::type::keyword` (`src/check.rs:3356`);
`variant-parent-of` returns a symbol and is typed `Option` of keyword (`:3308`). The floor is green because nothing
type-checks a consumer of those results.

1. Type both as what they return: `wat.type/symbol`, and `Option` of it.
2. **Find the class, not the two:** every verb this stone changed from returning a keyword to a symbol (reflection,
   `metadata-of`'s `:name`, `TypeInfo.name`, `Service.name`, and any other): its checker scheme against what its eval
   returns. List each, and cure each mismatch.
3. **A gate that would have caught it:** a test that, for each of those verbs, type-checks a program binding the
   verb's result to a function declared to take exactly the checker's stated type, and then **runs** it. A checker that
   says `keyword` while the runtime hands a `symbol` fails at run time there. Prove it red on the current tree first.

Floor, test-name set against `.floor/2026-10-05T07-34-12Z` (MISSING 0), clippy, ignores 18. A red caused by this
amend's own change is captured verbatim, cured, and followed by a **new** floor; any other red is a STOP. Append to the
SCORE, commit, **do not push**.
