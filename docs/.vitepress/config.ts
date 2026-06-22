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
