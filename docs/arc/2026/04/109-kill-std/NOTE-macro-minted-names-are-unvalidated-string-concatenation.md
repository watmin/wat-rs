# A macro-minted name is assembled by `string::concat`, and nothing checks what it spells

**Filed 2026-08-01, arc 278 (the namespacing wall). Grounded, not fixed.**

## The instance

`tests/diagnostics/probe_diagnostic_c3_macro_emits_record_def.wat` mints a defmacro `:t::mk` that is a
deliberate miniature of the C.3 `defservice` shape. From one input keyword it assembles **six** names by
`string::concat`, and got **five** right:

```clojure
req-name  (keyword/from-string (string::concat base-str "::Req"))     ;; :demo::Req     ✓
op-name   (keyword/from-string (string::concat base-str "::Op"))      ;; :demo::Op      ✓
acc-name  (keyword/from-string (string::concat base-str "::Req/n"))   ;; :demo::Req/n   ✓  BOTH separators, correct
go-var    (keyword/from-string (string::concat base-str "::Op::Go"))  ;; :demo::Op::Go  ✓
req-ty    (keyword/from-string (string::concat base-str "::Req"))     ;; :demo::Req     ✓ (an exact alias of req-name)
go-name   (keyword/from-string (string::concat base-str "/go"))       ;; :demo/go       ✗
```

`::` composes a namespace; `/` selects a member of a type (`Thread/join-result`, `CONVENTIONS.md:45`).
So `:demo/go` asserts a **type** `demo` that does not exist — while the same macro, one binding away,
spells `"::Req/n"` correctly with both separators in a single string.

## Why nothing caught it

Every instrument we have is blind to it, and each for a structural reason:

| instrument | why it cannot see the name |
|---|---|
| a grep over `.wat` | the name **is not in the source** — it exists only after expansion |
| a wat-fix codemod | reaches **only source forms**; a macro-generated name is unreachable (arc 294 9a — *hand-fix the generating macro*) |
| `--check` on the macro | validates the macro's own body; the *string it concatenates* is just data |
| the test suite | green for as long as the probe's assertions used the same wrong spelling |

It surfaced only when **registration-time enforcement** existed — the arc-278 `Registration::Unnamespaced`
wall rejected it at the door. That is late (expansion has already happened) and narrow (it polices the
namespacing axis only; a `/` pointing at a type that does not exist is still merely *unnamespaced* to it).

## Why this is not one probe's typo

`defservice` is the production instance of the same mechanism. `wat/service.wat` mints **14+** names via
`keyword/from-string` over concatenated strings — `:93` `:278` `:342` `:343` `:356` `:398` `:405` `:420`
`:449` `:596` `:610` `:612` `:618` `:619` — with separators supplied as string literals (`"::Op"`,
`"::Reply"`, `"::surface-forms"`), and it assembles parametric type heads the same way (`:699-701`,
`"wat::kernel::ThreadSelfPeer<" … "," … ">"`).

`CLAUDE.md` already records the failure this produces, from the other direction:

> *a macro-generated name is built by **string concatenation**, so it is where generics get silently
> mangled (`box-svc<T>::Record` instead of `box-svc::Record<T>`)*

Same mechanism, two failure modes: there the separator was in the **wrong position**; here it was the
**wrong kind**. Both are a grammar being spelled out by hand, one `concat` at a time, in a language that
has a grammar for names and no way to say it.

This is the sibling of arc 278's recurring class (*a string comparison with one side normalized and the
other not* — three instances in that arc alone). This one is its constructive twin: **a string
CONSTRUCTION with one part spelled wrong.**

## The ladder, and where we are on it

- **convention** — "compose namespaces with `::`, select members with `/`." Documented, and violated by a
  macro that observes it correctly five times out of six.
- **a check at construction** — where we are now. The namespacing wall rejects a bare minted name at
  registration. Real, but partial: it fires after expansion, and only on the one axis.
- **a shape the mistake cannot be written down in** — not built. If a name were assembled through a
  *constructor that knows the grammar* rather than raw `string::concat`, a macro could not spell a
  malformed one:

  ```clojure
  ;; sketch only — not proposed as final surface
  (namespaced-name  base "Req")        ;; -> :demo::Req      the ns/member distinction is
  (member-name      req-ty "n")        ;; -> :demo::Req/n    carried by the CONSTRUCTOR, not the caller
  ```

  That is the `reference_a_costly_shape_change_means_a_missing_constructor` shape: the reason every macro
  hand-spells separators is that there is no door that spells them for it.

## Status

**Filed, not decided.** The instance is fixed (`"/go"` → `"::go"`, builder-ruled, plus its three coupled
`.rs` assertions). The wall is what surfaced it and stays. Whether name-minting gets a constructor — and
whether `defservice`'s 14+ sites should route through it — is a ruling, not a cleanup, and it is the
builder's.

**Also observed, unrelated to the class:** `req-ty` is byte-identical to `req-name` in that macro — one
value bound twice. It carries a comment naming its *purpose* (the type-position splice) but not the fact
that it duplicates. Left alone; noted so it is not rediscovered.

## Sibling — the same family, one layer down

[`NOTE-a-parametric-head-is-bare-a-path-is-not.md`](NOTE-a-parametric-head-is-bare-a-path-is-not.md)
(filed 2026-08-04) is this note at the **Rust** layer: `TypeExpr::Path` carries its leading colon and
`TypeExpr::Parametric.head` does not, so a caller that handles both arms *symmetrically* — which is
how the match reads — produces a malformed name for the parametric case. 137 sites destructure that
head by hand and no `impl TypeExpr` exists to normalize through. Where this note is about a name
nobody validates, that one is about two halves of one enum disagreeing on the name's FORM.
