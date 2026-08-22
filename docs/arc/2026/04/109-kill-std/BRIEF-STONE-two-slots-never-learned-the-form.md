# BRIEF — two slots never learned the `:- [T …]` form

The last two things blocking ②-iii. Both are the shape γ-i already fixed for `fn`: **a slot that
accepts a type KEYWORD and was never taught the reference FORM.** Two error sites, three guards.

## ⛔ Read this first — the blocker list you may have seen is dead

`109/NOTE-2iii-is-blocked-*.md` names five blockers. **All five are closed.** I re-ran the codemod
today and measured the real list with the NOTE's own held-back-set method. Do not work from that
NOTE; it is a measurement dated 2026-08-21 and substrate has shipped under every entry.

**What the re-run actually found, and it is the whole list:**

```
defclause's RETURN slot is keyword-only     src/runtime.rs:7849-7862      ~10 files
vec / HashSet first ARG is keyword-only     src/collection/eval.rs:1869, 1960   33 sites, 2 files
```

With those two files-groups held back, **24 of 36 migrated files load and the floor is
4866/4866, 0 FAIL.** The migration itself is sound.

## The rooms

1. `src/runtime.rs:7849` — `defclause`'s return slot:
   ```rust
   match &rest[type_pos] {
       WatAST::Keyword(k, _) => parse_type_keyword(k)?,
       other => return Err(… "must be followed by a return type keyword; got {}" …),
   }
   ```
2. `src/collection/eval.rs:1869` (`:wat::core::Vector`) and `:1960` (`:wat::core::HashSet`) — same
   shape, a `matches!(&args[0], WatAST::Keyword(_, _))` gate with no `List` arm.
3. **`src/function/parse.rs:178` — THE EXEMPLAR.** γ-i's move: `parse_type_node(&sig[2])`.
4. `src/types/surface.rs:345` — its own comment names the door:
   *"`parse_type_node` is the substrate's one door that reads all four type node shapes."*

## The work

Teach all three guards to accept a `List` as well as a `Keyword`, routing through
`parse_type_node` — **the existing door, not a new one, and not a second hand-rolled reader.**
The keyword path must stay byte-identical; this is widening only.

⚠ `parse_type_keyword` and `parse_type_node` may not return the same type — check before assuming
the call is a drop-in, and if they differ, say what you did about it.

## STOP triggers — ship nothing further and report

- **STOP-1 — if `parse_type_node` is not reachable from `collection/eval.rs`** (a module boundary,
  a `pub(crate)` limit), STOP and report. Do not hand-roll a second List-reading path; that is the
  "second resolution path" this arc has rejected twice.
- **STOP-2 — if widening any of the three changes what the KEYWORD path accepts or rejects**, STOP.
  This is additive. A keyword that checked before must check identically after.
- **STOP-3 — if the codemod re-run (row 4) surfaces a THIRD class**, STOP and report the full list
  before touching it. That list is worth more than this stone.

## Acceptance

| # | what | expected |
|---|---|---|
| 1★ | `defclause` takes a form return | a probe with `-> (:wat::core::Vector :- [:wat::core::i64])` checks clean |
| 2★ | `vec` / `HashSet` take a form first-arg | ditto for both verbs |
| 3★★ | the keyword path is untouched | the same probes in KEYWORD spelling still check clean |
| 4★★ | **the corpus migrates whole** | re-run the codemod over all 52 stdlib files → stdlib LOADS, no held-back set |
| 5 | idempotent | second codemod pass = 0 changes |
| 6 | clippy | 0 under `-D warnings` |

**Row 3 is the row that decides it.** Rows 1, 2 and 4 all go green if you replace the guard with
something that accepts anything. Only row 3 proves the widening did not become a hole — a keyword
that was rejected before must still be rejected.

**Row 4 is the payoff and the real test.** Run the codemod exactly as recorded:

```bash
PATHS=$(find wat -name '*.wat' | sort | sed 's/.*/"&"/' | tr '\n' ' ')
printf '[%s]\n' "$PATHS" | target/release/wat ./wat-scripts/fixes/parametrics-take-a-type-vector.wat
```

⚠ Use `target/release/wat`, **not** `cargo wat` — the installed binary at `~/.cargo/bin` is stale and
lacks `keyword/to-type-form-colon`. It will fail with `UnknownFunction` and that is not a finding.

⚠ **Dry-run on a `/tmp` copy and `diff` it first** (R21). Expect **36 files, ~865 lines, 840 `:-`
emissions**, byte-identically idempotent. Then apply.

⛔ **Leave the corpus migration IN the working tree — do not revert it.** It is the deliverable
alongside the substrate fix; I floor them together.

## Boundaries

- `src/runtime.rs` (the one match), `src/collection/eval.rs` (the two guards), a probe under
  `wat-scripts/scratch-pad/`, and the corpus migration the codemod produces.
- Do NOT hand-edit any `.wat` in `wat/`. The codemod does that — R21. If a file needs a change the
  codemod does not make, that is a finding, not an edit.
- Do NOT run `scripts/floor.sh` or a full `cargo nextest` — I measure centrally. Your own checks are
  `target/release/wat --check` on your probes and on `wat/core.wat` after the migration.
- Do NOT commit, push, stash, revert or amend.

⚠ `no_loose_string_assert` has a false-positive class on `assert!(registry.contains("literal"))`. If
it fires, do NOT add a rune — ask through the door, whose argument is an enum.

Prefix long commands with `systemd-run --user --scope -q -p MemoryMax=16G -p MemorySwapMax=0 timeout 1800`.
Read exit codes DIRECTLY — never through a pipe, never after a trailing `; echo`.

## Your report

Rows 1-3 together with verbatim output — row 3 especially, since rows 1/2/4 pass for a guard that
accepts anything. The dry-run diff stats before you applied. Whether `parse_type_keyword` and
`parse_type_node` agreed, and what you did if not. The full stdlib-load result after migration. What
surprised you. Anything you inspected and left alone.
