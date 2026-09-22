# BRIEF — STONE 251.8d-ii (THIRD DRAW): the bootstrap

**Drawn 2026-09-21 against `main` @ `43fe5b8f2`.** Floor 5959/5959, clippy 0, census `no STOP-8`.
Delta baseline **18**. Predecessors: `…-the-bootstrap.md`, `…-REDRAWN-…`, and their SCOREs.

## ⛔ READ THE TWO PRIOR SCORES FIRST. DO NOT RE-LITIGATE WHAT THEY PROVED.

The second attempt **already established**, and you inherit it:

| | |
|---|---|
| the path list | `git ls-files \| grep -E '^wat/.*\.wat$'` → **64**. ⛔ `git ls-files 'wat/**/*.wat'` → **31** — git's `**` does NOT match `wat/*.wat`. **This glob has bitten twice.** |
| conversion | **64/64 changed**, 1315 s, rc 0; the applied run was **byte-identical** to the dry run |
| build | `cargo build --release` exit 0 |
| recovery | `git checkout -- wat/` + rebuild ⇒ working tool. **No stash needed.** |

**What killed it was the LOAD, and 255.8 cured that.** ⭐ A converted stdlib now **starts**: 255.8
overlaid all 64 converted copies, rebuilt, and `--check` returned only `MainSignatureError`.

## ⛔⛔ THE GATE THAT IS STILL OWED — "it starts" IS NOT "it works"

255.8 proved the converted binary **starts** and **type-checks a trivial file**. It did **NOT** prove
the converted `wat/fix.wat` can still **run a codemod**. ⛔ **That is the whole point of this stone.**

`wat/fix.wat` is `include_str!`'d into the binary. The flip rewrites **the codemod's own source**, and
the rewritten codemod is what must then rewrite everything else in 8d-iii. ⛔ **A conversion gate is
not a loading gate, and a loading gate is not a RUNNING gate.** 8d-i passed 2145/2145 on conversion
while its premise was false; 8d-ii passed conversion while the stdlib would not load. **Do not let
this stone pass on a green that is one layer short again.**

⛔ **`include_str!` means on-disk edits are INVISIBLE until `cargo build --release`.** Four stones
have lost time to this. **Rebuild between every step, and say that you did.**

## The work

1. **Apply** the codemod to live `wat/` (all 64 paths, listed explicitly). `cargo build --release`.
2. ⭐ **SELF-APPLICATION — the real gate.** With the converted stdlib compiled in, run the converted
   codemod against a copy of an **unconverted** file and prove it converts it correctly.
3. ⭐ **IDEMPOTENCE.** Run it a second time over the converted `wat/`: **0 changes**.
4. **Floor / clippy / census**, and the **179-file delta** re-measured. Baseline **18**.

## ⛔ MEASURED HAZARD — a macro that diverges only by the ARROW

`wat/holon/Ngram.wat` is **in the stdlib** and it is the one file 255.8's delta **opened**:
converted, it raises `DuplicateMacro :wat::holon::Ngram`. **Verified here, with the control:** an
**UNCONVERTED** copy at the same out-of-tree path is CLEAN, so it is not the copy-out-of-place
artifact.

⭐ **The mechanism, measured:** the converted macro body differs from the embedded one by
**3 `<-` + 2 `->` → 5 `:-`**. `macro_structurally_equivalent` now compares names through
`canonical_identity` and has a `Keyword`↔`Symbol` arm — **but that arm requires `id.is_reference()`,
and `<-` / `->` are not references.** So a body that differs only by the binder-marker flip compares
**unequal**.

⚠ **This may be TRANSITIONAL ONLY** — once `wat/` is converted and rebuilt, both sides are `:-` and
nothing compares across spellings. ⛔ **Determine which, by measurement, and SAY SO.** If it is
transitional, it still means **any file checked against a half-converted stdlib can show a spurious
duplicate** — which is exactly the state this stone passes through. If it is not, it is a defect that
reaches 8d-iii.

## ⚠ What this stone's green CANNOT see

255.8 left the wrong join **accepted for a symbol author**: `(wat.core.Option.expect …)` — no slash
at all — resolves, because identity makes `::expect`, `other_join_spelling` flips it to `/expect`,
and the registry holds that. **A converted stdlib can therefore carry a wrong-join call head and
still be green.** ⛔ **Out of scope to fix here** — it is drawn as the stone before 8d-iii — but
**do not cite your green as evidence that every converted join is right.** It is not that gate.

## The gate

- ⭐ **The converted codemod CONVERTS** — step 2, on a real file, diffed.
- ⭐ **Idempotent** — second pass, **0 changes**.
- `scripts/floor.sh` green; clippy `-D warnings --all-targets --workspace` 0; census `no STOP-8`.
- Delta **≤ 18**, re-measured. ⚠ **Report the sample you used and how you built it** — the last two
  stones disagreed on the baseline (161 vs 164 clean on the same nominal 179), because the list is
  rebuilt by hand every time. If you commit the list as a file, say so; that is a welcome fix.
- ⛔ **`wat/` IS converted in this stone.** That is the point. Everything else stays put: no `src/`
  behaviour change, no corpus outside `wat/`, **no 8d-iii**.

## Doctrine

- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** ⭐ 255.8 disclosed a 66-test red, captured it whole,
  and did not re-run it to green — and the red turned out to be **the ONE DOOR lints catching a
  second name parser**. **That disclosure is why the stone was accepted. Do the same.**
- ⛔ **REPORT WHAT A DOOR CANNOT EXPRESS.** Nine stones running.
- ⛔ **If this brief contradicts the code or the runner, THEY WIN.** **Sixteen corrections across
  thirteen stones** — 255.8's was that the brief named the wrong slot entirely and its own hint
  ("suspect the routing") was the right one. **Assume a seventeenth.**
- ⚠ **Reformat in its own commit, or not at all.** 255.8's diff read as 2,889 lines and was ~2,251
  semantic; one file read as 958 and was 58. It cost the weigh four instruments to get an honest
  number.
- Work only in `/home/john/work/holon/wat-rs`. **Do not push.**
