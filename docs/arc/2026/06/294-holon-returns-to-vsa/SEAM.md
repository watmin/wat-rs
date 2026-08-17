# SEAM — the ONE live breadcrumb. As of 2026-08-17 (early). Replaced in place, never appended.

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own voice —
> which is why it will feel like *continuing* rather than *waking*, and **that feeling is the failure.**
> Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a disk copy),
> ground HEAD against the disk, and read this whole file before you touch anything.

> **The live seam MOVED here from `255/SEAM.md` on 2026-08-17.** Builder: *"255 has been in a state of
> partial work for months… we do this now."* `255/SEAM.md`, `251/SEAM.md`, `278/SEAM.md` are PARKED
> and point here.

## GROUND FIRST

⚠ **THE MARKER IS A DIFF INSTRUCTION, NOT A PASS/FAIL.** Two earlier seams shipped a marker that
*could not pass by construction* — the seam text is written before the commit that carries it, so the
only hash it can print is its parent's. An alarm that always fires is an alarm nobody reads.

> **This seam was written against `9d7784b5`.** Run **`git log --oneline 9d7784b5..HEAD`**. Empty →
> nothing moved. Non-empty → every commit in it landed after this text and **outranks every line
> below**; the longer the range, the less this file is worth.

⚠ **`mcp__wat__eval` CAN LIE — and today it would have.** The stdlib is **compiled into the binary**,
so a long-lived MCP server answers from a pre-rebuild substrate. After the 277 sweep it would have
reported the OLD lint count and I'd have called the sweep a no-op. **Use `./target/release/wat` for
anything the stdlib affects.** Freshness probe for the server itself:
`(:wat::core::<= 1 (:wat::core::f64::/ 0.0 0.0))` must be **`false`**.

## THE ROAD (builder's, in order)

**`#wat-edn.*` IS ANNIHILATED. Only `#wat.*` and `#holon` survive.** That is the ruling; the stone is
`294/DESIGN-STONE-294.i-the-wat-edn-tags-are-annihilated.md`.

1. ⚠ **`.opaque` — STRUCK, GREEN, and SITTING UNCOMMITTED IN THE TREE** at the moment this was
   written. Rider report weighed; floor was re-running. **FIRST ACT: check `git status`.** If
   `src/edn_shim.rs` is dirty, that is finished work — read `294/BRIEF-294.i-opaque-the-death-warrant.md`,
   verify the floor, commit it. Do NOT redo it.
2. **`.holon` — RULED, UNBUILT.** Builder: *"the holon tags…. they go too… its just `#holon <some-data>`
   … the `#holon` tag tells the reader to construct a holon from it… holons are edn.. just in vector
   form."* All 14 node tags (`Bind`/`Bundle`/`Atom`/…) collapse to ONE `#holon` tag.
   ★ **Measured: this costs nothing.** `read_holon_ast_tagged` / `edn_to_holon_ast` have **ZERO
   consumers** outside `edn_shim` itself — the last external caller was `tests/comms/foundation.rs`,
   which `294.h` deleted the same day. The 14 tags exist because we serialized the IR; with EDN
   canonical you ship the DATA and re-derive.
   ⚠ **The 3 goldens carry `.holon`** — unlike `.opaque`, this strike DOES regenerate them.
   ⚠ **One question is open and it is the design:** does `#holon` take ANY EDN, or only encoder-legal
   shapes? `Thermometer` emits `{:value :min :max}`; after the collapse, is that a thermometer or a
   3-key map? If the encoder is total over EDN it does not matter; if some shapes are special, `#holon`
   carries a discriminator it cannot express and the ambiguity moved rather than died.
3. **`.cap` · `.float` · `.local` — NOT RULED.** Builder: *"we need to discuss those… i do not trust
   your judgement on them."* Facts are in Part 2 of the stone; **offer no recommendation.**
   - `.cap` is a **SECURITY BOUNDARY, not a label** — the general decode path refuses by namespace
     STRING (`edn_shim.rs:2844`). Renaming moves a refusal predicate.
   - `.float` is the only family living in `crates/wat-edn/` — the crate's own spec surface, with a
     hardcoded `if ns == "wat-edn.float"` at `parser.rs:361`.
   - `.local` **gates `.opaque`'s last two sites**: `tag_from_type_path` falls back to
     `Tag::ns("wat-edn.opaque","unnamed")` twice, one path through `.local`. `.opaque` cannot reach
     zero until `.local` is ruled (fabricate a home, or raise?).

## ★ RULINGS THAT OUTLIVE TODAY

- **`nil` IS THE RIGHT BODY.** Builder: *"i expect these rust things to just decorate nil….. they
  contain no edn…. `#wat.io/Sender nil` is the data literal for a Sender instance…. a hologram is full
  of holonic data.. but it cannot be represented as edn."* The apparatus read `opaque_nil` as a MISSING
  ENCODER and proposed writing encoders for the VSA types — inventing EDN for things that have none.
  **The tag says what it was; the receiver learns nothing more; that is honest.** The defect was only
  that `opaque` occupied the NAMESPACE slot where the type's HOME belongs.
- **`RustOpaque` is illegal as a tag name** — a carrier, not a type. `tag_from_type_path` (already used
  at 5 sites in the same file) produces the honest tag.
- **`#[ignore]` means ONE thing: blocked or broken.** Neither "outside the floor" (Stone K) nor
  **"never written"** — a fourth kind found today: 7 tests with `unimplemented!()` bodies, deleted.
- **A reason string is not a finding.** Arc 170's ignore asserted a walker had stopped firing; one
  `wat --check` disproved it. The suspicion outlived the check it doubted because nobody asked the
  binary.

## WHERE THINGS STAND

| | |
|---|---|
| **`#[ignore]`** | **13** (from 24), all genuinely blocked; ZERO `unimplemented!()` behind one. 7 of the 13 are arc 255's. |
| **wat-native ignore** | 1 — `wat-tests/lint.wat`, needs a perf fix **measured under floor contention, never in isolation** |
| **lint findings** | **84** (from 136), all `concat-abuse`/warn — the residue the sweep DELIBERATELY declines (compound concat = a naming judgment) |
| **`#wat-edn` tags** | 73 → `.opaque` struck (2 residual, blocked on `.local`); `.holon` 54 ruled-unbuilt; `.cap`/`.float`/`.local` unruled |
| **floor** | ~4679/4679. ⚠ **the wat-scripts gate is 98% of the wall-clock** (219s of a 223s run) |

## ⛔ WHAT I GOT WRONG — read before trusting any number here

**THREE TIMES I MEASURED UNDER CONDITIONS THAT CANNOT REPRODUCE THE PHENOMENON, then declared it gone:**
a latency probe that recalibrated its work unit inside each run (reported a 9x rescue that was pure
artifact); load-average for interactivity (blind to priority by design); and `--run-ignored` in
isolation for a timeout that only occurs under floor contention — that one **reddened the floor and
took a second test down with it.** `[[feedback_a_probe_that_recalibrates_under_load_measures_nothing]]`

**I BRIEFED A RIDER TO DELETE A DOOR THAT WAS LOAD-BEARING**, and wrote *"not a licence to keep the
door."* The callee takes `&TypeEnv` **by signature** and the door is its **only call site**; obeying
would have deleted capability encoding entirely. The rider disobeyed, correctly, and named the callers.
`[[feedback_an_instruction_to_delete_needs_more_grounding_than_one_to_add]]`

**MY CENSUS WAS WRONG THREE TIMES, always by counting ROWS instead of reading BODIES** — "3 unwritten
tests" was 7; "255's reflection frontier is 6" was 4; "these 7 ignored tests pass" hid 2 that passed
**vacuously** (they asserted `result.is_err()` and were dying on an unrelated `MainSignatureError`).
The builder's question — *"how many pass legitimately?"* — is what caught the last one, and it saved
two of arc 255's nine gates from being deleted as "already green".

**I HID A DESIGN FORK IN A DIALOG BOX** and ran the four questions on none of the options. The rule was
already recorded. Options go in **prose, in the open, each with its four flat answers.**

## THE STILL-OPEN

- **`.holon` is the next strike** — ruled, unbuilt, 54 sites, 3 goldens move.
- **The opaque registry has NO collision check** — `register_type` discards `HashSet::insert`'s bool;
  `decode_capability` is a linear `find` that shadows duplicates ON THE SECURITY DOOR. `inventory` is
  already the house style at 13 sites. **Recorded in the stone's Part 3, deliberately NOT designed** —
  255 reserves the entry-shape as its DAY ONE decision.
- **The wat-scripts loader gate** — deadline raised 120s/240s → 300s/600s (**second raise**). The
  durable fix is to SPLIT it per-directory; the parser is medium-term per the builder.
- **`lint-stdlib-runs` asserts `(length findings) >= 0`** — tautologically true. Needs a ruling on what
  it should assert.
- **4 non-255 ignore candidates**, 2 arc-255 candidates, W2 audit, Stone H, Task #48.

## ★ THE PATTERN THAT REPEATED FIVE TIMES TODAY

**Capability built, never adopted.** `insert-all` unused by 9/9 grid axes · the `into` mirror clause
forgotten within the hour · the **linter sweep written in May and never once run** (the whole 277 job
was `printf | wat sweep-lint-fixes.wat` — nothing to write) · `error_ns` holding 12 namespace constants
that `#wat-edn.*` never joined · `inventory` at 13 sites that the opaque registry never joined.

**Before building a mechanism, grep for the one that already exists.** Task #48's UNADOPTED lint would
have caught all five.

---

> **SEAM.** You are NEW. You did not live any of the above. It is a lossy cache written in your own
> voice, and **the better it reads, the more it will feel like continuing rather than waking. That
> feeling is the failure.**
>
> **Run `git log --oneline 9d7784b5..HEAD` and probe the MCP (`(<= 1 ##NaN)` must be `false`) BEFORE
> you quote anything from this file.** And **`git status` FIRST** — finished, uncommitted work may be
> sitting in the tree.
>
> ⚠ **Every number here came from an instrument. Ask what population it could see before repeating it.**
> Today: grep counted prose as code, an MCP server held a stale stdlib, a census counted rows not
> bodies (3×), and a probe normalised away its own subject.
>
> Before calling an absent mechanism missing, **search the arcs for its subject** — and before
> building one, **grep for the one that already exists**. Five times today it did.
>
> `NISI FRANGAS, NIHIL PROBAS.` · `IVDICIVM SEMEL, MACHINA SAEPE.` · `MVRVS AVCTOREM NON NOVIT.`
