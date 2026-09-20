#!/usr/bin/env python3
"""Generate clj_oracle/corpus.txt from the EDN spec's grammar.

Stone 218.7: do not extend the hand-list. Enumerate, then regen.clj
asks clojure.edn. One form per line (newlines cannot be expressed).
"""
from pathlib import Path

OUT = Path(__file__).with_name("corpus.txt")

# Spec constituent chars (the list 219 quoted) plus the omitted `: #`.
CONSTITUENT = list(". * + ! - _ ? $ % & = < >".split())
ALNUM_SAMPLE = list("aZ0")  # 0 is illegal as FIRST, legal as continue

rows: list[str] = []


def add(s: str) -> None:
    if s and s not in rows:
        rows.append(s)


# ── keep the original 72 as a prefix (don't drop coverage) ──
legacy = """
nil true false 0 -5 42 +7 9999999999
1.5 -2.0 1e10 1.5M 1/2 -3/4
"" "abc" "with \\"quote\\"" "tab\\tnl\\n" "héllo" "a 😀 b"
\\a \\newline \\space \\tab \\é \\😀
é foo foo-bar + - / my.ns/foo foo? *foo* 😀 é λ foo→bar
:foo :my/foo :a-b :a😀 :λ ::foo :/
() (1 2 3) (a (b c)) [] [1 2 3] {} {:a 1} {:a 1 :b 2} {[1 2] 3}
#{} #{1 2 3}
#inst "1985-04-12T23:20:50.52Z"
#uuid "f81d4fae-7dec-11d0-a765-00a0c91e6bf6"
#myapp/Foo {:x 1}
[a #_b c] (1, 2, 3)
##Inf ##-Inf ##NaN
4/2 6/3 1/1 0/5 -6/4 10/4 1/0
""".split()

# The legacy block above split() destroys quoted strings. Load from the
# committed file if present, else the unquoted tokens only.
existing = Path(__file__).with_name("corpus.txt")
if existing.exists():
    for line in existing.read_text().splitlines():
        add(line)

# ── symbols: first vs continue for each constituent ──
for ch in CONSTITUENT + list("abcXYZ"):
    add(ch)          # first (and only)
    add("a" + ch)    # continue
    add(ch + "a")    # first then letter

# 218.8 — `:` / `#` as body constituents. Enumerate interior, doubled,
# trailing, leading; symbols and keywords; combined with `.` / `-`.
# Do not paste the brief's table; generate the surface.
for p in (":", "#"):
    add("a" + p + "b")             # interior
    add("a" + p + p + "b")         # doubled
    add("x" + p)                   # trailing
    add("x" + p + p)               # trailing doubled
    add(":a" + p + "b")            # keyword interior
    add(":a" + p + p + "b")        # keyword doubled
    add(":x" + p)                  # keyword trailing
for sep in ".-":
    add("a" + sep + "b:c")
    add("a" + sep + "b#c")
    add("a:b" + sep + "c")
    add("a#b" + sep + "c")
    add(":a" + sep + "b:c")
add("a:b:c")
add("a#:b")
add("a:#b")
add("#x")                         # leading hash = dispatch
add("a:::b")
add(":a::b")
add("x:")
add("x::")
add(":x:")
add("a:/b")                       # colon-final on the prefix
add("foo/a:b")
add(":ns/a:b")
add("wat::core::x")
add(":wat::core::x")
add("foo#_bar")                   # one symbol, not discard

# slash counts 0 / 1 / 2 / 3
add("foo")
add("ns/foo")
add("a/b/c")
add("a/b/c/d")
add("clojure.core//")
add("/x")   # empty prefix
add("x/")   # empty name
add("/")    # bare slash (already in legacy)

# leading - + . then numeric vs non-numeric
add("-a")
add("+a")
add(".a")
add("-1")   # number
add("+1")
add(".1")   # float or error — oracle decides
add("+")
add("-")
add(".")

# ── integers ──
add("0")
add("1")
add("-1")
add("9223372036854775807")   # i64::MAX
add("9223372036854775808")   # MAX+1
add("-9223372036854775808")  # i64::MIN
add("-9223372036854775809")  # MIN-1
add("123456789012345678901234567890")
add("9223372036854775808N")
add("123456789012345678901234567890N")
add("42N")
add("0N")

# ── floats ──
add("1.5")
add(".5")
add("5.")
add("1e10")
add("1.5e-2")
add("1.5E+2")
add("1.5M")
add("0.0")
add("-0.0")

# ── strings ──
add(r'"\n"')
add(r'"\t"')
add(r'"\r"')
add(r'"\\"')
add(r'"\""')
add(r'"\q"')          # illegal escape
add(r'"\u0041"')
add(r'"\u00"')        # truncated
add(r'"\uXXXX"')      # not hex

# ── chars ──
add(r"\newline")
add(r"\space")
add(r"\tab")
add(r"\return")
add(r"\u0041")
add(r"\a")
add(r"\ ")            # backslash + space — spec forbids

# ── collections ──
add("(")
add("[1")
add("{:a")
add("{:a 1")
add("{:a}")
add("{:a 1 :a 2}")
add("#{1 1}")
add("#{1 1 2}")
add("(1, 2, 3)")
add("{,}")
add("[1, 2]")

# ── # dispatch ──
add("#{1}")
add("#_1 2")
add('#inst "1985-04-12T23:20:50.52Z"')
add('#inst "1985-04-12"')          # date-only — ruling
add('#inst "not-a-timestamp"')
add('#uuid "f81d4fae-7dec-11d0-a765-00a0c91e6bf6"')
add('#uuid "not-a-uuid"')
add("#myapp/Person {:first \"F\"}")

# ── NOT-EDN negative controls (must stay refused by wat-edn) ──
add("'x")
add("`x")
add("~x")
add("~@x")
add("@x")
add("^:m x")

OUT.write_text("\n".join(rows) + "\n")
print(f"wrote {len(rows)} rows to {OUT}")
