# DESIGN — the tag: `#wat.doc/Row` and `#wat.doc/Alias`

> **Builder, 2026-09-04:** *"can we print rust smart comments as an edn tagged value? … we can
> further impose not just triple back ticked end, but also a legitimate wat tagged edn record
> entity?"* → *"what's the wat edn tagged value that'll be used for metadata-maps?"*

## THE NAME IS NOT A CHOICE — the convention is already ruled

`src/edn/render.rs:43` states it: **`Tagged #wat.<home>/<TypeName>` — arc 294.i: per-type home,
not a shared bucket.** The tree is full of them: `#wat.check/ArityMismatch`, `#wat.ast/Program`,
`#wat.runtime/TypeMismatch`, `#wat.core/Span`, `#wat.core.Option/Some`.

So the home is **`wat.doc`**, and the type name follows this substrate's own noun. It calls these
things **rows**, everywhere and without exception — *"registry rows 571"*, *"alias rows 52"*,
`:wat::intrinsic::Row`. **`Row` it is.**

## ⛔ CORRECTED 2026-09-04 BY PROBE — `::` IS ILLEGAL AS AN EDN VALUE

This document first spelled every value as a wat FQDN keyword (`:wat::runtime::Purity::Pure`).
**`wat_edn::parse` refuses it** — `InvalidKeyword("keyword begins with :: ")` — so not one row
would have parsed. EDN keywords are `:name` or `:ns/name`. Every value below is now the ns/name
form the substrate ALREADY emits (`:wat.core/defn`, `:wat.config/set-capacity-mode`), built by the
same `::` → `.` transformation that builds every tag in the tree. Measured in
`[[SCORE-the-two-probes-and-a-third-muted-wall]]`.

## TWO TAGS, BECAUSE THERE ARE TWO KINDS OF ROW

```edn
;; a row that DECLARES its own properties
#wat.doc/Row {
  :added       "1.0.0"
  :purity      :wat.runtime.Purity/Preserving
  :determinism :wat.runtime.Determinism/Preserving
  :totality    :wat.runtime.Totality/Preserving
  :expand-time :wat.runtime.ExpandTime/Legal
  :category    :wat.runtime.Category/ControlFlow
  :args   [{:name a :type :wat.core/i64 :doc "the left operand"}]
  :ret    {:type :wat.core/bool :doc "whether a is strictly greater than b"}
  :syntax "(:wat::core::defclause :name [-> :T] ([args] body) ...)"
  :examples [{:src "(:wat::rete::i64::> 2 1)" :yields "true"}]
  :see [:wat.core/foldl]
}

;; a row that DELEGATES — it has NOWHERE to put an axis
;; ⚠ CORRECTED 2026-09-04 — this used to carry :args/:ret. It should not, and the reason is
;; measured: the resolution pass copies FIVE fields and NOT the signature, nothing compares an
;; alias's arity to its target's, and 16 of 52 alias rows already disagree — including two
;; aliasing the SAME target with arities 1 and 2, because one author wrote one @arg line and the
;; other wrote two. See [[NOTE-an-alias-restates-its-signature-and-nothing-checks-it]].
;; :examples STAY: an example at the alias name demonstrates that name.
#wat.doc/Alias {
  :added "1.0.0"
  :alias :wat.core/foldl
  :examples [{:src "(:wat::core::reduce + 0 [1 2 3])" :yields "6"}]
}
```

The split is not invented for this design — it is **already the enforced law**, and the arithmetic
shows it: `520 axes + 38 alias = 558 = every row`. Every row declares five axes **or** is an alias,
never both, never neither. `crates/wat-macros/src/wat_intrinsic.rs` refuses the contradiction today
with a real `compile_error!`, `DocError::AliasDeclaresAxis`.

★ **So the two tags do not ADD a wall — they change a checked rule into an unrepresentable state.**
`AliasDeclaresAxis` is a check that must be written, maintained, and remembered for every new
axis-like field. Two records with disjoint fields need no check at all: an `Alias` has no `:purity`
key to fill. That is one rung up, and it generalises to every future mutual exclusion for free.

## WHAT THE MAP BUYS THAT `@name value` CANNOT

- **Typed values, not bare tokens.** `:category :wat.runtime.Category/ControlFlow` names a real
  enum variant the reader can resolve. `@Category ControlFlow` is a token compared as a string —
  and *"a string comparison with one side normalized and the other not"* is this campaign's single
  most recurrent defect class.
- **`:args` stops being positional micro-syntax.** `@arg a :wat::core::i64 the left operand` is
  name/type/prose separated by spaces, with arity implied. **19 rows currently lie about their
  arity** because `#[wat_intrinsic]` derives it from the Rust signature shape. A vector of maps
  with an explicit `:rest` is declared, not inferred.
- **One shape, two homes.** `:wat::core::sort` already declares all five axes in a `{...}` doc-map
  in `wat/core.wat`. Rust and wat currently spell one concept two ways. After this they do not —
  which turns the FOURTH registry (`registry()` reading `sym.binding_metadata`) from *translating
  between two grammars* into *reading one shape from two places*.
- **A delimiter instead of an inference.** `@name value` has no start and no end; the parser guesses
  by line prefix. A fence has both.

## THE FENCE

    /// ```edn
    /// #wat.doc/Row { ... }
    /// ```

`edn` as a tag is precedented — `scheme` (×2) and `text` (×42) already appear in Rust doc comments
here and the build is green. And an unknown-to-rustdoc tag means the block is **not** compiled as
Rust, which is exactly right: it is data.

⚠ **This is the one thing to PROBE before writing anything** (FM 2-bis — a composition claim earns
its assertion empirically): the doctest gate is armed now, so an `edn` fence that rustdoc mishandles
goes RED at the floor rather than silently. Write one row in the new form, run
`scripts/floor.sh`, and read the result before the migration is briefed.

## THE OTHER UNPROBED CLAIM

`#[wat_intrinsic]` must parse EDN at **expand time**. Whether `wat-edn` can be a proc-macro
dependency is **not measured**. If it cannot, the shape changes before anything is written down —
that probe and the fence probe are the two gates on this whole effort.

## MIGRATION — the ratchet from day one, not a drop at the end

The builder's path: *"we support both syntaxes while we clean up... then... as we near the end... we
just drop support for the `@name val` syntax... and all heretics self identify."*

**With one change: freeze the `@`-form names in a shrink-only ratchet on day one.** Otherwise the
migration window is a period in which a NEW row can still be written in the old form, and the list
grows behind the cleanup. With the ratchet, heretics self-identify at every commit rather than once
at the end, and the final drop is mechanical because the list is already empty. Same instrument as
`REGISTRY_MEMBERSHIP_GAP_A`/`GAP_B`, which took the corpus 121 → 37.

★ **And the tool: reuse the proc-macro's OWN `@`-parser to emit the map.** Parsing with the exact
code that defines what `@` means makes the transform faithful *by construction* rather than
faithful-if-the-regex-is-right — which matters at 558 rows, and is the lesson of every census this
campaign got wrong.

## THE PART TO DESIGN COLD, BEFORE STARTING

`@example` ×459 + `@example-norun` ×139, each holding wat source with quotes and `#=>`. Inside an
EDN map those become strings that need escaping, and that is where a rushed migration goes quietly
lossy. Settle it up front: `:examples [{:src "…" :yields "…" :run false}]` with the escaping rule
written down, or examples keep their own fence outside the map. **Do not discover this at row 300.**

## THE ROUND TRIP THIS MAKES POSSIBLE

`#wat.doc/Row` is what is **written**; `:wat::intrinsic::Row` is what the registry **answers**.
Two records, one fact. `probe_can_doc_types_reconstruct_the_checker_scheme` already runs exactly
this shape for TypeSchemes at **432/432** — a doc-row → registry-row round trip is the same gate,
and it would make the declaration surface answerable to itself.
