# DESIGN — the speed floors are unfalsifiable, not wrong

**Status:** drawn 2026-09-09. Resolves **`3W1`** (circumspicere, L1) — the last instrument-cluster row.

## What the row says

*"THE CI SPEED FLOORS WERE MEASURED ON A JDK NOBODY RECORDED, AND CI NOW PINS A JDK THAT WAS NEVER
CHECKED AGAINST THEM."* The `FLOOR` array in `check-grid-speed.sh` derives, by its own comment, from
`GRID-native-vs-clara-2026-08-27T07-15-56Z.txt` — a manual local capture (`bbeb1997d`) taken through
an undocumented `PATH → JAVA_HOME → $HOME/opt/jdk-*` discovery. CI pins `distribution: temurin`,
`java-version: "21"`.

⭐ **And the sharpest part is the row's own**: that comment was written by a **prior `circumspicere`
cast**, whose cure — *"the old reason EXPIRED on 2026-08-27 when the `parity` job added Temurin 21"* —
is what opened this gap. `[[a-cure-can-carry-the-defect-one-level-down]]`.

## ⛔ Re-derived, and the framing must change

Verified this session:

- **29 recorded grids, `grep -il 'jdk|temurin|openjdk|java '` → 0.** None names a JDK. Confirmed.
- ⭐ **But this box runs `Temurin-21.0.12+8`, with `JAVA_HOME=/home/john/opt/jdk-21`** — the *same
  distribution and major version CI pins*, reached through exactly the `$HOME/opt/jdk-*` path the
  row names as undocumented.

**So the floors were plausibly measured under the very JDK CI uses.** The row's implication — that
CI pins something *never checked against them* — is **not established**, and may well be false.

**The defect is not that the numbers are wrong. It is that nobody can tell.** A measured bound whose
instrument is unrecorded cannot be re-derived, defended, or refuted — which is this repo's own
standing complaint, in `wat-rs/CLAUDE.md`: *"an unverifiable assertion is one that rots undetected by
construction,"* and `[[a-metric-without-its-instrument-cannot-be-rechecked]]`.

## What this delivers

1. **The grid records its JDK.** Whatever emits `GRID-*.txt` captures
   `java -version` (and the Clojure CLI version) into the file's header, so **every future capture is
   self-describing** and the next hand can re-derive rather than guess.
2. **The `FLOOR` comment states what is and is not known** — that the 2026-08-27 capture's JDK was
   not recorded; that the box which produced it runs Temurin 21.0.12 **today**, which is evidence
   about the box and **not proof about the capture**; and that the first grid recorded under (1)
   supersedes it as the provenance-bearing baseline.

## The one contract decision, pinned

⛔ **DO NOT RE-DERIVE THE FLOOR NUMBERS IN THIS STRIKE.** Re-measuring is a *performance* act needing
six samples, a quiet box, and its own scorecard — and doing it here would produce a second set of
numbers whose provenance is recorded while the *mechanism* that loses provenance stays unfixed.
**Fix the recording first; the next capture is then trustworthy by construction.**

⚠ **And do not claim the floors were measured on Temurin 21.** They probably were. *Probably* is
exactly what this strike exists to stop shipping.

## Out of scope = rejected

- **Re-deriving the floors** — see the pin.
- **Adding the speed gate to CI.** A separate question the file already discusses.
- **Back-filling JDK provenance into the 29 existing grids.** Unknowable; inventing it would be worse
  than the gap.
