# BRIEF — `defservice` compares and destructures types as DATA

Closes the class that blocked ②-iii. The door was minted in `c5b9b6552`
(`:wat::core::type-equal?`); this stone uses it.

## ⚠ First — "the 11 COMPARE sites" is a PROXY, not a measurement

I have quoted "11 COMPARE sites" all day. That number is the count of `keyword/to-string` calls in
`wat/service.wat`, which is **the rendering step**, not a classification of what each does with the
result. My classification of those eleven:

```
186 fqdn-str · 231 clause-head · 375 clause-key      NOT type work — leave alone
428 proto-str                                        DESTRUCTURE (the base / type-arg split)
608, 721, 725  the :hibernate return check           EQUALITY
740 state-parent-str                                 EQUALITY against a string literal
801 peers-surfaces                                   EQUALITY (surface names)
821 ty-str      the :peers ephemeral scan            DESTRUCTURE
963 handle-name-decl                                 ★ UNCLASSIFIED — you classify it
```

**Verify this table before working from it.** It is my reading, not a census, and this file has made
my counts wrong four times (23 / 53 / 110 / 94 for a different population). If a site's job differs
from what I wrote, the table is wrong and you should say so.

## ★ TWO jobs, TWO doors — do not use one for the other

**EQUALITY — "are these the same type?"** → the new verb:

```clojure
(:wat::core::type-equal? a b)     ;; two WatAST nodes → bool, spelling-agnostic
```

**DESTRUCTURING — "what is this type made of?"** → normalize to the form, then ordinary AST verbs.
Measured, working today:

```clojure
(:wat::core::ast->children (:wat::core::keyword/to-type-form-colon (:wat::core::keyword-node kw)))
;; :wat::kernel::Peer<sq::S::Op,sq::S::Reply>  →  head=:wat::kernel::Peer  argc=3
;;                                                  [head, :-, [args…]]
```

⚠ `keyword/to-type-form-colon` takes a **Keyword**. A field type that is already a form is a
**List** and must not be passed through it — branch on `ast-kind` and use it as-is.

## The site that blocked ②-iii

`service.wat:796-835`'s `:peers` scan is not a comparison at all — it is a hand-written parser:

```clojure
tail      (:wat::core::second (:wat::core::string::split ty-str "Peer<"))   ;; "S::Op,S::Reply>"
first-arg (:wat::core::first  (:wat::core::string::split tail ","))          ;; "S::Op"
(:wat::core::string::subs first-arg 0 (:wat::core::i64::- (:wat::core::string::length first-arg) 4))
```

It asks: *is this field's type a `Peer` whose first argument ends in `::Op`, and if so what is that
argument minus `::Op`?* Every step of that is available structurally — the head, the argument vector,
each argument's name. **Rewrite it as destructuring.** It is the reason `②-iii` could not migrate
`wat/`, and it is the highest-value site in this stone.

## What "done" looks like

1. Every EQUALITY site uses `type-equal?` and no longer renders a type to a string to compare it.
2. Every DESTRUCTURE site reads head/args structurally and no longer splits on `"<"`, `","` or `"Peer<"`.
3. ★ **Both spellings work at every rewritten site** — `Peer<S::Op,S::Reply>` and
   `(Peer :- [S::Op S::Reply])` must behave identically. This is the point of the stone; a rewrite
   that handles only the form spelling breaks the corpus as it stands today.
4. ★ **The checks still REJECT what they exist to reject.** `:hibernate` declaring the wrong return
   type must still `macro-error`; a `:peers` surface with no matching `:ephemeral` peer field must
   still `macro-error`. Prove each with a probe that FAILS.
5. `wat/service.wat` has no `string::split` on `"<"`, `","` or `"Peer<"` left in a type context.
6. The expansion of a monomorphic and a parametric service is unchanged where it should be — capture
   BEFORE, as the previous stones did.

⚠ **Row 4 is the one that bites.** Rows 1-3 measure that the checks still accept; only row 4
measures that they still refuse. A rewrite that returns "equal" unconditionally passes 1, 2, 3 and 6.

## Boundaries

- Do NOT run `scripts/floor.sh` or a full `cargo nextest` — the orchestrator measures centrally.
  ⚠ A scoped run is not the floor: `binary_id(wat::services)` was 128/128 green on a recent stone
  while the floor was red by six.
- Do NOT commit, push, stash, revert or amend. Leave everything in the working tree.
- Touch `wat/service.wat` only. `type-equal?` is minted and registered; do not modify it.
- Do NOT convert DECL-NAME / RUNTIME-ARG bindings — a separate, ruled stone.
- Sites 186 / 231 / 375 are clause parsing, not types. Leave them.

## Your own checks

`cargo build --bin wat`, then `--check` and RUN probes under `wat-scripts/scratch-pad/`, plus
`cargo nextest run --release -E 'binary_id(wat::services)'` and `-E 'binary_id(wat::kernel)'`
(the `service-parametric-*` deftests live in the latter — a recent stone was green on `services`
while `kernel` was red). Prefix long commands with
`systemd-run --user --scope -q -p MemoryMax=16G -p MemorySwapMax=0 timeout 900`.
Diagnostics go to **stderr** — judge by exit code AND empty output, never grep alone.

Delete any scratch `.wat` that must fail; `tests/lint/wat_scripts_fixes_load.rs` type-checks
everything under `wat-scripts/`.

## STOP triggers — ship nothing further and report

- **STOP-1.** If row 4 fails — a check stops rejecting what it should reject — STOP and report.
  Silently accepting is worse than the string comparison this replaces.
- **STOP-2.** If a site needs something `type-equal?` and structural destructuring cannot express,
  STOP and report what it needs. That is a third missing door and it is a finding, not something to
  work around with more string surgery.
- **STOP-3.** If my classification table above is wrong for any site, STOP on that site and report
  the correct job before rewriting it.

## Your report

Your verdict on my classification table, per site — including 963, which I did not classify. The
diff. Every acceptance row with verbatim output, row 4 especially. Both expansion diffs. What
surprised you. Anything you inspected and left alone, with the reason.
