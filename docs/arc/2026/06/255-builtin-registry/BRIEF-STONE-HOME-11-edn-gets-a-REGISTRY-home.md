# STONE HOME-11 — EDN gets a REGISTRY home (HOME-5 gave it a file home)

DRAWN 2026-08-27 against `4099d8435`.
**PRIOR ART:** `git log -1 8ddccaaa3` (HOME-5 — the file carve, already shipped) and
`git log -1 38f51c9fc` (Stone G — which this stone is the largest beneficiary of).

## ⛔ FIRST, A CORRECTION THAT REFRAMES THE CAMPAIGN

**"Home" has meant TWO different things and the orchestrator conflated them three times today.**

```
FILE-DOMAIN carve    loose root files  ->  src/<domain>/       HOME-5 edn · HOME-6 load · HOME-7 host
REGISTRY home        dispatch arms     ->  src/intrinsic/<ns>/ HOME-8 holon · HOME-9/10 math,stat,seq
```

HOME-5 **shipped on 2026-08-25** (`8ddccaaa3`) — `src/edn_shim.rs`, `wat_edn_bridge.rs`, `to_edn.rs`,
`to_edn_derive_tests.rs`, `runtime_error_edn.rs` are all GONE and `src/edn/` has six files. The
orchestrator called it "drawn but unbuilt" three times, having checked `src/intrinsic/edn` — the
wrong deliverable. **Its own DESIGN is now stale**: it opens *"five loose root files · NO `src/edn/`"*,
which describes a world that ended the day it was written.

**Only the REGISTRY kind serves arc 255.** A file carve tidies the tree; it does not make one name
addressable, so the blanket-accept at `src/resolve/walk.rs:268` waves through exactly as many names
after HOME-5 as before. This stone is the registry half.

## The move

```
:wat::edn::{read read-json read-foreign validate write write-pretty write-json write-json-natural}
:wat::edn::{ForeignRecord/get ForeignRecord/class
            ForeignVariant/variant ForeignVariant/enum-class ForeignVariant/fields}
                                             ->  src/intrinsic/edn.rs   (13 verbs)
```

**Nothing is renamed.** `:wat::edn::` is already the final spelling. Pure re-registration, same as
HOME-8 and HOME-10 — **no codemod, no RetirementEntry rows, no `.wat` corpus file. STOP-4.**

## ⛔ THE ONE CONTRACT DECISION — THE DECODE VERBS ARE PRODUCERS, AND THIS IS WHY STONE G EXISTS

`src/edn/render.rs` holds **17 `Provenance::RuntimeBuilt` construction sites** — the largest producer
cluster in the tree. Measured, inside the handlers these verbs dispatch to:

```
eval_edn_read          2 RuntimeBuilt      eval_edn_read_json      1
eval_edn_read_foreign  1 RuntimeBuilt      eval_edn_write          0
```

Decode verbs MINT values from text and stamp which verb made them. **Before Stone G a registry
handler could not carry provenance at all** (`NativeHandler` returned a bare `Value`), so this carve
would have silently downgraded every one to `SymbolBound` — a fact about a binding site rather than
about what manufactured the value. Stone E-iv did exactly that to four keyword verbs and had to
disclose it; Stone G reversed it.

**So: any handler whose body constructs a `TrackedValue` must RETURN `Result<TrackedValue, EvalBreak>`,
not a bare `Value`.** The macro's `sniff_return` forwards it un-rewrapped. Getting this wrong is
silent — the tests stay green and the provenance quietly dies. **STOP-1.**

Identify producers by what the body DOES, not by the verb's name. Report which you classified as
producers and the evidence for each.

## Rooms — verified against `4099d8435`

```
src/runtime.rs                 the 13 ":wat::edn::…" => arms
src/edn/render.rs              the handlers + all 17 RuntimeBuilt sites
src/edn/{bridge,error,contract,derive_tests,mod}.rs   the rest of the file home (HOME-5's work)
src/intrinsic/keyword.rs       ★ THE SHAPE TO COPY — the only existing home with a producer,
                               re-stamped by Stone G. Read it before writing a single handler.
src/intrinsic/math.rs          the plain shim shape (HOME-10), for the non-producers
src/intrinsic/mod.rs           `mod edn;`
src/macros/eval.rs             is_pure_total — measure; it has bitten five consecutive stones
src/rete/purity.rs             the KNOWN_UNREVIEWED ledger scans a UNION of match arms AND
                               #[wat_intrinsic] names, so a pure re-registration may need ZERO edits
                               (HOME-10 measured exactly that). Verify rather than assume.
```

## STOP triggers — each REJECTS

1. **STOP-1 — a producer handler returns a bare `Value`.** Its provenance dies silently and green.
2. **STOP-2 — you would change a verb's behaviour.** Registration only; report any obstruction.
3. **STOP-3 — a registry consistency test fires you cannot satisfy honestly.** Report it; never
   weaken an assertion to pass. This arc has spent two whole stones undoing exactly that.
4. **STOP-4 — you would write a codemod, a RetirementEntry row, or touch a `.wat` corpus file.**
5. **STOP-5 — a room's line number does not hold.**

## Acceptance

```bash
# 0. ★ THE HOME EXISTS — the deliverable, asked directly.
ls src/intrinsic/edn.rs
grep -c '#\[wat_intrinsic(' src/intrinsic/edn.rs          # 13
grep -cE '":wat::edn::[^"]*"\s*=>' src/runtime.rs         # 0

# 1. ★ PROVENANCE SURVIVES — the row this stone exists for.
#    Force a diagnostic on a value produced by `:wat::edn::read` and show its provenance renders
#    RuntimeBuilt{producer ":wat::edn::read"}, NOT SymbolBound. Paste it.
#    Then BREAK IT: make that handler return a bare Value, show the provenance degrade to
#    SymbolBound, restore, show it back. Report both outcomes — a green test proves nothing here.

# 2. every verb still RUNS with the same answers — a scratch-pad probe asserting each.
# 3. metadata-of answers for one verb.
# 4. cargo build --release --all-targets
```

## Report back with

Row 0 verbatim. **Row 1's degrade-and-restore, both outcomes.** Which verbs you classified as
producers and the evidence per verb. Every consistency test that fired and how you satisfied it.
What `is_pure_total` and the purity ledger needed. Anything this brief got wrong; what you did NOT
do, and why.
