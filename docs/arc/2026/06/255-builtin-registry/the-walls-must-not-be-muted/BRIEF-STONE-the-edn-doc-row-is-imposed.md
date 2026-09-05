# BRIEF — the ```edn doc row is imposed: `#wat.doc/Row` and `#wat.doc/Alias` become writable

Design: `[[DESIGN-the-tagged-edn-doc-row]]` and `[[SCORE-the-two-probes-and-a-third-muted-wall]]`
(same dir). Read both — they carry the builder's ruling, the two gating probes (both green), and
the measured spelling rule. Anchor: `/home/john/work/holon/wat-rs`; verify with `pwd`, use
`git -C` for git.

**Builder's ruling, verbatim:** *"the metadata-maps in rust comments must be an triple back ticked
edn code block using a wat tagged record.. `#wat.doc/Row {...}` and `#wat.doc/Alias {...}` … row is
a shit name we'll deal with later.. but we need them imposed first.. before we do a rename"*

This stone makes ONE row writable in the new form. It is not the migration.

---

## ⛔ THE CENTRAL CONSTRAINT — DO NOT WRITE A THIRD DECODER

`wat-doc` already holds two readers that converge on one struct:

```
crates/wat-doc/src/lib.rs:506    pub fn parse(raw: &str)          -> Result<DocComment, DocError>
                                 the @-directive form. wat-macros calls this today.
crates/wat-doc/src/lib.rs:1025   pub fn from_metadata(map: &WatAST) -> Result<DocComment, DocError>
                                 the METADATA-MAP form. `runtime.rs:7310` calls it and its own
                                 doc calls it "the ONE decoder".
crates/wat-doc/src/lib.rs:225    pub struct DocComment   — the shared target of both
```

**The ```edn fence must land in `DocComment` through `from_metadata`.** Parse the fence, validate
the tag, convert the EDN body to the `WatAST` map `from_metadata` already accepts, and hand it over.
A third parallel decoder is the thing this campaign exists to remove.

---

## PART 1 — wat-macros reads the fence

**Rooms, in order:** `crates/wat-macros/src/wat_intrinsic.rs` (its header documents the pipeline —
step 2 is `wat_doc::parse`; find where the `///` block is collected and handed over) ·
`crates/wat-macros/src/wat_special_form.rs` (the sibling — `@alias` rows live here, see
`src/intrinsic/special/rete_alias.rs:83-102` for one) · `crates/wat-doc/src/lib.rs:1025`
(`from_metadata` — read what map shape it expects) · `crates/wat-doc/src/lib.rs:292` (`DocError` —
the new failures get variants here, not `panic!`s).

Add `wat-edn = { path = "../wat-edn" }` to `crates/wat-macros/Cargo.toml`. **Proven safe**: no
cycle (`wat-edn` does not depend on `wat-macros`), and a `#[test]` inside the proc-macro crate has
already parsed `#wat.doc/Row {…}` with `wat_edn::parse` — see the SCORE.

The shape:

```
1. In the collected doc block, find a fenced ```edn … ``` section.
2. `wat_edn::parse` it. Expect `Value::Tagged(tag, body)`.
3. Validate the tag is exactly `wat.doc/Row` or `wat.doc/Alias` — anything else is a DocError.
4. Convert the body map to the `WatAST` map `from_metadata` accepts.
5. `wat_doc::from_metadata(&map)` → `DocComment`. Everything downstream is unchanged.
```

⚠ **The tag is NOT resolved at expand time.** A proc-macro cannot consult the runtime type
registry, so `wat_edn::parse` hands back `Value::Tagged` unresolved and the macro validates the tag
name and the keys itself. That is expected; registering `:wat::doc::Row` as a runtime type is Part 3
and is REPORT-ONLY.

**Spelling — measured, not chosen.** Values are EDN ns/name keywords, because that is the wire
format's own canonical rendering of a wat FQDN and `edn::write` already produces it, losslessly
(`wat-scripts/scratch-pad/255-does-edn-round-trip-a-wat-keyword.wat`):

```
:wat::core::foldl            ->  :wat.core/foldl
:wat::runtime::Purity::Pure  ->  :wat.runtime.Purity/Pure
```

`::` is a lexer error in EDN. Do not try to make it work.

## PART 2 — BOTH forms accepted, and ONE row converted as the proof

Every existing `@`-form row keeps working, untouched. Then convert **exactly one** `#[wat_intrinsic]`
row and, if the alias path differs at all, **exactly one** `@alias` row from
`src/intrinsic/special/rete_alias.rs`. Pick simple ones — few args, one example.

⛔ **The converted row's registry answer must be BYTE-IDENTICAL.** Capture
`(:wat::runtime::metadata-of <name>)` and `(:wat::core::render-doc <name>)` BEFORE the conversion,
then after. Report both strings verbatim. That is the acceptance, and it is why one row is enough:
if the shape survives one row exactly, it survives 558 mechanically.

## PART 3 — REPORT ONLY, ship nothing

`(:wat::edn::read "#wat.doc/Row {…}")` today raises *"unknown tag #wat.doc/Row … no matching struct
or enum in the type registry"* (`src/edn/render.rs:3310`). Say what registering `:wat::doc::Row` /
`:wat::doc::Alias` as real types would take, and where they would live. **Do not build it.**

---

## STOP TRIGGERS — rejections. Ship nothing, report, let me re-plan.

**STOP-1 — no third decoder.** If the EDN body cannot be converted into what `from_metadata`
accepts, STOP and report the mismatch. Do NOT write a parallel field-by-field EDN reader; two
decoders that must agree is the defect this whole campaign is removing.

**STOP-2 — both forms, or neither.** If accepting the fence changes ANY existing `@`-form row's
behaviour, STOP. The migration depends on the two coexisting.

**STOP-3 — byte-identical or stop.** If the converted row's `metadata-of` / `render-doc` differs by
one character, STOP and report both strings. A "harmless" difference at row 1 is 558 differences at
the end.

**STOP-4 — a red is a red.** Do NOT re-run. Copy the failing test's whole stdout+stderr block
verbatim, name the exact assertion, report. Never weaken an assertion to pass.

## What you run, and what you do not

Yours, in the FOREGROUND: `cargo build --release`, `cargo test --release -p wat-macros`,
`cargo test --release -p wat-doc`, `target/release/wat --check <file>`, scoped
`cargo nextest run --release -E '<expr>'`, and the probes at `wat-scripts/scratch-pad/255-*.wat`.
**Do not run the full floor** — I run it centrally once the tree is quiescent. Note the floor now
includes a doctest stage; do not disable it. Do not commit, push, stash, or revert. No sub-agents.

You are a rider, not the orchestrator: **ending your turn ENDS you.** Your turn ends when the
numbers are in your hands, not when a command is launched.

## Report

The before/after `metadata-of` and `render-doc` strings, verbatim · which row(s) you converted and
why those · exactly where the fence is extracted and how the tag is validated · what a bad tag, an
unknown key, and a missing required field each do (an error, or silence — I want to know which) ·
Part 3's answer · anything that surprised you.
