# 296 · DESIGN STONE J — a delivered program must be able to name its own source

> **STATUS: DRAWN, NOT BUILT.** Surfaced by Wave A of the recapture cascade — one of the 4 tests it
> refused to bless.

## THE DEFECT, TRACED END TO END

A wat child crashes. The parent is told:

```clojure
:message  "boom"
:location #wat.kernel/Location {:file "src/wat_edn_bridge.rs" :line 442 :col 38}
:frames   [#wat.kernel/Frame {:file "src/freeze.rs" :line 1441 :symbol ":user::main"}]
```

Neither field names the user's code. `wat_edn_bridge.rs:442` is
`WatAST::Keyword(path, crate::rust_caller_span!())` — *the Rust line that constructed the node.*

| step | what happens | span |
|---|---|---|
| 1 | source parsed with a real filename | **real** |
| 2 | AST → EDN forms for `spawn-program` | **dropped** |
| 3 | child decodes via `edn_to_watast_with` | `rust_caller_span!()` |
| 4 | child raises; Fault takes the node's span | the decoder's line |
| 5 | `location_to_edn` serialises `span.file/line/col` | faithfully wrong |

**Step 5 is correct code doing its job on corrupted input.** The serialiser is honest; the span was
already destroyed three steps earlier. The loss is not on the error's return path — it is at
**program delivery**.

## THIS IS NOT AN OVERSIGHT — IT IS A JUSTIFICATION THAT OUTGREW ITS BOUNDARY

`src/wat_edn_bridge.rs:409` says so in as many words:

> *"Span is not preserved — all reconstructed nodes carry `crate::rust_caller_span!()`. … type-check
> and resolution operate on the semantic structure."*

and at `:287`:

> *"`startup_from_forms` / `freeze` re-derives what it needs from the semantic structure, not the
> span."*

**Both statements are true.** Type-check does not need spans. Resolution does not need spans.

The false step is the one nobody wrote down: *therefore spans need not be preserved.* A delivered
program is not only resolved — it is **executed**, and execution produces diagnostics, and a
diagnostic without a source location is a diagnostic about our interpreter.

Same family as the rest of this arc: a claim that is true about the instrument's own concern,
generalised past its boundary
(`[[feedback_a_measurements_boundary_is_its_claims_boundary]]`).

## THE BLAST RADIUS IS EVERY CHILD, NOT ONE TEST

This is not a crash-path curiosity. **Every diagnostic from any program delivered as EDN forms names
`wat_edn_bridge.rs`** — every raise, every located error, every frame. A user debugging a spawned
process is sent into our decoder.

Likely introduced by arc 278's IPC migration (`045ef88b`, `a40c294e`), which moved delivery to
`spawn-program'(process)` with forms.

## ⛔ THE IGNORE MISDIAGNOSED IT — and that is why it survived

The test was parked as *"296-recapture-pending: golden asserts pre-stone-B rust-debug face."* The face
changes in that golden are real and stale. **The `:location` delta is not a face change** — it is a
span-loss regression wearing a face-change's clothes, and the ignore's reason string is what made it
invisible for the life of the cohort.

This is the campaign's law earning its keep: the finding lived in the **triage**, not in the green.

## THE REQUIREMENT — state it as a property, not a mechanism

> **A program that will be EXECUTED must be able to name its own source.**

Everything below is a candidate mechanism. The property is what must hold.

### The mechanisms, and what each costs

- **A — the forms carry spans.** `#wat.core/Span` already exists as a tagged record and the error
  faces use it, so EDN *can* carry one. Cost: every node on the wire grows; the forms encoding becomes
  noisy to read and to diff; it changes a wire format that other things now depend on.
- **B — deliver the source text; the child parses it.** Spans arrive natively because the child does
  a real parse. Cost: reverses an arc-278 decision, and the forms path exists for reasons (structured
  delivery, hygiene, no re-parse of untrusted text) that must be re-examined rather than assumed away.
- **C — carry the source IDENTITY, not per-node spans.** One origin record (file + the original text,
  or a base offset) travels with the delivery; the child attributes to it. Cheaper than A, keeps the
  structured path of B.

**I would build C**, on the four questions: it satisfies the property without inflating every node
(Simple), a diagnostic that names the child's actual file reads correctly to whoever spawned it
(Obvious, Good UX), and nothing claims a location it does not have (Honest). But **A vs B vs C is a
wire-format ruling and belongs to the builder**, and I have over-claimed on a wire question once
already in this arc.

### ⛔ The rung worth climbing, whichever mechanism wins

Do not merely make the span correct. **Make "an executable program with no source identity"
unrepresentable** — refused at delivery rather than accepted and lying later. A program that cannot
name its source cannot produce an honest diagnostic, and a delivery path that admits one has built
the situation that needs the patch.

## NEIGHBOURS — this is not a lone defect

- **#92** — *the wire decodes GENERIC-then-validate; it must decode EDN→WatAST→refine.* The generic
  decode is precisely why spans die: `edn_to_watast_with` cannot know this keyword came from line 3
  column 34, so it stamps where it stands. **J is #92's symptom, arriving from the other direction.**
- **#93** — *a child's `Reply::Failed` is destroyed in transit.* Same class: information that exists
  on one side of the process boundary and does not survive the crossing.

Whether J, #92 and #93 are one stone or three is the first thing to settle.

## STOP TRIGGERS

- **STOP-1 — a fix that makes the span *plausible* rather than *true*** (defaulting to the child's
  entry file, synthesising line 1). A believable wrong location is worse than an obviously absent one;
  that is the `field-N` lesson, one boundary out.
- **STOP-2 — the span survives delivery but not the RETURN trip.** The two directions are separate
  round trips; proving one says nothing about the other. Test both ends.
- **STOP-3 — the fix reaches only `spawn-program`.** Any other path that ships a program as forms has
  the same hole. Enumerate the delivery paths before declaring the class closed.
