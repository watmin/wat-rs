# SCORE — STONE 218.8: the 219 correction (`:` and `#` as symbol constituents)

Branch: `main`. **Committed, not pushed.** Stone `1bb428bf9`.
Drawn against `1b5ccb783`. Parent brief: `BRIEF-STONE-218.8-the-219-correction.md`.
Floor / workspace clippy / census: orchestrator's row (not run). 8d not started.

## The shape, and why

`is_symbol_continue` is per-byte and cannot see "doubled" or "final". A naive add of
`b':'` / `b'#'` would accept `a::b`, `wat::core::x`, and `x:` — three new
`clj:ERR / wat:OK` bugs.

**Chosen:** admit `:` and `#` as continue bytes, then **post-scan each namespaced
component** with `validate_colon_constituents` (`vocab.rs`), called from
`validate_name_body` → `parse_namespaced` and the public constructors.

- `#` unrestricted (interior, doubled, trailing).
- `:` legal only when neither doubled nor final **on a prefix or name**.

Whole-token scan is not enough: `a:/b` has no `::` and does not end in `:`, but
Clojure refuses it (`Invalid token: a:/b`) because the prefix `a:` is colon-final.
Per-component catches that. Lexer still rejects `::foo` at keyword start.

Constructor `::` → `.` translation (`translate_and_validate_ns`) is unchanged;
it runs *before* the colon scan, so `Keyword::ns("wat::core", "HashMap")` still
stores `wat.core`.

## Behaviour (oracle, this machine)

| input | clj | wat-edn |
|---|---|---|
| `a:b` `a#b` `a:b:c` `a##b` `x#` `:a:b` | OK | OK (parity) |
| `a::b` `x:` `:a::b` `:x:` `a:/b` `wat::core::x` | ERR Invalid token | ERR |
| `foo#_bar` | OK symbol `foo#_bar` | OK (was: `foo` + discard; both OK so the ward was blind) |

219 exemption **removed**. `every_exemption_names_a_reason` now asserts
`exemption("a:b").is_none()` and `exemption("a#b").is_none()`. A stale exemption
would have been invisible (`if wat == clj { continue }`).

## Corpus: 190 → 221. Loop to dry.

Generator enumerates interior / doubled / trailing / leading, symbols and
keywords, combined with `.`/`-`. Golden regen'd against `/usr/local/bin/clj`.

| round | action | new unexempted divergences |
|---|---|---|
| 1 | generate 190→221, regen golden, run ward | **0** |
| 2 | regenerate: **221 identical**, golden identical | **0** — DRY |

## Ward still CATCHES

Flipped parity row `OK\t42` → `ERR\t42`. Ward RED:

```
clj-oracle parity VIOLATED — wat-edn diverges from clojure.edn on 1 input(s)
  "42"	clj:ERR	wat:OK
```

Restored → green. (Not an exempted row.)

## Test count

Predicted delta: **+3** (`colon_constituents_match_the_oracle`;
`symbol_colon_and_hash_interior`; `symbol_doubled_or_final_colon_rejected`).
Rename `is_symbol_continue_rejects_colon` → `is_symbol_continue_admits_colon_and_hash` (0).

- `cargo nextest list --release -p wat-edn`: **352** (was 349)

## Walls I ran (not the floor)

- `cargo clippy --release --all-targets -p wat-edn --offline -- -D warnings`
  — first pass: `clippy::byte_char_slices` on `[b':', b':']` → `*b"::"`. Second pass: **exit 0**.
- `cargo test --release -p wat-edn --offline` — 352 unit+integration + 3 doctests, all pass.
- Ward injection — RED then restored.

Floor + workspace clippy + `census.sh --diff`: **not run** (brief: orchestrator,
uncontended; floor is the proof this crate does not move `.wat` source). Do not push.
8d does not start.
