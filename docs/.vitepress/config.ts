import { defineConfig } from 'vitepress'
import { en } from './en'
import { zh } from './zh'

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: "IndustryDB",
  description: "由 Rust 和 Polars 驱动的高性能数据库中间件",
  
  // Base URL for deployment
  base: '/',
  
  // Clean URLs without .html extension
  cleanUrls: true,
  
  // Last update timestamp
  lastUpdated: true,
  
  // Markdown configuration
  markdown: {
    theme: {
      light: 'github-light',
      dark: 'github-dark'
    },
    lineNumbers: true
  },

  // Multi-language support - 默认中文
  locales: {
    root: {
      label: '简体中文',
      lang: 'zh-CN',
      ...zh
    },
    en: {
      label: 'English',
      lang: 'en-US',
      ...en
    }
  },

  // Theme configuration
  themeConfig: {
    logo: '/logo.svg',
    
    // Social links
    socialLinks: [
      { icon: 'github', link: 'https://github.com/yourusername/industrydb' }
    ],

    // Footer
    footer: {
      message: 'Released under the MIT License.',
      copyright: 'Copyright © 2024-present IndustryDB Contributors'
    },

    // Search
    search: {
      provider: 'local'
    }
  },

  // Head tags
  head: [
    ['link', { rel: 'icon', type: 'image/svg+xml', href: '/logo.svg' }],
    ['meta', { name: 'theme-color', content: '#5f67ee' }],
    ['meta', { property: 'og:type', content: 'website' }],
    ['meta', { property: 'og:locale', content: 'zh-CN' }],
    ['meta', { property: 'og:title', content: 'IndustryDB | 高性能数据库中间件' }],
    ['meta', { property: 'og:site_name', content: 'IndustryDB' }]
  ]
})
