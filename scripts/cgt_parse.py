"""
Shared helper for running `cgt-tool parse` and returning structured JSON.

CGT_TOOL_BIN selects an explicit binary. Otherwise, use the newest local build
or use cargo with this repository's manifest.
"""

import json
import os
import subprocess
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent


def find_cgt_tool() -> list[str]:
    """Return command tokens for invoking cgt-tool."""
    if binary := os.environ.get("CGT_TOOL_BIN"):
        return [binary]
    name = "cgt-tool.exe" if os.name == "nt" else "cgt-tool"
    candidates = [REPO / "target" / profile / name for profile in ("debug", "release")]
    builds = [path for path in candidates if path.is_file()]
    if builds:
        return [str(max(builds, key=lambda path: path.stat().st_mtime_ns))]
    return ["cargo", "run", "--quiet", "--manifest-path", str(REPO / "Cargo.toml"), "--"]


def run_parse(path: Path) -> list[dict]:
    """Run `cgt-tool parse <path>` and return the JSON transaction list."""
    cmd = find_cgt_tool() + ["parse", str(path)]
    result = subprocess.run(cmd, capture_output=True, text=True, timeout=120)
    if result.returncode != 0:
        raise RuntimeError(f"cgt-tool parse failed: {result.stderr.strip()}")
    return json.loads(result.stdout)
