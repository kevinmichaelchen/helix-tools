import type { BaseLayoutProps } from 'fumadocs-ui/layouts/shared';

export function baseOptions(): BaseLayoutProps {
  return {
    nav: {
      title: 'Ixchel Tools',
    },
    links: [
      {
        text: 'Documentation',
        url: '/docs',
        active: 'nested-url',
      },
      {
        text: 'GitHub',
        url: 'https://github.com/kevinmichaelchen/ixchel',
      },
    ],
    githubUrl: 'https://github.com/kevinmichaelchen/ixchel',
  };
}
