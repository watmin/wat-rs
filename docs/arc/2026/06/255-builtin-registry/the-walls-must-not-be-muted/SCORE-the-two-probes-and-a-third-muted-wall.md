# SCORE — the two gating probes, both answered. And a THIRD muted wall.

> **Builder, 2026-09-04:** *"you've named your next steps - we see where they take us as we walk
> upon them.."*

`[[DESIGN-the-tagged-edn-doc-row]]` named two unmeasured probes as gating the whole effort. Both
are now measured, on a quiescent tree, and the scaffolding for both is reverted (`git status`
clean). One passed, one failed usefully, and running them turned up a wall nobody had counted.

## ✅ PROBE (a) — can a proc-macro crate parse EDN at expand time? YES, and the tag works.

`wat-macros` is `proc-macro = true` and already depends on `wat-reader` + `wat-doc`. `wat-edn` does
NOT depend on `wat-macros`, so there is **no cycle**. Added `wat-edn` as a dependency and called
`wat_edn::parse` from a real `#[test]` inside the proc-macro crate — **it compiles and it runs.**

⚠ *Compiling is not parsing.* The first version of this probe was `#[allow(dead_code)]`, so it
proved only that the dependency links. Converted to a `#[test]` before any conclusion was drawn —
`[[feedback_a_green_test_can_prove_nothing]]`.

**What it actually parses, measured row by row:**

```
⛔ ERR   {:purity :wat::runtime::Purity::Pure}        InvalidKeyword("keyword begins with :: ")
✅ OK    {:purity :wat.runtime.Purity/Pure}
✅ OK    {:purity "wat::runtime::Purity::Pure"}
✅ OK    #wat.doc/Row {:added "1.0.0"}                ← THE TAG PARSES
⛔ ERR   {:alias :wat::core::foldl}                   InvalidKeyword — ANY FQDN value
✅ OK    {:args [{:name a :type :wat.core/i64}]}      ← nested vector-of-maps parses
```

⛔ **wat's FQDN keyword spelling cannot appear as an EDN value.** EDN keywords are `:name` or
`:ns/name`; `::` is illegal. **`[[DESIGN-the-tagged-edn-doc-row]]` used `:wat::runtime::Purity::Pure`
throughout and would not have parsed a single row.** Found by a probe, before a migration tool
existed — which is the whole reason FM 2-bis demands one.

### THE SPELLING IS NOT A PREFERENCE — the tree already emits both, for different jobs

```
:op ":wat::runtime::extract-arg-types"          a STRING, when the FQDN is diagnostic PROSE
:key :wat.core/defn · :wat.config/set-capacity-mode · :user/main · :myapp/Pt
                                                an EDN ns/name KEYWORD, when it is a NAME AS DATA
```

A doc row's `:alias`, `:type`, `:category`, `:purity` are **names as data**. So they take the
ns/name form the substrate already emits — `::` → `.` for every segment but the last, then `/`:

```
:wat::core::foldl              ->  :wat.core/foldl
:wat::runtime::Purity::Pure    ->  :wat.runtime.Purity/Pure
:wat::core::i64                ->  :wat.core/i64
```

This is the identical transformation that already builds every tag in the tree
(`#wat.core.Option/Some`, `#wat.check/ArityMismatch`), so it is one rule, not a second convention.
`:doc` and `:message` stay strings — they are prose.

⚠ **This does NOT contradict "wat is FQDN, always."** That rule governs wat SOURCE. Inside an EDN
block the ns/name form IS the wat FQDN, rendered in the wire format's own spelling — which is what
`#wat.core.Option/Some` has always been.

## ✅ PROBE (b) — does an ```edn fence survive the armed doctest gate? YES.

Added a `#wat.doc/Row` in an ```edn fence to a live doc comment and ran `cargo test --doc --release`:
**exit 0, collected count UNCHANGED at 6, no warning.** The block was not taken as Rust. Precedented
— `scheme` (×2) and `text` (×42) already appear as tags in this tree.

## ⛔ AND THE THING THE PROBES FOUND — `cargo doc` IS A THIRD MUTED WALL

`cargo doc` was named as an unexercised surface in
`[[RULING-a-wall-that-cannot-run-is-not-a-wall]]` and left unmeasured. It is red:

```
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p wat --release   ->  exit 101

  43  unresolved link to …        intra-doc links pointing at items that do not resolve
  37  public documentation for …  public docs linking to PRIVATE items
  19  unclosed HTML tag           <T>, <TypeEnv>, <where> in prose — rustdoc EATS the text
   1  could not document …
 ───
 100  findings   (104 as plain warnings without -D)
```

★ **The 19 unclosed-tag findings are user-visible right now.** A generic parameter written in prose
as `<T>` is parsed as an HTML tag, and rustdoc silently drops it from the rendered page. The
published documentation has holes in it.

⛔ **This one CANNOT be armed at zero today** — `no_rc_use.rs`'s own rule: *"a lint raised at zero is
a wall, a lint raised at 1306 is a campaign."* 100 findings is a campaign. It belongs on the
`the-walls-must-not-be-muted` board as its own step, sequenced AFTER the doctest gate (armed) and
BEFORE or DURING the doc-comment migration — the 19 unclosed tags in particular are doc-comment
prose, which the migration is already going to rewrite.

## THE BOARD, updated

```
1  MEASURE   cargo test --doc                                          ✅ DONE — 3 red, fixed
2  FIX       real Rust runs; non-Rust tagged honestly                   ✅ DONE
3  ARM       doctests in scripts/floor.sh, at zero, sabotage-proven     ✅ DONE
3b ⛔ NEW     cargo doc — 100 findings, 3 classes. A campaign, not a wall. UNSEQUENCED.
4  CENSUS    the 12 #[ignore]s vs their own named follow-ups; and the 56 bare fences on
             private items that can never be collected                  ⬜
5  MIGRATE   #wat.doc/Row · #wat.doc/Alias — both gating probes now GREEN, with the
             spelling corrected to ns/name                              ⬜
```
