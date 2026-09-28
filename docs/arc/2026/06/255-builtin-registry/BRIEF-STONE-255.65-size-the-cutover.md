# BRIEF — STONE 255.65: MEASURE — size the Clojure cutover and order it into landable stones (nothing lands but the SCORE)

**Drawn 2026-09-27 against `main` @ `45da20c8f`.** **Executor: grok, via pulsare.** **MEASUREMENT ONLY.** Probes in a
`git worktree` under the session scratchpad if needed; never edit the main tree. Write the SCORE on `main`, commit
**only** it, then `pulsare_yield kind=scored`.

## Why (builder, 2026-09-27)

*"we are preparing for a serious perf upgrade — we need the clojure cutover to happen."* Layers 255.60–.64 walked the
wall and found the shape. The builder then ruled the end state, which is bigger than one layer. This stone sizes each
part, finds what each part touches, and proposes **an order of stones that each land green**.

## The end state (builder's rulings, 2026-09-27; `FINDING-the-shape-of-a-declared-signature.md` § "the cutover's type namespace", superseded where below says so)

1. **Names:** every `::` keyword that is a name becomes a faithful-Clojure symbol (arc 251.8d). The `::` wall makes the
   old spelling unspeakable.
2. **Homes:** `wat.type/` holds **only the hard primitives**: `i64`, `f64`, `String`, `bool`, `keyword`, `Vector`,
   `HashMap`, `HashSet`, `List`, `Tuple`, `AST` (was `:wat::WatAST`), and the like. **Other typed things live in their
   own homes:** `wat.time/Instant`, `wat.time/Duration`, `wat.uuid/UUID`, `wat.holon/HolonAST`; declared utilities
   `wat.core/Option`, `wat.core/Result`. **`wat.core` is utility, never a type home** (*"we put types in core because we
   didn't think this through"*).
3. **The new spelling is the canonical key.** `:wat::core::i64`, `:wat::WatAST`, `:wat::time::Instant` die. The parser
   reads `wat.type/i64` straight to the key `wat.type/i64` (**C1**). No second spelling survives behind it, so no
   comparison can ever see two spellings of one type. `type_denotation` (31 call sites) exists to reconcile two
   spellings; after C1 there is one.
4. **Position decides:** `(wat.type/Vector :- [wat.type/i64])` is the type in a type position and an **empty vector**
   anywhere else; `(wat.type/Vector :- [wat.type/i64] 1 2 3)` equals `[1 2 3]` (measured today with the old head:
   `true`). **After the cutover `(wat.core/Vector …)` is an unknown function.** The same holds for `HashMap`/`HashSet`/
   the other constructors that are types.
5. **F1 wall:** a primitive spelled anywhere but `wat.type/` in a type position is refused.

## Measure (counts and file:line; say measured or inferred)

1. **The substrate's own keys.** The `::` keyword literals in `src/` and `crates/` (255.60: 5,745 exact literals, 209
   files). Class them by what they key: a type (primitive → `wat.type/…`; time → `wat.time/…`; …), a function or form
   head, a variant, a namespace or prefix (`:wat::`, `:rust::`, reserved prefixes), error/remedy text, test expectations.
   For each class, what the new canonical spelling is under (2)/(3), and whether a mechanical rewrite is exact.
2. **Where canonical identity lives today:** `canonical_identity`, `type_denotation`, `canonicalize_type_kw`,
   `ns_to_wat_path`, `compose_variant`, `Identifier`, the EDN `#wat.core/…` tags, `format_type`, the registries' keys,
   `Value` class names (`AggregateValue.class`, `"wat::core::Vector"`-style strings). What C1 changes in each, and
   which become dead once there is one spelling.
3. **Constructors that are types:** every call-shaped `(:wat::core::Vector …)`, `HashMap`, `HashSet`, `PersistentVector`,
   `PersistentMap`, `List`, `Tuple` construction in the corpus and the stdlib (255.62: 4,107 for `Vector` alone). Which
   become `[…]` / `{…}` / `#{…}` literals and which become the typed form. And the Rust side that registers
   `:wat::core::Vector` as a callable.
4. **Homes that move:** `Instant`/`Duration` (`:wat::time::` → `wat.time/`), `Uuid` (to `wat.uuid/UUID`, including the
   case change, and where `Uuid`/`UUID` is spelled now), `HolonAST`, and any other type registered under `:wat::core::`
   that is not a hard primitive (list them; each needs a home; name the obvious one, and mark any that needs the
   builder).
5. **The runtime and wire surface:** EDN tags and printed type names (`#wat.core/Span`, `format_type` output,
   `type-of`/reflection results, error messages, goldens). What changes in what users and files **see**, and which
   `.edn` goldens pin it.
6. **The bootstrap:** the codemod lives in stdlib (`wat/fix.wat`); the stdlib is `include_str!`'d; the canonical keys
   are in Rust. Lay out the order that keeps a buildable binary at every step (255.63/.64: rules must be in the
   **unconverted** `fix.wat` before the conversion).
7. **Other branches.** List the branches with commits not on `main` (`git branch -a`, `git log main..<b> --oneline | wc
   -l`), and for the largest (`sns-sqs`, `reason/little-wat-findings`), estimate what a landed cutover means for their
   merge (files both touch). Report; do not merge.

## Output

1. The sizes, per part.
2. **A proposed sequence of stones**, each landable **green** on its own (floor, clippy, census, delta), with its
   dependencies, its expected blast, and its gate. The walls land last, when nothing screams. Say which stones are pure
   codemod, which are Rust, and which need a builder ruling (list those questions plainly).
3. The risks: a place where one spelling must survive (wire compatibility, persisted data, the other branches).

## Doctrine

- Read, measure, cite file:line. Say measured or inferred for every claim. A text count says it is a text count.
- Capture `rc=$?` on the **next** statement.
- **If this brief contradicts the code, the code wins. Say so plainly.**
- **STOP-1:** where the sequence needs a design choice the rulings above do not make, list it as a question for the
  builder, with the measured consequences of each answer, and do not choose.
- Write `SCORE-STONE-255.65-size-the-cutover.md` beside this brief and commit **only** it. Remove any worktree. **Do not
  push.**
