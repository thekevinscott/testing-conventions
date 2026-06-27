---
name: fetch-url
description: Capture any URL the user provides into sources/<slug>/ as raw bytes (raw.html or raw.pdf), a total transcript (transcript.md) extracted from those bytes, and a summary (summary.md) extracted from the transcript. Use whenever a URL is provided — when the user drops, pastes, shares, or asks to fetch / capture / save / read / research / archive a link, or when a research note contains URLs that need capturing. Falls back to the agent-browser skill when a plain fetch is blocked.
allowed-tools: Bash, Read, Write, Skill
---

# fetch-url

Whenever a URL shows up — the user pastes one, asks to capture/save/read/research a
link, or a note on disk lists URLs — capture it into `sources/<slug>/`.

The pipeline has three stages, run in order:

```
URL ──fetch──▶ raw.html / raw.pdf ──extract──▶ transcript.md ──summarise──▶ summary.md
     (script)                       (script)                    (you, the agent)
```

The first two stages are mechanical and handled by the bundled script. The third
(the summary) is a reasoning task **you** do, following the research discipline in
`notes/research/agents.md`.

## 1. Fetch + transcript (the script)

Run, once per URL, from the directory whose `sources/` folder should hold the capture
(the research notes dir — `sources/` is resolved relative to the current working dir):

```bash
uv run ${CLAUDE_SKILL_DIR}/fetch.py <url>
```

It computes a deterministic slug (the URL minus scheme/trailing-slash, every non
`[A-Za-z0-9._-]` char → `-`; one folder per exact URL), creates `sources/<slug>/`, and writes:

| File | Always? | Contents |
|---|---|---|
| `raw.html` *or* `raw.pdf` | yes | raw fetched bytes (PDF when the response is a PDF) |
| `transcript.md` | yes | YAML header (`url`, `title`, `fetched`, `raw`, `transport`, `capture_status`) + the **total** transcript — full body for HTML, full text for PDFs |

Fetch strategy, each step falling back to the next:

1. **curl** with a Chrome UA — fast; handles arxiv, blogs, most static pages and PDFs.
2. **agent-browser CLI** (real Chrome over CDP) — auto-tried when curl returns 4xx/5xx
   or a bot-wall (Cloudflare "Just a moment", captcha, etc.).
3. On a hard block the script writes a `FETCH FAILED` placeholder transcript and exits
   `3`, leaving `capture_status: blocked` in the header.

Check the exit code / `capture_status`. Exit `0` = good. Exit `3` = blocked or empty.

### When the fetch is blocked — use /agent-browser

If the script exits `3` (paywall / IP block / JS-gated page — common on dl.acm.org,
springer, sciencedirect, researchgate), recover with the **agent-browser** skill:

1. Invoke the `agent-browser` skill to open the URL in a real browser and extract the
   rendered page HTML (or the readable article text).
2. Save that HTML to `sources/<slug>/raw.html` (use `--slug <url>` to print the exact path).
3. Re-extract: `uv run ${CLAUDE_SKILL_DIR}/fetch.py --transcript <url>`.

If even agent-browser is blocked at the network layer, don't fight the bot detection —
say so and ask the user to paste the body into `raw.html`, then re-run `--transcript`.
For paywalled academic sources, an abstract-only capture is acceptable — record it and
mark the transcript `capture_status` accordingly.

## 2. Summary (you, the agent)

Read `sources/<slug>/transcript.md` and write `sources/<slug>/summary.md`. This is
research organisation, not capture — follow `notes/research/agents.md` exactly:

- Header: source **title, authors, URL, date, and a source-type tag** —
  e.g. `(peer-reviewed study)`, `(arXiv preprint)`, `(blog)`, `(industry report)`.
- Summarise what the source **claims**. Record exact numbers and key quotes **verbatim**
  (quote exactly; never fabricate a number, quote, or citation).
- Note the **method / evidence type**, scope, limitations, and any gaps or uncertainties.
- **Organise, don't synthesise.** Do not recommend, rank, predict, adjudicate between
  sources, or merge them into a conclusion of your own. Report disagreement; don't resolve it.
- If the transcript is partial/blocked, say so explicitly and summarise only what was captured.

## 3. Report back

Tell the user the slug, the transport used (curl / agent-browser), the `capture_status`,
and the three files written. Surface any blocked captures so they know which need a
manual paste.

## Batch mode

For many URLs, run the script once per URL (it is idempotent — re-running overwrites the
prior capture for the same slug), then write each `summary.md`. To capture every URL
already on disk, collect them first, e.g.:

```bash
grep -ohrE 'https?://[^ )<>"]+' *.md | sed 's/[.,]$//' | sort -u
```
