# BRIEF — STONE 1a-α: the sketch adopts the registry's DECLARED grammar

`signature_of_defn` gains ONE match arm, placed FIRST: a `Binding::Registered` row whose
`entry.syntax` is non-empty renders that string through the substrate's own reader. This adopts the
precedence `render-doc` has always used, so reflection's two renderers stop disagreeing — and it
retires a two-month-dead ASCRIPTION SLOT the sketch still teaches. ⚠ `match` itself is fully
supported and always was — what was retired is `match` ASSERTING A RETURN TYPE. Only `fn` declares
types.
You also build the gate that makes a malformed `@syntax` a red floor at authoring time.

DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-1a-alpha-the-sketch-adopts-the-declared-grammar.md`.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. Run every command you do run in
the FOREGROUND and block on it. The orchestrator builds, floors, clippies and recaptures goldens
centrally — you do not run `cargo build`/`test`/`nextest`/`clippy` or `scripts/floor.sh`. You may
run the pre-existing `./target/release/wat` and `--check` for a fast read. **You may not spawn
sub-agents.** Work only in `/home/john/work/holon/wat-rs`; verify with `pwd` first. Do not commit,
push, stash, revert, or `git checkout --` anything. Tree clean, floor green at 5119.

## Read in order

1. **The DESIGN above** — the contract decision is pinned there and it is narrow.
2. **`src/intrinsic/reflect.rs:456-470`** — `render-doc`'s precedence, which you are mirroring:
   `@syntax` verbatim when non-empty, else derive from `@arg`. **This is the shape to copy.**
3. **`src/reflect/verbs.rs:160-212`** — `signature_of_defn`'s arm chain, where you are working.
   Read all three existing `Registered` arms before adding a fourth; their ORDER is the logic.
4. **`src/intrinsic/special/binding.rs:28` · `fn_form.rs:45` · `match_form.rs:38`** — the three
   `@syntax` declarations that will now be rendered. Read the exact strings.
5. **`crates/wat-reader/src/parser.rs:238`** — `parse_one_with_file(src, file) -> Result<WatAST, ParseError>`.
   Already a dependency of `src/reflect/`.
6. **`wat-scripts/scratch-pad/255-can-the-reader-parse-a-syntax-grammar.wat`** — the committed probe
   proving all three strings read clean. You are implementing what it already demonstrated.
7. **`src/intrinsic/mod.rs:2389-2412`** (`every_special_form_carries_check_and_eval_impls`) — the
   ratchet shape to copy for your gate: walk `registry().all_entries()`, collect offenders by name,
   one assert with a message that NAMES them.

## The work

### 1 — the new arm in `src/reflect/verbs.rs`

Placed FIRST among the `Registered` arms — before the `!entry.args.is_empty()` arm:

```rust
Some(Binding::Registered { entry, .. }) if !entry.syntax.is_empty() => {
    let ast = wat_reader::parser::parse_one_with_file(entry.syntax, /* a source label */)
        .expect(/* the gate below guarantees this */);
    Ok(Value::Option(Arc::new(Some(Value::wat__WatAST(Arc::new(ast))))))
}
```

Choose the source label and the failure wording yourself; state both in your report. The `expect`
is load-bearing *because* the gate makes it unreachable — say so at the site, in one line, the way
the neighbouring arms carry their reasoning.

**Then correct the stale comment at `src/reflect/verbs.rs:170-181.`** It currently says the fossil
is *"its own stone"* and offers two candidate vehicles. This IS that stone, and the vehicle is
chosen. Rewrite it to record what now stands: `@syntax` first (a grammar), then `@arg` (which carries
a type and is therefore the wrong vehicle for a syntactic slot), then the `special_forms.rs`
deferral for rows not yet registered.

### 2 — the gate, in `src/intrinsic/mod.rs`'s `mod tests`

Walk `registry().all_entries()`; for every entry with a non-empty `syntax`, assert
`parse_one_with_file` returns `Ok`. Collect failures by NAME with the parse error, sort, one assert.

Then **sabotage it and report the result**: temporarily corrupt one `@syntax` to something
unparseable, confirm you can state exactly what the gate would say, and restore the file byte-for-byte.
⚠ You cannot run the test — so report the sabotage as *"predicted red, unverified"*. Naming it
honestly is the deliverable; claiming a run you did not make is not.

### 3 — predict the three goldens

The orchestrator recaptures `tests/wat_lang/wat_arc144_special_forms__{let,fn,match}.edn` centrally
with `UPDATE_EDN=1`. **Write your prediction of each file's new content into your report, exactly.**
Your prediction versus the capture is this stone's calibration row.

## Blast radius

`src/reflect/verbs.rs` (one arm + one comment rewrite) · `src/intrinsic/mod.rs` (one test). Nothing
else. No `.wat` corpus change. No registrations. No dispatch change. No checker change.

## STOP triggers — each REJECTS; ship nothing further on that point and report

**⛔ STOP-1 — `if` and `quasiquote` MUST NOT change.** `:wat::core::if` carries `@arg` ×3 and NO
`@syntax`, so the second arm must still answer for it. `:wat::core::quasiquote` is not registered at
all, so the `special_forms.rs` deferral must still answer for it. If your arm would capture either,
its guard is wrong — STOP. **These two are the load-bearing acceptance rows**: they are what
distinguishes "placed correctly" from "swallowed everything."

**⛔ STOP-2 — render `entry.syntax` VERBATIM. Do not splice the FQDN head in.** `@syntax` names the
form with its short head (`let`, `match`, `fn`); the displaced sketch used `:wat.core/let`. The
short head is CORRECT here — `render-doc` already ships it, and the two renderers agreeing is the
whole point. Re-authoring the string in the renderer mints a third rendering of a question the row
already answers.

**⛔ STOP-3 — if any of the three declared `@syntax` strings does not parse, STOP.** Do not add a
fallback arm, do not soften the gate, do not "handle" it. A grammar the reader refuses is a defect
in the declaration, and surfacing it is the finding.

**⛔ STOP-4 — `src/special_forms.rs` is not yours.** It still answers for the 23 unregistered rows.
Do not delete it, do not edit its rows, do not touch the dead `-> <T>` slot at line 171 — it dies when
the deferral arm stops being reachable, which is Phase 4a, not this stone.

**⛔ STOP-5 — do not author `@arg` for `let`/`fn`/`match`.** Their slots are syntactic positions with
no type; `@arg` carries a type (`and_form.rs`'s own shape is `@arg exprs… :wat::core::bool …`).
Filling those slots with type claims mints a lie, and refusing to is why this stone exists.

**STOP-6 — verbatim otherwise.** No signature tidying, no reordering of the untouched arms, no
opportunistic cleanup in either file.

## Report

The new arm verbatim (it is the stone's centre) · the rewritten comment verbatim · the gate verbatim
· the sabotage you performed and what you predict it would say · **your exact prediction of all three
golden files** · confirmation `if` and `quasiquote` cannot reach your arm, with the reason · the
source label and failure wording you chose and why · and what surprised you.

## Prior comparable

`BRIEF-STONE-3a-i-the-registry-at-the-front-of-lookup-form.md` — same file, same fn, same arm chain,
one stone ago. Copy its report shape.
