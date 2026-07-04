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
      { text: 'Monorepo', link: '/monorepo' },
      { text: 'Configure', link: '/guide/configure' },
      { text: 'Reference', link: '/reference/workflow' },
      { text: 'Explanation', link: '/explanation/' },
    ],
    sidebar: {
      '/': [
        {
          text: 'Tutorials',
          items: [
            { text: 'Getting Started', link: '/getting-started' },
            { text: 'Adopt on a monorepo', link: '/monorepo' },
          ],
        },
        {
          text: 'How-to Guides',
          items: [
            { text: 'Configure the rules', link: '/guide/configure' },
          ],
        },
        {
          text: 'Reference',
          items: [
            { text: 'Workflow', link: '/reference/workflow' },
            { text: 'Configuration', link: '/reference/config' },
          ],
        },
        {
          text: 'Explanation',
          items: [
            { text: 'The testing model', link: '/explanation/' },
            { text: 'Colocated tests', link: '/explanation/colocated-test' },
            { text: 'Coverage', link: '/explanation/coverage' },
            { text: 'Mutation', link: '/explanation/mutation' },
            { text: 'Isolation', link: '/explanation/isolation' },
            { text: 'Packaging', link: '/explanation/packaging' },
            { text: 'E2E attestation', link: '/explanation/e2e' },
            { text: 'Scoping and exemptions', link: '/explanation/scoping' },
          ],
        },
      ],
    },
    search: { provider: 'local' },
    outline: [2, 3],
  },
})
