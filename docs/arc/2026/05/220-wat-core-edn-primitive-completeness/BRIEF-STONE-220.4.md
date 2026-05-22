# BRIEF — Arc 220 Stone 220.4 — `:wat::core::List<T>` (LinkedList-backed; cross-type Eq with Vector)

**Stone scope (sonnet portion):** mint `:wat::core::List<T>` per Char (arc 220 Slice 2) precedent + add cross-type sequence-Eq/Hash with Vector per EDN spec §282-289. The largest stone in arc 220 — three internal parts: substrate plumbing, dispatch arms, bridge + tests.
**Type:** Sonnet Mode A.
**Time budget:** 90-150 min target; 180 min STOP.
**Depends on:** Stone 220.2 (Char `dd84fcf`), Stone 220.3 (`'` reader macro `c526b1f`).
**Calibration:** 13 stones at-or-below band. This is the largest single substrate addition in arc 220. Band 90-150.
**Unblocks:** Slice 5 (INSCRIPTION + paperwork) → arc 219b → arc 218 streaming → arc 217 Clojure-IPC.

## User direction baked in

- **L (locked):** `first/rest/conj` (Clojure-style; matches arc 146 dispatch). conj on List = PREPEND (vs Vector conj = APPEND).
- **I (locked):** Cross-type Eq accepted per EDN spec §282-289. `List(1,2,3) == Vector(1,2,3)` returns true. Hash invariant preserved via shared sequence-Hash function.
- **G (locked, Slice 3 shipped):** `'(1 2 3)` reader macro syntax now available; tests use both `'(1 2 3)` literal AND `(:wat::core::List/of 1 2 3)` constructor.

## Pre-flight verified (orchestrator-grep'd 2026-05-22)

### Naming pattern (modern arc convention)

User-visible: `:wat::core::List<T>`. Internal Rust variant: `Value::wat__core__List(Arc<LinkedList<Value>>)` — mirrors Char (`wat__core__Char`) + Uuid (`wat__core__Uuid`). Older variants like `Value::Vec(Arc<Vec<Value>>)` (for `:wat::core::Vector`) and `Value::wat__std__HashSet` are historical naming; modern `wat__core__*` pattern applies to new mints.

### Char-precedent arm sites (mirror exactly)

Per SCORE-STONE-220.2: 10 sites for Char variant. List adds parallel arms:

| # | File:approx-line | Pattern (Char) | List counterpart |
|---|---|---|---|
| 1 | `src/runtime.rs:~618` | `wat__core__Char(char)` variant | Add `wat__core__List(Arc<std::collections::LinkedList<Value>>)` next to it |
| 2 | `src/runtime.rs:~655` | `(Value::wat__core__Char(a), Value::wat__core__Char(b)) => a == b` | Same-type arm: `(Value::wat__core__List(a), Value::wat__core__List(b)) => a == b` |
| 3 | `src/runtime.rs:~762` | `Value::wat__core__Char(c) => c.hash(state)` | **MODIFIED HASH STRATEGY — see "Cross-type sequence-Hash" below** |
| 4 | `src/runtime.rs:~1044` | `Value::wat__core__Char(_) => "wat::core::Char"` | `"wat::core::List"` |
| 5 | `src/runtime.rs:~7103` | structural-eq arm | Same-type + cross-type arms (see below) |
| 6 | `src/runtime.rs:~15905` | `Value::wat__core__Char(c) => format!("\\{}", c)` | Render List as the EDN parens form: emit `"("` + space-join children + `")"` (delegates to per-Value render for children) |
| 7 | `src/edn_shim.rs:~412` + `~590` | parse direction: `Edn::Char(c) → Ok(Value::wat__core__Char(*c))` | `Edn::List(items) → Ok(Value::wat__core__List(Arc::new(items.into_iter().map(...).collect::<LinkedList<_>>())))` |
| 8 | `src/edn_shim.rs:~1631` | write direction | `Value::wat__core__List(xs) → OwnedValue::List(xs.iter().map(...).collect())` |
| 9 | `src/closure_extract.rs:~1493` | Char capture as `(:wat::core::Char/of "x")` form | List capture as `(:wat::core::List/of <items...>)` variadic form |
| 10 | `src/parser.rs` | (Char added Token::Char handling) | NO parser change — List uses `'(1 2 3)` per Slice 3 reader macro OR `(:wat::core::List/of ...)` constructor; both reduce to existing parser paths |

### Cross-type sequence-Hash (LOAD-BEARING NOVEL SURFACE)

Per EDN spec §282-289 + Clojure verified semantics (`(= '(1 2) [1 2])` returns `true`):
- `List(1,2,3) == Vector(1,2,3)` returns true
- Hash invariant requires equal values hash equal
- Therefore List + Vector MUST hash to same value when contents match

Current Vec hash at `src/runtime.rs:~775`:

```rust
Value::Vec(xs) => xs.hash(state),  // std Vec's Hash: len + elements
```

Outer hash impl at `~760` calls `std::mem::discriminant(self).hash(state);` FIRST — so Vec hashes (discriminant_Vec + len + elements). LinkedList would hash (discriminant_List + len + elements) — different prefix means different total hash even with identical contents.

**Fix:** introduce shared sequence-Hash discipline. Two options (sonnet picks):

**(α) Shared SEQ_TAG constant** — modify outer impl to skip discriminant for List/Vec; both arms use same constant SEQ_TAG instead:

```rust
fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    // For sequence types (Vec, List) skip discriminant and use shared SEQ_TAG
    // per EDN spec §282-289 (cross-type sequence equality).
    match self {
        Value::Vec(xs) => {
            const SEQ_TAG: u8 = 0xA5;  // arbitrary; just distinct from other discriminants
            SEQ_TAG.hash(state);
            for v in xs.iter() { v.hash(state); }
        }
        Value::wat__core__List(xs) => {
            const SEQ_TAG: u8 = 0xA5;
            SEQ_TAG.hash(state);
            for v in xs.iter() { v.hash(state); }
        }
        // All other variants: existing discriminant + per-variant hash
        _ => {
            std::mem::discriminant(self).hash(state);
            // existing match arms...
        }
    }
}
```

**(β) Helper function** — extract shared `hash_sequence(items, state)`:

```rust
fn hash_sequence<'a, H, I>(items: I, state: &mut H)
where H: std::hash::Hasher, I: IntoIterator<Item = &'a Value>
{
    const SEQ_TAG: u8 = 0xA5;
    SEQ_TAG.hash(state);
    for v in items { v.hash(state); }
}
```

Then both Vec and List arms call `hash_sequence(xs.iter(), state)` instead of discriminant + xs.hash.

(β) is cleaner. Sonnet picks; preserve the SEQ_TAG semantic + skip discriminant for these two variants.

**Discipline note:** modifying Value::Vec's hash changes in-process HashMap key hashes. wat-rs doesn't persist hash keys to disk; HashMap rehashes on insert per Rust semantics. Safe within session.

### Cross-type Eq arms

Modify PartialEq impl at `~654`:

```rust
// Existing same-type arms stay:
(Value::Vec(a), Value::Vec(b)) => a == b,
(Value::wat__core__List(a), Value::wat__core__List(b)) => a == b,

// New cross-type arms per EDN spec §282-289:
(Value::Vec(a), Value::wat__core__List(b)) => sequence_eq(a.iter(), b.iter()),
(Value::wat__core__List(a), Value::Vec(b)) => sequence_eq(a.iter(), b.iter()),
```

Helper:

```rust
fn sequence_eq<'a, I, J>(mut a: I, mut b: J) -> bool
where I: Iterator<Item = &'a Value>, J: Iterator<Item = &'a Value>
{
    loop {
        match (a.next(), b.next()) {
            (None, None) => return true,
            (Some(_), None) | (None, Some(_)) => return false,
            (Some(x), Some(y)) => if x != y { return false; },
        }
    }
}
```

Same shape for structural-eq arms at `~7102` area.

### Dispatch arms (per arc 146)

**Length** — `vector_length_inner` at `~7762`:

```rust
fn vector_length_inner(v: &Value) -> Result<Value, RuntimeError> {
    match v {
        Value::Vec(xs) => Ok(Value::i64(xs.len() as i64)),
        other => Err(...),
    }
}
```

Add `list_length_inner` mirror (takes Value::wat__core__List). Register `:wat::core::List/length` dispatch entry.

**Empty?** — `vector_empty_q_inner` at `~7867` — same pattern: add `list_empty_q_inner` + register.

**First** — at `runtime.rs:4525` dispatch routes to `eval_positional_accessor`. Verify if positional accessor handles List or only Vec; extend if needed.

**Rest** — `eval_vec_rest` at `runtime.rs:4537`. Extend or add `eval_list_rest` (returns List, not Vec).

**Conj** — arc 146 Dispatch table. Add List/conj arm with PREPEND semantic (`xs.push_front`). Distinct from Vector/conj which appends.

**Contains?** — extend Vec arm at `~8537` / `~8586` (whichever is contains?) to also handle List.

**Get** — extend per-index access to handle List (O(N) walk vs Vec O(1)).

### Constructor + dispatch entry (Char precedent)

`src/string_ops.rs` — add `eval_list_of` following `eval_char_of`:

```rust
pub fn eval_list_of(args: &[WatAST], env: &Environment, sym: &SymbolTable) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::core::List/of";
    let mut items = std::collections::LinkedList::new();
    for arg in args {
        items.push_back(eval(arg, env, sym)?);
    }
    Ok(Value::wat__core__List(Arc::new(items)))
}
```

Variadic. No arity restriction. Each arg evaluated, pushed to end of LinkedList.

`src/runtime.rs:~4570` dispatch entry: `":wat::core::List/of" => crate::string_ops::eval_list_of(args, env, sym),`

### HolonRepresentable for LinkedList<T> (mirrors HashSet at `src/comms/mod.rs:142`)

```rust
/// Arc 220 — `HolonRepresentable` for `LinkedList<T>`.
/// Mirrors HashSet impl shape; encodes as `HolonAST::Bundle(vec![T_holon, ...])`.
impl<T> HolonRepresentable for std::collections::LinkedList<T>
where T: HolonRepresentable + Send + 'static,
{
    fn to_holon_ast(&self) -> holon::HolonAST {
        let children: Vec<holon::HolonAST> = self.iter().map(|v| v.to_holon_ast()).collect();
        holon::HolonAST::bundle(children)
    }
    fn from_holon_ast(ast: &holon::HolonAST) -> Result<Self, WireError> where Self: Sized {
        match ast {
            holon::HolonAST::Bundle(items) => {
                let mut list = std::collections::LinkedList::new();
                for (i, item) in items.iter().enumerate() {
                    let v = T::from_holon_ast(item).map_err(|e| {
                        WireError::new(format!("LinkedList<T>[{}]: {}", i, e))
                    })?;
                    list.push_back(v);
                }
                Ok(list)
            }
            other => Err(WireError::new(format!("expected Bundle, got {:?}", other))),
        }
    }
}
```

## Working dir + constraints

- `/home/watmin/work/holon/wat-rs/`
- Branch: `arc-170-gap-j-v5-deadlock-state`
- Linux only; Zero Mutex; no `--no-verify`

## Your scope (sonnet)

Execute in 3 parts. Each part has internal verification; if friction surfaces mid-stone, you can write partial SCORE and report.

### Part A — Substrate plumbing (variant + Hash + Eq + render + closure_extract)

1. Add `Value::wat__core__List(Arc<std::collections::LinkedList<Value>>)` variant in runtime.rs after Char
2. Update outer Hash impl to use sequence-Hash for Vec + List (shared SEQ_TAG); helper function approach (β) recommended
3. Add same-type Eq arms + cross-type sequence_eq arms (List=Vector, Vector=List)
4. Same-shape arms for structural-eq, type_name, render
5. closure_extract Char-style List capture form

**Verify after Part A:** `cargo build --release` clean; `cargo test --release --lib -p wat` passes baseline tests (you'll add List-specific tests in Part C). The Hash change should not break any existing test — verify carefully.

### Part B — Dispatch + constructor

6. Add list_length_inner + list_empty_q_inner; register dispatch entries for `:wat::core::List/length` + `:wat::core::List/empty?`
7. Extend first/rest/conj/contains?/get polymorphic paths to handle List
8. Add eval_list_of variadic constructor in string_ops.rs; dispatch entry `:wat::core::List/of` in runtime.rs

**Verify after Part B:** all dispatch arms compile; sample test of `(:wat::core::List/of 1 2 3)` + `(length ...)` + `(empty? ...)` works.

### Part C — Bridge + tests + interop

9. HolonRepresentable<LinkedList<T>> impl in src/comms/mod.rs mirroring HashSet
10. edn_shim bridge: parse direction `Edn::List(...) → Value::wat__core__List(...)` at the 2 sites; write direction `Value::wat__core__List(...) → OwnedValue::List(...)` at the 1 site
11. `tests/wat_arc220_list.rs` integration test:
    - Construction: `'(1 2 3)` literal AND `(:wat::core::List/of 1 2 3)` constructor produce same value
    - Empty list
    - first/rest/conj (List conj prepends; Vector conj appends — different semantic verified)
    - length/empty?/contains?/get
    - **Cross-type Eq:** `(= '(1 2 3) [1 2 3])` returns true (List == Vector per EDN spec)
    - **Cross-type HashMap key:** `{(:wat::core::List/of 1 2) :a}` matches probe `{[1 2] :a}` (same key per Hash invariant)
    - EDN round-trip: parse `(1 2 3)` via wat-edn → wat__core__List → write → reparse → identical
12. `wat-tests/holon/list_round_trip.wat` — wat-source exercise with assert-eq!
13. `crates/wat-edn/interop-tests/src/bin/shape_matrix.rs` + `shape_matrix_reader.rs` + `consume_shapes.clj` + `produce_shapes.clj` — add `:list-3` shape (`Value::List` of 3 ints; verifies cross-language list round-trip)

### Verification (must run before SCORE)

1. `cargo build --release` — workspace clean
2. `cargo test --release --lib -p wat` — PASS with count += new List tests
3. `cargo test --release -p wat-edn` — 344/344 (unchanged)
4. `cargo clippy --release --all-targets -p wat-edn -- -D warnings` — 0 warnings
5. From `crates/wat-edn/interop-tests/`: cargo build + clippy clean
6. **Interop-tests 4 handshakes** (mandatory; shape_matrix gains `:list-3` probe per item #13):
   - `cd crates/wat-edn/interop-tests`
   - `cargo run --release --bin wat-edn-interop-tests | clojure -M clj/consume.clj`
   - `clojure -M clj/produce.clj | cargo run --release --bin reader`
   - `cargo run --release --bin shape_matrix | clojure -M clj/consume_shapes.clj`
   - `clojure -M clj/produce_shapes.clj | cargo run --release --bin shape_matrix_reader`

**NOTE on handshakes:** sub-agent piped-bash permission wall (218.6b/c/d/e/220.2 precedent — 6th stone). Ship the rest cleanly + write SCORE marking handshake row as "pending orchestrator-side verification". Orchestrator runs during scoring. Do NOT block.

**Write `docs/arc/2026/05/220-wat-core-edn-primitive-completeness/SCORE-STONE-220.4.md`** mirroring SCORE-STONE-220.2 shape.

## STOP triggers

- **STOP-1 (Hash change breaks existing tests):** if modifying Vec's hash arm breaks any existing test that uses Vec as HashMap key, report immediately + verify sequence-Hash arithmetic is consistent across both arms
- **STOP-2 (cross-type Eq test surprises):** if `List(...) == Vector(...)` returns false despite same contents, the helper function is broken — verify iterator equality logic
- **STOP-3 (dispatch arm cascade exceeds expected ~7 ops):** if first/rest/conj/length/empty?/contains?/get extension reveals MORE polymorphic ops needing List arms, report
- **STOP-4 (HolonAST encoding for LinkedList fails):** if encoder rejects LinkedList path (no impl), HolonRepresentable trait needs more than mirroring HashSet; report
- **STOP-5 (interop handshake fails on `:list-3`):** Clojure `clojure.edn/read` accepts `(1 2 3)` as a list natively; if shape_matrix `:list-3` probe fails, the wat-edn write or Clojure read has a real issue
- **STOP-6 (180 min elapsed):** wall-clock STOP

## Out-of-scope

- INSCRIPTION + USER-GUIDE + arc closure paperwork — Slice 5
- BigInt / BigDec wat-core types — deferred per DESIGN
- Performance optimization beyond LinkedList's natural O(1) cons / O(N) iter
- HolonAST schema extension — no new variants; List encodes via existing Bundle
- New runes (no candidates this stone)
- Touching wat-edn substrate — wat-edn handles List at the Value::List variant level; only wat-rs side adds the type

## Wat-clippy NOT gated

Per Stone 220.2 + 220.3 precedent + user direction 2026-05-22: the 115 wat-crate clippy warnings are arc 170 backlog visibility. NOT a Stone 220.4 verification gate.
