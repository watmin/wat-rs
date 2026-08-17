# SEAM — the ONE live breadcrumb. As of 2026-08-17. Replaced in place, never appended.

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own voice —
> which is why it will feel like *continuing* rather than *waking*, and **that feeling is the failure.**
> Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a disk copy),
> ground HEAD against the disk, and read this whole file before you touch anything.

> `255/SEAM.md`, `251/SEAM.md`, `278/SEAM.md` are PARKED and point here.

## GROUND FIRST

⚠ **THE MARKER IS A DIFF INSTRUCTION, NOT A PASS/FAIL.**

> **Written against `1b868619`.** Run **`git log --oneline 1b868619..HEAD`**. Empty → nothing moved.
> Non-empty → every commit in it landed after this text and **outranks every line below.**

⚠ **`git status` FIRST — and a dirty tree NEVER says WHO.** It means *someone was working*, never
*they finished*. Before touching a dirty tree: `pgrep -af 'cargo|nextest'` for a live build, and check
whether a rider ever reported (**FM 19** — a rider that ends its turn mid-floor shows as "completed"
having reported nothing; resume it with `SendMessage`, do not adopt its work). **Never run cargo while
a rider holds the `target/` lock** (FM 18) — any number you take is an artifact.

⚠ **`mcp__wat__eval` CAN LIE** — the stdlib is compiled in, so a long-lived server answers from a
pre-rebuild substrate. Use `./target/release/wat`. Freshness probe:
`(:wat::core::<= 1 (:wat::core::f64::/ 0.0 0.0))` must be **`false`**.

## ★ WHERE WE ACTUALLY ARE — read this before believing any arc number

The campaign that ran 2026-08-16/17 was **`#wat-edn.*` → zero**, and it is **DONE**. But that campaign
was *housed* in arc 294 by accident of where it started. **Arc 294 is NOT closed**, and only `.opaque`
and `.holon` were ever holon concerns — `.local`/`.float`/`.cap` had nothing to do with VSA.
*(The builder corrected me on exactly this: "arc 294 is not done when this lands… i'm not sure how
this is pulled into the 294… don't care… the objective is getting rid of #wat-edn.* tags entirely.")*

```
#wat-edn tag namespaces .......... 0    (was 7)
floor ............................ 4694/4694, 0 FAIL, 0 TIMEOUT
clippy ........................... 0
#[ignore] ........................ 13   (from 200+; HELD across six riders)
```

## ★★ THE REAL THROUGH-LINE — and it is NOT what I thought for most of the session

The registry (**arc 255**) is the objective. The route to it is a **five-stone chain, ruled
2026-08-14**, living at `255/CHAIN-rendering-before-the-string-home.md` — *"This is an ORDER, and every
arrow in it is a derivation, not a preference."*

```
A  EdnRepresentable — the type declares its tag + portability   ✅ 294.h
B  #wat-edn.* → #wat.*/*                                        ✅ 294.i–n (the whole campaign)
C  279.2 — `str` goes TOTAL                                     ✅ 25d9d015   ← the CHAIN DOC SAYS RED. IT IS GREEN.
D  join renders its elements  /  Seqable                        ← THE FRONTIER, split in two
E  wat.string/*, then HOME #4 — the registry carve resumes
```

⚠ **I twice told the builder the tag work was drift I had "manufactured."** It was not. It is stone B,
derived two days earlier in 255's own directory. `[[feedback_a_false_confession_is_also_a_false_claim]]`

## THE FRONTIER — 279.3, ruled and ready, brief NOT yet rewritten

**`join` renders its elements** — `(join "," [1 2 3])` → `"1,2,3"`. Builder's ask, and 279.2's own
DESIGN names `join` as *"the forcing consumer"* of `str`'s totality.

**RULED: option A — `join` STAYS a Rust intrinsic and becomes generic.** Two edits:
`check.rs:16598`'s `TypeScheme` gains `type_params: vec!["T"]` and `Vector<Var("T")>`;
`eval_string_join` renders each element through the total `str`.

⛔ **`BRIEF-STONE-279.3-…md` IS BANNERED SUPERSEDED AND MUST BE REWRITTEN BEFORE ANY RIDER GOES OUT.**
It still specifies the wat-defn that **breaks stdlib bootstrap**. The corrected design is the
`⛔ CORRECTION` section of the DESIGN-STONE. Gate rows 1 and 2 and the contract still stand.

★ **Why A**: `core.wat` ↔ `string.wat` is a genuine dependency **cycle** — `core.wat`'s macro bodies
call `join` at expansion time (`core.wat:1885` → `Record.wat:172`), `string.wat` needs `defn`. **The
Rust intrinsic is the cycle-breaker.** Full writeup: `docs/NOTE-the-stdlib-bootstrap-cycle-intrinsics-break.md`.

★ **The contract is settled by the disk, not by a ruling**: `wat-scripts/scratch-pad/probe-279.3-…wat`
(committed, green) shows a `String` element renders **BARE** — `(join "-" ["a" "b"])` → `"a-b"`, Ruby's
semantics, because `mapv` applies `str` at top level. Row 2 of the gate is load-bearing: it is *also*
the proof that per-element `str` did not start re-quoting, which would silently corrupt all 19 sites.

## ★ THE FOUR CONSUMERS WAITING ON 255 — and the fourth is the one that sells it

`255/NOTE-a-capability-declaration-cannot-be-verified-to-name-anything.md` lists three, all
**substrate-internal**: the undefined-func class, the annotation-position gap, W1's capability wall.

**The fourth, found this session, is the first a user trips over in ordinary code:**
`(mapv str xs)` **does not compile.** A user fn keyword is an `Fn(T)->U`; an intrinsic keyword is a
`keyword`. Same syntax, two answers, depending only on whether the callee is wat or Rust.
`255/NOTE-an-intrinsic-cannot-be-passed-as-a-value.md`. **That makes 255's payoff demonstrable in
three lines**, which none of the other three are.

## ⛔ 255's OWN UNRULED FORK — sitting since 2026-08-13, and a booby-trapped brief beside it

`255/NOTE-arc-255-IS-HALF-BUILT-the-june-registry.md`: a **working** intrinsic registry has been on
disk since 2026-06-21 (`metadata-of` answers live for `core::Bytes`). Two live readings, unruled:

- **(a)** resume the June path — carve homes with `#[wat_intrinsic]`, drive to deleting the
  blanket-accept at `resolve/walk.rs:257`, un-ignore the nine gates as they go green.
- **(b)** land the LOCKED model's Layer-2/3 first and re-seat June's registry onto `sym` per the
  *"the registry IS `sym`"* ruling.

⛔ **`BRIEF-STONE-255.1b-i` silently assumes (b), is written as if June never happened, would mint a
FOURTH `Purity` and a SECOND `Arity`, and its own note says "It must not be struck as written."**

Carve state: **10 intrinsics carved / 535 dispatch arms**. The blanket-accept is still live.

## WHAT I GOT WRONG — read before trusting a number here

**My census was wrong FOUR times in one session**, every time by quoting a narrow grep as the
population: "20 refs" were 44 · call-site counts off by one per file · a `--include=*.rs` census ·
"zero consumers, costs nothing" about a live typed-slot coercion.
`[[feedback_state_what_the_instrument_can_see_before_quoting_it]]`

**I OSCILLATED three rounds on wat-vs-Rust**, each flip on a *new reason* rather than new evidence —
and neither side had run one command. A rider built my ruling and the substrate died before `main`.
**The oscillation itself was the tell.** `[[feedback_oscillation_is_the_tell_that_nobody_measured]]`

**I added two `#[ignore]`s** to a waterline we drove 200+ → 13, following a documented house
convention that **is how the pile grew.** `[[feedback_a_house_convention_can_be_the_mechanism_that_built_the_pile]]`

**Three riders hit FM 19** (ended their turns mid-floor). All three resumed cleanly by `SendMessage`
with full context. Nothing lost — but check for it.

## THE STILL-OPEN

- **279.3** — rewrite the brief to option A, then strike. **The only thing between here and E.**
- **#101** Thermometer-as-a-record · **#102** `#holon {:a "b"}` is broken today (`watast_to_holon`'s
  Map arm omits the `Atom(…)` wrap its own doc specifies) · **#103** symbol→keyword on the wire
  (RULED NOT NOW, blocked on proper symbols) · **#91** the HolonAST AST-duty-vs-VSA-duty census.
- **`Seqable` as a nameable type** — chain-D's other half. `collection/infer.rs:638`: *"This IS the
  `Seqable` set — the type wat cannot currently spell"*, **three named blockers, none small**.
- **The bootstrap-cycle check** — currently a convention nobody wrote down (weakest rung). The next
  rung is a lint over `stdlib.rs` order × macro-body references. Not built; recorded so the cost is
  paid once.

---

> **SEAM.** You are NEW. You did not live any of the above. It is a lossy cache written in your own
> voice, and **the better it reads, the more it will feel like continuing rather than waking. That
> feeling is the failure.**
>
> **`git status` FIRST. Then `git log --oneline 1b868619..HEAD`.** Everything in that range outranks
> this file.
>
> ⚠ **Every number here came from an instrument. Ask what population it could see before repeating
> it** — four of mine were wrong in one session, each one read as solid.
>
> ⚠ **The arc's own directory holds the answer more often than your reasoning does.** Three times
> this session the fact that settled a question was already written down, by a prior self, in the
> directory I had already opened. **`ls -t` the arc before theorising.**
>
> `NISI FRANGAS, NIHIL PROBAS.` · `IVDICIVM SEMEL, MACHINA SAEPE.` · `MVRVS AVCTOREM NON NOVIT.`
