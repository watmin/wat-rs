# EXPECTATIONS — Arc 214 Slice 4 Stone 4.1 — `:wat::program::Env` typealias

Mode A target: 10/10 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | Typealias registered in `src/types.rs` | New `env.register_builtin(TypeDef::Alias(...))` call mirroring `:wat::core::Bytes` pattern; name = `:wat::program::Env`; expr resolves to `HashMap<keyword, HolonAST>` |
| 2 | Brief documentation comment | Comment block explains the typealias purpose; cites arc 214 Slice 4 forward-correction Q4; matches `:wat::core::Bytes` comment style |
| 3 | Probe 1 — type-keyword parses | `parse_type_expr(":wat::program::Env")` returns `Ok(...)` |
| 4 | Probe 2 — alias expands | `expand_alias` resolves `:wat::program::Env` to `Parametric { head: "wat::core::HashMap", args: [keyword, HolonAST] }` |
| 5 | Probe 3 — function signature accepts | A function declaring `:wat::program::Env` param type checks cleanly |
| 6 | Probe 4 — explicit-Atom literal accepted | `{:foo (:wat::holon::Atom 42)}` literal unifies with `:wat::program::Env` param |
| 7 | Probe 5 — empty `{}` accepted | Empty `{}` unifies with `:wat::program::Env` via HM (K + V fresh variables resolve from param) |
| 8 | Probe 6 — wrong V rejected | `{:foo "string"}` (V = String) fails at check with TypeMismatch naming V mismatch |
| 9 | WAT-CHEATSHEET updated | Brief mention in namespace section; references arc 214 Slice 4 forward-correction |
| 10 | All existing tests preserved | probe_arc215_* + probe_brace_map_literal + probe_hashmap_ctor_vector_symmetric all stay green |

## Independent prediction (calibration record)

**Target runtime:** 30-45 min Mode A
**Upper bound:** 60 min
**Confidence:** high

**Rationale:**
- Smallest possible stone — single file change (types.rs) + new probe file + tiny doc update
- Bytes precedent is direct (copy-pattern; substitute name + expression)
- Probes mirror standard type-existence + unification shapes
- No substrate machinery changes; just additive registration
- Risk factor: if registration order matters (some types must register before aliases that reference them), might need ordering work — but HashMap registers as a builtin parametric before any aliases, so likely fine

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]
- If overrun: where? [TBD]

## Out-of-scope rows

- Accessor verbs (Stone 4.2)
- spawn-program' (Stone 4.3)
- Kernel polymorphic verbs (Stone 4.4)
- Integration tests (Stone 4.5)
- WARD-PASS, INTERSTITIAL — orchestrator handles
- `:wat::process::Env` — separate concern

## Honesty deltas accepted

- Registration ordering surprise if HashMap not yet available at the Env-registration site (unlikely; flag if encountered)
- Probe 5's HM unification path may surface implementation details (e.g., does the param-type unification path see the alias expanded form first or the source form?); document what you find
- WAT-CHEATSHEET section placement (where exactly to mention this); sonnet picks the natural fit
