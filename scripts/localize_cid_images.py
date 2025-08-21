#!/usr/bin/env python3
"""
Walk all message.html files output by outlook-pst-cli and rewrite <img src="cid:..."> to
refer to a local file named after the content-id, saving as local_message.html next to the original.

Usage:
  python3 scripts/localize_cid_images.py <ROOT>

Notes:
  - The content-id is taken as-is after stripping optional surrounding angle brackets.
  - The replacement keeps only the filename (no directory), so the images are expected to be alongside message.html.
  - The script logs basic stats and warnings if no matching file is found (non-fatal).
"""
from __future__ import annotations

import argparse
import sys
from pathlib import Path
from typing import Iterable, List

try:
    from bs4 import BeautifulSoup  # type: ignore
except Exception as e:  # pragma: no cover - clear error for missing dep
    sys.stderr.write(
        "BeautifulSoup (bs4) is required. Install with: pip install beautifulsoup4\n"
    )
    raise


def strip_angle_brackets(s: str) -> str:
    s = s.strip()
    if s.startswith("<") and s.endswith(">"):
        return s[1:-1]
    return s


def process_file(path: Path) -> int:
    html = path.read_text(encoding="utf-8", errors="replace")
    soup = BeautifulSoup(html, "html.parser")

    changed = 0
    parent_dir = path.parent

    for img in soup.find_all("img"):
        src = img.get("src")
        if not src:
            continue
        if not src.lower().startswith("cid:"):
            continue

        cid_raw = src[4:]
        cid = strip_angle_brackets(cid_raw)


        # Optional: warn if the file is not present locally
        candidate = parent_dir / cid
        alt_candidate = parent_dir / cid.split("@")[0]  # Handle cases like "cid:png@456
        if candidate.exists():
            # Set src to the local filename (same directory as message.html)
            img["src"] = cid
            changed += 1
        elif alt_candidate.exists():
            img["src"] = cid.split("@")[0]  # Use the part before @
            changed += 1
        else:
            # Non-fatal; just inform via stderr
            sys.stderr.write(f"[warn] Missing local file for content-id '{cid}' in {parent_dir}\n")

    if changed:
        path.rename(parent_dir / "original_message.html")
        out_path = parent_dir / "message.html"
        out_path.write_text(soup.prettify("utf-8").decode("utf-8"), encoding="utf-8")
    return changed


def main(argv: List[str]) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "rootpath",
        default=".",
        help="Root directory to search recursively for message.html (default: .)",
    )
    args = ap.parse_args(argv)

    total_files = 0
    total_imgs = 0

    for msg_path in Path(args.rootpath).rglob("message.html"):
        total_files += 1
        changed = process_file(msg_path)
        total_imgs += changed

    print(
        f"Processed {total_files} message.html file(s); updated {total_imgs} img(s) with cid: sources.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
