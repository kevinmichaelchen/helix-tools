'use client';

import { Tweet } from 'react-static-tweets';
import type { TweetAst } from 'react-static-tweets/client';

interface TweetProps {
  id: string;
  ast?: TweetAst;
}

export function TweetEmbed({ id, ast }: TweetProps) {
  if (!ast) {
    // Fallback: link to the tweet if AST data isn't available
    return (
      <a
        href={`https://twitter.com/i/web/status/${id}`}
        target="_blank"
        rel="noopener noreferrer"
        className="inline-block text-blue-600 hover:underline"
      >
        View tweet
      </a>
    );
  }

  return <Tweet ast={ast} />;
}
