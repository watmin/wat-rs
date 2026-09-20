# AMEND — STONE 218.8: the `one-variant-separator` gate, folds INTO the stone

**Orchestrator's verification row, `1bb428bf9` + score `499bf3b1a` (neither pushed).**

## ⛔ FLOOR RED — 1 of 5924. Everything else passed.

```
     Summary [ 271.913s] 5924 tests run: 5923 passed (8 slow), 1 failed, 22 skipped
        FAIL [   0.099s] ( 215/5924) wat::lint one_variant_separator::only_identifier_rs_spells_the_variant_separator
```

**THE ARM, verbatim from `.floor/2026-09-20T06-12-08Z/ARM.txt`:**

```
🔥🔥🔥 A SECOND VARIANT SEPARATOR — 1 site(s) spell the `::` between an enum and
its variant OUTSIDE `crates/wat-reader/src/identifier.rs`.

If this site does NOT separate an enum from its variant, add a co-located
`// rune:lint(one-variant-separator, <category>) — <reason>` on the line or the one
above, with <category> one of: namespace | type-path | display | edn | not-a-name.
⛔ `variant` is NOT a category — a variant site routes through the door.

Offenders:

crates/wat-edn/src/vocab.rs:225  [DATA]  if bytes.windows(2).any(|w| w == *b"::") {
```

## ⭐ THE CAUSE IS YOUR OWN CLIPPY FIX — and that is not a criticism

Your SCORE records it: *"first pass: `clippy::byte_char_slices` on `[b':', b':']` → `*b"::"`."*

`[b':', b':']` does **not** match the gate's pattern. `*b"::"` **does**. So clippy's suggested
rewrite created a literal `::` spelling in a file the gate watches. ⛔ **Two gates disagreeing about
the same line is not something crate-level clippy could have told you** — you ran it, it passed, and
this is the orchestrator's row firing exactly where it should.

## The fix — one comment line, and THE PRECEDENT IS 11 LINES ABOVE IT

`crates/wat-edn/src/vocab.rs:214` already carries the identical rune, same file, same category:

```rust
// rune:lint(one-variant-separator, edn) — the wall's own literal example: translates the
// wat `::` namespace separator into strict-EDN `.` form.
let translated = ns.replace("::", ".");
```

`validate_colon_constituents` is **not** composing or decomposing an enum/variant name — it is the
EDN lexer's own constituent rule, rejecting a doubled colon inside a symbol body. **`edn` is the
right category**, matching the site above.

⚠ **Choose the category yourself and say why.** `not-a-name` is also arguable (the function
validates a body, it does not name anything). The orchestrator's reading is `edn` on the precedent,
but you wrote the function. ⛔ **Do NOT use `variant`** — the gate refuses it explicitly.

⚠ **Do NOT revert the clippy fix to dodge the gate.** `[b':', b':']` would satisfy `one-variant-
separator` by accident while re-introducing `clippy::byte_char_slices`. Trading one gate's red for
another's is the shape this campaign calls a cure that conflates two questions.

## The fold

⛔ **Fold into `1bb428bf9`.** Not a repair commit after — the stone must be green at its own landing.
Nothing is pushed; amending is safe.

## Everything else I verified, so you know what is already banked

| row | result |
|---|---|
| golden is the ORACLE'S | ✅ I regenerated with `/usr/local/bin/clj` — **byte-identical**, 221 rows |
| corpus generator idempotent | ✅ byte-identical, 221 rows |
| the rule, behaviourally | ✅ **29/29** — incl. `a:/b` refused, `wat::core::x` still refused, and all of 218.7 intact |
| `a/b:c` · `:a/b:c` · `foo#_bar` | ✅ parity with clj, checked directly against the REPL |
| `foo#_bar` behaviour change | ✅ clj reads it as one symbol `foo#_bar`; discard still works (`[1 #_2 3]` → `[1 3]`) |
| 219 exemption removed | ✅ `exemption("a:b").is_none()`, `exemption("a#b").is_none()` |
| **ward still CATCHES** | ✅ flipped **parity** row `OK\t42`→`ERR\t42` → RED; restored → green |
| no published rewrite | ✅ `origin/main` ancestor, 0 `refs/original` |
| clippy workspace-wide | ⏳ blocked behind the floor; re-run after the fold |
| `census.sh --diff` | ⏳ after the fold |

⭐ **`a:/b` was YOUR catch, not the brief's.** The brief's rule said "neither doubled nor final";
you found that a *whole-token* scan misses `a:/b`, where the **prefix** `a:` is colon-final, and
went per-component. That is a real improvement on the instruction you were given.

## After the fold

State `cargo test --release -p wat-edn` and crate clippy. The orchestrator re-runs the whole floor,
workspace clippy and the census uncontended. **Do not push.**
