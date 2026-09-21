# SCORE — STONE 255.8: a namespace that is also a type

Branch: `main`. **Committed, not pushed.** Drawn against `0d6da5f19`.
Parent: `BRIEF-STONE-255.8-a-namespace-that-is-also-a-type.md`.
**Not one live `.wat` converted.** `wat/` restored after every overlay (`git status -- wat/` empty).

## The door

A type annotation does not reach `reconstruct_call_path`. The annotation slot
rewrites a reference symbol with `ns_to_wat_path` (always `::`). Call position
stays on the join. `reconstruct_call_path`'s body is unchanged. The comment at
`src/types.rs` is unchanged: identity stays `::`; `my.Counter/Req` is not a method.

## Non-vacuity, one test

`tests/resolve/probe_arc255_8_namespace_is_also_a_type`:

- `(:wat::core::defrecord :my::Journal::Req …)` plus the annotation — CLEAN
- `(wat.core/defrecord my.Journal/Req …)` plus the annotation — CLEAN
- `wat.core.Option/expect` still resolves as a member and returns 7

Passed inside the green floor.

## Converted stdlib starts

64/64 copies from `/tmp/8d-ii-dry`, overlaid, `find wat -name '*.wat' -exec touch`,
release build showed `Compiling wat`. `./target/release/wat --check` of

```wat
(wat.core/defn user/main [] :- wat.core/i64 1)
```

returns only `MainSignatureError` (`:user::main` must return `:wat::core::nil`).
The same probe against the live rust-scheme stdlib returns that same error.
No `CheckErrors` and no `UnresolvedReferences` in `wat/`. Overlay trap restored
`wat/` and rebuilt. `wat_dirty_after 0`.

## 179-file delta — this tree, re-converted

Every 12th of the 8d-i census, 179 files. Codemod
`wat-scripts/fixes/to-faithful-clojure.wat` on `/tmp` copies, 31.94 s, rc 0.
Originals are the live tree. Baseline **49**.

| | 255.7 | this stone |
|---|---|---|
| originals clean | 161 / 179 | **161 / 179** |
| converted failures | 67 | **36** |
| **new** (orig clean, conv broken) | **49** | **18** |

**31 closed against 255.7. 1 opened:** `wat/holon/Ngram.wat`.
Originals that already failed: 18, all still fail. None of those 18 went green.

New kinds: MalformedDecl 8, UnresolvedReferences 4, ProgramBodyEvalFailed 3,
CheckErrors 1, DuplicateMacro 1, DuplicateType 1.

## Gate

| | |
|---|---|
| floor | `Summary [ 321.390s] 5959 tests run: 5959 passed (9 slow), 22 skipped` exit 0. `.floor/2026-09-21T23-40-53Z/` |
| clippy | `cargo clippy --all-targets --workspace -- -D warnings` exit 0 |
| census | `census-diff: no STOP-8` against `.census/2026-09-21T10-08-58Z.txt`. This run: `.census/2026-09-21T23-47-29Z.txt` (2201 files) |

An earlier floor the same day was red (66 failed, `.floor/2026-09-21T23-21-45Z/`).
That run was not repeated. The red was two real defects, fixed, then this floor.

## Fifteenth correction — the brief named the annotation

Startup of a converted stdlib is not the annotation slot. The slots that were
keyword-only, and what they do now:

- Boundary heads (`match`, `quote`, `quasiquote`, `matches?`, `make-rule`) are
  rewritten to the keyword the checker matches. Leaving `(wat.core/match …)` as
  a symbol made every arm infer as a value vector (`:wat::core::vec` is the
  vector-literal error's hardcoded callee).
- `detect_match_shape` reads arms through `parse_match_arm`, which already
  accepts a symbol variant head.
- Stdlib `defclause` / `extend-type` / scalar `def` reach runtime registration
  when the head is a symbol (`head_fqdn`). A keyword-only filter left the
  0-param defclause stub in place.
- `extend-type`'s type and protocol slots accept a symbol through
  `parse_type_node` (`ns_to_wat_path`).
- A defclause return slot accepts a namespaced symbol or a type-parameter
  letter (`T`). A bare value symbol (`n`) is still rejected.
- extend-type and defclause bodies are normalized after they are stored.
  The earlier pass never saw them.
- `wat.type/X` and `:wat::core::X` are one type at `is_subtype`, the
  equatable and orderable gates, and `extract_lazyable_elem`. `unify` already
  denotated. The gates did not, so a converted `wat.type/i64` failed a check
  whose message printed `:wat::core::i64`.
- A surface method tail that `decompose_variant` recognizes is not a method.
  `Store/ScanResponse.Success` was being accepted as a method of `Store`.
- Rekey of a stored `::method` to `/method` skips a variant path and a primed
  name (`IncrementRequest'`). Primed constructors are the positional form and
  stay `::`.
- `:wat::core::seqable->stream` is registry membership. It has a check arm and
  a runtime arm and no `TypeScheme`, so the scheme fold could not see it.

## What a door cannot express

A keyword that already contains both `::` and `/` becomes a symbol whose
`ns_to_wat_path` writes `::`. `reconstruct_call_path` puts the `/` back only
when the parent is a known type. Two spellings the registry holds anyway:

- `:wat::kernel::stdin-svc/start` (and stdout, stderr). `defservice` emits the
  slash. `stdin-svc` is not a type, so the symbol reconstructs to `::start`.
- `:wat::core::Fault/of`. The macro is stored with `ns_to_wat_path` (`::of`).
  The call reconstructs to `/of` because `Fault` is a type.
- `:rust::cache::Lru/new`. The shim registers the slash. The symbol
  reconstructs to `::new`, which is the retired spelling.

The other join is asked of the registry (a function, a defclause value, a
macro, an intrinsic entry, or a rust-deps symbol). It is not a second test
inside `reconstruct_call_path`. A retired spelling is not kept when the other
join is the binding. `canonical_identity` still does not fold `/` into `::`.
