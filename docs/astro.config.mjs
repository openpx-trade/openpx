import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

export default defineConfig({
  integrations: [
    starlight({
      title: 'OpenPX',
      description: 'Unified SDK for prediction markets — Rust, Python, TypeScript',
      social: [
        { icon: 'github', label: 'GitHub', href: 'https://github.com/openpx/openpx' },
      ],
      sidebar: [
        { label: 'Introduction', link: '/' },
        {
          label: 'Getting Started',
          items: [
            { label: 'Installation', link: '/getting-started/installation/' },
            { label: 'Quick Start', link: '/getting-started/quickstart/' },
          ],
        },
        {
          label: 'Guides',
          items: [
            { label: 'API Methods', link: '/guides/api/' },
            { label: 'WebSocket Streaming', link: '/guides/websocket/' },
            { label: 'SDKs', link: '/guides/sdks/' },
          ],
        },
        {
          label: 'Reference',
          items: [
            { label: 'All Types', link: '/reference/models/' },
            { label: 'Exchanges', link: '/reference/exchanges/' },
            { label: 'Errors', link: '/reference/errors/' },
          ],
        },
      ],
    }),
  ],
});
