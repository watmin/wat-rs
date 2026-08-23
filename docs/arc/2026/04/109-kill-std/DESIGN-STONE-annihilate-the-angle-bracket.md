# DESIGN — annihilate the angle bracket

> *"annihilate all support for angle brackets to communicate types from the language - they all go"*
> — the builder, 2026-08-23
>
> *"you do not need to find the callers - you need to find who permits their heresy - remove their
> expressions from the language and the heretics self identify"*

## The thesis

**`:-` is the parameterization operator. There is no other.** Angle brackets carry no meaning as type
syntax anywhere in wat — not in a declaration name, not in a type reference, not in a method-member
name, not as a call-site type application. The character `<` survives only as what it always was
outside this one concession: a comparison operator (`:wat::core::<`, `:wat::core::>=`) and half of the
arrows (`<-`, `->`).

## Where the strike lands — the PERMISSION, not the callers

③ walled the **type parser**; the comma strike walled the **comma**. Both left the `<…>` spelling
*lexing* — a name like `:wat::core::Vector<wat::core::i64>` still read as one keyword, and only a
later door refused it. That is why `find('<')` machinery still sits in `runtime.rs`, `check.rs`,
`types.rs` and `surface.rs`: it is all downstream of a token the lexer still agreed to build.

**The permission is two expressions, one in each lexer door:**

```rust
// crates/wat-reader/src/lexer.rs — lex_keyword
if prev_alpha { angle_depth += 1; }        // ← `<` after an identifier char OPENS A TYPE HEAD
// crates/wat-reader/src/lexer.rs — lex_symbol
if prev_type_head { angle_depth += 1; }    // ← the twin
```

Delete those two, and every angle-bracket type name in the language stops existing at the reader.
Everything downstream becomes unreachable-from-source, and the floor names it.

★ The discriminator is already written and already correct: **`<` opens a type head ONLY when
preceded by an identifier character** (`Vector<`, `make<`, `Thread'<`). An operator `<` follows `::`
or leads its token (`:wat::core::<`, `<-`, `<=`), so it never reaches the branch. That predicate is
the whole reason this is a two-line kill rather than a redesign.

## The measurement — the wall imposed, and the shell it returned

The wall was imposed and run against the entire corpus. **28 of 1798 `.wat`/`.wat.bad` files fall
outside it.** `wat/` — the whole standard library — is **not among them**; ②-iii and ③ already took it.

```
13  .wat.bad   negative fixtures
13  live .wat
 2  docs/arc/…/complected-2026-05-02/   archived snapshots, inert — LEAVE
```

### Acceptance surface, measured under the wall (not predicted)

| form | verdict |
|---|---|
| `:wat::core::Vector<wat::core::i64>` | ⛔ REFUSED |
| `(make<T> [x] -> :T)` | ⛔ REFUSED |
| `(:my::helper<wat::core::i64> 1)` — the turbofish | ⛔ REFUSED |
| `:wat::core::HashMap'<wat::core::i64>` — arc 214's primed head | ⛔ REFUSED |
| `a<b` | ⛔ REFUSED — **the honest cost; see below** |
| `:wat::core::<` · `:wat::core::>=` | ✅ lexes |
| `[x <- :wat::core::i64] -> :T` | ✅ lexes |
| `((:wat::core::Vector :- [:i64]) 1, 2, 3)` | ✅ `[1 2 3]` — the comma dual holds |
| `:wat::kernel::Peer'` · `foo/bar` | ✅ lexes |

⚠ **`a<b` is a NARROWING, and it amends the previous stone's row 3.** The comma stone guaranteed
`a<b` still lexed. It no longer does: a bare symbol may not contain `<` immediately after an
identifier character. This is indistinguishable from `Vector<` at the reader, and wat's less-than is
the prefix form `(:wat::core::< a b)`, so an infix `a<b` symbol was never meaningful. **The census
says it costs zero live sites** — but it is a real narrowing and is written down here, not discovered
later.

## The five classes of heresy

| | class | example | becomes |
|---|---|---|---|
| **A** | declaration name | `defn :test::make-3tuple<T>` | `defn :test::make-3tuple :- [T]` — γ-i's binder |
| **B** | type reference | `xs <- :wat::core::Vector<t::New>` | `xs <- (:wat::core::Vector :- [:t::New])` |
| **C** | method-member name | `(make<T> [self …] -> :T)` | `(make :- [T] [self …] -> :T)` |
| **D** | call-site type application | `(:test::make-3tuple<wat::core::bool> true)` | **DELETED** — `(:test::make-3tuple true)` |
| **E** | negative fixture | a `.wat.bad` whose subject IS the angle lexer | re-pointed at the NEW wall, or retired |

## Class D has NO replacement spelling, and that is a measured result

`canonical_callable_name` (`runtime.rs:4256`) **strips** the `<…>` suffix so lookup matches the bare
registered name; `split_type_params` (`runtime.rs:14502`) returns `(base, suffix)` and the suffix is
re-attached **only to build a diagnostic**. Nothing ever resolves *through* a call-site type argument.

Proven, not read (FM 2-bis) — the same fixture, declaration migrated to the binder and the turbofish
simply deleted:

```
(:wat::core::defn :test::make-3tuple :- [T] [mid <- :T] -> … )
(:test::make-3tuple true)        →  --check CLEAN, runs, returns "hello"
```

**Inference already did the work.** The turbofish was decoration all along — which is precisely why
arc 139 had to teach the lookup to throw it away.

## ⛔ The recorded codemod is 2-for-3, and its third arm is a REGRESSION VECTOR

`wat-scripts/fixes/angle-brackets-to-binder.wat` handles **A** and **B** correctly. On **D** it emits
a reference form where a *callable head* stands, and it double-colons an already-colonned arg:

```clojure
(:wat::test::assert-eq<:wat::core::i64> …)   →  ((:wat::test::assert-eq :- [::wat::core::i64]) …)
                                                                          ^^ two colons
(:test::make-3tuple<wat::core::bool> true)   →  ((:test::make-3tuple :- [:wat::core::bool]) true)
                                                →  ArityMismatch: expected 1 argument(s); got 2
                                                →  ':wat::core::bool' is a TYPE keyword, not a value
```

The NOTE on the seven macros already named why: *"a CALLABLE name with a type suffix — not a type
reference at all, and a form is not a name."* The codemod does not carry a CALL-HEAD role beside its
DECL-NAME and REFERENCE roles.

★ **This is the `fix.wat:502` shape again** — a recorded migration whose OUTPUT re-introduces an
illegal form every time it runs. It must be corrected, or the codemod must be forbidden from class D.

## ⛔ Sequencing — measured, and it decides the stone

**The codemod cannot run once the wall is up.** ③'s wall was at the type parser, so the codemod dodged
it by carrying its own renderer (its header says so). **This wall is at the LEXER**, and the codemod
reads its input through `read-string`. Measured with the wall up:

```
[#wat.kernel.LociDiedError/RuntimeError …UnknownFunction: type `:wat::edn::ForeignRecord`
 does not implement surface method `message`…]
```

It cannot read its own input, and its error path cannot even *report* that — a second finding
(`ForeignRecord` has no `message` arm) filed separately.

**Therefore the order is forced:**

```
1. wall DOWN (= clean main)   run the codemod over classes A + B
2. hand-fix                    classes C, D, E — the codemod must not touch D
3. wall UP                     apply the two-expression kill
4. floor                       the Rust screams enumerate the dead machinery
```

No stash dance is needed, because the wall is not committed: the tree starts wall-down.

## What ships in this stone, and what does not

**Ships:** the wall; the 26 non-archived files migrated; the codemod's class-D arm corrected or
fenced; a negative control proving the refusal and a positive control proving the operators survive.

**Does NOT ship — the PURGE, a sibling stone:** deleting `canonical_callable_name`,
`split_type_params`, `split_type_params_pub`, `split_name_and_type_params`,
`split_method_name_type_params`, and the `find('<')` splits in `check.rs` / `runtime.rs` / `types.rs`.
That work is only *legible* once the corpus is clean and the floor is green — the floor is the
instrument that says which of them is genuinely unreachable. **Out of this stone's scope. Tracked as
`STONE-purge-the-angle-machinery`, briefed the moment this stone is green.** ③'s type-parser wall
(`types.rs:4631`) STAYS — it is the backstop for names *minted* at expand time, which never pass
through the lexer at all.

## The four questions

- **Obvious?** YES. One operator parameterizes; a second spelling for the same thing is what a reader
  trips on. The error message teaches the replacement at the point of refusal.
- **Simple?** YES. Two expressions deleted. Not a new subsystem — the removal of a concession.
- **Honest?** YES, and the honesty is the `a<b` row: the wall narrows bare symbols, that is written
  down here with its measured cost, and the negative fixtures that tested the old permission are
  re-pointed rather than deleted (a negative control that CAN be kept MUST be kept).
- **Good UX?** YES. `Vector<i64>` now fails at the reader with a message naming `:-` and showing the
  three forms, instead of lexing cleanly and failing later somewhere else — or, worse, lexing cleanly
  and *working*, which is what kept this alive for months.
