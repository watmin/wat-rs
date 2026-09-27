# AMEND — STONE 255.56: STOP-2 ruled; resume the stone

**Builder, 2026-09-27.** The brief `BRIEF-STONE-255.56-the-operators-ask-the-classes.md` stands, with these changes.
Everything else in it is unchanged: the gates, Refuse, E-a, Z1, the deletion, the tests.

## 1. `wat/doctest.wat:118` — pin the type with a typed consumer

`got` and `want` are self-describing data whose type nothing pins (`eval-ast!`'s `T` is fresh; `=` is their only
consumer). The builder ruled that **types are headers on function definitions**: no typed `let`, no ascription
(`ann-form`), no string comparison. The comparison moves into a function whose parameters say what it compares:

```clojure
(wat.core/defn wat.doctest/matches? [got :- wat.core/Equatable  want :- wat.core/Equatable] :- wat.core/bool
  (wat.core/= got want))
```

(Write it in the spelling the surrounding file uses, or the target spelling, as the file allows.) The walk calls it
where `(= got want)` stood. `=` stays plain data equality.

## 2. `wat/rete/acc.wat:68`/`:87` — dropped from the work list

They are not Refuse sites: `infer_ordering` unifies `v` with `cur` (`i64`) first. My brief listed them in error (see
`WEIGH-STONE-255.56-…`). Do not change them. The latent rete issue (`Element.bindings` is untyped) is carried to rete.

## 3. The three `tests/resolve/…fix_source_local_rules__contract-0{6a,6b,7}` files are golden data, not programs

The builder: *"the entire notion of goldens is data equality."* They are the fix-source codemod's expected output.
Under Refuse their census rc flips to 1, because `a` and `b` have no types. **That flip is expected. It is not
STOP-1.** Do not edit them. Name the three flips in the SCORE. How goldens are compared (as data) and marked (not
type-checked as programs) is the next stone's measurement.

STOP-1 and STOP-2 otherwise stand as drawn: any other site outside step 5 goes red → STOP.

Finish as the brief says: floor, clippy, pre/post census, delta; SCORE appended; commit locally; do not push; then
`pulsare_yield kind=scored`.
