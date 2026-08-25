# DESIGN — STONE: a prose citation names a SYMBOL, not a LINE

> Named at the close of HOME #5 (`8ddccaaa3`) as the strike that follows it, and grounded here.
> The builder: *"you've named the next strike - we continue"*.

## THE SURFACE WORK

HOME #5 renamed five modules. **118 prose references across 61 files still name the old ones**, and
unlike the `string_ops` tail there is no keep-bucket: `string_ops.rs` was **deleted**, so prose
recording its removal was correct; these modules were **renamed**, so every reference names
something that does not exist under that name.

```
edn_shim             96      -> edn::render      /  src/edn/render.rs
runtime_error_edn    16      -> edn::error       /  src/edn/error.rs
wat_edn_bridge        4      -> edn::bridge      /  src/edn/bridge.rs
to_edn_derive_tests   2      -> edn::derive_tests
                    ────
                     118   across 61 files:  36 .rs · 20 .wat · 3 .md · 1 .sh · 1 .intueri
```

## ⛔ BUT THE REAL FINDING IS UNDERNEATH IT — 8 OF 12 LINE CITATIONS WERE ALREADY FALSE

Twelve of those references carry a **line number**. Every one was tested against what that line now
holds:

```
STILL TRUE
  edn_shim.rs:132        -> pub fn eval_edn_write_json_natural(          ✓ claim was about write-json-natural
  edn_shim.rs:144        -> /// `(:wat::edn::read s)` → `:T`             ✓
  runtime_error_edn.rs:64 -> impl ToEdn for RuntimeError                 ✓ claim: "HAND-WRITTEN at :64-83"
  wat_edn_bridge.rs:22   -> //! doc line                                 ✓

ALREADY FALSE
  edn_shim.rs:1773  ×5   -> `None => {`          claimed "refuses every Edn::Symbol" — it is at 2007
  edn_shim.rs:1899  ×2   -> `F: FnMut(Span) …`   claimed "EDN keyword reader"
  edn_shim.rs:964        -> `WatAST::Vector(…)`  claimed "read_edn stringifies the parser error here"
  edn_shim.rs:2651       -> `TypeExpr::Fn {…}`   claimed "the aggregate arm is guarded"
  edn_shim.rs:1008       -> `other => return Err` claimed a shared literal, "do NOT scatter it"
  edn_shim.rs:3490       -> `EdnReadErrorKind::Other(format!(` claimed a "stated discipline"
  edn_shim.rs:105        -> `args: &[WatAST],`   claimed a to_json_string consumption site
  edn_shim.rs:191        -> a RuntimeError ctor  claimed `eval_edn_read`
```

★ **The rename did not break these.** `git` recorded the moves at 92–99% similarity — the files'
contents did not move. **They were already wrong**, and the most-cited of them, `:1773`, is wrong by
**234 lines** and is repeated in five separate files. HOME #5 did not cause this; it made it legible.

★★ And the file says it about itself: `src/edn/render.rs:682`'s own comment reads *"the `Edn::Symbol`
arm ~:1440"* — a **third** wrong line number for the same arm, written inside the very file it
mis-cites.

## THE DERIVATION — and it is the whole stone

> **A line number is a claim fixed to a moment. A symbol name recomputes itself.**

`edn_shim.rs:1773` dies on the next insertion above it, silently, and *looks* more precise for having
a number. `edn::render::edn_to_value_caps` survives any edit that does not rename the function — and
when it does die, it dies **findably**: a reader greps the name and gets zero hits, instead of
getting a confident pointer to `None => {`.

That is R9's pin/derivation distinction applied to prose, and the measurement is 8 of 12.
`[[R9 DERIVAMVS NE MENTIAMVR]]`

**So the rename and the de-lining are one stone, not two.** Renaming `edn_shim.rs:1773` to
`edn/render.rs:1773` would leave a citation that is *still wrong* while now looking freshly
maintained — strictly worse than leaving it alone, because the staleness cue (an old module name)
would be gone.

## THE FORM

| what the prose says | becomes |
|---|---|
| `edn_shim::foo` | `edn::render::foo` |
| `src/edn_shim.rs` / `edn_shim.rs` | `src/edn/render.rs` |
| `edn_shim.rs:1773` (line **wrong**) | the SYMBOL that line meant — `edn::render::edn_to_value_caps`'s `Edn::Symbol` arm — **no number** |
| `edn_shim.rs:132` (line **right**) | `edn/render.rs` + the symbol; **still drop the number** |

⛔ **Drop the number even where it is currently correct.** Four of the twelve happen to still point
at the right line today; keeping those four teaches that a line citation is fine when you are careful,
which is exactly the belief that produced the other eight. The rule is the derivation, not the hit
rate. A citation names what it means.

## ⚠ THE `.wat` HALF IS COMMENTS, AND A RULES CODEMOD CANNOT REACH IT

Twenty of the 61 files are `.wat`, and every reference in them is inside a `;;` comment. **A comment
is not a node** — `wat/grep.wat`'s fact base is built from the form tree, so a rule cannot see one, by
construction. R21's "never hand-edit `.wat`" governs *structural* rewrites; this is prose the tool
provably cannot reach, and stone E set the precedent by hand-fixing its five comment lines.

## THE FOUR QUESTIONS

- **Obvious?** YES — a reference to a module that no longer exists is wrong on its face.
- **Simple?** YES — a rename, plus deleting a number and naming what it pointed at.
- **Honest?** YES, and this is the axis that matters: eight citations currently send a reader to the
  wrong place *with an air of precision*. Removing the number removes the false precision.
- **Good UX?** YES — a name that has rotted greps to zero. A line that has rotted greps to `None => {`.

## ACCEPTANCE

1. **Zero references to `edn_shim` / `wat_edn_bridge` / `runtime_error_edn` / `to_edn_derive_tests`**
   outside `docs/arc/**`. Derived: 118 at HEAD across 61 files.
2. **Zero `…\.rs:[0-9]+` citations of the five moved modules** — every one replaced by the symbol it
   meant. Derived: 12 distinct at HEAD, 8 of them already false.
3. **Each replaced symbol is RE-DERIVED, not guessed** — the rider must state, per citation, the
   symbol it now names and how it found it. A wrong symbol is worse than a wrong line.
4. **`docs/arc/**` untouched.** Immutable record; a past INSCRIPTION citing `edn_shim.rs` is a true
   statement about the world when it shipped.
5. **`src/edn/render.rs:682`'s self-referential `~:1440`** is corrected in the same pass — it is the
   worked example.
6. **No code changes.** Comments and docs only; a diff that touches an expression is out of scope.
7. Floor green **accounted BY NAME** (baseline 5057/5057, 19 skipped); clippy 0.

## OUT OF SCOPE — affirmatively cut

- **Every other stale line citation in the tree.** This stone covers the five moved modules. A
  general `file.rs:NNN` audit is a real and probably ugly question — measure it separately; do not
  let it ride in on this.
- **`docs/CONVENTIONS.md:1149` and `docs/MODULARIZATION-NOTES.md:20`**, which cite `edn_shim.rs` in
  *counts of loose root files*. Those are measurements of a past state, and both documents are stale
  in other ways (the README's layout diagram is stale six ways over). Doc-wide freshness is its own
  strike; these two lines get the rename only if they read as live, not as history.
