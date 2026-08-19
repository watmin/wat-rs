# DESIGN STONE — 118.B4-0 · `nth` becomes a Rust intrinsic; the wat clause becomes its ORACLE

**Route B, inserted 2026-08-18 ahead of B4-ii.** Not a choice between options — a prerequisite the
disk produced when B4-ii's codemod was struck and could not load.

## What forced it

B4-ii rewrote 44 sites to `(nth X n)` and the stdlib stopped starting:

```
StartupError   wat/service.wat:468, inside defservice's macro body
  #wat.runtime/UnknownFunction {:path ":wat::core::nth"}
```

A macro program body evaluates through `dispatch_keyword_head` — the Rust intrinsic dispatch.
`(first (drop X n))` works there because **both halves are intrinsics**. `nth` is not one:

```
                rust dispatch arms      wat definitions
  first                  1                    0
  second                 1                    0
  third                  1                    0
  last                   1                    0
  get                    1                    0
  nth                    0                    2      ← wat/core.wat:1417
```

## ⛔ MY FIRST FIX WAS A CATEGORY ERROR — recorded because the shape recurs

I added `":wat::core::nth"` to `is_pure_total` (`src/macros/eval.rs`) and rebuilt. The error changed
from *refused by the gate* to *UnknownFunction* — because that list is not "pure heads". Its own
header says it is **"the pure-total subset of `dispatch_keyword_head`"** — a filter over a population
`nth` was never in. Adding a wat-defined name to it asserts a membership that does not exist and
silently accomplishes nothing. **Reverted.**

The builder's question is what broke it open: *"is nth impure? … accessing a data structure is one of
the strongest definitions of purity."* Correct, and purity was never the axis — I had spent a whole
round four-questioning a totality distinction that the gate does not test. The gate's own first
comment admits `i64::/` because div-by-zero is *"a deterministic located abort, never a panic"*, and
its deny list is entirely effects: `:wat::kernel::*`, IO, spawning, `now`, random UUIDs, signals,
`apply`, `eval-ast!`. **`nth` fails nothing it tests. It was simply never a candidate.**

★ Once `nth` IS an intrinsic, that allow-list entry becomes correct — for the right reason. **Part of
this stone.**

## Two independent derivations — this is a *cannot*, not a preference

**1. From what `nth` now is.** B4-i promoted it to *the general positional door*. A door that cannot
be opened from an entire evaluation context is not general. Macro bodies reach only intrinsics;
therefore the general accessor must be an intrinsic. The alternative — spelling those sites
`(Option/expect (get X n) …)` — is the CONVENTION rung: *remember to write it differently in there.*
Builder, 2026-08-18: *"we do not do conventions - we do walls."*

**2. From the family.** `nth` is the only positional accessor that is not native. As **sugar** over
`Option/expect (get v i)` that was reasonable; wat is where sugar belongs. B4-i changed what it is.
The promotion is what turns non-nativeness from a fair choice into an inconsistency the family's own
membership exposes.

## The shape — the house's own, and I owe getting it right this time

```
nth-spec    wat/core.wat    THE ORACLE — correct-and-slow, B4-i's four arms and the Seqable walk
nth         src/runtime.rs  the native kernel — a dispatch arm beside first/second/third/last/get
            differential    the spec keeps the native honest
```

`wat/rete.wat:1508` is the recorded exemplar: `insert-all-spec` (wat oracle) / `insert-all'` (native
kernel) / `insert-all` (public). Here the public name IS the native, so there are two names, not
three. **B4-i's work is not discarded — it becomes the specification.** A wat clause with four arms
and an explicit `next`-walk is exactly what an oracle should look like.
`[[feedback_an_oracle_must_be_written_in_the_other_language]]`

## The ONE contract decision, pinned

**`nth` and `nth-spec` agree on every input, and a differential test proves it.** Same values, same
raises, same message, across all four receivers, in range and out. The native may be faster; it may
not be different.

## ⚠ THE CAPABILITY IS NOT `indexable()` — and this is a trap with a fuse

`first`/`second`/`third` route through `StreamContainer::indexable()`, and **B4-iii flips that to
`false` for Stream.** If `nth` shares that gate, the wall kills `nth` on a Stream three stones later,
silently, and the general door closes again.

`gettable()` is wrong too — it is `false` for Stream today, so `nth` would never reach a lazy seq.

**Mint a third capability** — general positional lookup by index — on the same narrow waist
(`src/collection/seq_container.rs`), so the exhaustiveness guarantee still forces both classifiers to
agree. `true` for Vector / PersistentVector / List / WatAstList / Stream; `false` for Tuple
(heterogeneous — a runtime index cannot be typed) and HashSet (unordered).

## ⚠ AN INTERACTION B4-iii MUST RULE — this stone does NOT decide it

If `nth` accepts a Stream, then **`(nth s 0)` is a `first`-equivalent**, and B4-iii's wall has a hole:

```
(if (empty? s) base (match (next s) ((Item v rest) … )))     with (nth s 0) available
```

is the 3× walk again, spelled around the closed door. That is the same "name the property, not the
symptom" failure the reachability wall taught, and closing `first` while leaving `(nth s 0)` open
would repeat it.

**B4-0 does not touch this.** It is a REPRESENTATION change — wat to Rust, semantics identical,
proven by the differential. The hole is newly *visible*, not newly *created*: `(first (drop s 0))`
was always available. **B4-iii owns the ruling**, and it now has to make it explicitly instead of
inheriting it.

## The four questions

- **Obvious? YES.** `nth` joins the five siblings it already belongs with; the wat text stays as the
  spec that keeps it honest.
- **Simple? YES.** One dispatch arm, one capability method, one rename, one differential test.
  Nothing new is invented — the oracle pattern and the capability waist both exist.
- **Honest? YES.** It removes a restriction rather than routing around it, and it says plainly that
  B4-i's clause is being demoted to a specification rather than deleted.
- **Good UX? YES.** `nth` works everywhere the language evaluates, including macro bodies — which is
  the property that made it "the general door" in the first place.

## ACCEPTANCE

| | assertion | instrument |
|---|---|---|
| 1 | ★ **a macro program body can call `(nth …)`** | a probe defmacro whose body indexes `ast->children`, mirroring `wat/service.wat:468` |
| 2 | ★ **differential: `nth` ≡ `nth-spec`** on Vector/PV/List/Stream, in range and past the end | a new `wat-tests/` differential, order-sensitive, with a non-vacuity control |
| 3 | Stream `nth` still visits exactly **i+1** cells | the force-shape probe, as B4-i proved for the wat clause |
| 4 | Vector/PV/List stay O(1) — no walk on the indexable path | read the dispatch; bench only if the read is ambiguous |
| 5 | `nth` on the allow-list is now **truthful** | it appears in `dispatch_keyword_head` |

Plus: floor ≥4756/0, clippy 0, ignores 13.

⚠ **Every run capped.** `systemd-run --user --scope -q -p MemoryMax=<N> -p MemorySwapMax=0 timeout <s>`

## Rooms

- `src/runtime.rs` ~5646 — the `first`/`second`/`third` dispatch arms; `nth` joins them, taking its
  index from an argument rather than a constant.
- `src/runtime.rs` ~15456 — `eval_positional_accessor`, which already takes an index parameter.
  **Read before reusing:** its Stream arm is index-0-only today and must walk for `nth`.
- `src/collection/seq_container.rs` — the capability waist; the new method and its two classifiers.
- `src/check.rs` — `nth` needs a `TypeScheme` registration it does not have as a wat clause.
- `src/macros/eval.rs` ~512 — the allow-list entry, now legitimate.
- `wat/core.wat:1393–1435` — the clause becomes `nth-spec`. **Its header's total-CONTRACT /
  partial-FUNCTION argument is correct and stays** — extend it, do not rewrite it.

## Out of scope — affirmative cuts

- **B4-iii's ruling on `(nth s 0)`** — named above, owned there.
- **The deeper class: a macro body cannot call ANY wat-defined function.** `nth` is one instance;
  the restriction is invisible until tripped. Bigger than this arc, tracked separately, and it must
  not hijack B4.
- **Widening `get`, or minting `Seqable/get`** — untouched.
