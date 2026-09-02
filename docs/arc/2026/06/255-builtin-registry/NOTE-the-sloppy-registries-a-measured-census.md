# NOTE — the sloppy registries: a measured census of every hand-rolled answer to "what is this name?"

> **Builder, 2026-09-01:** *"we seek to annihilate .... these... 'duplicates' ..... destroy......
> the registry must become the sole authority for these things.......... **there's many..... sloppy...
> attempts at doing this**......."*
>
> He is right, and the record was not. `[[RULING-the-registry-is-the-sole-authority]]`'s census had
> **ten** entries; a `solvere` cast then found an eleventh (`special_forms.rs`). This NOTE is the
> systematic sweep that should have produced that census in the first place, and it finds **nine
> more** — including a family of **five mutually-inconsistent hand-lists** that no document in this
> arc had ever named.

## The instrument, stated so the number can be re-derived

Two structural sweeps over `src/`, comments stripped, **not** a grep from memory
(`[[feedback_a_pattern_that_matches_a_subset_is_not_a_census]]`):

1. **Every `":wat::…"` string literal, by syntactic shape** — match-arm position (`"…" =>` / `"…" |`),
   bare array element, `==`/`!=` comparison, `starts_with`.
2. **Every `fn` taking a name-ish `&str`** (`name`/`head`/`verb`/`fqdn`/`k`/`key`/`op`), then read by
   hand for "does this answer one of the RULING's seven questions?"

```
5,275   ":wat::…" string literals in src/, across 166 files
  501   in MATCH-ARM position        729  as bare array elements
  133   in an == / != comparison      25  reached via starts_with
  322   fns taking a name — read by hand, ~20 answer a registry question
```

⚠ `src/intrinsic/mod.rs`'s 413 array elements are the **five absence ledgers**, not duplicates.
They are the instruments; they die at Phase 4b. `[[RULING-the-registry-is-the-sole-authority]]`'s
"a campaign that cannot tell a duplicate from a derivation deletes correct code" applies here.

---

## ⛔⛔ FAMILY 1 — "what kind of form is this head?" — FIVE hand-lists, and they DISAGREE

**None of these appears in any arc-255 document before this NOTE.**

| # | where | population |
|---|---|---:|
| 1 | `src/runtime.rs` · `is_mutation_head` | 9 names + `starts_with(":wat::config::set-")` |
| 2 | `src/freeze.rs` · `is_mutation_form` | 11 names + the same prefix |
| 3 | `src/freeze.rs` · `is_declaration_form` | 9 names |
| 4 | `src/declare/parse.rs` · `DECLARATION_HEADS` | 6 names |
| 5 | `src/declare/parse.rs` · `RUNTIME_DECLARATION_HEADS` | 8 names |

### ★★★ The drift, measured — this is not theoretical

`is_mutation_head` and `is_mutation_form` are **the same question in two files**, and their answers
differ by two names:

```
only in freeze::is_mutation_form :   :wat::core::def      :wat::core::defsurface
```

**`:wat::core::def` — the single most-used definitional form in the corpus (3,431 call sites) — is a
mutation to `freeze.rs` and is not one to `runtime.rs`.** Two files, one concept, opposite answers,
and nothing in the tree compares them. That is the RULING's *"eliminate every source of duplication
or inconsistency"* with a live instance attached.

And `DECLARATION_HEADS` vs `RUNTIME_DECLARATION_HEADS` differ by `:wat::core::do` and
`:wat::core::let` — a third and fourth spelling of an overlapping concept, in one file.

### The annihilation vehicle already exists, and the builder already ruled on it

`@Category` **has a `Declaration` variant**, and `Category` is **generated from wat**
(`crates/wat-doc/src/lib.rs:88` — `wat_enum_from!(pub enum Category, "../../wat/runtime-meta.wat",
":wat::runtime::Category")`), on the builder's own ruling *"wat is source of truth ... that's my
pick."* Its own comment states the principle this NOTE is applying:

> *"a generated enum cannot drift from its generator."*

So the annihilation is a query, not a mechanism:

```
five hand-lists  →  registry().lookup_entry(head).is_some_and(|e| /* on e.category */)
```

⚠ **`mutation` and `declaration` are NOT the same predicate** — mutation includes the loaders and
the config setters, declaration does not. One authority does not mean one predicate; it means both
predicates read the same declared axis instead of five hand-lists.

★★ **And Phase 1a is exactly the unblock.** Nine of the eleven distinct names across these five
lists are in the 23 unregistered `special_forms.rs` rows. The RULING's forced order applies without
modification: **registry can answer (1a) → consumer asks → the five lists die.**

---

## FAMILY 2 — "what is a builtin TYPE?" — two lists, and one uses a different SPELLING

| where | population | spelling |
|---|---:|---|
| `src/runtime.rs` · `is_builtin_primitive` | 37 | ⛔ **UNPREFIXED** — `"wat::core::bool"`, no leading colon |
| `src/check.rs` · `is_primitive_type_keyword_in_value_position` | 7 | prefixed — `":wat::core::bool"` |

The second is a 7-name subset of the first, written in the other spelling. Both are fragments of the
**frozen `TypeEnv`**, which is the derivation the RULING already protects — so this family's cure is
not "fold into the registry" but "ask the `TypeEnv` you are already fragmenting."

⚠ The unprefixed spelling is its own hazard: it is invisible to every `":wat::` census in this arc,
including both of mine. It was found only by reading the function.

---

## FAMILY 3 — a property, guessed from a PREFIX

| where | shape |
|---|---|
| `src/resolve/reserved.rs` · `is_reserved_prefix` | **membership** by prefix — ★ THE ARC'S FOUNDING TARGET |
| `src/rete/purity.rs` · `effectful_by_prefix` | **effectfulness** by 8 prefixes: `kernel` `io` `holon` `eval-` `load` `config` `stream` `rete` |

`:wat::holon::` is declared effectful **wholesale** — every VSA verb, including the pure ones the
registry has individually ruled on. A prefix cannot be right about 464 rows.

⚠ **The SEAM cites `is_reserved_prefix` at `src/resolve/walk.rs`. It lives at
`src/resolve/reserved.rs:42`**; `walk.rs` is its caller. Corrected in the same commit as this NOTE.

---

## ★ FAMILY 4 — the residue BEHIND a registry consult, and 16 rows of it are already dead

`src/macros/eval.rs` · `is_expand_time_legal` is **the migration pattern already working in this
tree** — it asks the registry FIRST and only then falls back to a 54-name hand-list:

```rust
if let Some(e) = registry().lookup_entry(head) { return matches!(e.expand_time, …); }
matches!(head, ":wat::core::=" | ":wat::core::not=" | … )   // 54 names
```

**16 of those 54 names are registered TODAY**, so the fallback arm is unreachable for them:

```
= not= and or not do fn match bool::to-string str show … (16 of 54)
```

★ Dead code that no lint can see, because reachability runs through a runtime lookup. It is
discoverable only by asking the registry — which is the argument for the registry, made by the
residue of a partial migration to it.

---

## The amended census — what the RULING's table should now read

```
ALREADY NAMED (RULING + the solvere amendment)
  RETE_OPS 74 · SPECIAL_FORMS 35 · register_builtins 350 · literal arms 118
  RETIREMENT_TABLE 144 · intrinsic_meta 37 · is_expand_time_legal 54
  effectful_by_prefix 8 · is_reserved_prefix

NEW IN THIS NOTE — nine more, none previously recorded
  is_mutation_head · is_mutation_form · is_declaration_form
  DECLARATION_HEADS · RUNTIME_DECLARATION_HEADS          ← five, mutually inconsistent
  is_builtin_primitive · is_primitive_type_keyword_in_value_position
  classify_constraint_head · is_where_form
```

⚠ **NOT duplicates, and the distinction still matters:** `constructor_meta`/`accessor_meta` derive
from the frozen `TypeEnv`; `step_list`'s 19 names declare a capability; the five absence ledgers are
the campaign's own instruments; `is_namespaced` (`name.contains("::")`) is a lexical fact about a
string, not a claim about the namespace.

## ⛔ What this NOTE does NOT claim

It does not draw a stone, and it does not say these die in Phase 1a. **It says the campaign's target
list was incomplete by nine, that one family of five is inconsistent on disk today, and that
`@Category` — already generated from wat, already the builder's ruling — is the vehicle for the
largest of them.**

★ The method is the transferable part: **two structural sweeps found in one pass what three separate
readings of the same tree had missed.** `[[feedback_impose_the_check_and_read_the_screams]]` — my
censuses have been wrong every time they were written from what I had been reading, and right every
time an instrument was imposed over the whole population.
