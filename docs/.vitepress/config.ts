import { defineConfig } from 'vitepress'
import llmstxt from 'vitepress-plugin-llms'

export default defineConfig({
  // Deployed to GitHub Pages at https://thekevinscott.github.io/testing-conventions/,
  // so assets must be served from the repo subpath, not the domain root. Without this,
  // every CSS/JS/font URL resolves to /assets/... (404) and the site renders unstyled.
  base: '/testing-conventions/',
  title: 'testing-conventions',
  description: 'One config-driven standard for how tests are structured, isolated, and measured across Python, TypeScript, and Rust.',
  cleanUrls: true,
  vite: {
    plugins: [
      // Emit an agent-facing entry point alongside the HTML build, per the
      // llmstxt.org standard: `llms.txt` (a link-rich index of every page) and
      // `llms-full.txt` (the whole docs concatenated as one markdown file).
      // Generated from these same pages at build time, so the agent digest
      // tracks the docs automatically — no second, hand-maintained corpus to
      // drift out of sync (#220).
      llmstxt({
        // Absolute URLs so an agent that fetched llms.txt from anywhere can
        // resolve every link; the `base` subpath above is appended after it.
        domain: 'https://thekevinscott.github.io',
        // AGENTS.md is the docs-authoring conventions for *contributors*; the
        // agent digest is for consumers *using* the shipped tool (#220). Keep
        // them distinct — different audience, different file.
        ignoreFiles: ['AGENTS.md'],
      }),
    ],
  },
  themeConfig: {
    nav: [
      { text: 'Getting Started', link: '/getting-started' },
      { text: 'How-to Guides', link: '/guide/' },
      { text: 'Reference', link: '/reference/' },
      { text: 'Explanation', link: '/explanation/' },
    ],
    sidebar: {
      '/': [
        {
          text: 'Tutorial',
          items: [
            { text: 'Getting Started', link: '/getting-started' },
          ],
        },
        {
          text: 'How-to Guides',
          items: [
            { text: 'Overview', link: '/guide/' },
            { text: 'Configure the rules', link: '/guide/configure' },
            { text: 'Extend the defaults', link: '/guide/extending' },
            { text: 'Isolate tests', link: '/guide/isolation' },
            { text: 'Run mutation testing', link: '/guide/mutation' },
            { text: 'Enforce conventions in CI', link: '/guide/ci' },
            { text: 'Use the CLI directly', link: '/guide/cli' },
          ],
        },
        {
          text: 'Reference',
          items: [
            { text: 'API', link: '/reference/' },
            { text: 'Defaults', link: '/reference/defaults' },
          ],
        },
        {
          text: 'Explanation',
          items: [
            { text: 'The testing model', link: '/explanation/' },
            { text: 'Why mutation testing', link: '/explanation/mutation' },
          ],
        },
      ],
    },
    search: { provider: 'local' },
    outline: [2, 3],
  },
})
