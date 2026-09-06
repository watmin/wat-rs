# DESIGN — STONE: three head spellings, one seam, and a canonical form that is the TARGET

> **Builder, 2026-09-06:** *"we have 3 flavors of heads right now… we are in the process of mass
> moving our lang to clojure… so - we need all three spellings handled correctly for now….. ensure
> they are wired up such that we can drop the keyword flavors for only symbol when we complete the
> clojure migrations."*

## HOW THIS SURFACED — the priority target was measured on the wrong artifact

The builder asked to point the doc-row printer at a real decorated item and look. `:wat::rete::step-payload`
(`src/rete/step_payload.rs`, a **1556-column** `@example`) rendered — prose, heredoc margins and
structure all correct — and its examples came out in the **dotted** spelling, because
`print.rs` converts (`wat_fqdn_to_edn_keyword`).

⚠ **Every fmt rule keys on the FQDN spelling.** So the arc's headline number —
*"614 doc examples, 0 over 120"* — was measured against `/tmp/fmt-614.wat`, a corpus **grepped from
the Rust source**, where examples are written in FQDN. **The migration will never produce that
spelling.** The width claim survives (indent is structural); **the SHAPE claim does not.**

## THE THREE SPELLINGS — measured, all three readable

```
:wat::core::+     kind=keyword   ast-name=":wat::core::+"     ← today's wat
:wat.core/+       kind=keyword   ast-name=":wat.core/+"       ← what the doc-row printer emits
wat.core/+        kind=SYMBOL    ast-name="wat.core/+"        ← the clojure TARGET
```

`ast-name` returns each **verbatim**. There is no canonicalisation anywhere.

## ⛔ AND ONLY SPELLING #1 GETS A RULED SHAPE — a three-way A/B on `let`

```
:wat::core::let          :wat.core/let            wat.core/let
  [a (…quote 1)            [a                       [a
   b (…quote 2)]             (…quote 1)               (…quote 1)
  a)                         b                        b
                             (…quote 2)]              (…quote 2)]
★ the ruled shape          a)                       a)
                         ⛔ fallthrough           ⛔ fallthrough
```

Same on `defn` — FQDN gives one arg per line with `<-` aligned; the clojure form puts both args on
one line.

⚠ **A single-arg `defn` probe showed no difference and nearly closed this as "it already works."**
One argument renders identically under both the ruled rule and the generic fallthrough; only a
two-argument form discriminates.
`[[feedback_a_green_from_a_mis_aimed_probe_is_indistinguishable_from_a_working_gate]]`

## THE CENSUS

```
33  string compares against a head FQDN, across 12 rule files — all match spelling #1 ONLY
13  rules bind the head through :wat::grep::Named
 0  fmt rules gate on the HEAD's kind   (defrecord's 2 NodeKind::Keyword tests are on defenum
                                         VARIANT TAGS, not heads — so the keyword/symbol axis
                                         is NOT in play for heads)
```

★ The kind axis being clear is what makes this a **name** problem only, and therefore a one-seam fix.

## ★★ THE CONTRACT DECISION — CANONICALISE TO THE TARGET, NOT TO TODAY

The builder's requirement is that dropping the keyword flavors later be **one edit**. That is
decided entirely by which spelling is canonical:

```
canonicalise to :wat::core::defn   →  when keywords are dropped, all 33 comparisons must be
                                      rewritten.  The migration is paid for TWICE.

canonicalise to wat.core/defn      →  when keywords are dropped, the canonicaliser becomes the
   ★ THIS ONE                         IDENTITY function and NOT ONE RULE CHANGES.
```

So the canonical form is **the clojure symbol spelling**, and the rules compare against
`"wat.core/defn"`. Today the canonicaliser folds three spellings into it; at the end of the migration
its other two arms are deleted and nothing downstream notices.

**Dropping a flavor must be the deletion of one arm in one function.** That is the acceptance.

## ⚠ THE DOOR DOES NOT ALREADY DO THIS — checked

`crates/wat-reader/src/identifier.rs` has `leaf`/`path`/`receiver`/`method`, and `one_name_grammar`
makes it the ONE parser. But it is **FQDN-shaped** — `leaf`/`path` split on `::`, which spellings #2
and #3 do not contain — and it is **not exposed to wat** (`:wat::holon::leaf` is a different verb).

So the canonicaliser is **new logic**, and where it lives is the design's second decision:

- it must be **ONE function**, so the builder's "drop a flavor" is one deletion;
- it must not become a **second name parser** — the class `one_name_grammar` exists for, and which
  went red on this arc's `:then` stone;
- the three-spelling knowledge must not leak into any rule file. **`grep -c` for a dotted or FQDN
  head literal in `wat-scripts/fmt/rules/` must be ZERO except the canonical one.**

## WHAT THIS STONE IS NOT

- **NOT the doc-row migration.** That is arc 255 and it needs `DocSpecialForm`'s reader (step 2) and
  the printer's per-example formatting wiring. **Named here because this stone is what makes those
  produce correctly-shaped output rather than merely narrow output.**
- **NOT a change to `ast-name`**, which correctly returns what was written.
- **NOT `print.rs`'s conversion.** The printer emitting the EDN spelling is right for EDN; the
  formatter is what must understand all three.

## FILES

```
the canonicaliser            ONE function, one home — the design's open decision
wat/grep.wat                 the fact the rules match on
wat-scripts/fmt/rules/*.wat  12 files, 33 comparisons, retargeted at the canonical form
wat-scripts/fmt/fixtures/    the same form in all THREE spellings, formatting identically
crates/wat-doc/examples/render_one.rs   the throwaway that surfaced this; kept so the
                                        DESIGN's citation is reproducible
```
