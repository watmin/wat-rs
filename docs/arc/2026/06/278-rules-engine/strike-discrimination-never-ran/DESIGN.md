# DESIGN — D2p: the row that makes the others mean anything has never run

> Drawn 2026-09-06 at HEAD `69431429a`. Source: vigilia 2026-09-05 D2p (`probare`, driven by
> `experiri`). **Mechanism diagnosed on disk at THIS HEAD, exactly.**

## The site says what it is for

`src/rete/reachability.rs:1655-1665`:

> *"⛔ **DISCRIMINATION.** A constant matching NEITHER fact must select nothing — otherwise the
> operand is being evaluated but not compared, and **the rows above prove nothing**."*

```rust
let never = src.replacen(&format!("::= :v {other}"), "::= :v :zeta", 1);
if never != *src {
    assert_eq!(raw_count(&never), Ok(0), "`{name}`: a constant equal to no fact must select NOTHING");
}
```

By its own words this row is what makes the two assertions above it meaningful. **It never executes.**

## The mechanism, exactly

The loop is `for (name, src, other) in [("keyword", KW, ":beta"), ("enum", EN, ":probe::E::B")]`.

| | the rule's constant | `other` passed in |
|---|---|---|
| KW | `(:wat::rete::core::keyword::= :v **:alpha**)` | `:beta` |
| EN | `(:wat::rete::core::enum::= :v **:probe::E::A**)` | `:probe::E::B` |

`other` holds **the miss FACT's value**, not the rule's constant. So `replacen` searches for
`"::= :v :beta"` in a source containing `"::= :v :alpha"`, finds nothing, and returns the string
unchanged. `never == *src`, the `if` is false, and the assertion inside is skipped — **on both
iterations, every run, since the row was written.**

## ★ And six siblings in the same file do it correctly

`:1328`, `:1399`, `:1443`, `:1547` and `:1654` all guard their rewrites with `assert_ne!` —
and `:1654` is **four lines above this one, in the same block**:

```rust
assert_ne!(src, oracle, "the rewrite must select the oracle");
```

An `assert_ne!` would have failed the moment this row was written. The `if` converts *"the rewrite
did not happen"* into silent success. **Eighth instance in this arc of the tree stating a rule and
applying it in one place** — and here the correct form is four lines away.

## The one contract decision, pinned

**Make the rewrite target the rule's constant, and guard it the way its siblings are guarded.**

- the loop carries the **rule's constant** (`:alpha`, `:probe::E::A`) — the thing actually present in
  the source — not the other fact's value;
- the `if` becomes `assert_ne!`, matching `:1654` four lines up;
- then the discrimination assertion runs.

## ⛔ And the outcome is genuinely unknown

Once the rewrite works, `raw_count(&never)` must be `Ok(0)`. **Nobody has ever observed it.**

- **`Ok(0)`** → the discrimination holds, the rows above become meaningful, and a check that never
  ran now runs.
- **anything else** → the operand is being evaluated but not compared, which is what the comment
  warned about — **an ENGINE defect behind a check that has never executed.** That is a finding, and
  it needs its own strike.

Report the number either way. **Do not adjust the expectation to match what comes back.**

## Scope

**IN:** the rewrite target, the `assert_ne!`, and the observed `raw_count`. Floor GREEN.

**OUT, affirmatively cut:** curing any engine defect this exposes; the other `if`-guarded rewrites in
this file if any exist (name them, do not sweep them here); census B, D–M; F2; A4.

## Note on stakes

`src/rete/mod.rs` wraps this file in `#[cfg(test)]`, so this is a test-quality defect — but the
property it fails to check is an ENGINE behaviour: that a keyword/enum constant is compared, not
merely evaluated. Nothing else in the file covers it.
