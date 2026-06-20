import { defineConfig } from 'vitepress'

export default defineConfig({
  // Deployed to GitHub Pages at https://thekevinscott.github.io/testing-conventions/,
  // so assets must be served from the repo subpath, not the domain root. Without this,
  // every CSS/JS/font URL resolves to /assets/... (404) and the site renders unstyled.
  base: '/testing-conventions/',
  title: 'testing-conventions',
  description: 'One config-driven standard for how tests are structured, isolated, and measured across Python, TypeScript, and Rust.',
  cleanUrls: true,
  themeConfig: {
    nav: [
      { text: 'Getting Started', link: '/getting-started' },
      { text: 'Guides', link: '/guide/' },
      { text: 'Reference', link: '/reference/' },
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
            { text: 'Isolate tests', link: '/guide/isolation' },
            { text: 'Exempt a file', link: '/guide/exemptions' },
            { text: 'Enforce conventions in CI', link: '/guide/ci' },
            { text: 'Mutation testing', link: '/guide/mutation' },
          ],
        },
        {
          text: 'Reference',
          items: [
            { text: 'API', link: '/reference/' },
            { text: 'Defaults', link: '/reference/defaults' },
          ],
        },
      ],
    },
    search: { provider: 'local' },
    outline: [2, 3],
  },
})
