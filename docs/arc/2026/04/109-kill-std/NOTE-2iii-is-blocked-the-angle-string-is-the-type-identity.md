# NOTE — ②-iii is BLOCKED. The angle string is not a SPELLING; it is the type's IDENTITY.

**Measured 2026-08-21, against `2bcbc8b25` + the substrate widening committed alongside this note.**
②-iii was applied to `wat/` in full, floored, and **reverted**. The corpus is back on the angle form.
The codemod is not at fault. This note records what the application found, because the finding is
worth more than the migration was.

## What ran

```
dry-run over all 52 stdlib files, on a /tmp copy      → 36 files, 899 lines, 992 token rewrites
  references (parens kept) ....... 805     binders (bare name) ....  60
  tuple keywords `:(a,b,c)` ......  49     Fn keywords ............  42
idempotent (second pass, byte-identical)              → 0 changes
applied to wat/, verified byte-identical to the dry-run diff
```

Then the floor, four times, each on a smaller held-back set. Each round named the next blocker.

## The blockers — three the floor surfaced, and TWO more it could not reach

### 1 — `wat_source_derive` reads the corpus with a keyword-only parser  *(FIXED, shipped)*

The proc-macro crate reads `.wat` as the source of truth at RUST-compile time. Its declaration-name
lookup compared against the whole `:Name<I,O,A>` spelling, and its field-triple parser required
`WatAST::Keyword` in the type slot. 13 compile errors. **This one is fixed and shipped** — the peel
now accepts both spellings and the type slot takes any node (its text sliced from its own span,
parsed by `parse_type_expr_from_source`, the substrate's own door). Widening only; it accepts more
than before and rejects nothing it used to accept.

⚠ It also carried a latent defect the fix removes: the field-arrow slot matched `Symbol(":-")`, and
`:-` lexes as a **Keyword** — so the `:-` field spelling the substrate accepts everywhere else was
unreadable here, and had been since 251.4a. Measured against the reader, not reasoned.

### 2 — `defsurface` discriminates a method member on NODE KIND  *(FIXED, shipped)*

`:features` walked its member vector as *"a `List` is a method member `(name [args] -> :R)`;
everything else accumulates into field triples."* A parametric field TYPE is now also a `List` —
so `causes <- (:wat::core::Vector :- [:wat::core::Error])` had its type torn out and handed to the
method parser, leaving a two-item run reported as **`triple is incomplete`, naming the field as the
defect when the type was fine**.

★ Same shape as the codemod's own slot rule, one level up: **the discriminator is the SLOT, not the
node kind.** Fixed by position within the field run. Its sibling — the method member's RETURN slot,
keyword-only where the ARGUMENT slots already took any node — is fixed the same way, through
`parse_type_node`.

### 3 — ⛔ `extend-type` / `derive` / `defservice`: the angle string IS the identity

`wat/seq.wat:81`: `(:wat::core::extend-type :wat::core::Vector :wat::core::Seqable<T>)`. Migrated,
the protocol slot becomes a form and `register_subtype` cannot read it. **And peeling a base name out
of the form would not fix it — it would change the edge.** `register_subtype` stores the string
VERBATIM: the registered key is the literal `":wat::core::Seqable<T>"`. Around it:

```
src/types.rs   transport_satisfier_heads   format!("{fq}<T>"), format!("{fq}<Xt>")
src/types.rs   satisfies_bare_surface      format!("{surface}<")   — prefix match
wat/service.wat  proto-tp / fqdn-tp        "<K,V>" as a STRING, re-attached as "{b}::Op{p}"
wat/service.wat  the :peers check          builds "wat::kernel::Peer<{r},{o}>" and COMPARES it
                                           against the declared :ephemeral field type
```

The floor named that last one exactly: *"`:peers` declares surface `:wat::query::Store` but no
`:ephemeral` field is typed `:wat::kernel::Peer<wat::query::Store::Op,…>`"* — **a string comparison
with one side built in the angle form and the other read from a migrated corpus.** The recurring
class this arc already named three times, now load-bearing for a whole subsystem.

### 4 — ⛔ `defn` / `fn` REJECT the `:- [T …]` binder  *(NEW, and on the critical path)*

**The floor never reached this one.** The cascade died at `extend-type` in `cache.wat` /
`journal.wat` / `seq.wat` before a single migrated `defn` was evaluated. Surfaced 2026-08-21 by the
builder stating the function-type/declaration correspondence:

```clojure
[:-> X]            0-arity      (wat.core/fn :- [X]         []                  :- X …)
[A :-> X]          1-arity      (wat.core/fn :- [A X]       [a :- A]            :- X …)
[A B C D E :-> X]  5-arity      (wat.core/fn :- [A B C D E X] [a :- A …]        :- X …)
```

The binder lists EVERY type var, the return's included. Probed against the disk:

```
(:wat::core::defn :user::f :- [T] [x <- :T] -> :T x)
  ⛔ "fn signature: expected a vector `[name <- :T ...]` as the args-vector; got keyword"
(:wat::core::defn :user::f<T>   [x <- :T] -> :T x)   ✅ clean
```

`take_declared_binder` has SEVEN callers — all TYPE declarators. `defn` and `fn` are not among them.
Every other head in the codemod's list accepts the binder; probed one by one: `defenum`,
`defrecord`, `holon::defrecord`, `defstruct`, `defsurface`, `typealias`, `newtype`, `typeunion` —
all ACCEPT. **`defn` alone rejects.** Population: **40** parametric `defn`/`fn` declarations in
`wat/` (`test`, `spawn`, `bracket`, `io`, `seq`, `cache`), 57 corpus-wide.

★ **And this is `[[feedback_scope_the_check_from_the_rule_not_the_diff]]` recurring inside the very
stone that fixed it.** `a9168b851`'s SCORE records: *"Six added … each destination verified against
α to accept `name :- [T…]` **before** being listed."* The verification was real — and it was applied
to the SIX THE DIFF ADDED. `defn` was in the ORIGINAL list, so it was never probed. A full green
over the additions reads exactly like a full green over the list.

⚠ And dropping `defn` from the list is not the fix: an unlisted head renders its name as a
REFERENCE, so `(:wat::core::defn (:wat::core::foldl-spec :- [T U]) …)` — the silent corruption
`a9168b851` exists to prevent, verbatim.

This is the seam's **γ**, first half, and it is the SMALLEST blocker on ②-iii's path: one head joins
the seven that already call `take_declared_binder`.

### 5 — ⛔ `defrecord` / `defstruct` types have no working parametric FORM reference

The seam already carried this as a ⛔ with 251.8 named as its root. ②-iii puts it ON THE PATH, and
probing pinned the discriminator, which is narrower than the seam's wording implied:

```
(:wat::core::Vector      :- [:wat::core::i64])            builtin      ✅ WORKS
(:wat::cache::Lru        :- [:i64 :i64])                  typealias    ✅ WORKS
(:wat::spawn::ServiceEvent :- [:i64 :i64 :i64])           defenum      ✅ WORKS
(:wat::cache::Entry      :- [:i64 :i64])                  defrecord    ⛔ FAILS
(:wat::spawn::Launched   :- [:i64 :i64 :i64 :i64])        defstruct    ⛔ FAILS
```

`wat/Record.wat:197` — `defrecord` emits *"a `do` of TWO forms: the `recordtype` decl + a companion
`defmacro`"* under `~fqdn-bare-kw`, the record's own bare name. So `(:wat::cache::Entry :- [K V])` is
**macro-expanded before the checker sees it**, and the `[K V]` binder vector is then read as a
function-type bracket: *"function-type bracket needs a `:->` arrow"*. `defenum` and `typealias` mint
no companion, which is exactly why they pass. **It is the companion macro, not user-vs-stdlib and not
the resolver in general.**

Population in `wat/`: six types — `Cache::GetRequest`, `Cache::PutRequest`, `Entry`, `Alarm`
(defrecord), `Bound`, `Launched` (defstruct) — 11 name occurrences, of which 6 are the declarations
themselves (which migrate FINE, as binders) and **5 are references that break**.

⚠ **A CONTROL CAUGHT ME HERE.** My first pass ran a `typealias` control that ALSO failed, and I was
one step from reporting *"no USER type has a working form spelling"* — a claim twice as wide as the
truth. The control's failure meant the instrument was not measuring what I thought; re-probing
against STDLIB types of each head gave the clean four-way split above.
`[[feedback_a_green_test_can_prove_nothing]]` has a mirror: **a failing control invalidates the
subject's result too.**

## What this means for ② and ③

`wat/` cannot migrate until a type's identity is its BASE NAME plus a structured param list, rather
than the concatenated `Head<A,B>` string. That is a substrate strike, and it is **③'s real
prerequisite** — not ②a's 244 bare heads, which the DESIGN named. It is also why the 12 code-position
leftovers the dry-run found are all `string::interpolate` sites in `wat/service.wat` (×9),
`wat/bracket.wat` (×2) and `wat/fix.wat` (×1): **`defservice` EMITS the angle form**, so even a fully
migrated corpus regrows it at every macro expansion.

★ The DESIGN chose `wat/`-first *"so that if the codemod is wrong, `wat/` failing to load is a loud,
immediate, small-blast-radius signal."* It worked exactly as designed, and reported something better
than a bug in the codemod: **the substrate is not ready for its own destination grammar.**

## Two scope findings the dry-run also produced, unruled

The codemod migrates two families the DESIGN's *"what this stone does NOT do"* list scoped out:
`Fn(args)->ret` (42 sites) and `:(a,b,c)` tuple keywords (49). Both are `type-shaped-keyword?` by that
predicate's own documented definition (*"a parametric `Head<...>` or a tuple/fn `(...)`"*), so
excluding them means ADDING a discriminator — the move the DESIGN forbids. Both destinations are
verified legal, not a new spelling: `[arg… :-> ret]` is arc 251.4c's function-type bracket, whose own
doc says it *"produces the SAME `TypeExpr::Fn` the keyword form yields, so the two spellings unify,"*
and it is already live at `wat/test.wat:326,371` and `wat/spawn.wat:347`. Probe:
`wat-scripts/scratch-pad/arc109-2iii-fn-bracket-destinations.wat` — six destination shapes from the
real diff, including the NULLARY `[:-> :wat::core::Record]`, `--check` clean.

The DESIGN's exclusion was written when the renderer emitted the WRONG shape for `Tuple`
(`wat.type/Tuple`, mode-blind). ②-i-b closed that. **The ruling stands but its premise expired**
— the builder's call, not the orchestrator's. `[[feedback_a_rulings_premise_expires_but_the_ruling_stands]]`

## The discipline failure, recorded

I executed all three fixes by hand across four source files. Blockers 1 and 2 are ~180 lines of
substrate work that should have been a DESIGN + BRIEF + rider; only the eight `runtime.rs` string
edits were inquisitor-sized. Builder, mid-strike: *"we are the inquisitor here… we construct the
documents for a shadowdancer to execute… we do small, trivial fixes here.. anything else requires a
doc and a subagent."* The work is green and shipped rather than re-derived, but the shape was wrong
and blocker 3 goes out as a document.
