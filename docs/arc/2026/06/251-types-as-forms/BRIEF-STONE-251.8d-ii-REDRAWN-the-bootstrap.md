# BRIEF — STONE 251.8d-ii (REDRAWN): the bootstrap, against a tree that has moved

**Drawn 2026-09-21 against `main` @ `402ed7e73`** (floor 5957/5957, clippy 0, census `no STOP-8`).
⛔ **Supersedes `BRIEF-STONE-251.8d-ii-the-bootstrap.md`**, whose strike correctly **STOPPED**.

## ⭐ WHY IT IS WORTH RE-ATTEMPTING — the blocker that stopped it is GONE

8d-ii stopped on a single, correct finding:

> *"`#wat.type/MalformedDecl` — malformed `:wat::core::defenum` declaration: **name must be a
> keyword; got symbol**. `parse_declared_name` matches only `WatAST::Keyword`… That is a
> checker/registry change, not a spelling flip. **STOP.**"*

**Measured today** — `(wat.core/defenum u/Color wat.enum/Pure :Red [] :Blue [])` → **`--check` clean.**
⭐ **255.1 fixed it.** And **EIGHT stones have landed since that stop:**

`255.1` identity is the pair · `255.2` every one-spelling slot · `255.3` the registry decides the
join · `255.4` one member join · `255.5` the position flag · `255.6` rete's `:when` adopts the door ·
`255.7` the rete validator adopts the door · `251.8d-i-b` the arrow is gone for rete binds.

**Delta over that span: 104 → 49.** ⛔ **None of it has ever been tested against the stdlib.**

## ⛔ THE ORCHESTRATOR JUST BOTCHED THE MEASUREMENT — read this before repeating it

To decide whether to redraw, the orchestrator ran `--check` on 64 **converted stdlib files
standalone** and got **46 failing** — then discovered the **originals already fail 37/64**, and that
30 of the 46 are `DuplicateType`: **the file colliding with the copy already embedded in the binary.**

⛔ **The first 8d-ii brief says exactly this, in a section the orchestrator wrote:**

> *"Running `--check` on a stdlib file STANDALONE answers a different question and gives artifacts…
> **The ONLY proof is: convert → rebuild → floor.**"*

⭐ **The warning was violated one stone after it was written. Do not repeat it.** A standalone
`--check` of `wat/**` proves nothing here, in either direction.

## ⚠ WHAT IS DIFFERENT THIS TIME, AND IT CUTS BOTH WAYS

The stdlib contains **rete clauses**, so converting it exercises `8d-i-b`, `255.6` and `255.7` **at
once** — the broadest real test any of them has had. ⭐ **That is the value.**

⛔ **And the hazard: a red could come from any of four stones, not one.** **ATTRIBUTE EVERY RED to
the stone whose behaviour it exercises**, and say which. A bare "floor red" here is nearly useless.

## The work

1. **Convert all 64** `wat/**/*.wat` by the tool, on **copies** first (dry-run, `diff`), then apply.
   ⚠ **Derive the path list**: `git ls-files | grep -E '^wat/.*\.wat$'` → **64**. ⛔ The orchestrator's
   original `git ls-files 'wat/**/*.wat'` returns **31** — git's `**` does not match `wat/*.wat`, and
   **you caught that last time.** ⚠ **Check `.wat.bad` too** — the glob gap you found in 255.4.
2. **`cargo build --release`.** The binary now embeds the converted stdlib.
3. ⭐ **PROVE THE SELF-APPLICATION.** After the rebuild the embedded `fix.wat` **is** the converted
   one, and it is the tool 8d-iii depends on. Re-run the codemod over a copy of an unconverted corpus
   file: it must still convert, and a second pass must change **0 files**. ⛔ **If the converted
   codemod cannot convert, 8d-iii has no tool — STOP AND REPORT.**
4. **The floor is the load proof.**

## ⛔ RECOVERY — proven, and read it BEFORE step 2

Your own 8d-ii stop verified it: `git checkout -- wat/ && cargo build --release` → **exit 0, 22.5 s,
binary alive, codemod functional.** Nothing is pushed, so this is always available.

⚠ **Do NOT `git stash` the conversion** — a partial stash leaves a **mixed** stdlib, and the load
order (`core.wat` first, 64 entries deep in `src/load/stdlib.rs`) makes it fail somewhere unrelated
to the cause. **If the floor reds, bisect by reverting halves — BY POSITION IN THE LOAD ORDER, not
alphabetically.**

## The gate

- ⭐ **`scripts/floor.sh` green. That is the load proof, and there is no cheaper one.**
- **Test-count delta: predict from the diff; expected 0** — a spelling flip adds no tests. ⚠ **A
  non-zero delta means something other than spelling moved. Report it.**
- clippy `-D warnings --all-targets --workspace` **0**; census `no STOP-8`. Run crate clippy **and
  the lint suite** yourself — `one_param_spec`, `one_variant_separator`, and your own
  `rete_bind_generators`.
- **Idempotence:** second pass over converted `wat/` changes **0 files**.
- **The self-application proof** from step 3, stated explicitly.
- ⛔ **ONLY `wat/**` moves.** `git status` proves it — no `tests/`, no `wat-scripts/`, no `src/`.
- ⚠ **Re-run the 179-file delta afterwards** (baseline **49**, re-converted with the current codemod).
  ⭐ **Some of the 49 may resolve once the stdlib is flipped** — and if the number moves, that is
  information about the corpus, not just the stdlib.

## Doctrine

- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Capture whole the first time; never re-run first.
- ⛔ **`wat/*.wat` is `include_str!`'d** — an on-disk edit is invisible until `cargo build --release`.
  **Four stones have lost time to this.**
- ⛔ **R21 — by the tool, never by hand.** These codemods are the builder's **merge infrastructure**.
- ⛔ **A CONVERSION GATE IS NOT A LOADING GATE.** 8d-i passed 2145/2145 on conversion and the premise
  was still false. **Only the floor answers this stone.**
- ⛔ **REPORT WHAT A DOOR CANNOT EXPRESS.** Seven stones running — and in 255.6 you were **right
  against the brief.**
- ⛔ **If this brief contradicts the code or the runner, THEY WIN.** **Thirteen corrections across
  eleven stones**, including the botched measurement above, made *by the brief's own author, against
  the brief's own warning.* **Assume a fourteenth.**
- Work only in `/home/john/work/holon/wat-rs`. **Do not push.**

## Out of scope

- **8d-iii** — the remaining ~2,076 corpus files. After this stone the tree is **mixed dialect**
  (`wat/` converted, the rest not). ⭐ **That is fine: both spellings type-check.** Do not tidy.
- **The residue (49)** — `UnresolvedReference` 23 (incl. `not-a-special-form` ×3, a **deliberate
  negative-test name that stays**), `TypeMismatch` 11, `defsurface` 8, other 7. ⚠ **4 of the
  TypeMismatch arrived from behind the rete wall — they were always broken.**
- **`fn` param annotations** — 7,028 sites, correctly still `<-`; they move with 8d-iii.
