# name-variants-rs Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Publish a native Rust crate `name-variants` on crates.io that mirrors the Python `name-variants` package — same 15-language lookup tables, same `lookup_key` semantics, zero runtime dependencies.

**Architecture:** A Python codegen script (`codegen/gen_rust.py`) reads the 15 Python table modules and emits `name-variants-rs/src/generated.rs` containing a flat `phf_map!` of every romanization variant → canonical script key. The Rust crate exposes `pub fn lookup_key(text: &str) -> Option<&'static str>` with exact, lowercase, and token-split matching. The generated file is committed to source — Rust users never need Python. CI has a drift-check step to ensure generated.rs stays in sync with the Python tables.

**Tech Stack:** Rust ≥1.70, `phf` 0.11 (macros feature), Python 3.11 (codegen only), GitHub Actions

---

## Task 1: Rust Crate Scaffold

**Files:**
- Create: `name-variants-rs/Cargo.toml`
- Create: `name-variants-rs/src/lib.rs`
- Create: `name-variants-rs/src/generated.rs`

**Step 1: Create `Cargo.toml`**

```toml
[package]
name = "name-variants"
version = "0.1.0"
edition = "2021"
description = "Multilingual name romanization lookup tables: Chinese, Japanese, Korean, Arabic, Vietnamese, Indian, Persian, Hebrew, Thai, Greek, Turkish, Russian, Indonesian/Malay"
license = "MIT"
repository = "https://github.com/SecurityRonin/name-variants"
keywords = ["names", "romanization", "transliteration", "nlp", "cjk"]
categories = ["text-processing"]

[dependencies]
phf = { version = "0.11", features = ["macros"] }
```

**Step 2: Create stub `src/lib.rs`**

```rust
//! Multilingual name romanization lookup tables.
//!
//! Maps romanization variants to their canonical native-script key so that
//! `Chen`, `Chan`, and `Tan` all resolve to `陈`.

mod generated;

/// Return the canonical script-form key for a name, or `None` if unknown.
///
/// Matching order:
/// 1. Exact match (handles native script input like `陈`)
/// 2. Lowercase match (handles `Chan`, `CHAN`)
/// 3. Token-by-token (handles `"Chan Wai Ming"` → checks `chan`)
///
/// # Examples
/// ```
/// use name_variants::lookup_key;
/// assert_eq!(lookup_key("Chan"), Some("陈"));
/// assert_eq!(lookup_key("Smith"), None);
/// ```
pub fn lookup_key(text: &str) -> Option<&'static str> {
    // stub — always returns None until generated map is wired in
    let _ = text;
    None
}
```

**Step 3: Create empty `src/generated.rs` placeholder**

```rust
// GENERATED FILE — do not edit by hand.
// Run: python codegen/gen_rust.py
// to regenerate from name_variants/*.py source tables.

use phf::phf_map;

pub(crate) static INDEX: phf::Map<&'static str, &'static str> = phf_map! {
    "__placeholder__" => "__placeholder__",
};
```

**Step 4: Verify it compiles**

```bash
cd name-variants-rs
cargo build
```
Expected: compiles with zero errors (stub returns None for everything)

**Step 5: Commit**

```bash
git add name-variants-rs/
git commit -m "chore(rust): crate scaffold with stub lookup_key"
```

---

## Task 2: Failing Rust Tests

**Files:**
- Modify: `name-variants-rs/src/lib.rs`

**Step 1: Add `#[cfg(test)]` block with failing tests**

Append to `src/lib.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // ── Chinese ──────────────────────────────────────────────────────────────
    #[test]
    fn chan_and_chen_same_key() {
        assert_eq!(lookup_key("chan"), lookup_key("chen"));
    }

    #[test]
    fn chan_resolves_to_simplified_chinese() {
        assert_eq!(lookup_key("chan"), Some("陈"));
    }

    #[test]
    fn hui_and_xu_same_key() {
        // 許/许: Xu (Mandarin), Hui (Cantonese), Kho (Hokkien)
        assert_eq!(lookup_key("hui"), lookup_key("xu"));
    }

    #[test]
    fn wang_and_wong_same_key() {
        assert_eq!(lookup_key("wang"), lookup_key("wong"));
    }

    // ── Korean ───────────────────────────────────────────────────────────────
    #[test]
    fn park_and_bak_same_key() {
        assert_eq!(lookup_key("park"), lookup_key("bak"));
    }

    #[test]
    fn lee_and_rhee_same_key() {
        assert_eq!(lookup_key("lee"), lookup_key("rhee"));
    }

    // ── Arabic ───────────────────────────────────────────────────────────────
    #[test]
    fn muhammad_and_mohammed_same_key() {
        assert_eq!(lookup_key("muhammad"), lookup_key("mohammed"));
    }

    // ── Russian ──────────────────────────────────────────────────────────────
    #[test]
    fn ivanov_and_ivanoff_same_key() {
        assert_eq!(lookup_key("ivanov"), lookup_key("ivanoff"));
    }

    // ── Case-insensitive ─────────────────────────────────────────────────────
    #[test]
    fn uppercase_input_matches() {
        assert_eq!(lookup_key("CHAN"), Some("陈"));
    }

    #[test]
    fn mixed_case_input_matches() {
        assert_eq!(lookup_key("Chan"), Some("陈"));
    }

    // ── Multi-word token split ────────────────────────────────────────────────
    #[test]
    fn full_name_resolves_via_token_split() {
        // "chan wai ming" → first token "chan" matches
        assert_eq!(lookup_key("Chan Wai Ming"), Some("陈"));
    }

    #[test]
    fn korean_full_name_resolves() {
        assert_eq!(lookup_key("Park Ji-sung"), lookup_key("park"));
    }

    // ── Unknown names ────────────────────────────────────────────────────────
    #[test]
    fn unknown_returns_none() {
        assert_eq!(lookup_key("Smith"), None);
        assert_eq!(lookup_key("Kowalski"), None);
        assert_eq!(lookup_key(""), None);
    }

    #[test]
    fn all_unknown_tokens_returns_none() {
        assert_eq!(lookup_key("John Smith"), None);
    }
}
```

**Step 2: Run tests to verify they fail**

```bash
cd name-variants-rs
cargo test
```
Expected: most tests FAIL with `assertion failed: left == right` (stub returns `None`). The `unknown_returns_none` test PASSES (already correct). That's correct RED state.

**Step 3: Commit RED tests**

```bash
git add name-variants-rs/src/lib.rs
git commit -m "test(rust): failing tests for lookup_key — RED"
```

---

## Task 3: Codegen Script

**Files:**
- Create: `codegen/__init__.py`
- Create: `codegen/gen_rust.py`

The script reads all 15 Python modules, builds the flat `variant → canonical` index, and emits valid Rust with a `phf_map!`. Must be run from the repo root.

**Step 1: Create `codegen/__init__.py`**

```python
# empty
```

**Step 2: Create `codegen/gen_rust.py`**

```python
#!/usr/bin/env python3
"""
Generate name-variants-rs/src/generated.rs from name_variants/*.py tables.

Usage (from repo root):
    python codegen/gen_rust.py

The generated file is committed to source — Rust users do not need Python.
Re-run whenever a Python table is updated.
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).parent.parent
PACKAGE_DIR = REPO_ROOT / "name_variants"
OUTPUT_PATH = REPO_ROOT / "name-variants-rs" / "src" / "generated.rs"

# Module path → variable name holding the dict
TABLE_MODULES: list[tuple[str, str]] = [
    ("chinese_surnames", "CHINESE_SURNAME_VARIANTS"),
    ("arabic_names", "ARABIC_NAME_VARIANTS"),
    ("japanese_surnames", "JAPANESE_SURNAME_VARIANTS"),
    ("korean_surnames", "KOREAN_SURNAME_VARIANTS"),
    ("vietnamese_surnames", "VIETNAMESE_SURNAME_VARIANTS"),
    ("indian_names_hindi", "INDIAN_NAMES_HINDI"),
    ("indian_names_tamil", "INDIAN_NAMES_TAMIL"),
    ("indian_names_bengali", "INDIAN_NAMES_BENGALI"),
    ("persian_names", "PERSIAN_NAME_VARIANTS"),
    ("hebrew_names", "HEBREW_NAME_VARIANTS"),
    ("thai_names", "THAI_NAME_VARIANTS"),
    ("greek_names", "GREEK_NAME_VARIANTS"),
    ("turkish_names", "TURKISH_NAME_VARIANTS"),
    ("russian_surnames", "RUSSIAN_SURNAME_VARIANTS"),
    ("indonesian_malay_names", "INDONESIAN_MALAY_NAME_VARIANTS"),
]


def load_table(module_name: str, var_name: str) -> dict[str, list[str]]:
    """Load a table dict from a name_variants module file."""
    module_path = PACKAGE_DIR / f"{module_name}.py"
    spec = importlib.util.spec_from_file_location(module_name, module_path)
    mod = importlib.util.module_from_spec(spec)  # type: ignore[arg-type]
    spec.loader.exec_module(mod)  # type: ignore[union-attr]
    table = getattr(mod, var_name)
    assert isinstance(table, dict), f"{module_name}.{var_name} is not a dict"
    return table


def build_flat_index(
    tables: list[tuple[str, dict[str, list[str]]]]
) -> dict[str, str]:
    """Build flat {variant_lowercase → canonical_key} index.

    Duplicates (same romanization under multiple entries) keep the first
    occurrence — this mirrors the Python _build_index() behavior.
    """
    index: dict[str, str] = {}
    for _name, table in tables:
        for canonical, variants in table.items():
            # canonical key maps to itself (for direct script-form lookup)
            if canonical not in index:
                index[canonical] = canonical
            for variant in variants:
                v = variant.lower().strip()
                if v and v not in index:
                    index[v] = canonical
    return index


def escape_rust_str(s: str) -> str:
    """Escape a string for use in a Rust string literal."""
    return s.replace("\\", "\\\\").replace('"', '\\"')


def generate(index: dict[str, str]) -> str:
    """Render the Rust generated.rs content."""
    entries = "\n".join(
        f'    "{escape_rust_str(k)}" => "{escape_rust_str(v)}",'
        for k, v in sorted(index.items())
    )
    entry_count = len(index)
    return f"""\
// GENERATED FILE — do not edit by hand.
// Run: python codegen/gen_rust.py
// to regenerate from name_variants/*.py source tables.
//
// Entries: {entry_count}

use phf::phf_map;

pub(crate) static INDEX: phf::Map<&'static str, &'static str> = phf_map! {{
{entries}
}};
"""


def main() -> None:
    tables = []
    for module_name, var_name in TABLE_MODULES:
        table = load_table(module_name, var_name)
        tables.append((module_name, table))
        print(f"  loaded {module_name}: {len(table)} entries", file=sys.stderr)

    index = build_flat_index(tables)
    print(f"  flat index: {len(index)} entries total", file=sys.stderr)

    content = generate(index)
    OUTPUT_PATH.write_text(content, encoding="utf-8")
    print(f"  wrote {OUTPUT_PATH}", file=sys.stderr)


if __name__ == "__main__":
    main()
```

**Step 3: Run the codegen**

```bash
python codegen/gen_rust.py
```
Expected output (stderr):
```
  loaded chinese_surnames: 200+ entries
  loaded arabic_names: 100+ entries
  ...
  flat index: 1000+ entries total
  wrote name-variants-rs/src/generated.rs
```

**Step 4: Inspect the generated file**

```bash
head -30 name-variants-rs/src/generated.rs
wc -l name-variants-rs/src/generated.rs
```
Expected: valid Rust, `phf_map!` with hundreds of entries, no `__placeholder__`.

**Step 5: Verify Rust still compiles**

```bash
cd name-variants-rs && cargo build
```
Expected: compiles (stub still returns None — tests still fail)

**Step 6: Commit**

```bash
git add codegen/ name-variants-rs/src/generated.rs
git commit -m "feat(codegen): gen_rust.py generates phf_map from Python tables"
```

---

## Task 4: Implement `lookup_key` in Rust

**Files:**
- Modify: `name-variants-rs/src/lib.rs`

**Step 1: Replace the stub implementation**

Replace the stub `lookup_key` body in `src/lib.rs`:

```rust
pub fn lookup_key(text: &str) -> Option<&'static str> {
    if text.is_empty() {
        return None;
    }

    // 1. Exact match (handles native script keys: 陈, 박, محمد)
    if let Some(key) = generated::INDEX.get(text) {
        return Some(key);
    }

    // 2. Lowercase match (handles Chan, CHAN, chan)
    let lower = text.to_lowercase();
    if let Some(key) = generated::INDEX.get(lower.as_str()) {
        return Some(key);
    }

    // 3. Token-by-token: "Chan Wai Ming" → try each token
    for token in lower.split_whitespace() {
        if let Some(key) = generated::INDEX.get(token) {
            return Some(key);
        }
    }

    None
}
```

**Step 2: Run the tests**

```bash
cd name-variants-rs && cargo test
```
Expected: all tests PASS

**Step 3: If any test fails, diagnose**

Common issues:
- Token with hyphen: `"Park Ji-sung"` → `split_whitespace` gives `["park", "ji-sung"]` → `"park"` matches on first token. ✓
- Duplicate collision (e.g. `"ng"` under both 黄 and 吴): `lookup_key("ng")` returns whichever was indexed first — same behavior as Python. Acceptable.

**Step 4: Run with output to see all results**

```bash
cd name-variants-rs && cargo test -- --nocapture 2>&1 | head -40
```

**Step 5: Commit GREEN**

```bash
git add name-variants-rs/src/lib.rs
git commit -m "feat(rust): implement lookup_key with exact/lowercase/token-split matching"
```

---

## Task 5: Crate Metadata + doc-tests

**Files:**
- Modify: `name-variants-rs/src/lib.rs`
- Create: `name-variants-rs/README.md`

**Step 1: Verify doc-test in lib.rs compiles and passes**

The doc-test was already added in Task 1:
```rust
/// ```
/// use name_variants::lookup_key;
/// assert_eq!(lookup_key("Chan"), Some("陈"));
/// assert_eq!(lookup_key("Smith"), None);
/// ```
```

```bash
cd name-variants-rs && cargo test --doc
```
Expected: PASSED

**Step 2: Create `README.md`**

````markdown
# name-variants

Multilingual name romanization lookup tables for Rust — Chinese, Japanese, Korean,
Arabic, Vietnamese, Indian, Persian, Hebrew, Thai, Greek, Turkish, Russian, Indonesian/Malay.

Maps romanization variants to their canonical native-script key so that
`Chen`, `Chan`, and `Tan` all resolve to `陈`.

## Usage

```rust
use name_variants::lookup_key;

assert_eq!(lookup_key("Chan"),      Some("陈"));
assert_eq!(lookup_key("chen"),      Some("陈"));   // case-insensitive
assert_eq!(lookup_key("park"),      Some("박"));
assert_eq!(lookup_key("Muhammad"),  Some("محمد"));
assert_eq!(lookup_key("Smith"),     None);

// Multi-word: first matching token wins
assert_eq!(lookup_key("Chan Wai Ming"), Some("陈"));
```

## Install

```toml
[dependencies]
name-variants = "0.1"
```

## Design

- **Canonical key is always the native script form** — `陈`, `박`, `محمد`
- **Zero runtime dependencies** — all data in a compile-time perfect hash map (`phf`)
- **Case-insensitive** — input is lowercased before lookup
- **`None` for unknowns** — explicit signal to fall back to fuzzy matching
````

**Step 3: Run all tests one final time**

```bash
cd name-variants-rs && cargo test && cargo test --doc
```
Expected: all PASSED

**Step 4: Commit**

```bash
git add name-variants-rs/README.md name-variants-rs/src/lib.rs
git commit -m "docs(rust): README and doc-test for lookup_key"
```

---

## Task 6: CI Drift Check

**Files:**
- Modify: `.github/workflows/ci.yml` (create if absent)

The drift check runs `python codegen/gen_rust.py` and fails if `generated.rs` changes — ensuring the Rust tables stay in sync with the Python tables.

**Step 1: Create / update `.github/workflows/ci.yml`**

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:

jobs:
  python-tests:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        python-version: ["3.11", "3.12", "3.13"]
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with:
          python-version: ${{ matrix.python-version }}
      - run: pip install -e ".[dev]"
      - run: pytest tests/ -v

  rust-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --manifest-path name-variants-rs/Cargo.toml
      - run: cargo test --doc --manifest-path name-variants-rs/Cargo.toml

  codegen-drift:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with:
          python-version: "3.12"
      - run: python codegen/gen_rust.py
      - name: Fail if generated.rs changed
        run: |
          if ! git diff --exit-code name-variants-rs/src/generated.rs; then
            echo "ERROR: name-variants-rs/src/generated.rs is out of sync."
            echo "Run 'python codegen/gen_rust.py' and commit the result."
            exit 1
          fi
```

**Step 2: Commit**

```bash
mkdir -p .github/workflows
git add .github/workflows/ci.yml
git commit -m "ci: Python tests, Rust tests, and codegen drift check"
```

---

## Task 7: Publish to crates.io

**Step 1: Verify the crate passes `cargo publish --dry-run`**

```bash
cd name-variants-rs
cargo publish --dry-run
```
Expected: `Packaged X files` with no errors

**Step 2: Tag and publish**

```bash
git tag rust-v0.1.0
git push origin rust-v0.1.0

cd name-variants-rs
cargo publish
```

**Step 3: Verify on crates.io**

```bash
cargo add name-variants
# in a scratch project:
cargo test
```

**Step 4: Update edng's `Cargo.toml`**

In `~/src/edng/Cargo.toml` (or the relevant workspace member):
```toml
[dependencies]
name-variants = "0.1"
```

Replace any inline name resolution logic with:
```rust
use name_variants::lookup_key;

fn same_name_key(a: &str, b: &str) -> bool {
    match (lookup_key(a), lookup_key(b)) {
        (Some(ka), Some(kb)) => ka == kb,
        _ => false,
    }
}
```

**Step 5: Commit edng change**

```bash
cd ~/src/edng
git add Cargo.toml Cargo.lock
git commit -m "feat: use name-variants crate for cross-script name resolution"
```
