# BRIEF — 294.j RELAND · render DATA, not source forms

**You are a rider, not the orchestrator. Ending your turn ENDS you** — nothing wakes you, no
notification is coming. Run every verification in the **FOREGROUND** and block on it.

Work in `/home/watmin/work/holon/wat-rs/`. **Do not commit, push, stash, or revert.**

## Read first

`DESIGN-STONE-294.j-the-shim-forgets-the-algebra.md` — and specifically **the `⛔ CORRECTION` section
at the end**, which supersedes the encode/decode design in the body above it. The body's blast-radius
measurements, deletions and gate discipline all still stand.

## THE TREE ALREADY HOLDS MOST OF THIS WORK — you are amending, not starting

A previous rider struck this stone and left its work uncommitted in the tree. **It was right about
almost everything** and its work stays:

- the 14-tag deletion, the reader collapse, the mode-selector removal
- the three `pub` export deletions (STOP-3 was run: nothing outside this repo names them)
- the 3 regenerated goldens, the line-number goldens, the lint fixes
- `#[ignore]` count held at 13

**Exactly one thing was wrong: the encode/decode composition**, and it was wrong because *my stone told
it the wrong functions*. Do not re-litigate the rest.

## What is wrong, measured

```
(:wat::holon::Thermometer 50.0 0.0 100.0)  →  "(:wat.holon/Thermometer 50.0 0.0 100.0)"
```

That is a **wat source form on the wire**, and the builder has ruled it illegal. Decoding it
structurally yields a `Bundle`, because `runtime.rs:19711` is unconditional:

```rust
WatAST::List(items, _) => HolonAST::bundle(items.iter().map(watast_to_holon).collect()),
```

A round-tripped Thermometer then answers `Bundle/children` — which *raises* on a real Thermometer.
That is the crash at `wat-tests/service-cache-hologram.wat:121`, reproduced on three independent runs.

## The corrected design — the builder's model

> *"we do not transmit bind, bundle, atom, etc etc.... **they are data**"* …
> *"`#wat.holon {:key1 "val1"}` — this is both edn and holon"* …
> *"the only things that need tags are stuff like thermometers as they convey **constructor details
> about the data**"*

```
encode   from_holon_item(h) → Ok(data)  ⇒  #wat.holon <data>
         HolonAST::Thermometer          ⇒  #wat.holon/Thermometer {:value v :min lo :max hi}
         HolonAST::SlotMarker           ⇒  #wat.holon/SlotMarker  {:min lo :max hi}
         anything else                  ⇒  RAISE

decode   #wat.holon <data>              ⇒  to_holon (derive the vector)
         #wat.holon/Thermometer {…}     ⇒  HolonAST::Thermometer
         #wat.holon/SlotMarker  {…}     ⇒  HolonAST::SlotMarker
```

`Bind`/`Bundle`/`Atom`/`Permute`/`Blend` appear on a wire in **no form** — not tags, not call forms.

## The mechanism exists; adopt it, do not write it

- **`from_holon_item`** — `runtime.rs:16641`, `pub(crate)`. The holon→**data** inverse. Handles
  `Map`/`Set`/`Vector`/`List`/`Tuple`, keywords, symbols, `Char`, scalars. **Errors** on anything that
  is not data — and its message already enumerates the set:
  `"unclassified HolonAST (bare Bundle, non-classifier Bind, Permute, Thermometer, Blend, or other composite)"`
- **the holon-side lift** for decode — the same one `:wat::holon::literal` (`#holon <form>`, arc 294.b)
  uses via `to_holon_inner`. Find it and reuse it; do not hand-roll a second one.

`holon_to_watast` / `watast_to_holon` **stop being the encode/decode path.** Leave them alone
otherwise — they have 8 legitimate `runtime.rs` callers. If the `pub(crate)` widening the previous
rider added is now unused, revert just that.

## The RAISE arm is the wall — build it, do not soften it

A holon that is neither data nor a directive **must raise on encode.** Do not fall back to a Bundle,
a nil, or a best-effort rendering. `[[feedback_a_negative_control_that_can_be_kept_must_be_kept]]` —
and unrepresentable beats guarded.

## Bare `#holon` is not available and this is not a style choice

`crates/wat-edn/src/value.rs:382`: *"per the EDN spec, user tags MUST be namespaced — there is no
`Tag::new(name)` because a no-namespace tag is invalid input."* `Tag::namespace` is required at the
type level. Census: `Tag::new` 0 uses, `Tag::ns` 33. Use `Tag::ns("wat.holon", …)`.

## The gate

| # | assertion |
|---|---|
| 1 | `grep -rn 'wat-edn\.holon' src/ crates/ tests/ wat/ wat-scripts/ wat-tests/` → **0** |
| 2 | **no wat source form on any wire** — encoding any HolonAST never produces a leading `(` |
| 3 | `(:wat::holon::Thermometer 50.0 0.0 100.0)` encodes to `#wat.holon/Thermometer {:value 50.0 :min 0.0 :max 100.0}` and decodes back to a **real Thermometer** (not a Bundle) |
| 4 | a data holon round-trips: `{:key1 "val1"}` → `#wat.holon {:key1 "val1"}` → equal holon |
| 5 | a non-data, non-directive holon **RAISES** on encode |
| 6 | ⛔ **`wat-tests/service-cache-hologram.wat` GREEN on BOTH tiers** — `on_thread` *and* `on_process`. This is the load-bearing row; it is the only test that proves similarity survives encode→wire→decode |
| 7 | floor GREEN via `scripts/floor.sh` — the **Summary line** |
| 8 | `cargo clippy --release --all-targets` → **0** |
| 9 | `grep -rnE '^[[:space:]]*#\[ignore' tests/ src/ crates/ benches/ --include=*.rs \| wc -l` → **13** |
| 10 | the probe `tests/value/probe_arc294_holon_bare_leaf_read.rs` updated to the corrected design, GREEN, **zero `#[ignore]`** |

Row 6 is the one that failed last time. Row 3 is why.

## What you report

- the `git diff` of `src/edn_shim.rs` encode/decode arms
- the measured wire string for a Thermometer, a data holon, and a leaf — **verbatim**
- the round-trip result for each
- what the RAISE arm does on a bare `Bundle` — the error, verbatim
- `service-cache-hologram` on both tiers, Summary lines
- floor Summary verbatim; clippy count; `#[ignore]` count
- honest deltas

## STOP triggers — ship nothing on these; report and stop.

- **STOP-1 — `from_holon_item` cannot reach a shape the corpus actually ships.** Name the shape and
  the site. Do not widen it speculatively.
- **STOP-2 — the decode lift for `#wat.holon <data>` does not exist or is not reusable.** Name what
  you found. Do not hand-roll a second holon builder.
- **STOP-3 — row 6 still fails after the corrected design.** Then the model has a gap the builder must
  rule on. Capture the far-side crash verbatim and stop; do not chase it with a fallback.
- **STOP-4 — the `#[ignore]` count moves off 13.** A finding about this brief, not a step.
- **STOP-5 — a red you did not intend. Do NOT re-run.** `scripts/floor.sh` kept the untruncated log at
  `.floor/latest/`. Copy the failing test's **entire** stdout+stderr **verbatim**, name the exact arm,
  report. There is no such thing as a known flake.

## Out of scope — do not touch

**Thermometer as a record / kwargs.** The builder ruled: land this first, that as its own stone.
`(:wat::holon::Thermometer 50.0 0.0 100.0)` stays positional at all 38 call sites. Only its **wire
form** changes here.
