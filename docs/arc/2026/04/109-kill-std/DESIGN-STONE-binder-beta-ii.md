# DESIGN — arc 109, binder strike β-ii: `defservice` stops deriving parametricity from its own NAME

**Status: DRAWN 2026-08-21. Option B ruled by the builder after four questions.** Written against
`9741507da`.

## The problem, measured

`:wat::service::defservice` (`wat/service.wat:180`) decides whether a service is parametric by
asking whether its NAME ends in `>`, then carries the type parameters as a STRING — brackets and
commas included — and interpolates that string into every type name it generates.

```clojure
fqdn-parametric? (string::ends-with? fqdn-str ">")                      ; :217
fqdn-base        (first (string::split fqdn-str "<"))                   ; :220
fqdn-tp          (string::subs fqdn-str (length fqdn-base) (length …))  ; :222 → "<K,V>"
…
state-ty-str     (string::interpolate "{b}::State{p}" :b fqdn-base :p fqdn-tp)   ; :437
```

That is why `wat/cache.wat:195`'s `lru-svc<K,V>` can refer to `lru-svc::State<K,V>` at all — the
macro rebuilt the name by concatenation.

**Arc 109 deletes the angle brackets.** The moment `<K,V>` leaves the name and becomes `:- [K V]`,
every one of those derivations has nothing to read.

## The size, and my three wrong numbers

I told the builder **15**, then **4**, then **~50**. The first mixed in ③'s concerns; the second
counted where the string is BUILT rather than where it is USED. The measured figure:

```
fqdn-tp  consumers   40
proto-tp consumers   10
extraction sites      4   (:217 :220 :222 fqdn · :364 :368 :372 proto)
```

★ Same error shape as the codemod bug found this session: **I read the definition and reported it
as the call-site count.** `[[feedback_a_file_count_is_not_an_item_count]]`

## Why the consumers cannot be ported mechanically

They interpolate the params into a **name string**:

```clojure
(string::interpolate "{b}::State{p}" :b fqdn-base :p fqdn-tp)   ; → "…::State<K,V>"
```

A param LIST cannot be interpolated into a name. The emission target itself has to change from a
name to a FORM — `(…::State :- [K V])` — which is a structural change at each site, not a
substitution.

⚠ **And one site is not a port at all.** `:741`:

```clojure
transport-param (if (string::contains? fqdn-tp "<T>") …
                    (if (string::contains? fqdn-tp "<T,") …
```

It detects a transport parameter by SUBSTRING-MATCHING the bracket text, comma included. This is
exactly the failure class `holon/CLAUDE.md` names — *"a macro-generated name is built by string
concatenation, so it is where generics get silently mangled"* — running in production. Porting it
requires deciding what the macro is actually asking (is `T` positional? named-by-convention?), which
is a ruling, not a translation.

## The four questions — run on three options, shared premise checked first

**Shared premise, and it expires:** all three assume `defservice` keeps minting generated type names
by string concatenation. That dies at ③ — once a parametric type reference is a FORM, no
concatenation can produce one. Any option leaving params inside a name string is scaffolding ③ deletes.

- **A — accept `:- [K V]`, convert straight back to the internal `<K,V>` string.**
  Obvious YES (a transparent shim) · Simple YES (two sites) · **Honest NO** — advertises binder
  support while the params remain a substring of a name, and cannot survive ③. It is the
  "don't build what you're about to rewrite" that `NOTE-generic-bracket-syntax-edn` killed its own
  pipe-separator interim for. **Fails.**
- **B — the params become a real list; emissions build forms.**
  Obvious YES · Simple YES (it REMOVES a mechanism) · Honest YES (parametricity stops depending on
  how the name is spelled) · Good UX YES (precondition for ③ inside the stdlib's largest macro).
  **4–0 — RULED.**
- **C — retire parametric `defservice`; make `lru-svc` monomorphic.**
  Obvious NO · Simple YES · **Honest NO** — drops a capability to avoid work, and
  `wat-tests/service-cache-lru.wat` starts the service, dials two clients and exercises put/get at
  concrete types. **Fails.**

## The decomposition — four strikes, each floor-bisectable

- **β-ii-a — the extraction sites produce a LIST, alongside the existing string.** Both live,
  nothing downstream moves. **Provably inert**: no emission changes, so the floor must be green
  with zero golden churn. This is the stepping stone the other three stand on.
- **β-ii-b — the 40 `fqdn-tp` type-position emissions become forms.** In batches the floor can
  bisect. The bulk of the work.
- **β-ii-c — the 10 `proto-tp` sites**, same shape, same discipline.
- **β-ii-d — `:741`'s transport-param detection.** A ruling, not a port. Drawn last, deliberately,
  because β-ii-b will have taught us what the macro actually needs from a param list.

Then `defservice` accepts `:- [K V]` and `<K,V>` retires from `wat/cache.wat:195` with the rest of
the corpus at ②-iii.

## Out of scope, affirmatively

- **The other ~11 angle-string sites in this file** (`Peer<`, transport, admin, response, status
  names at `:636 :752 :798 :843 :1724 :2076 :2570`). They build OTHER parametric type names and
  belong to ③'s general cut, not to the binder.
- **`defn`** — γ, and now known to be two capabilities: the declaration binder and call-site type
  application, the second unmeasured.

---

## ⛔ AMENDMENT 2026-08-21 — THE DECOMPOSITION WAS BACKWARDS, AND THE INTRINSIC IT NEEDED SHOULD NOT EXIST

β-ii-a as briefed built a param LIST *from* the bracketed string. It was struck, and it failed
twice, and the second failure is what exposed the design error.

**Failure 1 — the F5 gate refuses a wat-level helper outright.** The rider added a top-level
`defn` (the first option my brief offered) and the whole stdlib went down — 3029 failures:

> *"malformed defmacro: program-body macro purity check failed at definition: keyword head
> `:wat::service::tp-suffix->syms` refused at macro expand time — not on the pure-combinator
> allow-list (default-deny F5 gate, arc 249 stone 249.2b-i)"*

**Failure 2 — a closure does not rescue it.** Rebound as a `let`-bound `fn` inside the macro's own
`let`, it passed definition and then failed during expansion of `lru-svc<K,V>`:
`TypeMismatch … :expected "wat::core::fn"` — `mapv` will not take a bare builtin-primitive keyword
as a value in this position.

So **every wat-level spelling of that helper is blocked**: a `defn` is refused at definition, a
closure at use. On the original decomposition the only way out was a new INTRINSIC.

### ★ The builder's question — "what needs to become an intrinsic?" — is what broke it open

**Nothing does.** The string derivation exists only because the params live INSIDE A NAME. Once they
arrive as `:- [K V]`, extracting them is `ast->children` on the bracket — and `ast->children` is
already allow-listed for macro bodies (**145 uses**), as is `ast-name` (**131**). The intrinsic I
was about to draw would have propped up the representation this arc is deleting.

### The inverted decomposition

| | old (struck) | new |
|---|---|---|
| a | build the LIST from the string | **accept `:- [K V]`; params via `ast->children`. DERIVE the legacy string from the list** |
| b | convert ~50 emissions, list bolted on | consumers move to forms in batches; the derived string shrinks as they go |
| c | proto sites | the last consumer goes; string and name-splitting die together |

The list becomes the source of truth immediately and the string becomes a **compatibility shim** —
so the thing that gets deleted is the shim, not a bolt-on that has to be threaded in later. All ~50
consumers keep working untouched throughout, because they still read a string.

★ It also kills `:741` naturally. `(string::contains? fqdn-tp "<T>")` becomes a membership test over
a list of symbols — the question the macro was always actually asking, which is why the substring
test needed a comma variant (`"<T,"`) to paper over position.

### Two hazards measured while grounding this

- **The clause fold rejects unknown keys.** `defservice` folds `[:key val …]` and calls
  `macro-error` on anything not in `known-clauses` (`:290`). A leading `:-` must be peeled from
  `clauses` BEFORE the fold, or it is rejected as an unknown clause.
- ★ **`:wat::core::macro-error` EXISTS**, and is the established idiom (`wat/core.wat:632`, `:782`).
  That closes the open question in `NOTE-a-macro-cannot-diagnose-with-option-expect.md`: a macro CAN
  raise a structured error; `Option/expect` was simply the wrong tool. Recorded there.
