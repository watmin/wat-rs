# SCORE — 109 step ①b: the remaining three

Rider: ~15 min, one flight, no STOP triggered. Every row re-run by the orchestrator's own hand.

| # | what | result |
|---|---|---|
| 1 | `PersistentVector [T] 1 2 3` | ✅ `#wat.core/PersistentVector [1 2 3]` |
| 2 | `PersistentMap [K V] "a" 1` | ✅ `#wat.core/PersistentMap {"a" 1}` |
| 3 | `Tuple [T1 T2] 1 "a"` | ✅ `[1 "a"]` |
| 4 | ★ **the NOTE's gap** | ✅ see below |
| 5 | bracket-less forms unchanged | ✅ all three byte-identical output |
| 6 | step ①'s three untouched | ✅ `Vector` / `Vector` positional / `HashSet` |
| 7 | ★ the collision the rider found | ✅ `(Tuple [1 2 3] "tag")` → `[[1 2 3] "tag"]` |
| 8 | `collection/eval.rs` net-zero | ✅ |
| 9 | floor | ✅ **4818/4818, 69.7s** |
| 10 | clippy | ✅ 0 |
| 11 | rustfmt | ✅ parity after I fixed 3 lines of my own (see deltas) |
| 12 | goldens | ✅ 7 bumped — and the delta was NOT the net (see below) |

## ★★ ROW 4 — the gap `109/NOTE-typed-literal-constructors.md` FILED AND COULD NOT FIX

Filed 2026-07-18: *"a literal constructor cannot declare its element/value type… you cannot write a
literal whose declared element type is a common supertype holding heterogeneous elements."*

Its own worked example, run today, with the bracket-less form as the control:

```
(PersistentMap 0 (A :x 1) 1 (B :y "s"))
  → parameter value #2 expects :user::A; got :user::B          ← the note's exact failure

(PersistentMap [:wat::core::i64 :wat::core::Record] 0 (A :x 1) 1 (B :y "s"))
  → #wat.core/PersistentMap {0 #user/A {:x 1} 1 #user/B {:y "s"}}
```

**Two record types under one declared supertype.** The note is closed by this stone.

★ And it was closed **without touching unification**, which was STOP-1's whole worry. The rider found
`assignable` (`check.rs:15036`) — pre-existing directional up-cast, `is_subtype`-based, already used
by `check_vector_literal_against` / `check_map_literal_against` / `check_tuple_constructor_against`
for annotation-directed checking. With a bracket present, elements are checked via `assignable`
against the DECLARED type; without one, the old `unify`-against-a-fresh-var path is untouched. It
extended an existing mechanism rather than inventing one.

## ★ THE RIDER CAUGHT A COLLISION MY BRIEF WOULD HAVE SHIPPED

Step ①'s `unwrap_type_param_bracket` treats a leading `WatAST::Vector` as a bracket **unconditionally**
— safe there, because `Vector`/`HashMap`/`HashSet` always required a leading type keyword, so a vector
literal was never legal in that slot. **Not true for these three.** The rider found a live test —
`tests/collection/probe_arc216_stone7_tuple_roundtrip.rs`, `(:wat::core::Tuple [1 2 3] "tag")` — whose
first Tuple element IS a `Vector<i64>` literal. Unconditional splicing would have silently
reinterpreted it: STOP-2, shipped.

Instead it wrote `is_type_bracket_candidate` — a leading vector is a bracket only if **non-empty and
every element is a bare `WatAST::Keyword`**. Made `pub(crate)` and called from BOTH check and runtime
(7 uses / 6 uses), so the two can never disagree about what a bracket is. Verified: the collision case
returns `[[1 2 3] "tag"]`.

### ⚠ The residual ambiguity is REAL, bounded, and measured — not waved off

The discriminator is a heuristic, not a proof. Measured:

```
(Tuple [1 2 3] "tag")   → data vector, passes through   ✓
(Tuple [:a :b] "tag")   → read as a TYPE bracket; `:a` is not a type → error
```

So **a data vector of bare keywords cannot be the first element of these three** during the additive
window. The rider grep-verified no site in `wat/`, `wat-scripts/` or `tests/` does this, so it is
vacuous today — and it resolves BY RULE at step ③, when the bracket becomes mandatory and the first
slot is unambiguously the type list. Recorded here so ③ inherits it as a known property rather than
a surprise. This is the CHECK rung, and ③ is what lifts it to no-form.

## ⚠ THE GOLDENS: THE DELTA WAS NOT THE NET — the first time that bit today

```
check.rs    net +183     ← WRONG to apply
            11 hunks; SIX at 13943–14148 sit BELOW the pins at 13488/13506 and shift nothing
            true delta +79   → 13567, 13585
runtime.rs  net +28      ← correct, both hunks above the pin → 25244
```

The seam's standing warning — *"one stone's numstat said −64 while only −12 sat above the pinned
site; confirm which hunks PRECEDE each pinned line; never apply the net"* — earned its keep. Had I
applied +183 the goldens would have gone red a second time with a number that looked derived.

⚠ **And then I moved them again myself.** Fixing three rustfmt nonconformances of my own in
`runtime.rs` added 6 lines above the pin: 25244 → **25250**. Re-floored rather than assumed.

## Honest deltas

- **Every line number in my brief was correct this time** — the rider confirmed each by matching
  surrounding code. Nine numbers, zero wrong, against six wrong two stones ago.
- **`Tuple` arity mismatch** reuses the existing `check_tuple_constructor_against`, so a bracket/value
  arity mismatch is a hard `ArityMismatch` — never truncated, never padded, consistent with every
  other arity check in the file.
- **`collection/eval.rs` is net-zero again** — the rider spliced at the dispatch call site, not inside
  the callee ctors, mirroring step ①'s own convention.
- **rustfmt**: 3 lines of mine in `runtime.rs`, found by CONTENT-diffing the offending lines against
  HEAD rather than by line number — the instrument that misled me earlier today. Parity restored 16=16.

## Where the parametric bracket now stands

**All six constructors accept `(Head [type…] …values…)` in both the checker and the runtime**, and
every bracket-less form still behaves exactly as before. Step ① is complete.

Next, in order: the `is_holon_arg_canonical` arm (`runtime.rs`), which gates ② — after the codemod
every `Vector` in a holon-constructor argument reaches it; then ② the corpus codemod; then ③ the angle
form becomes illegal.
