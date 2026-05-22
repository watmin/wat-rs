# BRIEF — Arc 220 Stone 220.5 — `:wat::core::Char` atomization gap fix

**Stone scope (sonnet portion):** add `:wat::core::Char` to the `is_atomizable` predicate at `src/check.rs:3623`; add 3 probe tests proving Char is now fully atomizable as a primitive (Atom-able + HashMap-key-able + HashSet-element-able). One-line predicate addition + one new test file.
**Type:** Sonnet Mode A.
**Time budget:** 15-25 min target; 35 min STOP.
**Depends on:** Stone 220.2 (`dd84fcf` — Char shipped) + Stone 220.4 (`31089d9` — List shipped + 14/14 PASS, also surfaced the predicate gap).
**Calibration:** 14 stones at-or-below band; this is smallest yet (1 substrate line + 3 probes). Band 15-25.
**Unblocks:** Slice 5 (INSCRIPTION + USER-GUIDE + cross-references) — arc 220 closure can honestly say "Char is fully atomizable."

## Gap surfaced 2026-05-22 (user question)

User asked: *"the atom of a char is.... (:wat::holon::Atom \\N) ?.."*

Investigation: `:wat::core::Char` (Stone 220.2) added the runtime Value variant + `Char/of` constructor + Hash/PartialEq/Eq impls. BUT — the **check-time atomizability predicate** at `src/check.rs:3623` was never extended to include Char. Without this:

- `(:wat::holon::Atom \\N)` would fail check (Atom requires atomizable T)
- `HashMap<wat::core::Char, V>` would fail check (HashMap requires atomizable K)
- `HashSet<wat::core::Char>` would fail check (HashSet requires atomizable T)

Per `feedback_no_known_defect_left_unfixed`: known defect can't be deferred. Stone 220.5 closes the gap.

## Pre-flight verified (orchestrator-grep'd 2026-05-22)

### Current `is_atomizable` predicate

`src/check.rs:3623-3661`:

```rust
fn is_atomizable(ty: &TypeExpr) -> bool {
    match ty {
        TypeExpr::Path(p) => matches!(
            p.as_str(),
            // Primitives (arc 215 baseline)
            ":wat::core::i64"
                | ":wat::core::f64"
                | ":wat::core::bool"
                | ":wat::core::String"
                | ":wat::core::keyword"
                // HolonAST and WatAST (arc 215 baseline)
                | ":wat::holon::HolonAST"
                | ":wat::WatAST"
                // Uuid — hashable primitive (arc 207)
                | ":wat::core::Uuid"
                // Type variables and inference sentinels — can't prove non-atomizable
                | ":wat::type::Infer"
        ),
        TypeExpr::Var(_) => true,
        TypeExpr::Parametric { head, args } => match head.as_str() {
            "wat::core::HashSet" => args.len() == 1 && is_atomizable(&args[0]),
            "wat::core::Vector" => args.len() == 1 && is_atomizable(&args[0]),
            "wat::core::HashMap" => {
                args.len() == 2 && is_atomizable(&args[0]) && is_atomizable(&args[1])
            }
            _ => false,
        },
        TypeExpr::Fn { .. } => false,
        TypeExpr::Tuple(elements) => elements.iter().all(is_atomizable),
    }
}
```

`:wat::core::Char` is MISSING from the `matches!` arm.

### Runtime Hash + Eq for Char (Stone 220.2 — shipped)

`src/runtime.rs:846` — Char hash arm exists:

```rust
Value::wat__core__Char(c) => c.hash(state),
```

Plus PartialEq + Eq arms exist (Stone 220.2). Runtime side is complete; only the check-time predicate gate is missing.

### Test pattern precedent

`tests/wat_arc220_char.rs` (Stone 220.2 — 312 lines, 10 tests) is the canonical Char test file shape. New atomization tests go in a separate file (`tests/wat_arc220_char_atomization.rs`) to keep concerns separate; same helper functions (pipe_pair, drain_lines, etc.).

## Working dir + constraints

- `/home/watmin/work/holon/wat-rs/`
- Branch: `arc-170-gap-j-v5-deadlock-state`
- Linux only; Zero Mutex; no `--no-verify`

## Your scope (sonnet)

Execute 2 mechanical edits:

### 1. Extend `is_atomizable` to include Char

`src/check.rs:~3638` — add one line to the `matches!` arm BEFORE the type-variable line:

```rust
// Uuid — hashable primitive (arc 207)
| ":wat::core::Uuid"
// Arc 220 Stone 220.5 — Char is a primitive; hashable per Stone 220.2's
// Value::wat__core__Char(c) => c.hash(state) at src/runtime.rs:846.
| ":wat::core::Char"
// Type variables and inference sentinels — can't prove non-atomizable
| ":wat::type::Infer"
```

### 2. Add probe tests

Create `tests/wat_arc220_char_atomization.rs` mirroring `tests/wat_arc220_char.rs` shape. 3 substantive probes:

#### Probe 1 — `(:wat::holon::Atom \N)` round-trip

Construct an Atom holding a Char value. Verify the resulting `Value::holon__HolonAST(...)` contains the expected atom shape. Compare against the equivalent Atom constructed from an i64 leaf as a sanity check.

```wat
(:wat::core::let
  [atom-c    (:wat::holon::Atom \a)
   atom-i    (:wat::holon::Atom 42)
   _         (:wat::core::assert (:wat::core::not (:wat::core::= atom-c atom-i)))]
  :wat::core::nil)
```

#### Probe 2 — `HashMap<Char, i64>` insert + lookup (the "char frequency" pattern)

```wat
(:wat::core::let
  [tally  (:wat::core::assoc (:wat::core::HashMap :wat::core::Char :wat::core::i64) \a 3)
   tally2 (:wat::core::assoc tally \b 7)
   a-val  (:wat::core::get tally2 \a)
   b-val  (:wat::core::get tally2 \b)
   _      (:wat::core::assert-eq (:wat::core::Some 3) a-val)
   _      (:wat::core::assert-eq (:wat::core::Some 7) b-val)]
  :wat::core::nil)
```

#### Probe 3 — `HashSet<Char>` insert + contains?

```wat
(:wat::core::let
  [vowels  (:wat::core::HashSet :wat::core::Char \a \e \i \o \u)
   has-a   (:wat::core::contains? vowels \a)
   has-z   (:wat::core::contains? vowels \z)
   _       (:wat::core::assert has-a)
   _       (:wat::core::assert (:wat::core::not has-z))]
  :wat::core::nil)
```

(Use Tools::canonical assertion form per existing tests; adapt fixture exact syntax to whatever the existing wat_arc220_char.rs / wat_arc220_list.rs patterns demonstrate.)

### Verification (must run before SCORE)

1. `cargo build --release` — workspace clean
2. `cargo test --release --lib -p wat` — PASS (no regression from predicate extension)
3. `cargo test --release --test wat_arc220_char` — 10/10 PASS (Stone 220.2 tests unchanged)
4. `cargo test --release --test wat_arc220_char_atomization` — 3/3 PASS (new probes)
5. `cargo test --release --test wat_arc220_list` — 23/23 PASS (sanity, unrelated)
6. `cargo test --release -p wat-edn` — 1/1 PASS (unchanged; this stone is wat-rs only)
7. `cargo clippy --release --all-targets -p wat-edn -- -D warnings` — 0 warnings
8. **wat-crate clippy intentionally NOT gated** — pre-existing arc 170 backlog stays visible per user direction

**Write `docs/arc/2026/05/220-wat-core-edn-primitive-completeness/SCORE-STONE-220.5.md`** mirroring SCORE-STONE-220.4 shape (but smaller — match the actual 4-row scorecard from EXPECTATIONS).

## STOP triggers

- **STOP-1 (lib test regression):** if `cargo test --release --lib -p wat` fail count goes UP from baseline 827/0 → diagnostic + report. Adding to atomizable should be additive — anything that breaks suggests something downstream was assuming Char was NOT atomizable.
- **STOP-2 (Char/i64 atom equality conflicts):** if Probe 1 finds `atom(Char('a')) == atom(i64(42))` (cross-type Eq leakage), STOP — that would mean Value's PartialEq has cross-type collision (would be a Stone 220.2 bug, surface for orchestrator decision).
- **STOP-3 (35 min elapsed):** wall-clock STOP.
- **EXTRA — interop handshakes NOT required for this stone** — wat-edn surface untouched; wat-rs-only predicate change. Don't run the 4 handshakes.

## Out-of-scope

- Slice 5 paperwork (INSCRIPTION + USER-GUIDE + cross-references) — separate; runs after Stone 220.5 ships
- Any List-related atomization (List is NOT atomizable; per HolonRepresentable<LinkedList<T>> it lowers to Bundle, not Atom — out of scope)
- Wat-edn changes (untouched)
- New runes (no candidates this stone)
- Documentation beyond test comments + SCORE
