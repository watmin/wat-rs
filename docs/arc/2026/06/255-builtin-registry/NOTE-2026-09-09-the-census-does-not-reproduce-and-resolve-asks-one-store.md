# ⛔ CORRECTED 2026-09-09 — THE HEADLINE BELOW WAS WRONG. THE CENSUS REPRODUCES.

**This NOTE was published claiming the 143/35 census did not reproduce. It does. My instrument was
defective, and the defect is the finding worth keeping.** The original text is preserved below the
correction, because the wrong instrument is the whole lesson.

## The instrument defect — an early `return` that short-circuits four checks

`is_resolvable_call_head` (`src/resolve/walk.rs:269`) is a CHAIN. The blanket is its FIRST rung and
it is an **early return**:

```rust
if is_reserved_prefix(head) { return true; }   // ← never falls through
if sym.get(head).is_some() { return true; }    // 2628 stdlib + user functions
if sym.has_unit_variant(head) { return true; }
if macros.contains(head) { return true; }
/* surface methods, single-segment accessors */
```

On clean main the blanket returns `true`, so the four rungs below it **never mattered for a
reserved-prefix name**. `BRIEF-STONE-resolve-asks-the-registry.md`'s recipe —

```rust
return registry().lookup_entry(head).is_some();   // ⛔ still an early RETURN
```

— keeps the early return and makes it return **`false`**, which severs `sym.get` and the three rungs
under it for every `:wat::*` name in the corpus. That is not "resolve asks the registry." That is
"resolve asks the registry INSTEAD OF everything else."

## The measurement, both ways, same tree, same 845 files

```
                                                    files    names
clean main (baseline)                                  36        —
⛔ registry as an early RETURN   (the brief's text)    600      309
✅ registry as a GATE, falling through                  97       14
```

`845 = wat-scripts/ (702) + wat-tests/ (81) + wat/ (62)`. The arc's own 2026-09-07 figure was
**143/35**; the fall-through instrument now measures **97/14**, and the difference is stone ①′ (the
membership facet, +75 rete names) landing in between. **The census held, and got better.**

The corrected instrument, verbatim:

```rust
if is_reserved_prefix(head) && crate::intrinsic::registry().contains(head) {
    return true;
}
// fall through to sym.get / unit variants / macros / surface methods
```

★ **The terminal step of the queue must be written this way.** "Delete the blanket and let `resolve`
ask `registry().contains`" is ambiguous between the two shapes above, and the shapes differ by
**564 files**. The blanket does not become a *replacement*; it becomes a **gate that falls through**.

## ★★★ THE REMAINING WORK IS 14 NAMES IN 97 FILES

```
② the `:- [...]` BOUNDARY — 4 names · 23 occurrences · pass = normalize.rs:461
   wat.type/Tuple 9 · wat.type/i64 7 · wat.type/String 5 · wat.type/Vector 2

③ real verbs with dispatch arms and no rows — 4 names · 350 occurrences · pass = walk.rs
   :wat::eval-ast! 337 · :wat::eval-with-defs! 6 · :wat::core::stream->pvec 5 · :wat::string::= 2

④ A FIFTH FAMILY THE DESIGN NEVER NAMED — 6 names · 8 occurrences · pass = walk.rs
   :wat::eval::walk 2 · :wat::core::Option.Some 2 · :wat::rete::f64::>X 1 ·
   :wat::kernel::panic! 1 · :wat::eval-step! 1 · :wat::core::i64/to-string 1
```

★ `:wat::rete::f64::>X` is **not a typo** — it is `wat-scripts/scratch-pad/probe-f64-comparator-bogus-head.wat`,
a deliberate witness whose own header records that it *"TYPE-CHECKS despite `:wat::rete::f64::>X`
never existing."* Under the gate instrument it goes RED. That is arc 255's founding promise —
*"the undefined-func class dies as a side effect"* — becoming visible for the first time.

★ `:wat::core::Option.Some` is **the dot spelling**, already in the corpus and already unresolvable.
It is the migration's own target, and it is in the worklist before the flip, exactly as the seam's
ordering caveat predicted.

## ② is real, and the DESIGN still names the wrong pass

Every one of the four type names carries `"namespaced symbol ref — not a builtin, not a registered
function (arc 251)"` — `src/resolve/normalize.rs:461` — **not** `walk.rs`'s `"call head …"`. They are
`WatAST::Symbol`s (`wat.type/Tuple`), refused by `resolve_namespaced_symbol`, which asks
`is_resolvable_call_head` **"may this symbol be rewritten to this keyword FQDN?"** — a call-head
predicate answering a question about a TYPE position.

`walk.rs:87` carries the arc-109 `:-` type-reference guard. `normalize.rs` runs FIRST and has none.
The walk.rs comment already names this class — *"the expander was taught this first; the resolver is
a SECOND, INDEPENDENT consumer of the same shape and was not"* — and normalize is its **third**
consumer. `[[feedback_a_slot_with_two_implementations_is_two_slots]]`

⚠ Corpus-wide (1952 files, incl. `tests/`) the family is **eight**: `+ bool · nil · f64 · HashMap`.
Four is a property of the 845 scope, not of the defect.

## The DESIGN's own arithmetic

`DESIGN-the-blanket-dies-in-three.md` states **35 distinct names** and decomposes
**11 (rete) + 4 (type) + 4 (verbs) = 19**. Sixteen were unaccounted for; ①′ has since absorbed the
rete eleven, and the fall-through measurement resolves the rest — the residue is the ④ family above.

---

# ─── ORIGINAL TEXT, PRESERVED AS WRITTEN (headline REFUTED above) ───

The original claimed: *"the census does not reproduce — 564 of 845 files break, on 309 names,"* and
concluded that `registry().contains` was insufficient because *"46 of the 309 refused names are
defined by a `defn` in an embedded stdlib file"* — naming `:wat::test::assert-eq`,
`:wat::core::map-indexed`, `:wat::fix::structural?` among them.

**Every one of those 46 was a false positive manufactured by the early `return`.** A dump of
`symbols.functions` taken at step 7 — immediately before `normalize_symbol_refs` — shows **2628
registered functions, with `:wat::test::assert-eq`, `:wat::fix::structural?`,
`:wat::core::map-indexed`, `:wat::core::take-while` and `:wat::core::remove` all PRESENT.** The
stdlib's surface was in `sym` the whole time; my instrument simply never let resolve reach it.

The one observation that survives intact, and that led to the correction:

> `(:wat::test::assert-eq 1 1)` → resolve says unresolved, while `(:wat::test::assert-eq 1 1 "x")` →
> the type-checker reports its exact arity. Two passes, one process, one name, disagreeing.

That contradiction was real and it was the thread worth pulling. The conclusion drawn from it —
*"resolve asks one store; the truth is a union"* — was **wrong**: resolve asks a five-rung chain that
already IS the union, and the instrument had cut four rungs off it.

★ **The lesson, and it is the expensive one:** I built the instrument from the brief's own prose,
measured 4× the expected result, and wrote the discrepancy up as a finding about the CODE instead of
first suspecting the INSTRUMENT. A number that disagrees with the record by 4× is a claim about the
measuring device until the device has been controlled. The control that would have caught it costs
one line: *does a name I KNOW resolves today still resolve under the instrument?*
`[[feedback_a_green_from_a_mis_aimed_probe_is_indistinguishable_from_a_working_gate]]`
