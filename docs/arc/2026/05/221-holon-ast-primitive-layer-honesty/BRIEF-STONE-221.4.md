# BRIEF — Arc 221 Stone 221.4 — wat-rs ripple for Keyword + Nil + Tag + Uuid arms

**Stone scope (sonnet portion):** wat-rs ripple for the Keyword/Nil/Tag leaves shipped in Stone 221.3 (holon-rs `fa48b39`). Three new `value_to_atom` arms (Keyword, Nil, Uuid via `Bind(Tag("uuid"), String(hex))` per arc 221 doctrine correction), `is_atomizable` extensions, cascade-arm fixes (substrate-as-teacher: rust exhaustive-match compiler will surface them), 3 doc-comment refreshes, new probe file. **Wat-rs ONLY this stone — holon-rs untouched.**
**Type:** Sonnet Mode A.
**Time budget:** 60-90 min target; 120 min STOP.
**Depends on:** Stone 221.3 holon-rs commit `fa48b39` (Keyword/Nil/Tag leaves + cascade arms in holon-rs). Workspace cargo will fail-fast on exhaustive-match cascade sites in wat-rs that need new arms — the failure list IS the brief.
**Calibration:** Per `feedback_stone_briefs_cite_prior_score`, read **Stones 221.1, 221.2, 221.3 SCOREs** (in `docs/arc/2026/05/221-holon-ast-primitive-layer-honesty/`). Stone 221.2 was the closest precedent (wat-rs side, ~35 min). Stone 221.4 is ~2× scope: 3 new value_to_atom arms instead of 2, cascade larger (Keyword/Nil/Tag all need arms vs just Char), Uuid arm uses the new doctrine shape.

## Working dir + constraints

- **Working dir: `/home/watmin/work/holon/wat-rs/`** (NOT holon-rs!)
- Branch: `arc-170-gap-j-v5-deadlock-state` (already current)
- Linux only; no `--no-verify`.
- DO NOT commit. Orchestrator commits after independent scoring.
- DO NOT touch holon-rs files this stone (Stone 221.3 already shipped there).
- DO NOT modify Stone 221.5's scope (Symbol/String canonical-bytes seed — separate fix).

## Pre-flight verified (orchestrator-grep'd 2026-05-22)

### Surface state post-Stone-221.3

- **`Value::Unit`** exists at `src/runtime.rs:387`. This is wat's nil value. Needs `value_to_atom` arm → `HolonAST::Nil`.
- **`Value::wat__core__keyword(Arc<String>)`** at `src/runtime.rs:390`. Stored content **includes the leading colon** (per `src/runtime.rs:7111` constructor: `format!(":{}", s.as_str())`). The new `value_to_atom` arm must strip the colon at the boundary, or use `HolonAST::keyword(&k)` (which already strips it per Stone 221.3 constructor).
- **`Value::wat__core__Uuid(uuid::Uuid)`** at `src/runtime.rs:616`. Currently has NO `value_to_atom` arm (arc 207 false-flag latent since 2026-05-17). Closes here via tagged composition.
- **No `Value::wat__core__Tag` variant exists.** Tag is only an EDN-side dispatch marker; it appears in composition (`Bind(Tag, payload)`), never as a wat user-value. No standalone `value_to_atom Tag` arm needed.
- **Only 3 `HolonAST::Symbol(":...")` sites in wat-rs total**, ALL doc comments (no producers):
  - `src/runtime.rs:10490` — doc comment
  - `tests/probe_arc214_slice4_stone2_env_get_trio.rs:322` — test comment
  - `tests/wat_arc201_structured_signature_types.rs:23` — test comment
- **Zero** `HolonAST::Symbol("nil")` producer sites.
- **Zero** `HolonAST::Symbol("#...")` producer sites.

The "consumer ripple" turns out to be far smaller than substrate-wide; it's the value_to_atom arms + the compiler-driven cascade.

### Cascade sites known (Stone 221.1 + 221.2 precedents)

The compiler will surface exhaustive-match sites that lack arms for `HolonAST::Keyword`, `HolonAST::Nil`, `HolonAST::Tag`. Known precedent locations from Stone 221.2's cascade work for Char:
- `src/hologram.rs:232` (`find_first_thermometer`)
- `src/edn_shim.rs:~1809` (`holon_ast_to_edn`)
- `src/edn_shim.rs:~1919` (`edn_holon_tag_to_ast` reader)
- `src/runtime.rs:~14782` (`holon_to_watast`)
- `src/runtime.rs:~15631` (statement-length)
- Possibly `src/runtime.rs:~8487` / `~13488` / `~13672` (Bool-arm neighbors)

Sonnet: don't pre-enumerate. Run `cargo build --release -p wat`, read the E0004 list, mirror Stone 221.2's arm style. Trust the compiler — `feedback_substrate_as_teacher`.

## Your scope (sonnet)

### 1. `value_to_atom` Keyword arm (`src/runtime.rs:~13800`)

Place after the Char arm (line 13828). The Value variant stores keyword with leading colon; `HolonAST::keyword(&k)` strips it.

```rust
Value::wat__core__keyword(k) => HolonAST::keyword(&k),
```

Doc comment cites Stone 221.3 holon-rs commit `fa48b39` and arc 221 doctrine.

### 2. `value_to_atom` Nil arm

After the Keyword arm.

```rust
Value::Unit => HolonAST::Nil,
```

Doc comment names that Value::Unit is wat's nil; this maps to HolonAST::Nil (Stone 221.3 leaf, not the pre-arc-221 `Symbol("nil")` convention).

### 3. `value_to_atom` Uuid arm — tagged composition per doctrine

Closes arc 207 false-flag. Uses `Bind(Tag("uuid"), String(hex))` shape per the doctrine correction (NOT `Bind(Atom(Symbol("#uuid")), Atom(String(...)))` — that notation was retired; bare-leaf payloads).

```rust
Value::wat__core__Uuid(u) => HolonAST::bind(
    HolonAST::tag("uuid"),
    HolonAST::string(u.to_string()),
),
```

Doc comment cites the doctrine correction inscribed at DESIGN-221 § "Forward-correction 2026-05-22 (notation refinement)".

### 4. `is_atomizable` extensions (`src/check.rs:~3623`)

Add to the matches arm:

```rust
| ":wat::core::keyword"
```

Doc comment cites Stone 221.4 value_to_atom Keyword dispatch.

Nil: check whether `:wat::core::nil` or `:wat::core::Unit` is the type-system surface. If the type for `Value::Unit` is checkable, add it. If `nil` isn't a first-class type the user instantiates, skip and surface as a question — Unit may already be in the path. **Do not invent a new type name.**

### 5. Cascade-arm fixes (compiler-driven)

After steps 1-4, `cargo build --release -p wat` will emit E0004 errors for exhaustive-match sites missing Keyword/Nil/Tag arms. For each:

- **Leaf-passthrough sites** (e.g., `find_first_thermometer`, statement-length): `| HolonAST::Keyword(_) | HolonAST::Nil | HolonAST::Tag(_) => <leaf-result>` — mirror the existing Char arm in the same site
- **`holon_to_watast`**: each leaf → its WatAST equivalent. Keyword → `WatAST::Keyword`; Nil → `WatAST::Nil` (if it exists) or equivalent; Tag → `WatAST::List([Symbol("#"), name])` or whatever Stone 221.2's holon_to_watast Char arm patterns map to. **If the cleanest mapping isn't obvious for any of the three, STOP and surface as a question.**
- **`holon_ast_to_edn` + `edn_holon_tag_to_ast`**: emit + parse the EDN tagged representation. Keyword → bare keyword; Nil → bare nil; Tag → bare tag-symbol. Mirror Stone 221.2's Char arms.

Iterate `cargo build --release -p wat` until clean. The fail-count is the progress meter (`feedback_substrate_as_teacher`).

### 6. Doc-comment refreshes (3 sites)

Update the 3 stale doc comments naming the pre-arc-221 convention:

- `src/runtime.rs:10490` — refresh to reflect the new variant
- `tests/probe_arc214_slice4_stone2_env_get_trio.rs:322` — refresh
- `tests/wat_arc201_structured_signature_types.rs:23` — refresh

Comment-only; no logic changes.

### 7. New probe file `tests/wat_arc221_keyword_nil_tag_atomization.rs`

Mirror Stone 221.2's `wat_arc221_char_atomization.rs` shape. Probes (5+ minimum):

1. **Keyword atom round-trip:** `(:wat::holon::Atom :foo) == (:wat::holon::Atom :foo)`; `(:wat::holon::Atom :foo) != (:wat::holon::Atom :bar)`; `(:wat::holon::Atom :foo) != (:wat::holon::Atom "foo")` (Keyword leaf distinct from String leaf)
2. **Nil atom round-trip:** `(:wat::holon::Atom nil) == (:wat::holon::Atom nil)`; `(:wat::holon::Atom nil) != (:wat::holon::Atom :nil)` (Nil leaf distinct from Keyword)
3. **Uuid atom via tagged composition:** `(:wat::holon::Atom <uuid-val>)` produces a HolonAST equal to itself; round-trip; distinct from another uuid. Verify the encoding is `Bind(Tag("uuid"), String(hex))` shape (not Atom-wrapped).
4. **HashMap<keyword, i64>** insert + lookup (verifies Keyword atomization works at runtime)
5. **HashSet<keyword>** insert + contains?
6. **HashMap<Uuid, String>** insert + lookup (closes arc 207 false-flag — first time this works at runtime)

If `HashSet<Nil>` is meaningful in the type system (typically not — Nil is a unit), skip it or add an honest note.

### 8. Verification (must run before SCORE)

From `/home/watmin/work/holon/wat-rs/`:

```
cargo build --release -p wat
cargo test --release --lib -p wat
cargo test --release --test wat_arc220_char
cargo test --release --test wat_arc221_char_atomization
cargo test --release --test wat_arc221_keyword_nil_tag_atomization
cargo test --release -p wat-edn
cargo clippy --release --all-targets -p wat-edn -- -D warnings
```

All must be clean. Pre-existing wat-clippy backlog (115 warnings) stays gated per arc 218 discipline; don't gate this stone on it.

**Holon-rs untouched** — `git -C /home/watmin/work/holon/holon-rs/ diff --name-only` must be empty.

**Write `wat-rs/docs/arc/2026/05/221-holon-ast-primitive-layer-honesty/SCORE-STONE-221.4.md`** mirroring SCORE-STONE-221.2.md / 221.3.md shape (10-row scorecard per EXPECTATIONS).

## STOP triggers

- **STOP-1 (existing wat-rs test regression beyond planned):** if `cargo test --release --lib -p wat` shows ANY pre-existing test regression not caused by this stone's intentional changes, STOP + diagnostic + report. NOTE: per Stone 221.3's discipline learning, "tests broken by this stone's intentional substrate change" ≠ "pre-existing failures"; surface them as Delta entries with HONEST framing ("Stone 221.4 substrate change broke N tests in <file>; they passed on baseline"). Fix in-flight only if mechanical AND the fix is honest + correct + non-masking.
- **STOP-2 (load-bearing probe fails):** if any of the 6 probes fails its load-bearing assertion (especially Uuid round-trip — the arc 207 false-flag close), STOP + diagnostic + report.
- **STOP-3 (120 min elapsed):** wall-clock STOP.
- **STOP-4 (holon-rs touched accidentally):** if `git -C /home/watmin/work/holon/holon-rs/ diff --name-only` shows any changes from this stone, STOP and report.
- **STOP-5 (unclear holon_to_watast mapping for Keyword/Nil/Tag):** if the round-trip-safe mapping isn't obvious, STOP and surface — better to ask than invent.

## Out-of-scope

- holon-rs changes (Stone 221.3 already shipped there at `fa48b39`)
- Stone 221.5 — Symbol/String canonical-bytes seed distinction (separate substrate-doctrine fix in holon-rs)
- Stone 221.6 — INSCRIPTION (Phase B closure; blocked on arc 223 + arc 222 per spawn-block discipline)
- Arc 222 + arc 223 work (these are spawn children of arc 221; they execute AFTER 221.5 ships, per the spawn-block chain)
- Wat-edn changes — wire format already handles EDN literals
- BOOK / USER-GUIDE updates — Stone 221.6
- Pre-existing wat-clippy backlog (115 warnings) — gated separately per arc 218 discipline
