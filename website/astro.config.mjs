// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

// https://astro.build/config
export default defineConfig({
  site: 'https://flaglite.dev',
  integrations: [
    starlight({
      title: 'FlagLite',
      logo: {
        light: './src/assets/logo-light.svg',
        dark: './src/assets/logo-dark.svg',
        replacesTitle: true,
      },
      social: [
        { icon: 'github', label: 'GitHub', href: 'https://github.com/faiscadev/flaglite' },
      ],
      customCss: [
        './src/styles/custom.css',
      ],
      sidebar: [
        {
          label: 'Getting Started',
          items: [
            { label: 'Introduction', slug: 'docs/introduction' },
            { label: 'Quickstart', slug: 'docs/quickstart' },
            { label: 'Installation', slug: 'docs/installation' },
          ],
        },
        {
          label: 'SDKs',
          items: [
            { label: 'JavaScript / TypeScript', slug: 'docs/sdks/javascript' },
            { label: 'Python', slug: 'docs/sdks/python' },
            { label: 'Go', slug: 'docs/sdks/go' },
            { label: 'Rust', slug: 'docs/sdks/rust' },
          ],
        },
        {
          label: 'API Reference',
          items: [
            { label: 'Overview', slug: 'docs/api/overview' },
            { label: 'Authentication', slug: 'docs/api/authentication' },
            { label: 'Flags', slug: 'docs/api/flags' },
            { label: 'Evaluation', slug: 'docs/api/evaluation' },
          ],
        },
      ],
      // Landing page link in header
      components: {
        SiteTitle: './src/components/SiteTitle.astro',
      },
      head: [
        {
          tag: 'link',
          attrs: {
            rel: 'icon',
            href: '/favicon.svg',
            type: 'image/svg+xml',
          },
        },
      ],
    }),
  ],
});
