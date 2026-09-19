# RULING — the first slash separates namespace from name

**Builder's ruling, 2026-09-19.** Recorded with the measurements that make it cheap. **Not yet
implemented** — see *When* below; stone 251.8c is in flight with the counterpart.

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
