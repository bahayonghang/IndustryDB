import { defineConfig, type DefaultTheme } from 'vitepress'

export const en = defineConfig({
  lang: 'en-US',
  description: 'High-performance database middleware powered by Rust and Polars',

  themeConfig: {
    nav: nav(),
    sidebar: sidebar(),
    
    editLink: {
      pattern: 'https://github.com/yourusername/industrydb/edit/main/docs/:path',
      text: 'Edit this page on GitHub'
    },

    footer: {
      message: 'Released under the MIT License.',
      copyright: 'Copyright © 2024-present IndustryDB Contributors'
    },

    docFooter: {
      prev: 'Previous page',
      next: 'Next page'
    },

    outline: {
      label: 'On this page'
    },

    lastUpdated: {
      text: 'Last updated',
      formatOptions: {
        dateStyle: 'short',
        timeStyle: 'short'
      }
    }
  }
})

function nav(): DefaultTheme.NavItem[] {
  return [
    {
      text: 'Guide',
      link: '/en/guide/what-is-industrydb',
      activeMatch: '/en/guide/'
    },
    {
      text: 'API Reference',
      link: '/en/api/connection',
      activeMatch: '/en/api/'
    },
    {
      text: 'Examples',
      link: '/en/examples/quick-start',
      activeMatch: '/en/examples/'
    },
    {
      text: 'About',
      items: [
        { text: 'Architecture', link: '/en/about/architecture' },
        { text: 'Contributing', link: '/en/about/contributing' },
        { text: 'Changelog', link: '/en/about/changelog' }
      ]
    }
  ]
}

function sidebar(): DefaultTheme.Sidebar {
  return {
    '/en/guide/': {
      base: '/en/guide/',
      items: [
        {
          text: 'Introduction',
          collapsed: false,
          items: [
            { text: 'What is IndustryDB?', link: 'what-is-industrydb' },
            { text: 'Getting Started', link: 'getting-started' },
            { text: 'Features', link: 'features' }
          ]
        },
        {
          text: 'Core Concepts',
          collapsed: false,
          items: [
            { text: 'Connection Management', link: 'connection' },
            { text: 'Configuration', link: 'configuration' },
            { text: 'CRUD Operations', link: 'crud-operations' },
            { text: 'DataFrame Integration', link: 'dataframe' }
          ]
        },
        {
          text: 'Database Support',
          collapsed: false,
          items: [
            { text: 'PostgreSQL', link: 'postgres' },
            { text: 'SQLite', link: 'sqlite' },
            { text: 'MSSQL', link: 'mssql' }
          ]
        },
        {
          text: 'Advanced',
          collapsed: false,
          items: [
            { text: 'Performance Optimization', link: 'performance' },
            { text: 'Error Handling', link: 'error-handling' },
            { text: 'Type Safety', link: 'type-safety' }
          ]
        }
      ]
    },
    '/en/api/': {
      base: '/en/api/',
      items: [
        {
          text: 'API Reference',
          items: [
            { text: 'Connection', link: 'connection' },
            { text: 'DatabaseConfig', link: 'config' },
            { text: 'CRUD Methods', link: 'crud' },
            { text: 'Utilities', link: 'utilities' },
            { text: 'Exceptions', link: 'exceptions' }
          ]
        }
      ]
    },
    '/en/examples/': {
      base: '/en/examples/',
      items: [
        {
          text: 'Examples',
          items: [
            { text: 'Quick Start', link: 'quick-start' },
            { text: 'Configuration', link: 'configuration' },
            { text: 'Basic CRUD', link: 'basic-crud' },
            { text: 'Advanced Queries', link: 'advanced-queries' },
            { text: 'DataFrame Operations', link: 'dataframe-operations' },
            { text: 'Multiple Databases', link: 'multiple-databases' }
          ]
        }
      ]
    },
    '/en/about/': {
      base: '/en/about/',
      items: [
        {
          text: 'About',
          items: [
            { text: 'Architecture', link: 'architecture' },
            { text: 'Contributing', link: 'contributing' },
            { text: 'Changelog', link: 'changelog' },
            { text: 'License', link: 'license' }
          ]
        }
      ]
    }
  }
}

export const search: DefaultTheme.AlgoliaSearchOptions['locales'] = {
  root: {
    placeholder: 'Search docs',
    translations: {
      button: {
        buttonText: 'Search',
        buttonAriaLabel: 'Search'
      },
      modal: {
        searchBox: {
          resetButtonTitle: 'Clear',
          resetButtonAriaLabel: 'Clear',
          cancelButtonText: 'Cancel',
          cancelButtonAriaLabel: 'Cancel'
        },
        startScreen: {
          recentSearchesTitle: 'Recent',
          noRecentSearchesText: 'No recent searches',
          saveRecentSearchButtonTitle: 'Save',
          removeRecentSearchButtonTitle: 'Remove',
          favoriteSearchesTitle: 'Favorites',
          removeFavoriteSearchButtonTitle: 'Remove'
        },
        errorScreen: {
          titleText: 'Unable to fetch results',
          helpText: 'Check your network connection'
        },
        footer: {
          selectText: 'to select',
          navigateText: 'to navigate',
          closeText: 'to close'
        },
        noResultsScreen: {
          noResultsText: 'No results for',
          suggestedQueryText: 'Try searching for',
          reportMissingResultsText: 'Missing results?',
          reportMissingResultsLinkText: 'Let us know'
        }
      }
    }
  }
}
