# BRIEF — STONE 251.9: the catch-all that ate the declaration

> Read `DESIGN-STONE-251.9-the-catch-all-that-ate-the-declaration.md` first. The probe is committed
> and RED: `tests/resolve/probe_arc251_stone9_symbol_head_declaration.rs` (3 rows, `#[ignore]`d).

## THE WORK, IN ONE PARAGRAPH

A form whose head is a `WatAST::Symbol` is not recognised as a declaration. `is_declaration_form`
reads the head as `WatAST::Keyword(k, _) => k.as_str(), _ => return false`, so a symbol head takes
the catch-all, the form is classified `FormOutcome::Evaluated` instead of `::Declared`, it evaluates,
and the session grows by nothing — no error, no diagnostic. Give the head ONE door that reads either
spelling, route every head reader through it, and the silent path stops existing.

## READ IN ORDER — the rooms

```
src/declare/parse.rs:199   is_declaration_form — THE SUBJECT. `_ => return false` at :206 is the
                           defect verbatim. Read :191 is_declaration_head and the two constants
                           above it FIRST: DECLARATION_HEADS (6 entries) and
                           RUNTIME_DECLARATION_HEADS (8). Their shared doc records a SHIPPED bug
                           where ONE LIST ANSWERED TWO QUESTIONS — a top-level `let` classified
                           Declared, its value discarded, `wat --repl` printing nothing. Do not
                           collapse them; this stone adds a door, not a list.
src/declare/parse.rs       further head readers: 236 · 267 · 278 · 362 · 413 · 546 · 563 · 666 ·
                           689 · 888
src/declare/preregister.rs 77 · 123 · 278 · 415 · 488
src/declare/register.rs    274 · 285 · 290 · 531 · 778 · 786 · 1767
crates/wat-reader/src/identifier.rs   receiver() / method() / is_reference() — the accessors the
                           door reads. 251.8b made these fields, not derivations.
src/edn/render.rs:3520     ns_to_wat_path — the symbol→FQDN map. Read its HALT first
                           (`277/HALT-a-keyword-is-a-keyword-is-withdrawn.md`) so you know exactly
                           why it is safe HERE and nowhere else.
```

## SKETCH

```rust
/// The FQDN of a form's head, whatever spelling wrote it. `None` = this node names nothing.
pub(crate) fn head_fqdn(node: &WatAST) -> Option<Cow<'_, str>> {
    match node {
        WatAST::Keyword(k, _) => Some(Cow::Borrowed(k.as_str())),
        WatAST::Symbol(id, _) if id.is_reference() => {
            Some(Cow::Owned(crate::edn::render::ns_to_wat_path(id.receiver(), id.method())))
        }
        _ => None,
    }
}
```

Then each room becomes `let Some(head) = head_fqdn(&items[0]) else { return … };` and the existing
comparison is unchanged. **Delete the `_` arm at each site you convert** — that is the point of the
stone; a site that keeps its catch-all keeps its silent path.

## WHY THE MAP IS SAFE HERE — the one thing to hold while you work

`ns_to_wat_path` cannot disambiguate a `Type/member` name. It is safe at THIS door because **every
caller immediately tests set-membership or literal equality** (`DECLARATION_HEADS.contains(&head)`,
`head == ":wat::core::def"`). A mis-mapped head therefore yields *"not this form"* — the same answer
the catch-all gives today — never a WRONG form. The moment a caller uses the result for anything
else, that property is gone: see STOP-3.

## BLAST RADIUS

`src/declare/*.rs` only, plus wherever `head_fqdn` lands (put it beside `is_declaration_head`). No
new types. No behaviour change for a Keyword head anywhere — every existing keyword-headed program
must render and resolve byte-identically.

## THE POPULATION IS THE COMPILER'S, NOT THE ROOM MAP'S

The room map above is **hand-classified from a grep and is not to be trusted as a census** — this
campaign produced four census errors that were each a pattern rather than a site. Removing the `_`
arm makes the compiler emit the real list. **Any site the map missed is a finding to record in the
SCORE, not a surprise to absorb.**

## STOP TRIGGERS — rejection criteria, not permission slots

- **STOP-1 — a NAME reader.** `parse.rs` 70 · 240 · 244 · 372 · 554 · 674, `register.rs` 584 · 1810,
  `function/parse.rs` 731 · 1006 · 1053 · 1341 · 1356 read a declared NAME, not a head. They are
  affirmatively out of scope (the DESIGN says why: a symbol NAME's FQDN is undecidable against
  today's registry keying, and names already fail LOUDLY). If you find yourself widening one to make
  something pass, STOP — the thing that needs it is out of scope.
- **STOP-2 — a MARKER keyword.** `:guard` / `:ensure` (`function/parse.rs:417·426`) and metadata-map
  KEYS (`parse.rs:322`) are keywords forever. The rule is: *a keyword that NAMES something becomes a
  symbol; a keyword that IS a marker stays one.* Widening one of these is a defect, not progress.
- **STOP-3 — `head_fqdn`'s result used for anything but set-membership or literal equality.** That is
  the entire safety argument. If a room needs the head for a lookup, a registration key, or a
  rendered string, STOP and report it — that room is a different stone.
- **STOP-4 — a TYPE reader.** `parse_type_keyword` at `parse.rs` 586 · 718 · 875 and
  `function/parse.rs:511` belong to 251.2 / 251.3, both STRIKE-READY with their own probes.
- **STOP-5 — the floor goes red on a KEYWORD-headed program.** This stone adds an accepted spelling;
  it removes none. A red on legacy input means the door changed behaviour it should not have.

## PRIOR ART TO COPY FOR SHAPE

`251.8a` (`BRIEF-STONE-251.8a-one-door.md`) — the same move one layer down: four `contains('/')`
classifiers replaced by one door (`namespace()` / `is_reference()`). Copy its structure.
