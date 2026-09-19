# RULING — the first slash separates namespace from name

**Builder's ruling, 2026-09-19. ✅ STRUCK AND LANDED the same day, after 8c closed.**
Floor **5920/5920** (predicted +1, confirmed), clippy 0, census `no STOP-8`.
**The score is at the bottom of this document** — including the two things that measurement changed
about the ruling's own framing.

> *"everything to the left of the first `/` is the namespace, everything to the right is the name…
> pathological names are tolerated gracefully… the rule is simple: **the first slash separates
> namespace from name**."*
>
> ```
> wat.core//                →  [wat.core, /]
> user/whatever/name/here/  →  [user, whatever/name/here/]
> ```

---

## What the code does TODAY — the opposite

`crates/wat-reader/src/identifier.rs:150`, in `Identifier::bare`, with its own comment conceding the
rule is a choice rather than a law:

```rust
// The tuple is derived ONCE, here, from today's split (last `/`).
let (ns, name) = match flat.rfind('/') {
    Some(slash) => (flat[..slash].to_string(), flat[slash + 1..].to_string()),
    None => (BOUND_NAMESPACE.to_string(), flat.clone()),
};
```

**`rfind` — the LAST slash.** On the builder's own examples the two rules part company badly:

| spelling | today (`rfind`, last) | ruled (`find`, first) |
|---|---|---|
| `wat.core//` | `["wat.core/", ""]` — **empty name** | `["wat.core", "/"]` |
| `user/whatever/name/here/` | `["user/whatever/name/here", ""]` — **empty name, path swallowed into the namespace** | `["user", "whatever/name/here/"]` |
| `wat.core/+` | `["wat.core", "+"]` | `["wat.core", "+"]` — **agree** |
| `wat.map/get` | `["wat.map", "get"]` | `["wat.map", "get"]` — **agree** |

Today's rule answers a pathological input with an **empty name** and a namespace that has eaten the
path. That is not a tolerated name; it is a silently malformed one.

## ⭐ WHY THIS IS CHEAP — measured, not assumed

The two rules can only disagree on a spelling with **more than one slash**. Measured across the tracked
`.wat` corpus, comments stripped, code positions only:

| | count |
|---|---|
| slashed identifiers in code | **6,292** |
| of those, with **2+ slashes** | **15** |
| of those 15, actual identifiers | **0** |

All 15 are file paths inside string literals (`wat-scripts/fmt/run.wat`,
`wat-scripts/scratch-pad/277-file-width-census.wat`, …) plus one prose fragment
(`defrecord/defstruct/…`). **Not one real identifier in this tree has more than one slash.**

⛔ **So this ruling migrates NOTHING.** It changes no existing identifier's namespace or name. What it
changes is the *answer to a question nobody had defined* — and it replaces an accidental answer (empty
name) with a stated one (the name is whatever follows the first slash, pathological or not).

⚠ **Instrument note, because it nearly went in the record wrong.** A first pass counted **3,984**
multi-slash "identifiers" and would have made this look like a large migration. That count was
`target/release/wat`, `src/rete/vocabulary.rs` and friends — **file paths in comments**. Stripping
comments and restricting to code positions took it to 15, and reading those 15 took it to 0. This is
the campaign's own recurring defect (`[[feedback_ask_the_tool_that_owns_the_fact]]`): a pattern over
source text answers "what matches", never "what is".

## The sites that implement the split — four, all in one file

| site | what it splits | note |
|---|---|---|
| `identifier.rs:150` `Identifier::bare` | the stored `(ns, name)` tuple | **the ruling's primary target** |
| `identifier.rs:300` `receiver()` | *"everything before the `/` of a surface-method call head"* (`:S/mk` → `:S`) | a **different concept** — method dispatch — that happens to use the same rule |
| `identifier.rs:310` `method()` | everything after | ditto |
| `identifier.rs:542` (test) | mirrors `bare`'s rule in its expectation | must move with it |

⚠ **`receiver`/`method` are method dispatch, not namespacing.** For every single-slash spelling — which
is all of them — they compute exactly what namespace/name compute, so the concepts coincide today and
nothing distinguishes them. **Whether the ruling governs them too is a question to settle when it is
struck**, not to assume here. Making them first-slash keeps one rule in the file; leaving them
last-slash means two rules that agree on all real input and diverge only on pathology, which is the
shape this ruling exists to kill.

## Why the rule is better, on its own terms

- **It is total.** Every spelling has a first slash or none; both cases have an answer, and neither
  answer is empty.
- **It is one sentence.** The current behaviour cannot be stated without saying "last", which invites
  "last of what?" the moment a name contains a slash.
- **It tolerates pathology gracefully**, which is the builder's stated intent: a weird name stays a
  weird *name*, instead of corrupting the *namespace*.
- **It matches the surface's own grammar.** `ns/name` reads left to right; the namespace is the prefix.

## When — AFTER 251.8c, and it does not disturb it

**8c is in flight with the pulsare counterpart.** Measured: this ruling does **not** collide with it.

- 8c's targets are `src/check.rs`'s `infer_list` and the call-head path; this ruling's target is
  `crates/wat-reader/src/identifier.rs`. Different files.
- `is_reference()` is `namespace() != BOUND_NAMESPACE`. A spelling with **no** slash still takes
  `BOUND_NAMESPACE` under either rule, and every slashed identifier in the tree has exactly one slash,
  so `is_reference()` returns the same answer for **every** input that exists. 8c's behaviour is
  untouched.

⛔ **Do not strike this while 8c is open** — not because it conflicts, but because two hands in the
identifier/resolve neighbourhood at once is how the 7h desync happened. Queue it as its own small stone.

## Acceptance, when it is struck

1. `Identifier::bare` splits on the **first** slash; the test at `:542` mirrors it.
2. The builder's two examples are pinned as cases: `wat.core//` → `["wat.core", "/"]`,
   `user/whatever/name/here/` → `["user", "whatever/name/here/"]`.
3. A **non-vacuity control**: a single-slash case that must be unchanged (`wat.core/+`), proving the
   test would notice a regression rather than passing because nothing moved.
4. The `receiver`/`method` question answered explicitly in the score — governed by the ruling, or
   deliberately not, with the reason.
5. Floor green, clippy 0, census `no STOP-8` — the orchestrator's row.


---

# ⭐ SCORE — struck 2026-09-19

Floor **5920/5920** (predicted +1 from the diff, confirmed by `cargo nextest list`), clippy
`-D warnings --all-targets --workspace` **0**, census **`no STOP-8`**.

## ⛔ TWO THINGS MEASUREMENT CHANGED ABOUT THIS DOCUMENT'S OWN FRAMING

Recorded because the brief above was written from reading, and both corrections came from running.

### 1. This was NOT "a question nobody had defined" — the tree already answered it, twice, the ruled way

The section *"What the code does TODAY — the opposite"* implied one rule existed and it was
last-slash. **Three different answers were live for `a/b/c`:**

| layer | `a/b/c` | ruled rule? |
|---|---|---|
| `crates/wat-edn/src/parser.rs:409` — EDN **text** | **illegal**: `more than one / in a/b/c` | n/a — the EDN spec, correct as-is and left alone |
| `crates/wat-edn/src/vocab.rs:179` `split_namespaced` — the JSON bridge | `body.find('/')` → `(a, b/c)` | ✅ **already first-slash** |
| `crates/wat-reader/src/identifier.rs:150` — the wat surface | `flat.rfind('/')` → `(a/b, c)` | ❌ **the outlier** |

⭐ So the ruling is not the adoption of a new convention. It makes **the wat surface agree with the
data layer that owns EDN compliance** — which is precisely the campaign 251 is running. That is a
strictly stronger justification than the one this document was written with.

`crates/wat-edn/src/value.rs:308` goes further and names the builder's first example in its own
comment: *"The name `/` itself (division, Clojure `clojure.core//`) is the one exception."* The EDN
layer had already ruled `wat.core//` → `[wat.core, /]`. Only the surface disagreed.

### 2. Acceptance row 4 (`receiver`/`method`) was answered by MEASUREMENT, not preference

This document called them *"a different concept — method dispatch"* and left the question open.
**They are not only method dispatch.** `src/resolve/normalize.rs:500` reads:

```rust
// Split on the LAST `/` → (namespace, local_name).
let namespace = wat_reader::identifier::receiver(symbol_text);
let local_name = wat_reader::identifier::method(symbol_text);
```

⛔ **`receiver`/`method` ARE the namespace splitter on the normalizer's path.** Flipping
`Identifier::bare` alone would have left **one process holding two different namespaces for the same
string** — the stored tuple's and the normalizer's. **They move with the ruling**, and the reason is
that collision, not a view about whether dispatch and namespacing are the same concept. The doc
comment on `receiver` now says exactly this, so the next reader does not re-open it.

## The change — 4 sites, all in one file, plus one stale comment

| file | what |
|---|---|
| `crates/wat-reader/src/identifier.rs:150` `bare` | `rfind('/')` → `find('/')`; the comment now states the rule and why last-slash was wrong |
| `identifier.rs` `receiver()` | `rfind` → `find`; doc records the `normalize.rs` collision that forced it |
| `identifier.rs` `method()` | `rfind` → `find` |
| `identifier.rs` module doc | ⭐ the rule is stated ONCE, with all four cases, where it lives |
| `src/resolve/normalize.rs:499` | comment said *"Split on the LAST `/`"* — it now states the rule it actually follows |

⭐ **Why a 4-site edit and not a corpus census:** `tests/lint/one_name_grammar.rs` fails any second
implementation of the split (`rfind('/')`, `rsplit_once('/')`, …) anywhere in the tree. The one-door
gate is what made changing the rule cheap. A sweep confirmed **zero** other real sites.

## Acceptance, row by row

| # | row | result |
|---|---|---|
| 1 | `bare` splits on the first slash; the test at `:542` mirrors it | ✅ — and the mirror was **fixed**: it derived `expected_ns` with `rfind`, so it passed under **either** rule and checked nothing. It now derives from the **rule**, not from the implementation |
| 2 | the builder's two examples pinned | ✅ `the_first_slash_separates_namespace_from_name`, written as **literals**, replacing `wat_core_double_slash_is_the_current_last_slash_split` (whose own docstring read *"not the builder's model… Reported, not fixed"*) |
| 3 | a non-vacuity control | ✅ `the_ruling_leaves_every_single_slash_spelling_untouched` — 5 single-slash spellings plus a bare name. Revert `bare` to `rfind` and row 2 reds while this stays green, which is the discrimination it exists to give |
| 4 | the `receiver`/`method` question answered explicitly | ✅ **governed by the ruling** — see §2 above; forced by `normalize.rs`, measured |
| 5 | floor green, clippy 0, census `no STOP-8` | ✅ 5920/5920, 0, `no STOP-8` |

## Behaviour confirmed on the built binary, not inferred

| probe | before | after |
|---|---|---|
| `wat.core//` | `:wat::core/::` — **unresolved**, empty name, unspellable | **rc=0, resolves** — namespace `wat.core`, name `/`, i.e. **division**, exactly `clojure.core//` |
| `:wat::core::/` (control, same name) | rc=0 | rc=0 — **agrees** |
| `wat.core/nope-not-real` (negative control) | rc=1 | rc=1 `:path ":wat::core::nope-not-real"` — **the resolution is not vacuous** |
| `user/whatever/name/here/` | `:user/whatever/name/here::` — empty name | `:user::whatever/name/here/` — the builder's example |
| `a/b` | `:a::b` | `:a::b` — **unchanged** |
| `a/b/c` | `:a/b::c` | `:a::b/c` |

⚠ **Two probes on the way to this table were mis-aimed and did not count** — a bare `(println …)`
with no `main` hit the entry-point gate, and a malformed `defn` return annotation hit a Doctrine-1
type-keyword error. Both returned non-zero *for reasons unrelated to the split*, which is
`[[feedback_a_green_from_a_mis_aimed_probe_is_indistinguishable_from_a_working_gate]]` — the class
that produced the wrong 8c brief hours earlier. **Each row above was read for WHICH error fired.**

## What this did NOT change

- **The EDN text parser still rejects 2+ slashes** per the EDN spec. "Tolerated gracefully" governs
  how a name that reaches us **is split**, not whether the strict data notation must accept it.
  Those are different questions and conflating them would be finding 40's shape.
- **No corpus migration.** Re-confirmed: 0 real identifiers in the tree carry more than one slash.
