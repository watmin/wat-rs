# AMEND — STONE 251.8d-i: three stale goldens + two stale doc comments. Folds INTO the stone.

**Orchestrator's verification row, `22118e145` + score `1d056b05b` (neither pushed).**

## ⛔ FLOOR RED — 3 of 5930. Everything else PASSED, including the gate.

```
     Summary [ 277.484s] 5930 tests run: 5927 passed (8 slow), 3 failed, 22 skipped
  FAIL wat::types struct_restricted::struct_restricted_ctor_restriction_fires_on_illegal_caller
  FAIL wat::types struct_restricted::struct_restricted_per_field_restriction_fires_on_illegal_caller
  FAIL wat::types struct_restricted::struct_restricted_empty_sections_honored
```

⭐ **All three are the SAME class, and YOUR CHANGE IS RIGHT.** The only delta is the diagnostic
**prose**. Everything structural is byte-identical — `:callee ":my::Token"`,
`:enclosing-fn ":user::bad-mint"`, `:prefixes [":my::issuer::"]`, same `:location` span:

```
expected: "An entry ending in `::` is a namespace prefix (caller FQDN must start with it);
           an entry without trailing `::` is an exact-FQDN match."
actual:   "An entry without `/` is a namespace prefix (caller FQDN must start with it);
           an entry containing `/` is an exact-FQDN match."
```

⛔ **DO NOT REVERT THE MESSAGE.** `src/check/error.rs:721` is now the truth and the old sentence is a
**lie** after this stone: entries are symbols discriminated by `/`, and `ends_with("::")` is gone.
Reverting the prose to satisfy three goldens would make the diagnostic describe a matcher that no
longer exists — a cure that trades a red for a falsehood.

⚠ **Note the message still displays `[:my::issuer::]`** — the corpus is not converted, so the entry
is still the old keyword. That is **correct and is the dual-dialect canonicalization working**:
`:my::issuer::` contains no `/`, so it is treated as a prefix, and the restriction **fires**. The
behaviour under test is intact; only the sentence describing the rule moved.

## The fix — 3 goldens + 2 stale doc comments

| file | what |
|---|---|
| `tests/types/struct_restricted__struct_restricted_ctor_restriction_fires_on_illegal_caller.edn` | re-golden |
| `tests/types/struct_restricted__struct_restricted_per_field_restriction_fires_on_illegal_caller.edn` | re-golden |
| `tests/types/struct_restricted__struct_restricted_empty_sections_honored__case_c.edn` | re-golden |
| `tests/types/struct_restricted.rs:15` | ⚠ module doc still says *"Whitelist entry ending in `::` → caller FQDN must START WITH the prefix."* |
| `tests/kernel/wat_arc198_def_restricted.rs:16` | ⚠ same stale sentence — **this file's tests PASS**, so nothing would ever catch it |

⛔ **The two doc comments are the ones that matter more than the goldens.** A golden fails loudly the
moment it drifts. A comment describing a retired rule rots silently and is read by the next hand as
fact — `[[feedback_a_comment_can_ship_a_gap_as_a_law]]`. `wat_arc198_def_restricted.rs` is green;
only a human reading it will ever notice.

⚠ **Re-golden by regenerating, not by hand-editing the sentence.** If these goldens have a
regeneration path, use it and say so; a hand-typed expectation is how a golden stops being evidence.

## The fold

⛔ **Fold into `22118e145`.** Not a repair commit after — the stone must be green at its own landing.
Nothing is pushed; amending is safe.

## ⭐ WHY YOUR OWN WALLS DID NOT CATCH THIS — and it is not a criticism

You ran `--test kernel wat_arc198_def_restricted` (the **defn** path) and
`--lib restriction_entry_match_tests` (the **matcher**). Both green, correctly. The three reds are
`wat::types struct_restricted` — the **defstruct** path, which this stone also changed
(`types/defstruct.rs`, unifying the drop-vs-error split).

⇒ **The brief told you to unify two paths and then named the test surface of only one.** That is the
orchestrator's defect, not yours. For the record, the selector that would have caught it:
`cargo test --release -p wat --test types struct_restricted`.

## Everything else I verified — the gate is GREEN

| row | result |
|---|---|
| ⭐ **corpus census, MY independent run** | ✅ **2145 / 2145 OK — 0 A · 0 B · 0 C · 0 other** |
| idempotence | ✅ 60-file spread of already-converted copies, **second-pass-changed = 0** |
| **ZERO tracked `.wat` conversions** | ✅ only 4 **new** fixtures + `wat/fix.wat` (the engine) |
| prefix admits `ns.Type/method` | ✅ — your catch; my brief's rule would have red the stdlib |
| prefix denies a sibling namespace | ✅ `my.kernelish/sneaky` refused, `whitelist [my.kernel]` shown |
| exact entry admits its named caller | ✅ |
| silent drop → hard error | ✅ `MalformedForm: ":restricted-to entries must be keywords or symbols"` |
| no published history rewritten | ✅ `origin/main` ancestor, 0 `refs/original` |

## After the fold

State `cargo test --release -p wat --test types struct_restricted` and crate clippy. I re-run the
whole floor, workspace clippy and `census.sh --diff` uncontended. **Do not push. Do not start 8d-ii.**
