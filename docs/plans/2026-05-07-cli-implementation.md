# name-variants CLI Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add an `nv` CLI to both the Python package and Rust crate, with `nv lookup` (key + all variants) and `nv match` (variant comparison), plus a full README with badges and publish to PyPI and crates.io.

**Architecture:** Seven tasks in order: (1) add Traditional Chinese forms to data, (2) add `lookup_all` to Python API, (3) update Rust codegen to emit a VARIANTS map + add `lookup_all` to Rust lib, (4) Python CLI using argparse (zero new deps), (5) Rust binary using clap, (6) README with blazehash-style badges, (7) publish both packages. The CLI output format is `KEY\tvariant1, variant2, ...` (tab-separated for machine parsing).

**Tech Stack:** Python argparse (stdlib), clap 4 (Rust), hatchling, cargo, twine/OIDC

**Design doc:** `docs/plans/2026-05-07-cli-design.md`

---

## Task 1: Traditional Script Forms in Chinese Data

Add Traditional Chinese character forms as the first variant in each relevant entry. This makes `lookup_key("陳")` return `"陈"` and ensures Traditional forms appear in CLI output.

**Files:**
- Modify: `name_variants/chinese_surnames.py`
- Modify: `tests/test_lookup.py`

**Step 1: Write the failing tests**

```python
# Add to tests/test_lookup.py

def test_traditional_chinese_resolves_to_simplified():
    """Traditional forms should resolve to the Simplified canonical key."""
    assert lookup_key("陳") == "陈"
    assert lookup_key("劉") == "刘"
    assert lookup_key("張") == "张"
    assert lookup_key("楊") == "杨"
    assert lookup_key("趙") == "赵"
    assert lookup_key("吳") == "吴"
    assert lookup_key("鄭") == "郑"
    assert lookup_key("許") == "许"


def test_traditional_and_simplified_same_key():
    """Traditional and Simplified forms of the same surname share one key."""
    assert lookup_key("陳") == lookup_key("陈")
    assert lookup_key("劉") == lookup_key("刘")
    assert lookup_key("許") == lookup_key("许")
```

**Step 2: Run tests to verify they fail**

```bash
pytest tests/test_lookup.py::test_traditional_chinese_resolves_to_simplified -v
```
Expected: FAIL — `AssertionError: None == "陈"`

**Step 3: Add Traditional forms to `chinese_surnames.py`**

Prepend the Traditional character to the variants list for each entry that has a distinct Traditional form. Find the entry, add the Traditional form as the FIRST variant (so it appears first in CLI output):

```python
# Entries to update — Traditional form prepended to variants list:
"陈": ["陳", "chen", "chan", "tan", "chin", "zen", "chern"],
"刘": ["劉", "liu", "lau", "lew", "low", "liew"],
"张": ["張", "zhang", "chang", "cheung", "cheong", "teo", "tio", "chong"],
"杨": ["楊", "yang", "yeung", "yeong", "yong", "young", "io"],
"赵": ["趙", "zhao", "chao", "chew", "chu", "chiu", "tio"],
"黄": ["黃", "huang", "wong", "ng", "oei", "uy", "wee", "way"],
"吴": ["吳", "wu", "ng", "goh", "ngo", "woo", "ou"],
"郑": ["鄭", "zheng", "cheng", "teh", "tay", "tee", "ching"],
"许": ["許", "xu", "hui", "kho", "khoo", "heui", "hee"],
"冯": ["馮", "feng", "fung", "fong", "foong"],
"邓": ["鄧", "deng", "tang", "teng", "ding"],
"谢": ["謝", "xie", "tse", "chia", "sia", "ze"],
"罗": ["羅", "luo", "lo", "law", "loh"],
"萧": ["蕭", "xiao", "hsiao", "siu", "sieu", "sew"],
"蒋": ["蔣", "jiang", "chiang", "cheung", "tsiang"],
"苏": ["蘇", "su", "soo", "soh", "see"],
"叶": ["葉", "ye", "yap", "ip", "yip", "jip"],
"吕": ["呂", "lu", "lui", "loo", "lv"],
"钟": ["鍾", "zhong", "chung", "tung", "tsong"],
"卢": ["盧", "lu", "lo", "loh", "luu"],
"龙": ["龍", "long", "loong", "lung"],
"陆": ["陸", "lu", "luk", "look"],
"华": ["華", "hua", "wah", "wa"],
"韩": ["韓", "han", "hon", "hann"],
"钱": ["錢", "qian", "chien", "chin"],
"谭": ["譚", "tan", "tam", "tham"],
"关": ["關", "guan", "kwan", "kwaan"],
```

Note: Some entries in the current file (e.g. `王`, `李`, `孙`, `林`) use the same character in both scripts — do NOT add a Traditional form to those.

**Step 4: Run tests to verify they pass**

```bash
pytest tests/test_lookup.py -v
```
Expected: all PASS including the two new tests

**Step 5: Verify the existing `test_all_romanizations_are_lowercase` still passes**

```bash
pytest tests/test_lookup.py::test_all_romanizations_are_lowercase -v
```
Expected: PASS — CJK characters are unaffected by `.lower()` so they pass the check

**Step 6: Commit**

```bash
git add name_variants/chinese_surnames.py tests/test_lookup.py
git commit -m "feat(data): add Traditional Chinese script forms as variants"
```

---

## Task 2: `lookup_all` Python Function

Add `lookup_all(text)` to the public API — returns `(canonical_key, variants_list)` for CLI output, or `None` if unknown.

**Files:**
- Modify: `name_variants/__init__.py`
- Modify: `tests/test_lookup.py`

**Step 1: Write the failing tests**

```python
# Add to tests/test_lookup.py
from name_variants import lookup_all


def test_lookup_all_returns_key_and_variants():
    result = lookup_all("Chan")
    assert result is not None
    key, variants = result
    assert key == "陈"
    assert "chen" in variants
    assert "chan" in variants


def test_lookup_all_traditional_shows_in_variants():
    key, variants = lookup_all("陈")
    assert key == "陈"
    assert "陳" in variants


def test_lookup_all_unknown_returns_none():
    assert lookup_all("Smith") is None
    assert lookup_all("") is None


def test_lookup_all_multiword():
    result = lookup_all("Chan Wai Ming")
    assert result is not None
    assert result[0] == "陈"
```

**Step 2: Run tests to verify they fail**

```bash
pytest tests/test_lookup.py -k "lookup_all" -v
```
Expected: FAIL — `ImportError: cannot import name 'lookup_all'`

**Step 3: Implement `lookup_all` in `name_variants/__init__.py`**

Add after the `lookup_key` function:

```python
def lookup_all(text: str) -> tuple[str, list[str]] | None:
    """
    Return (canonical_key, all_variants) for a name, or None if unknown.

    Examples:
        lookup_all("Chan")   → ("陈", ["陳", "chen", "chan", "tan", ...])
        lookup_all("Smith")  → None
    """
    key = lookup_key(text)
    if key is None:
        return None
    for table in ALL_TABLES.values():
        if key in table:
            return key, table[key]
    # Key found in index but not in any table — shouldn't happen
    return key, []
```

Also add `lookup_all` to `__all__`:

```python
__all__ = ["lookup_key", "lookup_all", "ALL_TABLES"]
```

**Step 4: Run tests to verify they pass**

```bash
pytest tests/test_lookup.py -v
```
Expected: all PASS

**Step 5: Commit**

```bash
git add name_variants/__init__.py tests/test_lookup.py
git commit -m "feat: add lookup_all() — returns canonical key + all variants"
```

---

## Task 3: Rust VARIANTS Map + `lookup_all`

Update the codegen to emit a second map `VARIANTS` (canonical → variants slice), add `lookup_all` to `lib.rs`, regenerate, run tests.

**Files:**
- Modify: `codegen/gen_rust.py`
- Modify: `name-variants-rs/src/lib.rs`
- Regenerate: `name-variants-rs/src/generated.rs`

**Step 1: Write the failing Rust tests**

Append to `name-variants-rs/src/lib.rs` tests block:

```rust
#[test]
fn lookup_all_chan_returns_variants() {
    let (key, variants) = lookup_all("Chan").unwrap();
    assert_eq!(key, "陈");
    assert!(variants.contains(&"chen"));
    assert!(variants.contains(&"陳"));
}

#[test]
fn lookup_all_unknown_returns_none() {
    assert!(lookup_all("Smith").is_none());
}
```

**Step 2: Run to verify they fail**

```bash
cd name-variants-rs && cargo test 2>&1 | grep -E "FAILED|error"
```
Expected: compile error — `lookup_all` not defined

**Step 3: Update `codegen/gen_rust.py` to emit the VARIANTS map**

Add a new `generate_variants` function and update `generate` and `main`:

```python
def build_variants_map(
    tables: list[tuple[str, dict[str, list[str]]]]
) -> dict[str, list[str]]:
    """canonical_key → variants list (in table order)."""
    variants_map: dict[str, list[str]] = {}
    for _name, table in tables:
        for canonical, variants in table.items():
            if canonical not in variants_map:
                variants_map[canonical] = variants
    return variants_map


def generate(index: dict[str, str], variants_map: dict[str, list[str]]) -> str:
    """Render the full generated.rs content."""
    # INDEX entries (variant → canonical)
    index_entries = "\n".join(
        f'    "{escape_rust_str(k)}" => "{escape_rust_str(v)}",'
        for k, v in sorted(index.items())
    )

    # VARIANTS entries (canonical → &[variants])
    variants_entries = "\n".join(
        f'    "{escape_rust_str(k)}" => &[{", ".join(f\'"{escape_rust_str(v)}"\'  for v in vs)}],'
        for k, vs in sorted(variants_map.items())
    )

    return f"""\
// GENERATED FILE — do not edit by hand.
// Run: python codegen/gen_rust.py
// to regenerate from name_variants/*.py source tables.
//
// INDEX entries: {len(index)}
// VARIANTS entries: {len(variants_map)}

use phf::phf_map;

pub(crate) static INDEX: phf::Map<&'static str, &'static str> = phf_map! {{
{index_entries}
}};

pub(crate) static VARIANTS: phf::Map<&'static str, &'static [&'static str]> = phf_map! {{
{variants_entries}
}};
"""
```

Update `main()` to build and pass `variants_map`:

```python
def main() -> None:
    tables = []
    for module_name, var_name in TABLE_MODULES:
        table = load_table(module_name, var_name)
        tables.append((module_name, table))
        print(f"  loaded {module_name}: {len(table)} entries", file=sys.stderr)

    index = build_flat_index(tables)
    variants_map = build_variants_map(tables)
    print(f"  flat index: {len(index)} entries, variants: {len(variants_map)} keys", file=sys.stderr)

    content = generate(index, variants_map)
    OUTPUT_PATH.write_text(content, encoding="utf-8")
    print(f"  wrote {OUTPUT_PATH}", file=sys.stderr)
```

**Step 4: Run codegen**

```bash
cd /Users/4n6h4x0r/src/name-variants
python codegen/gen_rust.py
```
Expected: `flat index: 3510+ entries, variants: 1000+ keys`

**Step 5: Add `lookup_all` to `name-variants-rs/src/lib.rs`**

Add after `lookup_key`:

```rust
/// Return the canonical key and all known variants for a name, or `None` if unknown.
///
/// # Examples
/// ```
/// use name_variants::lookup_all;
/// let (key, variants) = lookup_all("Chan").unwrap();
/// assert_eq!(key, "陈");
/// assert!(variants.contains(&"chen"));
/// ```
pub fn lookup_all(text: &str) -> Option<(&'static str, &'static [&'static str])> {
    let key = lookup_key(text)?;
    let variants = generated::VARIANTS.get(key)?;
    Some((key, variants))
}
```

**Step 6: Run tests to verify all pass**

```bash
cd name-variants-rs && cargo test 2>&1
```
Expected: all tests pass (including the two new ones)

**Step 7: Commit**

```bash
cd /Users/4n6h4x0r/src/name-variants
git add codegen/gen_rust.py name-variants-rs/src/generated.rs name-variants-rs/src/lib.rs
git commit -m "feat(rust): add VARIANTS map to codegen and lookup_all to lib"
```

---

## Task 4: Python CLI

Add `name_variants/cli.py` using argparse (zero new dependencies). Wire up as `nv` entry point.

**Files:**
- Create: `name_variants/cli.py`
- Modify: `pyproject.toml`
- Create: `tests/test_cli.py`

**Step 1: Write the failing tests**

```python
# tests/test_cli.py
"""CLI integration tests using subprocess."""
import subprocess
import sys


def run(args: list[str], stdin: str | None = None) -> subprocess.CompletedProcess:
    return subprocess.run(
        [sys.executable, "-m", "name_variants.cli"] + args,
        input=stdin,
        capture_output=True,
        text=True,
    )


def test_lookup_known_name():
    result = run(["lookup", "Chan"])
    assert result.returncode == 0
    lines = result.stdout.strip().split("\n")
    assert len(lines) == 1
    key, variants = lines[0].split("\t")
    assert key == "陈"
    assert "chen" in variants
    assert "陳" in variants


def test_lookup_unknown_name():
    result = run(["lookup", "Smith"])
    assert result.returncode == 0
    assert result.stdout.strip() == "-"


def test_lookup_stdin_mode():
    result = run(["lookup"], stdin="Chan\nSmith\nPark\n")
    assert result.returncode == 0
    lines = result.stdout.strip().split("\n")
    assert len(lines) == 3
    assert lines[0].startswith("陈\t")
    assert lines[1] == "-"
    assert lines[2].startswith("박\t")


def test_match_same_names():
    result = run(["match", "Chan", "Chen"])
    assert result.returncode == 0
    assert result.stdout.strip() == "match"


def test_match_different_names():
    result = run(["match", "Chan", "Smith"])
    assert result.returncode == 1
    assert result.stdout.strip() == "no match"


def test_match_unknown_name():
    result = run(["match", "Smith", "Jones"])
    assert result.returncode == 1
    assert result.stdout.strip() == "no match"


def test_version_flag():
    result = run(["--version"])
    assert result.returncode == 0
    assert "0.1.0" in result.stdout or "0.1.0" in result.stderr
```

**Step 2: Run tests to verify they fail**

```bash
pytest tests/test_cli.py -v
```
Expected: FAIL — `ModuleNotFoundError` or `No such file or directory`

**Step 3: Create `name_variants/cli.py`**

```python
"""
nv — name-variants command-line tool.

Usage:
    nv lookup [NAME]       resolve name to canonical key + variants
                           reads stdin if NAME omitted
    nv match NAME1 NAME2   exit 0 if same key, exit 1 otherwise
    nv --version
"""

from __future__ import annotations

import argparse
import sys

from name_variants import __version__, lookup_key, lookup_all


def _format_result(text: str) -> str:
    """Format one lookup line: KEY\\tvariant1, variant2 ... or -"""
    result = lookup_all(text.strip())
    if result is None:
        return "-"
    key, variants = result
    return f"{key}\t{', '.join(variants)}"


def cmd_lookup(args: argparse.Namespace) -> int:
    if args.name:
        print(_format_result(args.name))
    else:
        for line in sys.stdin:
            line = line.rstrip("\n")
            if line:
                print(_format_result(line))
    return 0


def cmd_match(args: argparse.Namespace) -> int:
    ka = lookup_key(args.name1)
    kb = lookup_key(args.name2)
    if ka is not None and kb is not None and ka == kb:
        print("match")
        return 0
    print("no match")
    return 1


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="nv",
        description="Multilingual name romanization lookup",
    )
    parser.add_argument("--version", action="version", version=f"%(prog)s {__version__}")

    sub = parser.add_subparsers(dest="command")

    # lookup
    p_lookup = sub.add_parser("lookup", help="resolve name(s) to canonical key + variants")
    p_lookup.add_argument("name", nargs="?", help="name to look up (reads stdin if omitted)")

    # match
    p_match = sub.add_parser("match", help="test if two names are variants of the same entity")
    p_match.add_argument("name1")
    p_match.add_argument("name2")

    return parser


def main() -> None:
    parser = build_parser()
    args = parser.parse_args()

    if args.command == "lookup":
        sys.exit(cmd_lookup(args))
    elif args.command == "match":
        sys.exit(cmd_match(args))
    else:
        parser.print_help()
        sys.exit(1)


if __name__ == "__main__":
    main()
```

**Step 4: Add `__version__` to `name_variants/__init__.py`**

Add near the top (after the docstring):
```python
__version__ = "0.1.0"
```

**Step 5: Add entry point to `pyproject.toml`**

Add:
```toml
[project.scripts]
nv = "name_variants.cli:main"
```

**Step 6: Reinstall in dev mode**

```bash
pip install -e .
```

**Step 7: Run tests to verify they pass**

```bash
pytest tests/test_cli.py -v
```
Expected: all 7 PASS

**Step 8: Smoke test the installed binary**

```bash
nv lookup Chan
nv match Chan Chen && echo "exit 0"
nv match Chan Smith || echo "exit 1"
printf "Chan\nPark\nSmith\n" | nv lookup
```

**Step 9: Commit**

```bash
git add name_variants/cli.py name_variants/__init__.py pyproject.toml tests/test_cli.py
git commit -m "feat: nv CLI — lookup and match subcommands (Python/argparse)"
```

---

## Task 5: Rust Binary

Add `src/main.rs` and wire up `[[bin]]` in `Cargo.toml`. Uses `clap` derive API.

**Files:**
- Modify: `name-variants-rs/Cargo.toml`
- Create: `name-variants-rs/src/main.rs`

**Step 1: Add `clap` to `Cargo.toml` and declare the binary**

Add to `name-variants-rs/Cargo.toml`:

```toml
[[bin]]
name = "nv"
path = "src/main.rs"

[dependencies]
phf = { version = "0.11", features = ["macros"] }
clap = { version = "4", features = ["derive"] }
```

**Step 2: Write the failing integration test**

Add to `name-variants-rs/src/lib.rs` tests block:

```rust
// These will compile but not yet pass (main.rs doesn't exist yet)
// The real binary tests run via `cargo test --test integration` or subprocess.
// For now, add a unit test that exercises the same logic:
#[test]
fn match_logic_chan_chen() {
    let ka = lookup_key("chan");
    let kb = lookup_key("chen");
    assert_eq!(ka, kb);
    assert!(ka.is_some());
}
```

Run to confirm it passes (it will, based on existing impl):
```bash
cd name-variants-rs && cargo test match_logic_chan_chen 2>&1
```

**Step 3: Create `name-variants-rs/src/main.rs`**

```rust
use clap::{Parser, Subcommand};
use name_variants::{lookup_all, lookup_key};
use std::io::{self, BufRead};

#[derive(Parser)]
#[command(name = "nv", about = "Multilingual name romanization lookup", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Resolve name(s) to canonical key + all variants.
    /// Reads stdin line by line if NAME is omitted.
    Lookup {
        /// Name to look up
        name: Option<String>,
    },
    /// Test if two names are variants of the same entity (exits 0 if match).
    Match {
        name1: String,
        name2: String,
    },
}

fn format_result(text: &str) -> String {
    match lookup_all(text.trim()) {
        Some((key, variants)) => format!("{}\t{}", key, variants.join(", ")),
        None => "-".to_string(),
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Lookup { name: Some(n) } => {
            println!("{}", format_result(&n));
        }
        Command::Lookup { name: None } => {
            let stdin = io::stdin();
            for line in stdin.lock().lines() {
                let line = line.expect("failed to read stdin");
                if !line.is_empty() {
                    println!("{}", format_result(&line));
                }
            }
        }
        Command::Match { name1, name2 } => {
            let ka = lookup_key(&name1);
            let kb = lookup_key(&name2);
            if ka.is_some() && ka == kb {
                println!("match");
                std::process::exit(0);
            } else {
                println!("no match");
                std::process::exit(1);
            }
        }
    }
}
```

**Step 4: Build and verify**

```bash
cd name-variants-rs && cargo build 2>&1
```
Expected: compiles cleanly

**Step 5: Smoke test the binary**

```bash
./target/debug/nv lookup Chan
./target/debug/nv match Chan Chen && echo "exit 0"
./target/debug/nv match Chan Smith || echo "exit 1"
printf "Chan\nPark\nSmith\n" | ./target/debug/nv lookup
./target/debug/nv --version
```
Expected output:
```
陈	陳, chen, chan, tan, chin, zen, chern
match
exit 0
no match
exit 1
陈	陳, chen, chan, tan, ...
박	park, bak, pak
-
nv 0.1.0
```

**Step 6: Run all Rust tests**

```bash
cd name-variants-rs && cargo test 2>&1
```
Expected: all pass

**Step 7: Commit**

```bash
cd /Users/4n6h4x0r/src/name-variants
git add name-variants-rs/Cargo.toml name-variants-rs/src/main.rs
git commit -m "feat(rust): nv binary — lookup and match subcommands (clap)"
```

---

## Task 6: README

Write the root `README.md` following the blazehash badge template. Also update `name-variants-rs/README.md` to include the binary usage.

**Files:**
- Create: `README.md`
- Modify: `name-variants-rs/README.md`

**Step 1: Create root `README.md`**

```markdown
[![PyPI](https://img.shields.io/pypi/v/name-variants.svg)](https://pypi.org/project/name-variants/)
[![Crates.io](https://img.shields.io/crates/v/name-variants.svg)](https://crates.io/crates/name-variants)
[![CI](https://github.com/h4x0r/name-variants/actions/workflows/ci.yml/badge.svg)](https://github.com/h4x0r/name-variants/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Sponsor](https://img.shields.io/badge/sponsor-h4x0r-ea4aaa?logo=github-sponsors)](https://github.com/sponsors/h4x0r)

**Multilingual name romanization lookup tables — Python and Rust.**

Maps every known romanization variant of a name to its canonical native-script form, so `陳 / 陈 / Chen / Chan / Tan`, `박 / Park / Bak`, `Nguyễn / Nguyen`, and `محمد / Muhammad / Mohammed` are treated as the same entity.

```bash
pip install name-variants   # Python
cargo install name-variants # Rust
```

---

## CLI

```bash
$ nv lookup Chan
陈	陳, chen, chan, tan, chin, zen, chern

$ nv lookup Smith
-

$ printf "Chan\nPark\nMuhammad\n" | nv lookup
陈	陳, chen, chan, tan, chin, zen, chern
박	park, bak, pak
محمد	muhammad, mohammed, mohamed, mohammad, mohamad, mehmed, mehmet

$ nv match Chan Chen && echo "same entity"
match
same entity

$ nv match Chan Smith || echo "different"
no match
different
```

Output format: `CANONICAL_KEY\tvariant1, variant2, ...` — tab-separated for `cut -f1` / `cut -f2`. Unknown names print `-`. `nv match` exits 0 on match, 1 on no match.

## Python API

```python
from name_variants import lookup_key, lookup_all

lookup_key("Chan")      # → "陈"
lookup_key("陳")        # → "陈"   (Traditional resolves to Simplified)
lookup_key("Park")      # → "박"
lookup_key("Muhammad")  # → "محمد"
lookup_key("Smith")     # → None

key, variants = lookup_all("Chan")
# key      → "陈"
# variants → ["陳", "chen", "chan", "tan", "chin", "zen", "chern"]
```

## Rust API

```rust
use name_variants::{lookup_key, lookup_all};

assert_eq!(lookup_key("Chan"),     Some("陈"));
assert_eq!(lookup_key("Smith"),    None);

let (key, variants) = lookup_all("Chan").unwrap();
// key      == "陈"
// variants == ["陳", "chen", "chan", "tan", "chin", "zen", "chern"]
```

## Languages

| Script | Entries | Example |
|--------|--------:|---------|
| Chinese (Han) | 200+ surnames | `陈 / 陳 → chen, chan, tan` |
| Arabic | 100+ given names | `محمد → muhammad, mohammed` |
| Japanese | 150+ surnames | `田中 → tanaka` |
| Korean | 100+ surnames | `박 → park, bak, pak` |
| Vietnamese | 80+ surnames | `nguyễn → nguyen` |
| Hindi/North Indian | 80+ names | `शर्मा → sharma, sarma` |
| Tamil | 60+ names | `சுப்பிரமணியம் → subramaniam` |
| Bengali | 60+ names | `চট্টোপাধ্যায় → chattopadhyay, chatterjee` |
| Persian/Farsi | 80+ names | `حسین → hossein, hussein` |
| Hebrew | 80+ names | `יצחק → yitzhak, isaac` |
| Thai | 80+ names | `ประยุทธ์ → prayuth, prayut` |
| Greek | 70+ names | `Κωνσταντίνος → konstantinos, constantine` |
| Turkish | 70+ names | `Çelik → celik` |
| Russian/Slavic | 80+ names | `Иванов → ivanov, ivanoff` |
| Indonesian/Malay | 60+ names | `suharto → soeharto` |

## Design

- **Native script is always the canonical key** — `陈`, `박`, `محمد`, not romanizations
- **Traditional forms included as variants** — `陳` maps to `陈`; no opencc required
- **Zero runtime dependencies** — pure Python dicts / compile-time `phf` hash map in Rust
- **`None` for unknowns** — explicit signal to fall back to fuzzy matching
- **Lowercase romanizations** — callers casefold before lookup (the library does this internally)
```

**Step 2: Update `name-variants-rs/README.md`**

Add a CLI section before the Install section:

```markdown
## CLI

Install the `nv` binary:

```bash
cargo install name-variants
```

```bash
nv lookup Chan          # 陈	陳, chen, chan, tan, chin, zen, chern
nv lookup Smith         # -
nv match Chan Chen      # match (exit 0)
nv match Chan Smith     # no match (exit 1)
printf "Chan\nPark\n" | nv lookup
```
```

**Step 3: Commit**

```bash
git add README.md name-variants-rs/README.md
git commit -m "docs: root README with badges, CLI examples, API reference"
```

---

## Task 7: Publish to PyPI and crates.io

**Step 1: Bump versions to 0.2.0** (CLI is a meaningful new feature)

Update `name_variants/__init__.py`:
```python
__version__ = "0.2.0"
```

Update `pyproject.toml`:
```toml
version = "0.2.0"
```

Update `name-variants-rs/Cargo.toml`:
```toml
version = "0.2.0"
```

**Step 2: Regenerate Rust (version doesn't affect generated.rs, but rebuild to verify)**

```bash
cd name-variants-rs && cargo build && cargo test 2>&1
```
Expected: all pass

**Step 3: Run full Python test suite**

```bash
pytest tests/ -v 2>&1
```
Expected: all pass

**Step 4: Build Python package**

```bash
pip install build twine
python -m build
python -m twine check dist/*
```
Expected: `PASSED` for wheel and sdist

**Step 5: Publish Python to PyPI**

```bash
python -m twine upload dist/*
```
Enter PyPI credentials when prompted (or use API token).

Verify:
```bash
pip install --upgrade name-variants
nv lookup Chan
```

**Step 6: Run cargo publish dry-run**

```bash
cd name-variants-rs && cargo publish --dry-run 2>&1
```
Expected: `Packaged N files ... warning: aborting upload due to dry run`

**Step 7: Publish Rust to crates.io**

```bash
cd name-variants-rs && cargo publish 2>&1
```
Requires prior `cargo login` with crates.io API token.

**Step 8: Tag the release**

```bash
git tag v0.2.0
git push origin v0.2.0
```

**Step 9: Commit version bumps**

```bash
git add name_variants/__init__.py pyproject.toml name-variants-rs/Cargo.toml
git commit -m "chore: bump versions to 0.2.0 for CLI release"
git push
```

**Step 10: Verify both published**

```bash
pip install --upgrade name-variants && nv --version
cargo install name-variants && nv --version
```
Expected: `nv 0.2.0` from both
