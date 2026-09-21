# AMEND — STONE 251.8d-i-b (3): 2 reds, both on the new gate's own self-tests.

**Orchestrator's verification row, `e57e04f52` (not pushed).**

## ⭐ 10 → 2, and everything substantive is done

```
     Summary [ 288.322s] 5946 tests run: 5944 passed (8 slow), 2 failed, 22 skipped
```

| row | result |
|---|---|
| the 8 `sift_rules` reds | ✅ **closed** — the `query.wat` generator now emits `:-` |
| the 2 byte goldens | ✅ ⭐ **you read the diff first** — one line each, `(?c <- :celsius)` → `(?c :- :celsius)`, with the `defrecord` param **unchanged on both sides**. **That is the discipline the amend asked for**: you proved it was the converted arrow before re-goldening, so the golden records a verified change and not an unexamined one |
| ⭐ **the generator gate** | ✅ `tests/lint/rete_bind_generators.rs` — walks the **constructor**: `` `(~NAME <- `` is RED when `NAME` is bound to `symbol-node "?…"`. **Two non-vacuity floors AND three detector self-tests** (the real shape hits; a `fn [~d-sym <-` param does not; `` `(~fact-sym :- `` does not). ⭐ **A wall with a positive and two negatives is a wall.** |

## ⛔ THE 2 REDS — the detector's own INPUT trips two text walls

```
tests/lint/rete_bind_generators.rs:64   no_inlined_edn
tests/lint/rete_bind_generators.rs:192  no_loose_string_assert
tests/lint/rete_bind_generators.rs:213  no_loose_string_assert
```

**They are not goldens. They are SOURCE-TEXT FIXTURES for a text detector:**

```rust
:64   let t = t.strip_suffix("(:wat::core::")?      // a parser literal, opens with `(`
:192  let src = "  fact-sym (:wat::core::symbol-node \"?fact\")\n  cond `(~fact-sym <- ~tkw)]\n";
:213  assert!(qvars.contains("fact-sym"), …)
```

⛔ **`no_inlined_edn`'s rubric — "move the EDN into a co-located `.edn` file" — is WRONG HERE, and
following it would break the test.** These strings are **wat source FRAGMENTS**, deliberately
malformed (`cond \`(~fact-sym <- ~tkw)]` has an unmatched `]`). They are the detector's *input*. An
`.edn` file would have to *parse*, and this does not and must not.

⭐ **A rune is the right answer here — and it is the FIRST time this arc has said that.** Every
previous wall in this campaign was routed through the door rather than runed (255.2's
`is_binder_marker`, 255.4's `one_param_spec`). **The difference: those sites were doing the thing
the wall guards. These are not.**

⛔ **But rune PER SITE with the reason, never the file.** Each rune states *why this string is a
detector fixture and not inlined data*. A blanket suppression is how a wall dies.

⚠ **`:213` may not need a rune at all.** `assert!(qvars.contains(…))` is loose *because it is*: it
proves one member and would pass if the set held ten wrong ones. ⭐ **`assert_eq!` on the whole set
is stronger and needs no rune** — `:194` already does exactly that (`assert_eq!(uses, vec![(2,
"fact-sym")])`). **Prefer tightening over exempting; rune only what cannot be tightened.**

## The fold

⛔ **Fold into `e57e04f52`.** Not a repair commit. Nothing is pushed.

## Gate

- `scripts/floor.sh` green; clippy `-D warnings --all-targets --workspace` 0; census `no STOP-8`.
- ⛔ **Every rune names its site's reason.** `no_inlined_edn` / `no_loose_string_assert` both accept
  a per-site rune — **use the form the wall itself documents.**
- ⚠ **The generator gate must still be non-vacuous after the change** — re-run
  `every_walking_gate_declares_non_vacuity` and `rete_bind_generators`. A rune that silences the
  detector's own self-test would hollow the wall this stone exists to build.
