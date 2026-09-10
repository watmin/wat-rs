# DESIGN — the codemod ASKS, and the flip is a PAIR

Steps ② and ③. Every primitive below was measured on the live tree before this was written.

## ⛔ FIRST, A CORRECTION TO MY OWN CLAIM

I wrote that after the composition door, *"the flip is one line in `compose_variant`."* **Measured:
it is TWO.** Composition now goes through my door; decomposition goes through the grammar's
general-purpose accessors:

```rust
identifier::leaf(name)  →  name.rsplit("::").next()      crates/wat-reader/src/identifier.rs:284
identifier::path(name)  →  name[..name.rfind("::")]                                        :290
```

and `TypeEnv::variant_parent_enum` splits with exactly those — deliberately, its own doc says, because
*"the one-name-grammar lint bans a hand-rolled `rfind` outside `identifier.rs`."*

So if `compose_variant` writes `Option.Some` while `path`/`leaf` still read `::`, then
`variant_parent_enum(":wat::core::Option.Some")` splits to `(:wat::core, Option.Some)`, finds no enum,
and answers `None`. **The registry would stop recognising the names it just composed.**

⚠ And `path`/`leaf` cannot simply flip — they split EVERY namespaced name, not just variants.
The flip therefore needs a variant-specific DECOMPOSER paired with the variant-specific COMPOSER:
`compose_variant`'s own inverse, beside it, so the separator remains one decision in one file.

★ This is the composition door's thesis completing itself. The grammar had ten decomposers and no
composer; it now has a composer whose inverse is a general accessor that does not know it is about
variants. Naming the pair is what makes the separator movable.

## ⛔⛔ CORRECTED — IT IS NOT A PAIR. THE DECOMPOSITION SIDE IS ~45 SITES.

This design said *"the separator is one decision, and it is spelled in exactly these two function
bodies."* **Measured after ③a landed: FALSE.** `variant_parent_enum` was not the only decomposer.

```
src/record/construct.rs:261  if !head.contains("::") { return None; }      ← a THIRD spelling
src/record/construct.rs:264  let type_path    = identifier::path(head);
src/record/construct.rs:265  let variant_name = identifier::leaf(head);
                             then `types.get(type_path)` → `TypeDef::Enum`
```

That is the VARIANT CONSTRUCTOR path, decomposing exactly as `variant_parent_enum` did — and the
`contains("::")` guard above it spells the separator a third time in three lines.

Wider census, on the tree with ③a landed:

```
raw separator splits (contains/rfind/rsplit/split on "::") outside identifier.rs ...... 45
files that do that AND touch `TypeDef::Enum` ........................................... 7
```

★ **The composition door collapsed 15 sites into 1. The decomposition side has ~45 split sites and
has not been collapsed at all** — most are NAMESPACE splits, which are correct and must stay
general; an unknown subset are VARIANT splits, which must move to `decompose_variant`.

⚠ **So the flip is not two lines and it is not two bodies.** Before ③b can land, the ~45 need
splitting by KIND — variant separator vs namespace separator — and that is a census, not a guess.
`[[feedback_a_pattern_that_matches_a_subset_is_not_a_census]]`

★★ ③a is still correct and still the right first step: it pairs the composer and routes the
registry's own lookup through the pair. It is a STEP, not the set — and this correction exists
because I wrote "exactly these two" without counting.

## ② THE CODEMOD — it ASKS, it does not match

`wat-scripts/fixes/variant-colons-to-dot.wat`, shaped after
`wat-scripts/fixes/bare-variant-to-qualified.wat` (arc 296 N — the same family, and its
`rename-five` / `rewrite-each` / `main` skeleton is the one to copy).

⛔ That precedent hardcodes FIVE names. **We have 493 distinct spellings over 9,946 occurrences**, so
ours enumerates from the source and asks the substrate per candidate.

### The algorithm, with every primitive MEASURED

```
for each path on stdin:
  src   = (:wat::io::read-file path)
  forms = (:wat::core::read-string src)                → ReadOutcome::Forms
  walk the form tree collecting KEYWORD leaves:
      (:wat::core::ast-kind node)   → "keyword"
      (:wat::core::ast-name node)   → ":wat::core::Option::Some"   ⚠ WITH the leading colon
  for each distinct leaf name `full`:
      bare   = (:wat::string::subs full 1 (:wat::string::length full))
                                    → "wat::core::Option::Some"    ⚠ from-string REFUSES a colon
      ask    = (:wat::runtime::variant-parent-of (:wat::keyword::from-string bare))
      Some p → parent = (:wat::keyword::to-string p) → "wat::core::Option"  ⚠ NO leading colon
               leaf   = (subs bare (+ (length parent) 2) (length bare))     → "Some"
               new    = (:wat::string::concat ":" parent "." leaf)          → ":wat::core::Option.Some"
               src    = (:wat::fix::rename-keyword-exact full new src)
      None   → leave it ALONE
  (:wat::io::write-file path src)
```

Every arrow above was run. The three ⚠ lines are the boundaries that will bite a guess:
`ast-name` yields the colon, `from-string` rejects it, `to-string` omits it.

★ **The `None` branch is the entire reason this is a codemod and not a `sed`.**
`:wat::cache::Cache::GetRequest` is a `defrecord` in a `:messages` block, identical in shape to a
variant. It answers `None` and survives untouched. A text rewriter renames it and nothing fails —
it just means something else afterwards.

### Idempotence, and why it holds

After the rewrite the token is `:wat::core::Option.Some`. Re-running asks
`variant-parent-of` about it; `path`/`leaf` split on `::`, giving `(:wat::core, Option.Some)`, which
is not an enum → `None` → untouched. **The codemod is idempotent BEFORE the flip.**

⚠ **After the flip it is idempotent for the opposite reason** — the name is already correct and the
old token is gone. Both directions are safe, but they are safe for DIFFERENT reasons, and a rerun
between the two states is the case to think about, not to assume.

## ③ THE FLIP — and it lands WITH the codemod

```
1  compose_variant           writes `.`
2  the paired decomposer     reads `.`   (variant_parent_enum stops using generic path/leaf)
3  the corpus                the codemod, above
```

⛔ **These are ONE landing.** Either half alone breaks the stdlib — measured: flipping only the
composer took `wat/core.wat` down at load with `macro :wat::core::format — program body eval failed`.
`wat/fix.wat`'s **STASH-DANCE** (its header, lines 38–52) is the supported path:

```
1  git stash push -m "rust change" <the flip>     # old separator restored
2  cargo build --release                          # old substrate + the NEW codemod verb
3  printf '[...every path...]' | cargo wat ./wat-scripts/fixes/variant-colons-to-dot.wat
4  git stash pop
5  cargo build --release && cargo test
```

⛔ **Dry-run step 3 on a `/tmp` COPY and `diff` it first.** At 9,946 sites the diff is the only
thing that can show the rewrite is exactly the structural change intended.

⚠ Step 3 must list EVERY path — a missed file breaks the build, and `every_wat_scripts_file_loads`
plus the stdlib load are what will say so.

## What must NOT happen

⛔ **No regex over `Ns::Upper::Upper`.** The lookalike above is why. Ask, or do not rewrite.

⛔ **No second decomposer.** The pair lives beside each other in `identifier.rs` or the separator
stops being one decision — which is the entire defect the door stone just closed.

⛔ **Do not flip the general `path`/`leaf`.** They split every namespaced name in the substrate.

## Out of scope = REJECTED

- **Retiring the `::` variant spelling from the reader.** After the corpus is migrated and green;
  a separate, later cut.
- **`wat-fmt` cleanup of the rewritten files.** The edits are span-faithful token replacements;
  surrounding whitespace is the formatter's job, as `fix.wat`'s header says.
