#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = [
#     "markdownify>=0.13",
#     "beautifulsoup4>=4.12",
# ]
# ///
"""Capture a single URL into sources/<slug>/ as raw bytes + a total transcript.

Pipeline (deterministic, mechanical — no judgement):

    URL ──curl──▶ raw.html / raw.pdf ──extract──▶ transcript.md

The *summary* step (transcript.md ──▶ summary.md) is NOT done here. Summarising
is a reasoning task the calling agent performs, following the research rules in
notes/research/agents.md. This script only does the fetch + transcript.

Usage:
    uv run fetch.py <url>                  # fetch + transcript
    uv run fetch.py --slug <url>           # print the slug/dir for <url> and exit
    uv run fetch.py --transcript <url>     # re-extract transcript from existing raw file
                                           #   (use after /agent-browser saved raw.html)

Fetch strategy (each step falls back to the next):
    1. curl with a Chrome UA (fast; works for arxiv, blogs, most static pages)
    2. agent-browser CLI (real Chrome over CDP) when curl is blocked / JS-gated
    3. on hard block, write a placeholder transcript + exit 3 so the agent surfaces it
       and can fall back to the /agent-browser skill or a manual paste.

Slug rule (deterministic, collision-free, reversible): strip scheme + trailing
slash, then replace every char outside [A-Za-z0-9._-] with '-', capped at 120
characters with a SHA-256 suffix past that. One folder per exact URL, and
transcript.md's `url:` frontmatter carries the exact URL.
"""
from __future__ import annotations

import hashlib
import re
import subprocess
import sys
import time
import urllib.parse
from pathlib import Path

from bs4 import BeautifulSoup
from markdownify import markdownify as md_convert

SOURCES = Path.cwd() / "sources"

CHROME_UA = (
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 "
    "(KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
)

# Markers that mean "you got a bot-wall, not the content".
BLOCK_MARKERS = (
    "just a moment",
    "enable javascript and cookies",
    "captcha",
    "cf-browser-verification",
    "access denied",
    "are you a robot",
)


# ----- slug -----------------------------------------------------------------

# Raising SLUG_MAX past 120 pushes a capture's deepest path over Windows's 260-character
# MAX_PATH, and `actions/checkout` then fails the release with "Filename too long".
SLUG_MAX = 120


def compute_slug(url: str) -> str:
    u = re.sub(r"^https?://", "", url.strip())
    u = u.rstrip("/")
    slug = re.sub(r"[^a-zA-Z0-9._-]", "-", u)
    if len(slug) <= SLUG_MAX:
        return slug
    digest = hashlib.sha256(url.strip().encode()).hexdigest()[:8]
    return f"{slug[: SLUG_MAX - 9].rsplit('-', 1)[0]}-{digest}"


# ----- fetch ----------------------------------------------------------------

def curl_fetch(url: str, dest: Path) -> tuple[int, str]:
    """curl <url> into dest. Return (http_code, content_type)."""
    fmt = "%{http_code}\t%{content_type}"
    r = subprocess.run(
        [
            "curl", "-sSL", "--max-time", "60", "--compressed",
            "-A", CHROME_UA,
            "-H", "Accept: text/html,application/xhtml+xml,application/pdf,*/*",
            "-o", str(dest), "-w", fmt, url,
        ],
        capture_output=True, text=True, timeout=90,
    )
    code, _, ctype = (r.stdout or "\t").partition("\t")
    try:
        return int(code), ctype.strip()
    except ValueError:
        return 0, ctype.strip()


def agent_browser_fetch(url: str) -> str:
    """Render <url> in agent-browser (real Chrome) and return the HTML string."""
    def ab(*args, timeout=60):
        return subprocess.run(
            ["agent-browser", *args], capture_output=True, text=True, timeout=timeout
        )

    open_r = ab("open", url, timeout=90)
    if open_r.returncode != 0:
        return ""
    ab("wait", "--load", "networkidle", timeout=25)
    html_r = ab("get", "html", "html", timeout=30)
    return html_r.stdout or ""


def looks_blocked(http_code: int, raw: bytes) -> bool:
    if http_code == 0 or http_code >= 400:
        return True
    if not raw:
        return True
    head = raw[:4000].decode("utf-8", errors="replace").lower()
    return any(m in head for m in BLOCK_MARKERS)


# ----- transcript extraction ------------------------------------------------

def html_to_transcript(raw: bytes) -> tuple[str, str]:
    """Whole-page HTML → (title, markdown). Strives for a TOTAL transcript:
    only non-content tags are stripped; the full body is converted."""
    s = raw.decode("utf-8", errors="replace")
    soup = BeautifulSoup(s, "html.parser")
    title = soup.title.string.strip() if (soup.title and soup.title.string) else ""
    for tag in soup(["script", "style", "noscript", "svg", "canvas", "iframe", "form"]):
        tag.decompose()
    root = soup.body or soup
    md = md_convert(str(root), heading_style="ATX")
    md = re.sub(r"\n{3,}", "\n\n", md).strip()
    return title, md


def pdf_to_transcript(pdf_path: Path) -> tuple[str, str]:
    """PDF → (title, plaintext) via pdftotext (-layout preserves reading order)."""
    r = subprocess.run(
        ["pdftotext", "-layout", "-enc", "UTF-8", str(pdf_path), "-"],
        capture_output=True, text=True, timeout=120,
    )
    text = (r.stdout or "").strip()
    title = ""
    for line in text.splitlines():
        if line.strip():
            title = line.strip()
            break
    return title, text


def is_pdf(http_ctype: str, url: str, raw: bytes) -> bool:
    if "pdf" in http_ctype.lower():
        return True
    if url.lower().rstrip("/").endswith(".pdf") or "/pdf/" in url.lower():
        return True
    return raw[:5] == b"%PDF-"


# ----- output ---------------------------------------------------------------

def write_transcript(outdir: Path, url: str, title: str, body: str,
                     raw_file: str, transport: str, status: str) -> None:
    header = "\n".join([
        "---",
        f"url: {url}",
        f"title: {title or '(none)'}",
        f"fetched: {time.strftime('%Y-%m-%d')}",
        f"raw: {raw_file}",
        f"transport: {transport}",
        f"capture_status: {status}",
        "---",
        "",
        "",
    ])
    (outdir / "transcript.md").write_text(header + body + "\n")


def transcript_only(url: str) -> int:
    """Re-extract transcript.md from an already-saved raw file (raw.html / raw.pdf)."""
    outdir = SOURCES / compute_slug(url)
    pdf, html = outdir / "raw.pdf", outdir / "raw.html"
    if pdf.exists():
        title, body = pdf_to_transcript(pdf)
        write_transcript(outdir, url, title, body, "raw.pdf", "re-extract", "ok")
    elif html.exists():
        title, body = html_to_transcript(html.read_bytes())
        status = "ok" if body.strip() else "empty — page had no extractable text"
        write_transcript(outdir, url, title, body, "raw.html", "re-extract", status)
    else:
        print(f"no raw.html / raw.pdf in {outdir}", file=sys.stderr)
        return 2
    print(f"  re-extracted {outdir / 'transcript.md'}")
    return 0


def capture(url: str) -> int:
    outdir = SOURCES / compute_slug(url)
    outdir.mkdir(parents=True, exist_ok=True)
    print(f"[{outdir.name}] {url}")

    code, ctype = curl_fetch(url, outdir / ".tmp")
    raw = (outdir / ".tmp").read_bytes() if (outdir / ".tmp").exists() else b""

    if is_pdf(ctype, url, raw) and not looks_blocked(code, raw[:5] and b"%PDF-" or raw):
        # PDF path — curl is enough; agent-browser can't help with a PDF byte stream.
        (outdir / ".tmp").rename(outdir / "raw.pdf")
        title, body = pdf_to_transcript(outdir / "raw.pdf")
        status = "ok" if body.strip() else "empty — pdftotext found no text (scanned?)"
        write_transcript(outdir, url, title, body, "raw.pdf", "curl", status)
        print(f"  -> raw.pdf + transcript.md ({status})")
        return 0 if body.strip() else 3

    transport = "curl"
    if looks_blocked(code, raw):
        print(f"  curl blocked (http {code}); trying agent-browser…", file=sys.stderr)
        rendered = agent_browser_fetch(url)
        if rendered:
            raw = rendered.encode("utf-8", errors="replace")
            transport = "agent-browser"

    (outdir / "raw.html").write_bytes(raw)
    (outdir / ".tmp").unlink(missing_ok=True)

    if not raw or looks_blocked(0 if transport == "agent-browser" else code, raw):
        write_transcript(
            outdir, url, "(fetch failed)",
            f"FETCH FAILED — curl http {code}, agent-browser fallback "
            f"{'also blocked' if transport == 'agent-browser' else 'not reached'}.\n\n"
            "Raw response saved to `raw.html`. Recover with the /agent-browser skill "
            "(interactive render) or paste the page body into `raw.html` and re-run "
            "`uv run fetch.py --transcript <url>`.\n",
            "raw.html", transport, f"blocked — http {code}",
        )
        print(f"  BLOCKED (http {code}). See transcript.md for recovery steps.", file=sys.stderr)
        return 3

    title, body = html_to_transcript(raw)
    status = "ok" if body.strip() else "empty — page had no extractable text"
    write_transcript(outdir, url, title, body, "raw.html", transport, status)
    print(f"  -> raw.html + transcript.md (transport={transport}, {status})")
    return 0 if body.strip() else 3


def main() -> None:
    args = sys.argv[1:]
    if not args:
        print(__doc__)
        sys.exit(2)
    if args[0] == "--slug":
        url = args[1]
        slug = compute_slug(url)
        print(f"slug: {slug}")
        print(f"dir:  {SOURCES / slug}")
        return
    if args[0] == "--transcript":
        sys.exit(transcript_only(args[1]))
    sys.exit(capture(args[0]))


if __name__ == "__main__":
    main()
