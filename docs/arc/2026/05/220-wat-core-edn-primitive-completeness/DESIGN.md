# Arc 220 — `:wat::core` EDN primitive completeness (Char + List)

**Status:** Active (slice 1 = this DESIGN.md, committed 2026-05-22).
**Trigger date:** 2026-05-22.
**Predecessors:** arc 164 (List SKIPPED 2026-05-08 conditionally), arc 218 (wat-edn IMPECCABLE work).
**Successor:** unblocks arc 219b (EDN spec conformance + differential interop suite).

## The triggering signal

During the arc 218 IMPECCABLE sweep, the user surfaced the question *"are we out of spec?"* by quoting the EDN spec verbatim on symbol/keyword constituent characters. Empirical testing (`crates/wat-edn/examples/spec_probe.rs`) confirmed wat-edn rejects 3 spec-legal forms (`:foo:bar`, `:foo#bar`, `foo:bar`) that `clojure.edn/read` accepts. That's the L1 substrate violation arc 219b will close.

But auditing the broader EDN primitive → wat-type mapping surfaced a deeper gap:

| EDN primitive | wat-edn `Value` variant | `:wat::core::*` type | Status |
|---|---|---|---|
| `nil` | `Value::Nil` | `:wat::core::nil` | ✓ |
| `true`/`false` | `Value::Bool` | `:wat::core::Bool` | ✓ |
| integer | `Value::Integer` | `:wat::core::Int` (etc.) | ✓ |
| float | `Value::Float` | `:wat::core::Float` | ✓ |
| string | `Value::String` | `:wat::core::String` | ✓ |
| **character** | `Value::Char` | **MISSING** | ✗ |
| keyword | `Value::Keyword` | `:wat::core::Keyword` (assumed) | ✓ |
| symbol | `Value::Symbol` | `:wat::core::Symbol` (assumed) | ✓ |
| **list** | `Value::List` | **MISSING** | ✗ |
| vector | `Value::Vector` | `:wat::core::Vector<T>` | ✓ (arc 109 slice 1f) |
| map | `Value::Map` | `:wat::core::HashMap<K,V>` | ✓ (arc 216) |
| set | `Value::Set` | `:wat::core::HashSet<T>` | ✓ (arc 216) |
| tagged | `Value::Tagged` | encoding doctrine (Bind+Atom) | ✓ (arc 216.7) |
| `#inst` | `Value::Inst` | `:wat::core::Instant` (assumed) | ✓ |
| `#uuid` | `Value::Uuid` | `:wat::core::Uuid` | ✓ (arc 207) |

**Two gaps: `Char` and `List`.** Without them, a wat program receiving EDN over the wire CANNOT round-trip:
- Receive `\c` (a character) → must collapse to String (lossy)
- Receive `(1 2 3)` (a list) → must collapse to Vector (lossy: loses parens-vs-brackets distinction)

Arc 217 (Clojure-IPC bridge) is the consumer that surfaces this. Without Char + List, wat code reading EDN from a Clojure parent CAN'T faithfully represent the message structure.

## Why arc 164 was SKIPPED (and why the skip's conditions are met)

**arc 164's SKIPPED rationale (2026-05-08):**
- AST/substrate: zero need for List (Vec is workload-correct)
- Wat user-data: narrow signal (5 sites use `:rest` recursion; fold idiom covers most)
- Heritage alone doesn't justify
- Mitigations exist without minting a new type (fold refactor; Vec view; persistent vector)

**The skip was conditional on future revisit when:**
> *"(a) the language has stabilized + the ergonomic surface is settled, AND (b) the performance angle named below has surfaced as a real bottleneck in real workloads."*

Arc 220's trigger is **neither (a) nor (b)** — it's a NEW kind of signal:

**(c) EDN cross-language round-trip integrity.** Arc 217 (Clojure-IPC bridge) and arc 219b (EDN spec conformance) both require wat to faithfully represent EDN's full type vocabulary. A list-vs-vector collapse at the wat layer is a CORRECTNESS issue, not an ergonomics or perf one. Fold idiom doesn't help here.

Per `feedback_inscription_immutable`: arc 164's SKIP inscription stays as historical record. Arc 220 is a NEW arc; its DESIGN cites arc 164 + names the new trigger (c).

## Architectural decisions

### Decision 1 — Char: BMP-only via existing wat-edn discipline

`:wat::core::Char` wraps Rust `char` directly. The BMP-only constraint inherits from Stone 218.6b (`crates/wat-edn/src/writer.rs:307-339` panics on `cp > 0xFFFF`; `crates/wat-edn/src/lexer.rs:355-370` rejects supplementary-plane char literals). The wat-side constructor enforces the same BMP-only rule at construction time — symmetric strictness across all three layers (wat-edn parser, wat-edn writer, wat-core constructor).

Rationale: cross-language interop with Clojure (which rejects `\😀`). Per Stone 218.6b precedent.

### Decision 2 — List: `std::collections::LinkedList<Value>` backing

`:wat::core::List<T>` wraps `Arc<LinkedList<Value>>` (matching the `Arc<...>` pattern used by other wat collection variants — see `arc 216 Stone 216.5b/c` for the precedent). LinkedList gives:
- O(1) cons (head-prepend)
- O(1) head/tail decomposition (the cited `:rest` workload)
- O(N) iteration (acceptable for sequence ops)
- Per-node allocation overhead (Rust LinkedList is doubly-linked) — acceptable given the consumer pattern (head/tail recursion in EDN data, not high-frequency Vec mutation)

**Equality semantics per EDN spec §282-289:** sequences (lists AND vectors) are equal when same count + same ordinal pairs. So `List == Vector` when contents match. Implementation: `Hash` + `PartialEq` for `Value` already handle Vector via `Arc<Vec<Value>>` and HashSet via `Arc<HashSet<Value>>` (arc 216 Stone 216.5a `impl Hash for Value`); extend to handle `List` variant with sequence-equality (compare ordinal pairs across List + Vector boundaries).

### Decision 3 — Holon encoding via shared Sequential

`wat/holon/Sequential.wat:1-7` defines the holon-encoding shape for ordered sequences (bind-chain with positional Permute). Both List AND Vector encode through Sequential at the holon layer. **No new holon-encoding work needed for arc 220** — Sequential already handles the encoding side.

What IS new: the type-system layer distinguishes List from Vector (so wat programs can pattern-match / construct one vs the other); the holon layer collapses them to the same encoding (so VSA operations don't care about parens-vs-brackets).

### Decision 4 — wat-edn ↔ wat-core bridge

Add bidirectional translation:
- `wat_edn::Value::Char(c)` ↔ `wat::Value::wat__core__Char(c)`
- `wat_edn::Value::List(items)` ↔ `wat::Value::wat__core__List(Arc::new(items.into_iter().collect::<LinkedList<_>>()))`

The bridge lives in `src/edn_shim.rs` alongside the existing wat ↔ wat-edn translation paths. No new module; extends existing patterns.

## Slice plan (4 slices)

### Slice 1 — DESIGN (this document)

Committed 2026-05-22 as the slice-1 artifact. Names the trigger, cites arc 164 SKIP, surfaces the EDN-primitive mapping gap, locks the four architectural decisions, scopes slices 2-4.

### Slice 2 — `:wat::core::Char` primitive

**Substrate work:**
- `Value::wat__core__Char(char)` variant added to wat's value enum
- Constructor function (`:wat::core::Char/of` or per arc 207 Uuid precedent) — accepts BMP-only chars; rejects supplementary-plane with clear diagnostic
- Type registry entry for `:wat::core::Char` (with appropriate width/dispatch metadata)
- Parser support — wat source code can write a Char literal? (open: does wat-source need a literal syntax for Char, or only constructor function? Per arc 207 Uuid precedent: constructor function only. Char is rare in wat source; primarily needed for EDN ingest.)
- Walker / check support if needed
- Dispatch arms: equality, hashing, display
- `src/edn_shim.rs` bridge: `wat_edn::Value::Char(c)` → `wat__core__Char(c)` on parse; reverse on write
- Tests:
  - Construction with BMP char succeeds
  - Construction with supplementary-plane char panics with BMP-only diagnostic
  - Round-trip via wat-edn (parse `\c` → wat__core__Char → write → parse → identical)
  - Equality, hashing
  - Cross-language: interop-tests/shape_matrix includes a `:char-bmp` shape

**Estimated:** 30-50 min sonnet.

### Slice 3 — `:wat::core::List<T>` primitive

**Substrate work:**
- `Value::wat__core__List(Arc<LinkedList<Value>>)` variant added
- Type registry entry for `:wat::core::List<T>` (parametric per arc 109 slice 1e four-of-five parametric heads)
- Parser support — `(1 2 3)` already parses to wat-edn `Value::List` in wat-edn; bridge that to wat's `wat__core__List`. Wat-source-level: does wat source support `(...)` as List literal? (open: arc 215 added `[...]` Vector literal; `{...}` Map literal; `#{...}` Set literal. Per arc 215 precedent, `(...)` could become List literal in wat source.)
- Holon-rep extends Sequential to wrap `LinkedList<Value>` (verify Sequential.wat handles both Vec + LinkedList input shapes)
- Dispatch arms: length, empty?, first, rest, conj (List/conj = prepend; Vector/conj = append — semantic difference is intentional), contains?, get, equality (cross-type with Vector per EDN spec)
- Hash impl extends Value's existing Hash to handle List with sequence-hashing (same as Vector)
- `src/edn_shim.rs` bridge: `wat_edn::Value::List(items)` ↔ `wat__core__List(Arc::new(items.into_iter().collect()))`
- Tests:
  - Construction (empty, single, multi)
  - Head/tail decomposition (first/rest)
  - Cons (prepend)
  - Equality with Vector (cross-type per EDN spec)
  - Round-trip via wat-edn (parse `(1 2 3)` → wat__core__List → write → parse → identical)
  - Hash compatibility with Vector (same-contents hash equal)
  - Cross-language: interop-tests/shape_matrix includes a `:list-3` shape

**Estimated:** 60-90 min sonnet (larger than Char because more dispatch arms + cross-type equality logic).

### Slice 4 — INSCRIPTION + USER-GUIDE + cross-references

- INSCRIPTION.md inscribed at arc closure
- `crates/wat-edn/docs/USER-GUIDE.md` updated — Char + List sections in the type-mapping appendix
- `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION-CHANGELOG.md` row added (wat language spec changelog)
- Cross-reference notes:
  - arc 164's DESIGN.md: add a top-of-file note pointing at arc 220 ("List minted under arc 220 per EDN-conformance trigger; this arc's SKIP rationale stands for AST/substrate scope")
  - arc 218 INSCRIPTION (when it closes): note that the BMP-only Char discipline propagated to wat-core in arc 220
- Memory update if any rune-style discipline emerges

**Estimated:** 20-30 min sonnet (paperwork pattern from prior arc closures).

## Out-of-scope

- **NOT minting `:wat::core::BigInt` / `:wat::core::BigDec`** — these exist in wat-edn (`Value::BigInt`/`BigDec` backed by `num_bigint` types). The wat-core mapping for these is a separate question (likely defer; user hasn't surfaced demand). If arc 219b's differential interop suite shows a gap, that becomes a follow-up arc.
- **NOT extending the type system for ordered sequences as a trait** — both List and Vector implement specific dispatch arms; no `:wat::core::Sequence` trait abstraction. Sequential is the holon-encoding abstraction, not a wat-type-system abstraction.
- **NOT touching wat-edn `Value::List`** — it already exists and is correct. Arc 220 only adds the wat-core side + the bridge.
- **NOT addressing the original arc 164 perf workload** — that's still a SKIPPED-conditionally separate matter. Arc 220's slice 2-3 don't optimize `:rest` performance; they add a new type that happens to have efficient `:rest`. The existing Vector-based `:rest` workload stays as-is.

## Verification matrix (acceptance criteria for arc closure)

- `cargo build --release` — workspace clean
- `cargo test --release --lib -p wat` — passes with new test count
- `cargo test --release -p wat-edn` — unchanged (wat-edn untouched by arc 220 except shim usage)
- `cargo clippy --release --all-targets -p wat -- -D warnings` — 0 warnings
- New round-trip tests for Char + List via wat-edn — green
- interop-tests/shape_matrix bin includes `:char-bmp` + `:list-3` shapes — handshakes PASS bidirectionally with Clojure
- Cross-type equality test: `Vector(1,2,3) == List(1,2,3)` per EDN spec

## Open questions (resolve before slice 2)

1. **Char literal syntax in wat source.** Per arc 207 Uuid precedent (constructor function only, no literal syntax) — does arc 220 follow the same pattern, or mint a literal? Recommendation: constructor-only for now (Char rare in wat source); revisit if a real wat-source workload surfaces demand.

2. **List literal syntax in wat source.** Per arc 215 precedent (`[...]` for Vector, `{...}` for Map, `#{...}` for Set added in 215.x), parens `(...)` could become List literal. BUT — `(...)` is ALSO function-application syntax in wat. Conflict resolution needed. Recommendation: constructor-only for List in wat source (`(:wat::core::List/of 1 2 3)`); the bare `(1 2 3)` stays as application syntax. Wat-edn parser preserves the List/Vector distinction on the wire; wat source uses explicit constructors.

3. **Cross-type equality with HashMap keys.** If `List(1,2,3) == Vector(1,2,3)` per EDN spec, what happens when both are used as HashMap keys? Per Hash invariant, equal values must hash equal. Implementation must hash both List and Vector via the same sequence-hashing function. Acceptable per arc 216 Stone 216.5a precedent (Value's Hash already handles cross-type semantics for HashSet/HashMap).

## Decision authority

These design decisions are inscribed by orchestrator (Claude Opus 4.7) under user direction 2026-05-22 ("ok - 220 with four slices - i agree"). Slice 2-3 BRIEFs may surface refinements; substantive changes update this DESIGN.md with a "Forward-correction" appendix per `feedback_inscription_immutable` (this is a DESIGN.md, not an INSCRIPTION.md — DESIGNs are living docs).

---

*The EDN spec speaks; wat answers. Char + List are the missing words.*
