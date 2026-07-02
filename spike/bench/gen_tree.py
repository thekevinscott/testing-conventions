#!/usr/bin/env python3
"""Generate the deterministic sample docs tree (~30 md files, ~300KB)."""

from pathlib import Path

ROOT = Path(__file__).parent / "tree"

PARA = (
    "The quick brown fox jumps over the lazy dog. "
    "Pack my box with five dozen liquor jugs. "
    "How vexingly quick daft zebras jump.\n\n"
)

def main():
    for section in ("guide", "reference", "internals"):
        for i in range(10):
            p = ROOT / section / f"doc-{i:02d}.md"
            p.parent.mkdir(parents=True, exist_ok=True)
            body = f"# {section} doc {i:02d}\n\n" + PARA * (40 + i * 5)
            p.write_text(body)
    total = sum(f.stat().st_size for f in ROOT.rglob("*.md"))
    print(f"{len(list(ROOT.rglob('*.md')))} files, {total} bytes")

if __name__ == "__main__":
    main()
