import type { BaseLayoutProps } from 'fumadocs-ui/layouts/shared';

export function baseOptions(): BaseLayoutProps {
  return {
    nav: {
      title: 'Helix Tools',
    },
    links: [
      {
        text: 'Documentation',
        url: '/docs',
        active: 'nested-url',
      },
      {
        text: 'GitHub',
        url: 'https://github.com/kevinmichaelchen/helix-tools',
      },
    ],
    githubUrl: 'https://github.com/kevinmichaelchen/helix-tools',
  };
}
