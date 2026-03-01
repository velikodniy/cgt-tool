"""
Shared helper for running `cgt-tool parse` and returning structured JSON.

Locates the cgt-tool binary using the same strategy as cross-validate.py:
  1. Local release build (target/release/cgt-tool)
  2. PATH lookup
  3. Fallback to `cargo run`
"""

import json
import subprocess
from pathlib import Path


def find_cgt_tool() -> list[str]:
    """Return command tokens for invoking cgt-tool."""
    release_bin = Path.cwd() / "target" / "release" / "cgt-tool"
    if release_bin.exists():
        return [str(release_bin)]
    if subprocess.run(["which", "cgt-tool"], capture_output=True).returncode == 0:
        return ["cgt-tool"]
    return ["cargo", "run", "--quiet", "--"]


def run_parse(path: Path) -> list[dict]:
    """Run `cgt-tool parse <path>` and return the JSON transaction list."""
    cmd = find_cgt_tool() + ["parse", str(path)]
    result = subprocess.run(cmd, capture_output=True, text=True, timeout=120)
    if result.returncode != 0:
        raise RuntimeError(f"cgt-tool parse failed: {result.stderr.strip()}")
    return json.loads(result.stdout)
