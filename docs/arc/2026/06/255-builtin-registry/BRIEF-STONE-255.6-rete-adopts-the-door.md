# BRIEF — STONE 255.6: rete's clause grammar adopts the door

**Drawn 2026-09-21 against `main` @ `e3c40e3da`** (floor 5946/5946, clippy 0, census `no STOP-8`).
Delta baseline **66**. ⛔ **`ReteCheckErrors` — 21 — is now the largest single blocker to 251.8d.**

## ⭐ THIS IS 255.2's RULE, APPLIED TO A GRAMMAR 255.2 NEVER REACHED

255.2's rule: **"any slot that accepts exactly one spelling must accept both."**
255.1's door: **`canonical_identity`** — one key from either spelling.

⛔ **`src/rete/` calls `canonical_identity` ZERO times.** The rule exists, the door exists, and rete
has neither. Everything below follows from that one fact.

## THE EVIDENCE — declined THREE times, correctly each time

The executor reported this in 255.2, 255.3 and again in 8d-i-b, and refused to force it every time:

> *"Rete's `:when` parser still wants a **keyword** head / a `::`-keyword type.
> `head-keyword?` converts those independently of the arrow. Not an annotation question."*

**The failures are on CORRECTLY CONVERTED clauses:**

```
(vrm/F (?k :- :k))                  ← symbol-headed fact pattern
(?fact :- weather/ColdAndWindy)     ← fact-bind type is a symbol, not :ns::Type
```

⭐ **Nothing is wrong with the conversion.** The corpus is right and the parser is behind it.

## THE THREE SLOTS — and 8d-i-b already routed one

| slot | today | state |
|---|---|---|
| the binder **arrow** | `crate::types::is_binder_marker(&items[1])` | ✅ **ROUTED by 8d-i-b** |
| the clause **HEAD** (`clause.rs:404`) | `WatAST::Keyword(head_kw, _) => match head_kw.as_str()` — a **Symbol head falls through to `Unrecognized`** | ⛔ |
| the fact-bind **TYPE** (`clause.rs:~366`) | `keyword_payload(&items[2])` then `kw.contains("::")` — a **Symbol type is not a keyword payload** | ⛔ |

⭐ **One of three is already done, by the same executor, one stone ago.** That is the shape to
repeat — and it is why this stone should be small.

⚠ **`expr_is_provably_boolean` (`clause.rs:282`) has the same gate** —
`let Some(WatAST::Keyword(head, _)) = items.first() else { return false }`. ⛔ **Do not assume it is
the same fix.** It answers *"is this provably boolean"*, and a wrong `false` there is a **silent**
wrong answer, not a diagnostic. **Measure whether converted clauses reach it before touching it.**

## The work

1. **Canonicalize the clause head before dispatch.** Symbol **or** Keyword → one key, through
   **`canonical_identity`**. ⛔ **Not a second `match` arm for symbols** — that is the two-door defect
   255.4's 32-red came from, and the one 255.2 refused on `one_param_spec`.
   ⚠ `classify_constraint_head` is already documented as *"Vocabulary via the ONE DOOR, never a
   literal list here"* — **the pattern is in this file already. Follow it.**
2. **Canonicalize the fact-bind type.** ⛔ The `kw.contains("::")` test is a **spelling** test: it
   asks *"does this look like a namespaced type?"* by counting colons. Under the flip
   `weather/ColdAndWindy` is the same type and has none. ⭐ **Ask the registry — 255.1 made
   `is_known_type` answerable and 255.3 taught the join.** Replace the character test.
3. **RE-RUN THE DELTA** (baseline **66**) **+ the classification table**, one named tree.
   ⚠ **`ReteCheckErrors` 21 should collapse. If it does not, the remaining cause is a finding** —
   name it and do not force it, as you have three times.

## The gate

- ⭐ **THE DELTA (baseline 66) + classification.** ⛔ **Re-convert the sample with the CURRENT
  codemod** — 8d-i-b moved the live corpus *and* the tool, so a stale converted tree measures a
  corpus that no longer exists (`[[feedback_diff_like_against_like]]`).
- ⛔ **NON-VACUITY BOTH WAYS:** a converted clause `(vrm/F (?k :- :k))` **parses**, and a genuinely
  malformed one **still fails**. ⭐ The existing negative control — `(?k <- :k)` → `MalformedClause`
  — **must stay red**; this stone must not re-admit the retired arrow by widening.
- **A rete clause in BOTH dialects type-checks identically** — the rust-scheme corpus still loads.
- `scripts/floor.sh` green; clippy `-D warnings --all-targets --workspace` 0; census `no STOP-8`.
  Run crate clippy **and the lint suite** — especially `one_param_spec` and the new
  `rete_bind_generators` — yourself.
- Predict the test delta; confirm with `cargo nextest list`.
- ⛔ **NOT ONE `.wat` CONVERTED.** This is a parser stone. Fixtures expected.

## Doctrine

- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Capture whole the first time.
- ⛔ **ONE DOOR.** This arc's most expensive red (255.4, 32 tests) came from adding a **second**
  consult where one existed. **`canonical_identity` is the door.**
- ⛔ **A GENERATOR IS NOT TEXT.** 8d-i-b shipped a red because a macro *synthesised* the old
  spelling. ⚠ **If rete clauses are built anywhere by a constructor, the codemod cannot see them** —
  `tests/lint/rete_bind_generators.rs` is the wall you built for exactly that. **Check whether clause
  HEADS are synthesised too.**
- ⛔ **REPORT WHAT A DOOR CANNOT EXPRESS.** Six stones running.
- ⛔ **If this brief contradicts the code or the runner, THEY WIN.** **Eleven corrections across nine
  stones**, the last being a design decision the orchestrator made without standing. **Assume a
  twelfth.**
- Work only in `/home/john/work/holon/wat-rs`. **Do not push.**

## Out of scope

- **8d-ii / 8d-iii** — they wait on the delta.
- **The registry residue** — declaration names, `mem-store::start`, `defsurface :messages`, and
  `:wat::core::not-a-special-form` ×3 which is a **deliberate negative-test name that stays**.
- **`fn` param annotations** — still `<-` at **7,028** sites, correctly; they move with 8d-iii.
