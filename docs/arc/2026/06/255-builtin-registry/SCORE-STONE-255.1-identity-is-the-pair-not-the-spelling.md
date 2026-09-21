# SCORE — STONE 255.1: a name's IDENTITY is the (namespace, name) PAIR, not its spelling

Branch: `main`. **Committed, not pushed.** Drawn against `main` @ `779b084e3` (draw `748a64160`).
Parent: `BRIEF-STONE-255.1-identity-is-the-pair-not-the-spelling.md`.
Floor / workspace clippy / `census.sh --diff`: orchestrator's row (not run).
Crate clippy run here. 8d-ii and 8d-iii not started. **No `.wat` converted.**

## Gate: the 179-file delta

Same sampling as the FINDING: every 12th path of the 8d-i census `files.txt` (2145/12 = **179**).
Originals are the **live tree** (the census `tree/` copies are already converted — comparing
`tree/` to `tree-after-1` is converted-vs-converted and reports a false 0). Converted copies:
`/tmp/8d-i/census/tree-after-1`. Timed one file first: converted counter-actor **0.15 s**.

| | FINDING baseline (old binary) | this stone |
|---|---|---|
| originals clean | **161 / 179** | **161 / 179** |
| of those, still clean after conversion | 57 | **60** |
| ⛔ **REGRESSIONS** | **104 (65%)** | **101 (63%)** |

The 161 matching is the same spread. The count moved **104 → 101**. Files that only had gap 1
(declaration-name `MalformedDecl`) and no later gap become clean; files that had gap 1 **and**
gap 2/3a/3b stay red under the later error. The regression **count** is not the residue
**kind**.

### Which gaps closed, which remain (classified on the 101)

After identity + the name-as-data verb (`keyword::to-string`) + nature-root identity:

| kind | n | gap |
|---|---|---|
| **MalformedDecl** on a **declaration name** (`name must be a keyword`) | **0** | **gap 1 CLOSED** |
| `UnresolvedReference` | 68 | **gap 3a / 3b remain** |
| `MalformedDecl` on a **later slot** (`:nature` was one; now `expected -> … got keyword` on surface methods, etc.) | 13 | residue: Keyword-only **non-name** slots (`->` vs `:-`) |
| `ReteCheckErrors` / `CheckErrors` / OTHER | 14 | not identity |
| `UnknownNamedType` | 3 | residue after a macro now expands |
| `ProgramBodyEvalFailed` | 3 | **gap 2 almost closed** (was 54 before `keyword::to-string` identity) |

Demonstration, FINDING's own file `wat-tests/counter-actor-proof-process.wat`:

```
live original : --check → rc=0
converted     : no longer MalformedDecl "name must be a keyword; got symbol"
                now UnresolvedReference (17 refs) — gap 3a: `counter/Request`,
                `wat.enum/Pure`, `counter/dispatch` walked as references
```

Converted `examples/console-demo/wat/main.wat`: **rc=0** (nil-unify + wat.type members).

## The door — one key, both spellings

`canonical_identity` in `src/edn/render.rs` (next to `ns_to_wat_path`):

- Clojure `wat.core/Option` and dotted-keyword `:wat.core/Option` → `:wat::core::Option`
- Rust-scheme keywords **with `::`** are already the key, **including**:
  - variant paths (`:wat::kernel::StdIn.read-frame::Request` — `.` is the enum/variant separator)
  - surface-op aliases (`:wat::kernel::StdOut::write/Request` — `/` is Type/method in the leaf)
  - unprefixed parametric heads (`wat::kernel::StdOut::write/Request` → prefix `:`, do not
    split on `/` as if clojure)

Do **not** clojure-round-trip a rust-scheme path: `.` and `/` both become `::` and freeze
dies with `UnknownNamedType` on `StdOut::write::Request`. That was the freeze break on the
first land; the early-return is `s.contains("::")`, not `::` AND no `/`.

`parse_declared_name` accepts Keyword **or** Symbol and stores the identity. TypeEnv
`contains` / `get` / `classify` key on identity. `wat.type` is **not** rewritten to
`wat.core` at the identity door.

Compile-time sibling in `crates/wat-source-derive` (cannot import `wat` — cycle). Dual-read
of defenum/defrecord/defstruct/typealias heads and names. Converted `runtime-meta.wat`
`(wat.core/defenum wat.runtime/Kind …)` is the 8d-ii compile door.

## `wat.type` is a real namespace

- `register_builtin_leaf(":wat::core::X")` also inserts `:wat::type::X`. `nil` is unit
  `Tuple([])`, so extra `register_use_declared_leaf` for both spellings; parse stores
  identity and collapses denotation `:wat::core::nil` → `Tuple([])` so unify matches.
- `contains(":wat::type::i64")` and `contains("wat.type/i64")` are **true**.
  `contains("wat.type/nope")` is **false**.
- Non-member Display: `"not a member of wat.type: …"` — probe
  `wat_type_non_member_is_refused_as_not_a_member`.
- The three `strip_prefix(":wat::type::")` sites collapsed to `type_denotation`.
  Remaining production hit: **1** (`edn/render.rs` `type_denotation`). Gate
  `no_fourth_wat_type_strip_prefix` allows ≤3.
- Unify Path: `p1 == p2` **or** denotation equal, so `wat.type/i64` unifies with
  `wat.core/i64`.

### One position, not two — **residue**

`wat.type/Vector` **annotates** (`--check` rc=0). In **call** position
`(wat.type/Vector 1 2)` is `UnresolvedReference` `"call head — not a builtin, not a
registered function"`. `is_resolvable_call_head` does not consult TypeEnv membership.
Identity on that predicate was tried and reverted (freeze risk). Not smuggled back.
A type that is a member in annotation and absent as a call head is the ruling's
two-position defect, still open.

## Gap 3a — `:restricted-to` resolution **did not close**

Identity makes a symbol name the same TypeEnv key. It does **not** stop the resolver
walking a name-position symbol as a reference. Converted counter-actor is the witness
(`counter/Request`, `wat.enum/Pure` → `UnresolvedReference`). The whitelist FINDING
(exact FQDN entry must exist) is the same shape. Builder: validation pass out of
scope; resolution half **not** closed here. 8d-iii.

## Converted stdlib, without converting `wat/`

`--check /tmp/8d-ii/tree/wat/core.wat` as a **user** file: **`DuplicateType
:wat::kernel::Location`** — the symbol declaration name is the **same key** as the
embedded keyword stdlib. Before this stone that file died `MalformedDecl name must be
a keyword; got symbol`. Embedding still waits on 8d-ii (and on gap 3a/3b). Not applied
to `wat/`.

## Test count

Predicted +6 from the diff; `cargo nextest list --release -p wat`: **5314**.

| test | |
|---|---|
| `types::tests::stone_255_1_wat_type_has_members` | lib |
| `types::tests::stone_255_1_declared_symbol_name_is_the_same_key` | lib — `(wat.core/defenum t/Color wat.enum/Pure :Red :Blue)` → key `:t::Color` |
| `restriction_entry_match_tests::identity_is_the_pair_not_the_spelling` | lib |
| `probe_255_1_identity::{wat_type_i64_annotates, wat_type_non_member_is_refused_as_not_a_member, no_fourth_wat_type_strip_prefix}` | `--test types` |

`cargo test --release -p wat --test kernel wat_arc198_def_restricted` — **10 passed**.

## Walls I ran (not the floor)

- `cargo clippy --release --all-targets -p wat --offline -- -D warnings` — **exit 0**.
- Timed one `--check` first; never two wat CLIs.
- Identity / probe tests above: pass.
- Floor + workspace clippy + `scripts/replay/census.sh --diff`: **not run** (orchestrator).

Do not push. Do not start 8d-ii. Do not convert a `.wat` file.
