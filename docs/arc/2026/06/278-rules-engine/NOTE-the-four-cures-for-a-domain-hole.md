# NOTE — the four cures for a domain hole, and which one to reach for

**Filed 2026-08-05. NOT a work item — the method for one.** Task **#64** (*"Every core primitive's
domain hole becomes a faced outcome — totality as the language's finality"*) has been on the board
as an aspiration with no method. The rete work has been producing the method as a side effect, and
this note pins it down while the context is live so #64 does not have to re-derive it.

Builder, this session: *"core's expressions are going to be massively scrutinized pending our rete
work…. the rete work is showing us how to build total forms that can be completely compiled into a
jump table…. once we get the expressivity we want in rete… we may just rewrite a ton of the core
forms to force totality everywhere — **that's not a now thing**."*

⛔ **It is not a now thing. Do not start #64 from this note.** This is a captured method, filed so
the eventual strike has a starting point instead of a blank page.

## The four cures, each with a shipped exemplar

By 2026-08-05 all four exist on the disk with worked examples. They are **not interchangeable**; the
choice is forced by what the op *is*.

### 1. Monomorphise — delete the hole

The generic op is partial *because* it is generic. `:wat::core::>` raises on incomparable operands;
`:wat::core::i64::>` cannot, because any two i64s compare.

- **Reach for it when** the partiality comes from admitting types that may not relate.
- **Exemplar:** the per-type comparator families; the ruling in
  `DESIGN-STONE-where-admits-only-rete-ops.md` (*"Monomorphising … deletes the domain hole"*).
- **Cost:** N rows instead of 1, forever, and every call site must name the type.
- ⚠ **Does not generalise.** It works only where the hole IS the polymorphism. `first` is per-type
  and *still* partial — an empty `PersistentVector` has no first element regardless of spelling.

### 2. Outcome enum — name every hole as a variant the caller faces

The op returns a sum type; the caller must `match`.

- **Reach for it when** the op is a **measurement** whose codomain would otherwise *absorb* its own
  undefined case — i.e. the sentinel is indistinguishable from a real answer.
- **Exemplar:** `CosineOutcome{Similarity, Degenerate, DimensionMismatch}`. The wall exists because a
  guarded `0.0` **means "unrelated"** in cosine's own codomain, and genuine unrelatedness measures
  `-0.0086`. The two were indistinguishable to a caller.
- **Cost:** every caller matches; the return type stops composing directly.
- **Kin:** `RecvOutcome`/`SendOutcome`/`SpawnOutcome` — the same cure at the IPC layer.

### 3. Caller-supplied fallback — the caller chooses what undefined means

The op takes a mandatory `:undefined` marker plus a value to substitute.

- **Reach for it when** the hole is real, the caller is better placed than the substrate to say what
  it means, and an enum would be ceremony at a call site that just wants a number.
- **Exemplar:** `(:wat::rete::core::i64::/ a b :undefined 0)`; `(… f64::/ …)`;
  `(… holon::cosine a b :undefined 0.0)`.
- **This is Ruby's `fetch(key, default)`, generalised.** wat's accessor trio maps exactly:
  `get` → `Option<T>` ≡ `a[i]`; `nth` raises ≡ `a.fetch(i)`; the fallback form ≡ `a.fetch(i, default)`.
- ⚠ **One spelling only.** `:undefined` is the marker (builder-ruled 2026-08-05). A second spelling
  of one marker is a second door around every wall built on the first (arc 179's `()`).

### 4. Answer the question exactly — the predicate's cure

A **predicate** may absorb its undefined case, *if* the honest answer to the question actually asked
is a definite `false`.

- **Reach for it when** the op returns a bool whose question has a defensible answer on the
  undefined input.
- **Exemplar:** `coincident?` on a dimension mismatch returns `false` — *"an undefined comparison is
  not below the floor, so the honest answer to the question actually asked ('are these the same
  point?') is `false`, by documented total contract."*
- **Builder's ruling, 2026-08-02:** `★★ THE MEASUREMENT IS FULL; THE PREDICATE IS EXACT.`
- ⚠ **This is the cure most easily abused.** It is legitimate for `coincident?` and was a *lie* for
  cosine's guarded `0.0` — and the difference is not the shape, it is whether the sentinel is
  **distinguishable from a real answer in that op's codomain**. Applying cure 4 where cure 2 belongs
  is exactly the defect the cosine wall was built to remove.

## ★ The discriminator, in one line

> **Is the undefined case distinguishable from a real answer in this op's own codomain?**
> If NO → cure 2 or 3 (the caller must be told, or must choose).
> If YES, and the op is a predicate whose question has a definite answer → cure 4.
> If the hole exists only because the op is polymorphic → cure 1.

## What #64 will additionally need, and does not have

1. **A census.** How many core verbs are partial? Unmeasured. `purity.rs`'s `total` list is the
   nearest thing and it is a *whitelist*, not an inventory of the complement — and it was measurably
   wrong in both directions as recently as 2026-08-05 (three f64 comparators missing; generic `<`
   falsely present).
2. **A ruling on the arity break.** Cure 3 changes call-site arity. For a core verb with existing
   callers that is a migration, not an addition — unlike the rete surface, where the row is new.
3. **The compilation question, which is the builder's actual driver.** Whether `compiled_where`
   (#49a) *inlines* wat-level defns or requires every head to be a Rust opcode is **undecided**, and
   it decides whether cure-3 forms must be intrinsics. Grounded 2026-08-05: the vocabulary already
   carries two wat-level verbs (`map`, `reduce`) without incident, and `nth`'s body bottoms out in
   two intrinsics one hop down — so nothing on the disk yet establishes that a wat-level verb blocks
   compilation. **If it does, the class is at least those three, not one.**

4. **★ A CURE-3 BLOCKER, measured 2026-08-05 when `nth` was attempted and STOPped.** The fallback
   arm substitutes on `Err(EvalBreak::Diagnostic(e))`. **`Option/expect` does not return an `Err` —
   it `panic_any`s** (`runtime.rs`'s `expect_panic`, whose own doc says *"then `panic_any`s. Never
   returns."*). A panic unwinds straight past the arm; the only `catch_unwind` sites are at
   spawn/sandbox boundaries, nowhere near this dispatch.

   So **cure 3 cannot be applied to any verb whose partiality is expressed as a panic** — it reaches
   only verbs that fail by returning `Err`. `nth` is the first case found and it is certainly not the
   last: `Option/expect` is a general-purpose escape hatch, so every wat-level verb defined as
   `Option/expect (…)` inherits the same block.

   This is directly the builder's compilation driver: **you cannot compile a jump table whose opcodes
   may unwind.** It is also R53's territory one layer down — *"a failure must show its true face as a
   matchable VALUE, never a raise"* — the recv'/send' walls annihilated that class at the IPC layer
   and `Option/expect` is a survivor of it in core. Kept honest: `expect` is *documented* to panic, so
   it is **not** a hidden failure and not a mask; the no-hidden-failures LAW is not violated. It is a
   **compilability** problem, not an honesty one, and #64 will have to decide whether cure 3 requires
   `Option/expect`'s panic to become a returned value first.

   **Meanwhile the capability is not missing, only the ergonomics:** `:wat::rete::core::PersistentVector/get`
   is minted, returns `Option<T>`, and is already total — so indexed access with a default is
   expressible today via `get` + `match`, just verbosely. `nth` would have been the sugar.

   ### ⛔ RULED 2026-08-05 — and it dissolves the "blocker" above

   Builder: *"i feel like… we just disallow expect in rete … we'll purge from wat-core later."*
   And, reasoning to it: *"expect needs to return an enum…. but….. option already does?… do we just
   flat out kill expect?"*

   **That is the whole answer, and it inverts the framing above.** `expect` is not a partial op
   awaiting cure 3. **`Option<T>` IS cure 2, already applied** — the domain hole is already a named,
   matchable variant. **`expect` is the DISCARD of an already-faced outcome**: it takes something
   total and hands back something partial. There is nothing to add to it; there is something to stop
   doing.

   So a fallback row wrapping `nth` was **the wrong idea, not a blocked one** — it would have
   legitimised a panic inside a rule condition by dressing it in a total signature. The STOP was more
   correct than it looked.

   **⛔ NEVER MINT a rete row for `Option/expect`, `Result/expect`, or any verb defined in terms of
   them** (`nth` is the known instance; the transitive rule below finds the rest).

   **And nothing needs building — the machinery already refuses them, grounded 2026-08-05:**
   - `:wat::core::Option/expect` is **absent from `purity.rs`'s `total` list**, so the third conjunct
     refuses it the moment S7 arms.
   - `classify_fn` walks a wat `defn`'s body (`FunctionBody::Wat(body_ast) => classify_expr(…)`), so
     **`nth` inherits non-totality transitively and automatically** — no special case, no allowlist.
   - Both conjuncts are UNARMED today, which is the only reason an `expect` inside a `where` passes
     right now (it is pure ∧ deterministic — a panic is neither impure nor nondeterministic).

   **The measured scope of the later core purge, so it is not discovered cold:** 168 `Option/expect`
   + 27 `Result/expect` registrations, and call sites across `wat/` 185 · `wat-scripts/` 207 ·
   `tests/` 153 · `wat-tests/` 23 · `src/` 49. A real crusade, correctly cut out of the rete work.
   `Option/try` already exists as the non-discarding sibling — the purge's target form, not a thing to
   invent.

## Related, on the disk

- `DESIGN-STONE-total-the-third-axis.md` · `BRIEF-total-column-honest.md` — the axis and its audit.
- `DESIGN-STONE-where-admits-only-rete-ops.md` — the per-type ruling and *what "total" MEANS*
  (builder-ruled, stricter than IEEE).
- `BRIEF-cosine-outcome-wall.md` · `DESIGN-recv-outcome-wall.md` — cure 2, twice.
- `BRIEF-f64-fallback-rows.md` · `DESIGN-STONE-the-vsa-seam-opens.md` — cure 3, and its three
  distinct runtime failure modes (raise / non-finite return / outcome-enum return).
