import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'testing-conventions',
  description: 'One config-driven, deterministic standard for how tests are structured, isolated, and measured — across Python, TypeScript, and Rust.',
  cleanUrls: true,
  themeConfig: {
    nav: [
      { text: 'Getting Started', link: '/getting-started' },
      { text: 'Guides', link: '/guide/' },
      { text: 'Reference', link: '/reference/' },
      { text: 'Migrations', link: '/migrations' },
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
          ],
        },
        {
          text: 'Reference',
          items: [
            { text: 'API', link: '/reference/' },
            { text: 'Migrations', link: '/migrations' },
          ],
        },
      ],
    },
    search: { provider: 'local' },
    outline: [2, 3],
  },
})
