# Arc 242 — `lexeme-role-doctrine`

**Status:** OPEN (spawn-block child of arc 241 per `feedback_spawn_block_winding`; arc 241 paused at Stone 241.12 STRIKE-READY; arc 241 resumes after arc 242 closes).

**Bar:** REMARKABLE. The arc INSCRIBES foundational law governing all future type names in the substrate.

## The two doctrines this arc inscribes

### Doctrine 1 — bare lexeme = value; keyword lexeme (`:wat::core::*`) = type

The lexical FORM signals the ROLE:
- Bare lexeme (no `:` prefix) is a **value** — `42`, `true`, `false`, `"hello"`, `nil` (post-arc)
- Keyword lexeme (`:wat::core::*` namespace-qualified) is a **type** — `:wat::core::i64`, `:wat::core::bool`, `:wat::core::nil`, `:wat::core::String`

The `:` prefix is "name reference" syntax; the namespace says which name. Value lexemes carry no prefix because they ARE the value, not a reference to one.

### Doctrine 2 — scalar types lowercase; non-scalar/container types PascalCase

Within the type-namespace, case signals structure:
- **Scalar (single atomic value)**: lowercase — `i64`, `f64`, `bool`, `char`, `nil`, `keyword`
- **Non-scalar (sequence/container)**: PascalCase — `String` (sequence of chars), `Vector<T>`, `HashMap<K,V>`, `HashSet<T>`, `Tuple<...>`

Rationale: String is structurally `[Char]` (a sequence of chars; carries length, indexing, iteration). PascalCase tracks the container-ness. Char is the scalar atom; String is the container of atoms.

## Combined

| Lexical form | Role | Example |
|---|---|---|
| Bare lowercase | scalar value | `42`, `true`, `nil` |
| Bare PascalCase | (reserved; no current use) | — |
| Keyword lowercase (`:wat::core::*`) | scalar type | `:wat::core::i64`, `:wat::core::nil` |
| Keyword PascalCase (`:wat::core::*`) | container type | `:wat::core::String`, `:wat::core::Vector` |

## Stones

Single stone — the work is tightly coherent:

- **Stone 242.1** — bare `nil` lexer support + `:wat::core::nil` value-position migration + `:wat::core::Char` → `:wat::core::char` HARD CUT + doctrine inscription
- **Stone 242.2** — INSCRIPTION (closes arc 242; arc 241 resumes at Stone 241.12)

## Concrete cleanups in this arc

1. Add bare `nil` lexer support as primitive nil literal (currently parsed as bare symbol; needs primitive-value treatment)
2. Migrate all `:wat::core::nil` VALUE-position uses → bare `nil` (cascade, ~50-150 sites estimated)
3. **PRESERVE** `:wat::core::nil` TYPE-position uses (primitive lowercase; matches i64/bool convention)
4. HARD CUT `:wat::core::Char` → `:wat::core::char` (scalar; must be lowercase per Doctrine 2)
5. Inscribe BOTH doctrines in INTERSTITIAL + memory (`project_lexeme_role_doctrine.md`)

## Outstanding case-audit candidates (NOT this arc; flagged for future)

- `:wat::core::Uuid` — scalar opaque identifier — should be `uuid` per Doctrine 2; queued elsewhere (arc 109 territory per user direction)
- `:wat::core::Duration`, `:wat::core::Instant` — scalar time-values — queued elsewhere (arc 109 territory)

These will consume the doctrine when they land. Arc 242 inscribes the rule; future arcs apply it.

## The dividend — arc 241's apparatus does its first downstream work

Stone 241.10's `src/remedy/` + ranked-remedy schema + RETIREMENT_TABLE is FOUNDATIONAL. Arc 242's Char retirement is **one line** appended to `RETIREMENT_TABLE`:

```rust
(":wat::core::Char",              ":wat::core::char"),
```

The substrate teaches "did you mean: `:wat::core::char` [retirement replacement]" automatically. Zero additional Display work. Arc 241's investment pays its first downstream dividend.

## What this unblocks

- **Stone 241.12** (paused; defalias mint) resumes after arc 242 closes
- **Stone 241.13** INSCRIPTION closes arc 241 (Phase 4)
- **Arc 237.8b** reopens after Stone 241.13 (per `feedback_no_regression_until_arc_done`)
- **Future EDN-fidelity work** consumes Doctrine 1 + Doctrine 2 as foundational
- **Future case-audit arcs** (Uuid, Time family) consume Doctrine 2 as the rule

## The spawn-block discipline (per `feedback_spawn_block_winding`)

Arc 242 was spawned during arc 241's active context (Stone 241.12 in flight when surfaced). Per the doctrine: arc 241 CANNOT close until arc 242 closes; spawn-by-nature. Arc 241 paused; arc 242 wind-forwards; arc 241 resumes.

The discipline holds: no jumping between arcs; wind depth-first; INSCRIPTION is the last stone of each.

## Predicted band for arc 242

| Stone | Class | Predicted |
|---|---|---|
| 242.1 | Lexer addition + cascade migration (~50-150 sites) + Char HARD CUT + doctrine inscription | 60-150 min Mode A |
| 242.2 | INSCRIPTION (orchestrator-direct paperwork) | 30-60 min |

Arc 242 total: 90-210 min. The arc is tightly scoped; the doctrine is the lift.
