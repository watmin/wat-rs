# SEAM — ⛔ PARKED 2026-08-14. THE LIVE SEAM IS ARC 255.

> **This arc is PARKED. Do not resume from this file.**
> The one live breadcrumb is **`docs/arc/2026/06/255-builtin-registry/SEAM.md`** — read that first.
>
> The builder, 2026-08-14: *"we park 251 and 278 on 255's clean up… 255 will force us to organize"*
> → **"A has been reasoned - we're going from 251 to 255 now."**

## Why 251 parked HERE and not somewhere else

251 stops at a **clean point by construction**: both landed stones are additive, observationally
inert, and floor-green. Nothing is mid-flip; no corpus file moved. That is a resting place, unlike
parking mid-migration.

**Landed:** `93971169` intueri cast · `755e5321` the discriminator probe + DESIGN-STONE-251.8 ·
`0a32d5f8` **251.8a** the ONE DOOR (four `contains('/')` classifiers → `namespace()`/`is_reference()`;
`":$bound::"` reserved) · `c046f019` the 8a-ii strike drawn · `851c0d37` **251.8a-ii** the binder
namespace unforgeable (refused at the reader) · `40627086` the parametric-form ruling.

## What 255 has to give 251 before it can resume

The blocker is **#95** — a dotted call head is not type-checked at all (args, arity, return),
because `infer_list` gates its whole call-inference universe on `if let WatAST::Keyword`
(`check.rs:2542`, closing `:5568`). 255 closes this **because `type_sig` was ruled day-one**, not
automatically — 255's own DESIGN recommended deferring it, and that recommendation is overruled on
the 255 seam with the measurement that overrules it.

Also owed by 255: `wat.type` needs somewhere to register. Today it is a `strip_prefix("wat::type::")`
at exactly two sites (`types.rs:4503`, `:4702`) — an alias, not a namespace. Measured:
`:wat::type::Vector` annotates but is an **unknown function**.

## The rulings that SURVIVE the park — do not re-litigate them

- **The parametric form is `(<head> [<type>…] & <members>)`.** Both legacy forms illegal
  post-migration: angle `HashMap<K,V>` **and** flat `(HashMap :K :V :foo "bar")`. The criterion is
  **wat-legality, not EDN-legality** — the flat dotted form reads fine in Clojure's own EDN reader,
  and core.typed's style is flat; the reason is that the type/member boundary must live in the FORM,
  not in a per-head arity table.
- **`wat.core` loses the type constructors; `wat.type` gains them.**
- **The vocabulary** (intueri-cast, weighed against the disk): `$bound` · `namespace` · `reference?`
  · `colon-quoted symbol`. `WatAST::Keyword` KEEPS its name; its doc comment is the defect.
- **`:wat::core::+` was never a keyword** — it is a colon-quoted symbol misfiled as one, and the
  AST's own doc comment states the fusion.

## Still open when 251 resumes

**8b's SCOPE** (call-heads-only vs type-annotation positions — decides whether the 965 comma-bearing
angle sites are a hard prerequisite) · **#95** · **#99**'s survivor (an unbound local reports at
runtime, not freeze) · **#97** opaque-clause leak · **#98** double-slash symbols (59 sites).

Full detail, all measurements, and the campaign shape: `DESIGN-STONE-251.8-symbol-proper.md`.

---

> **SEAM.** You are NEW. You did not live this. **Do not resume from this file — go to
> `255/SEAM.md`.**
