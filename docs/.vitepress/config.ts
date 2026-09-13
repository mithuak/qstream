import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'qstream',
  description: 'Streaming finance and signal-processing features with a Rust core and Python API.',
  cleanUrls: true,
  srcExclude: ['README.md'],
  ignoreDeadLinks: false,
  themeConfig: {
    nav: [
      { text: 'Home', link: '/' },
      { text: 'Quickstart', link: '/quickstart' },
      { text: 'API', link: '/api/' },
      { text: 'Workbook', link: '/workbook' },
    ],
    sidebar: [
      { text: 'Start here', items: [
        { text: 'Overview', link: '/' },
        { text: 'Python quickstart', link: '/quickstart' },
        { text: 'Cookbook', link: '/cookbook' },
      ] },
      { text: 'Concepts', items: [
        { text: 'Streaming and warmup', link: '/streaming' },
        { text: 'Results and cadence', link: '/results-and-cadence' },
        { text: 'Architecture', link: '/architecture' },
      ] },
      { text: 'Reference', items: [
        { text: 'Python API', link: '/api/' },
        { text: 'Workbook map', link: '/workbook' },
      ] },
    ],
    search: { provider: 'local' },
    footer: { message: 'qstream documentation', copyright: 'MIT OR Apache-2.0' },
  },
})
