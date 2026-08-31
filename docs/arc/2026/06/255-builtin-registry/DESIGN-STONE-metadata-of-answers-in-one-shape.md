# DESIGN — STONE: `metadata-of` answers in ONE shape, and `:defined-in` stops lying

> Closes two NOTEs at once, because one edit closes both:
> `NOTE-metadata-of-returns-two-shapes-depending-on-which-store-answered.md` and the seam's named
> *"should not wait"* item.

## The two defects, both in one function

`eval_metadata_of` consults two stores. **They disagree about what a value IS, and one of them
asserts a fact it never measured.**

```clojure
(type (get (metadata-of :wat::core::sort$native)  :purity))  ;; => "wat::runtime::Purity"   decoded
(type (get (metadata-of :wat::string::capitalize) :purity))  ;; => "wat::WatAST"            raw AST
```

```rust
put(":defined-in", …to_enum_value(&DefinedIn::Rust));   // spliced CONSTANT, beside derived fields
```

Both render plausibly. **A consumer cannot tell either by looking** — which is what makes them one
defect wearing two faces.

## THE ONE CONTRACT DECISION — pinned

**Both branches emit from a TYPED structure, and neither decodes anything itself.**

```
registry branch:  IntrinsicEntry  -> map        (today, unchanged)
wat branch:       raw WatAST      -> map        (today — the defect)
wat branch:       DocComment      -> map        (the fix)
```

`wat_doc::from_metadata` already produces a `DocComment` with typed fields (`purity: Purity`,
`totality: Totality`, …). It is **already called at registration and its result already discarded.**
The wat branch calls the same function on the stored map and emits from the result.

⛔ **NOT by decoding the AST at the reflection layer.** That is a third decoder for a contract that
already has one, and it drifts the first time an axis gains a variant. The whole point of `wat-doc`
is one decoder; adding a second inside `runtime.rs` would rebuild the drift it prevents.

## `:defined-in` — derived, because the branch genuinely knows

Each branch knows its own provenance without guessing: the registry branch is reached only by a
`#[wat_intrinsic]` entry (`Rust`); the wat branch is reached only from `binding_metadata` (`Wat`).
That is a fact at the site, not an inference.

## ⚠ `:layer` is NOT in this stone, and the reason is the point

`Layer` is `Substrate | Userland`. **A branch cannot know it** — `:wat::string::capitalize` is a
*substrate* wat def, and a user program's `defn` would be *userland*, and both arrive through the
same wat branch. The only way to answer today is a **name-prefix guess** (`:wat::` ⇒ Substrate),
which is precisely the `effectful_by_prefix` shape this arc has been draining.

So `:layer` stays the hard-coded `Substrate` it is today — **accidentally true**, because the one
wired registration path is the stdlib one. Its unblocking condition is explicit and matches the
`defined_in` worklist entry's own logic: **build it when a user-program def path is wired**, so the
registration context can supply the answer rather than a prefix inventing it.

★ Fixing `:defined-in` and pointedly not fixing `:layer` in the same commit is the honest split: one
is knowable at the site, the other is not, and shipping a guess beside a fact would put the surface
right back where it started.

## What ships

1. The wat branch emits from `from_metadata`'s `DocComment`, matching the registry branch's typed
   emission key for key.
2. Both branches emit `:defined-in` from their own provenance.
3. A map with **no doc-axis key** (`{:restricted-to […]}` — 4 live in the corpus) keeps today's
   behaviour exactly: raw, un-decoded, unvalidated. Same predicate as the registration gate.

## THE FOUR QUESTIONS — flat YES/NO

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **both branches emit from a typed structure** | YES | YES | YES | YES | ✅ **ADMITTED** |
| decode the AST inside `eval_metadata_of` | YES | **NO** | **NO** | — | ⛔ **DISQUALIFIED** |
| leave the shapes, document the difference | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |
| also derive `:layer` from the name prefix | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |

- **decode-in-place Simple? NO / Honest? NO** — a third decoder for a one-decoder contract; drifts
  at the next variant.
- **document-it Honest? NO** — the defect is that a consumer *cannot tell by looking*; prose in a
  doc does not reach the consumer holding the value.
- **`:layer` by prefix Honest? NO** — a guess presented as provenance, in the very field whose job
  is to stop guessing. It is `effectful_by_prefix` reborn in the reflection surface.

## Acceptance

| what | command | expected |
|---|---|---|
| ★ one shape, both stores | `(type (get (metadata-of X) :purity))` for an intrinsic **and** a wat verb | **identical** |
| the other axes too | same for `:totality` `:determinism` `:expand-time` `:category` | identical — converging one key only MOVES the defect |
| `:defined-in` discriminates | `metadata-of` on `sort$native` vs `capitalize` | `Rust` vs **`Wat`** |
| capability maps untouched | `metadata-of` on a `{:restricted-to …}` verb | unchanged from today |
| `:layer` deliberately unchanged | — | still `Substrate`; NOT guessed |
| floor | `scripts/floor.sh`, exit read UNPIPED | 5109/5109, 0 failed |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |
