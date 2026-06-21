# DESIGN — `first`/`second`/`third` become BARE, raising (forced forward from the 251 note)

## What
`first`/`second`/`third` on the runtime-length sequences (`Vector`/`List`/`PersistentVector`/`WatAstList`) stop
returning `Option<T>` and instead return **bare `T`, raising on empty/out-of-range** — exactly like `nth`
already does. `get` remains the lone `Option`-returning safe accessor. `Tuple`-`first` is unchanged (already
bare-total: arity proves presence). `HashSet` stays rejected (∅ — unordered, no "first").

This was recorded as deferred in `251-types-as-forms/NOTES.md` (*"the first not being an option is a legit arc…
251 can record the need"*). The container annihilation we are on (drift kill `75356ecc` + the registry) put the
accessor contract in our hands right now — so the deferral is **forced forward to high priority and done now,
not in 251.** (251's note gets marked completed when this ships.)

## Why — argued, not asserted (the builder asked for the case)
1. **Ergonomics is the actual want, and in a typed language raise is the only way to give it.** Ruby's `first`
   returns `nil`; a Ruby dev *experiences* `xs.first.foo` as zero-ceremony — nil just flows. wat has **no untyped
   nil** (`nil` is not an inhabitant of `Vector<T>`), so we cannot replicate "first returns nil." The two typed
   options are `Option<T>` (preserves Ruby's nil-*semantics* but discards its *feel* — every use is an unwrap) or
   **bare-raise** (preserves the feel — `(first xs)` used directly; blows up only when there is no first, exactly
   as `nil.foo` would in Ruby). The named convenience accessor should be convenient; raise is the faithful port.
2. **Decomplect — it kills a real redundancy.** Today `first` and `get 0` BOTH return `Option<T>`; `first` adds
   nothing but a name over `get 0`. After the flip, `first` is the bare ergonomic accessor and `get` is the one
   `Option` path. One safe path, named raising accessors — Hickey-simple (un-braided), strictly fewer concepts.
3. **Consistency with `nth`.** `nth` is already bare-raising — `core.wat:497`: *"there IS an i-th element; give
   it or fail."* `first`/`second`/`third` ARE `nth 0/1/2`. `first`-Option-while-`nth`-bare is a split with no
   principle under it; this removes it.
4. **The 251 open question has one answer.** The note asked: bare ⇒ raise, or a typed nil? wat is typed and has
   no `nil` valid in `Vector<T>`, so **bare ⇒ raise**. Settled by the type system, not preference.
5. **Honest-typing is satisfied, not violated.** Raise-on-precondition is NOT a type lie — `nth` does it and we
   call it honest; the doctrine forbids *silent* dishonesty (made-up fields, sentinel garbage on empty), not a
   partial function that fails loudly on a documented precondition. The safe path (`get`→`Option`) stays, so the
   type system still forces **assert** (`first`/`nth`, raise) or **handle** (`get`, Option). Only the ergonomic
   default moves.

Four-questions: Obvious ✓ ("first" means the first; the Option-wrapper was the unobvious part) · Simple ✓ (one
safe path + bare accessors; kills the first/get redundancy) · Honest ✓ (raise-on-precondition, safe path kept) ·
Good UX ✓ (Ruby/Clojure muscle memory, no ceremony). All four hold.

## The change (two functions)
- `eval_positional_accessor` (`runtime.rs:10944`): Vec/List/PV/WatAstList arms return the **bare element**,
  **raising** on out-of-range (a `RuntimeError`, the shape `nth` raises), instead of `Value::Option(...)`. Tuple
  arm unchanged. (This supersedes the `Option` return that `75356ecc` gave PV/WatAst — the drift fix's PARITY
  holds; only the shared return shape changes Option→bare.)
- `infer_positional_accessor` (`check.rs:9991`): Vec/List/PV/WatAstList arms return bare `T` (not `Option<T>`).
  Tuple arm unchanged. HashSet still `_ =>` rejected.
- Consider reimplementing as `nth`-sugar (`first` = `nth … 0`) — same raise path, one source of truth. Decide in
  the strike; the contract is what's pinned.

## The fallout sweep (the bulk; the cascade is the worklist)
Flipping the two functions reddens the type-checker at **exactly** the affected callsites — every
`(Option/expect … (first <seq>) …)` (now double-wrapping a bare T) and every `(match (first <seq>) (Some…)(None…))`
(now matching a T). **Tuple-`first` callsites never error** (always bare). So the cascade IS the classification —
no need to pre-sort sequence-vs-tuple by hand.

Gross counts (real affected set ≤ these; the cascade gives the exact number): **243 wat** (`wat/` + `wat-tests/`
+ `examples/`) + **~159 Rust-embedded** (`tests/*.rs` 79, `src/*.rs` 80).
- **wat source → `fix-wat`** (the targeted wat codemod). Two transform classes: `(Option/expect -> T (first x)
  m)` → `(first x)` (mechanical); `(match (first x) (Some v) e1 (None) e2)` → switch to `(get x 0)` (the empty
  case genuinely wants the safe path). The mechanical class is a clean fix-wat rule (build it if absent —
  dogfood); the match class needs per-site judgment.
- **Rust-embedded wat strings → hand** (no Rust-source wat-parity tool yet — accepted). `tests/*.rs` + `src/*.rs`.
- **Core probes updated**: `probe_seq_container_parity`, `probe_seq_container_registry` (they currently `Option/
  expect` first → become bare).

## Decomposition (a campaign, not one strike)
1. **Flip-core** (this strike): the 2 functions + the new RED probe + update the 2 core probes → green. Isolated
   semantic change.
2. **Cascade sweep**: fix-wat over wat source (build the unwrap rule if needed); hand the Rust-embedded; drive
   the fail-count to zero.
3. **Floors green** (lib/deftest/nursery/deporder), commit per phase, push.
4. **Mark `251-types-as-forms/NOTES.md` completed** — assert done, with the resolution (typed ⇒ bare ⇒ raise).

## The RED probe (flip-core contract)
`tests/probe_first_bare_accessors.rs`: `(first <non-empty seq>)` used **bare** → the element (no Option/expect);
`(first <empty seq>)` → **raises**; across Vector/List/PV. RED at HEAD (first returns Option → using it bare is a
type error). GREEN after the flip. Tuple-first bare still works (regression guard).

## Downstream lint rule (queued — born from this cut; arc-277 fix-wat territory)
The flip makes `first`/`second`/`third` and `get 0/1/2`+assert *exactly* equivalent, which creates a clean
idiom-enforcement rule (spotted watching the cascade do its inverse). **Build after first-bare lands.**

PROMOTE → the bare named accessor (both sides bare-raise; exact equivalence):
- `(:wat::core::nth xs 0)` → `(:wat::core::first xs)`   (`1`→`second`, `2`→`third`)
- `(:wat::core::Option/expect -> T (:wat::core::get xs 0) "…")` → `(:wat::core::first xs)`  (`1`/`2` likewise)

GUARD (the non-rule — must NOT fire here, or it changes behavior):
- `(:wat::core::match (:wat::core::get xs 0) ((Some v) …) (None …))` STAYS `get` — legitimate empty-handling;
  rewriting to `first` would raise and delete the `None` branch.

The principle the rule enforces: **`get` asks "is there one?" (Option); `first`/`second`/`third` assert "give me
the one" (bare, raise).** The lint fires only when a `get`/`nth` result is *asserted present* (Option/expect, or
a literal 0/1/2 index on `nth`), never when it is *matched for emptiness*. A `fix-wat` autofix; keeps the corpus
on the idiom side of the contract this cut establishes.

## Done = green (flip-core)
`probe_first_bare_accessors` green; the 2 core probes updated + green; `cargo build --release` clean. (The wider
suite goes red until the cascade sweep — that is the sweep's worklist, tracked, not a regression.)
