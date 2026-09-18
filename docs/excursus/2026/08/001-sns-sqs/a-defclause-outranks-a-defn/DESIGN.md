# DESIGN — a defclause outranks a defn, and nothing stops it

**Drawn 2026-09-18.** Builder, on being told the namespace rename was under way: *"ok... so.... fix the
name... but that doesn't fix the defclause flaw?...."* — **correct, and here is the witness that proves
the two are separate.** **NOT STRUCK.**

## The two fixes do different jobs

| | what it fixes | what it leaves |
|---|---|---|
| `the-stdlib-vends-only-wat` (in flight) | makes **stdlib** names unforgeable — `:wat::` is reserved, so no user can declare one | the **mechanism**, live everywhere else |
| **this stone** | the mechanism | — |

## ⛔ THE WITNESS: a consumer breaks a LIBRARY'S OWN call site

Measured 2026-09-18 on the release binary. A library, in a namespace with no reserved-prefix
protection (i.e. any namespace a library may actually use):

```wat
;; lib.wat
(:wat::core::defn :mylib::greet  [s <- :wat::core::String] -> :wat::core::String s)
(:wat::core::defn :mylib::caller []                        -> :wat::core::String (:mylib::greet "hi"))
```

Loaded normally → **exit 0**. Loaded by a consumer that declares a clause on the library's name:

```wat
(:wat::load-file! "lib.wat")
(:wat::core::defclause :mylib::greet ([n <- :wat::core::i64] -> :wat::core::i64 n))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::core::let [_ (:mylib::caller)] nil))
```

→ **`NoMatchingClauseAtCallSite`, located INSIDE `lib.wat`.** ⭑ **The library author's own internal call
site is broken by their consumer**, and the author has no defence: only `:wat::` is reserved.

## The mechanism, already located by the Tier B spike

```
check.rs:800   preregister_defclause_in_env runs over the CONSUMER's forms
               its idempotence guard consults only `defclause_registrations`, so a name
               declared elsewhere as a `defn` is ABSENT and the registration lands
               `register_defclause` then inserts UNCONDITIONALLY — and consults no flag
check.rs:6042  defclause dispatch TAKES PRECEDENCE over the existing scheme
```

⚠ **`redef_allowed` is irrelevant** — measured at `true`, `false` and unset: identical. This is not the
redefinition path; it is a second, unguarded door into the same place.

★ **`defn` redefinition is refused. Type shadowing is refused twice over (`ReservedPrefix` and
`UnnamespacedName`). `defclause` is the one registration path with no stdlib-wins guard at all.** That
asymmetry — not the stdlib's namespace — is the flaw.

## Why this matters more after promotion, not less

Queue and topic were exactly this shape before promotion: libraries in `:queue::` / `:demo::`, loaded by
consumers via `load-file!`. `brackets`-on-queue would multiply it. **Every library this project vends
outside `:wat::` is exposed to its own consumers today**, and the promotion programme makes more of them.

## What this stone must decide — and it is a real design question, not a bug-swat

1. **Should a `defclause` on an existing `defn` be REFUSED, or should it EXTEND?** Refusing is safe and
   may break legitimate use; extending is what a reader probably expects but needs the clause to be
   type-compatible. ⭑ **Measure whether anything in the corpus relies on the current behaviour before
   choosing** — `wat --grep` over the form, not a text grep.
2. **Whose decision is "mine"?** A library author cannot reserve a prefix today. Does the fix give
   authors a guard, or does it make the precedence rule stricter for everybody? The second is smaller.
3. **What does the diagnostic say?** Today the error surfaces *inside the library*, blaming a file the
   consumer did not write. Whatever the rule becomes, the message must name the **consumer's**
   declaration as the cause.

## Trap-doors

1. ⛔ **Do not fold this into the rename stone.** They fix different things and one floor must not prove
   two.
2. **`defclause` is load-bearing** — `wat/rete/`, `Record.wat` and the generic-head machinery use it.
   Changing precedence without measuring the corpus will break something far from here.
3. **The spike left a scarier variant unsettled**: a consumer clause that *matches* the signature may
   re-point dispatch into consumer code **at runtime while type-checking green**. That is silent
   behaviour substitution, not a type error, and it was scoped out of the spike. **Settle it here or
   name it again.**
4. **A green floor proves little** — the corpus may simply not contain the collision. Argue from the
   path.

## Out of scope

The rename (in flight) · Tier B (its surface goes to 0 by the rename; re-run the spike's count there) ·
the queue promotion.
