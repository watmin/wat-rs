# BRIEF — STONE 218.8: the 219 correction (`:` and `#` as symbol constituents)

**Drawn 2026-09-20 against `main` @ `1b5ccb783`** (floor 5921/5921, clippy 0, census `no STOP-8`).
Follows 218.7, which reported arc 219's premise failure and was forbidden from acting on it.

## THE BUILDER HAS RULED — 219 REOPENS

He verified it in a live REPL, and not merely that Clojure *reads* them — that they are **definable
and callable**:

```clojure
user=> (defn a:b [x] (+ x 1))
#'user/a:b
user=> (a:b 2)
3
user=> (defn a#b [x] (+ x 1))
#'user/a#b
user=> (a#b 2)
3
```

> *"ok - so - i agree with 219 concern"*

Arc 219 removed `:` and `#` from `is_symbol_continue` citing a spec summary that omitted
*"Additionally, `: #` are allowed as constituent characters in symbols other than as the first
character."* That made `wat-edn` stricter than the EDN spec **and** stricter than Clojure.

## ⛔⛔ IT IS NOT A ONE-LINE REVERT. A NAIVE ONE SHIPS THREE NEW BUGS.

Adding `b':'` and `b'#'` to `is_symbol_continue` makes `wat-edn` accept **`a::b`**, **`wat::core::x`**
and **`x:`** — **all three of which Clojure REFUSES**. Under the ward's doctrine that is three new
`clj:ERR / wat:OK` rows, i.e. three new bugs. The real rule, derived from the oracle (Clojure 1.12.4,
`*read-eval*` false), not from the spec sentence:

| input | Clojure | | input | Clojure |
|---|---|---|---|---|
| `a:b` | ✅ | | `a#b` | ✅ |
| `a:b:c` | ✅ | | `a##b` | ✅ |
| `a.b:c` · `a-b:c` | ✅ | | `x#` · `x##` | ✅ |
| `a#:b` · `a:#b` | ✅ | | `#x` | ❌ (dispatch — leading, already correct) |
| `:a:b` · `:a#b` | ✅ | | | |
| **`a::b`** · **`a:::b`** · **`:a::b`** | ❌ `Invalid token` | | | |
| **`x:`** · **`x::`** · **`:x:`** | ❌ `Invalid token` | | | |

⭐ **THE RULE:**
- **`#`** — unrestricted as a body character (interior, doubled, trailing all legal).
- **`:`** — legal as a body character **only when it is neither DOUBLED nor FINAL**.

⚠ `is_symbol_continue` is a **per-byte** predicate and cannot see "doubled" or "final". **The rule
does not fit there.** Where it goes is yours to decide — a post-scan validation on the assembled
body, a small state machine in the lexer, or a check in `validate_*`. **Report the shape you chose
and why.** Do not contort the per-byte predicate into something that silently accepts `a::b`.

## ⭐ WHY THIS IS SAFE — measured

- **`wat::core::x` STAYS REFUSED, by both.** Clojure refuses it (`Invalid token`) and so will we.
  ⇒ **The 85 `::`-bearing symbol tokens across 27 `.wat` files do not move.** They are 8d's, and 8d
  retires them.
- **Blast on identifiers is ZERO.** 218.7 measured it: **0 real `a:b` / `a#b` identifiers** exist in
  the tracked corpus. Nothing in wat source changes.
- ⚠ `wat-edn` does **not** read `.wat` source — `wat-reader` does. This stone cannot affect the
  corpus. Verify that claim rather than trusting it: the floor is the proof.

## The work

1. **Implement the rule** at whatever layer can actually express it (see the warning above).
2. **Extend the generated corpus.** `generate_corpus.py` must emit the `:`/`#` surface — interior,
   doubled, trailing, leading, in symbols AND keywords, combined with `.`/`-`. The table above is the
   minimum; **generate it, do not paste it.**
3. **Regenerate `golden.txt` with real `clj`** (`/usr/local/bin/clj`). Never hand-write a verdict.
4. **Remove the now-stale exemption.** `a:b` / `a#b` become **parity**, so their `exemption()` arm
   must go, and `every_exemption_names_a_reason` — which currently asserts
   `exemption("a:b").is_some()` — must be updated to assert `.is_none()`.
   ⛔ **A stale exemption is invisible to the ward** (218.7's reported weakness: an exempted row is
   skipped once verdicts differ, so an exemption whose divergence has vanished never fires). **This
   is that exact case. Leaving it would be the first instance of the weakness biting.**
5. **Loop to dry.** Last generation round finds 0 new divergences; state the rounds.
6. **Update the record.** `vocab.rs`'s doc comment says *"The 219 ruling STANDS until the builder
   reopens it"* — he has. Update it, and note the reopening in arc 219's `DESIGN.md`.

## The gate

- `scripts/floor.sh` green; clippy `-D warnings --all-targets --workspace` **0** — the orchestrator's
  row, uncontended. ⚠ **218.7 landed with a clippy error the executor did not see** because the brief
  assigned clippy to the orchestrator. **Run `cargo clippy --release --all-targets -p wat-edn -- -D
  warnings` yourself before scoring**; it is cheap and it caught the last one.
- Predict the test-count delta from the diff, confirm with `cargo nextest list`.
- `census.sh --diff` → `no STOP-8`.
- Corpus size before → after; round-to-dry table.
- ⛔ **Prove the ward still CATCHES**, the valid way: flip a **PARITY** row's golden verdict (e.g.
  `OK\t42` → `ERR\t42`), confirm RED, restore. ⚠ Flipping an *exempted* row proves nothing — the
  orchestrator did exactly that and nearly filed a false finding; the loop `continue`s at
  `if wat == clj` before ever reaching `exemption()`.

## Out of scope — affirmatively cut

- **Duplicate map keys in `.wat` SOURCE.** The builder ruled it (*"the duplicate key in a map literal
  makes sense - the user generated code that should be considered an error"*) and it is **real**:
  `(wat.core/length {:a 1 :a 2})` **type-checks clean today** while `wat-edn` and Clojure both refuse
  it. But that is **`wat-reader`/the checker**, a different crate and an unmeasured blast radius.
  Filed as `FINDING-duplicate-keys-are-legal-in-wat-source.md`. **Not this stone.**
- **8d**, the `::` retirement, and anything touching `.wat` files.
- **Performance.** Do not benchmark here.
