import { fetchTweetAst } from 'static-tweets';
import type { TweetAst } from 'react-static-tweets/client';

/**
 * Fetch tweet AST data at build time
 * This should be called during static generation (getStaticProps)
 *
 * @param tweetId - The Twitter/X tweet ID
 * @returns Promise<TweetAst | null> - The tweet AST data to pass to TweetEmbed
 *
 * @example
 * ```tsx
 * // In your MDX or page component
 * const tweetAst = await getTweetAst('1234567890');
 *
 * // Then use it:
 * <TweetEmbed id="1234567890" ast={tweetAst} />
 * ```
 */
export async function getTweetAst(tweetId: string): Promise<TweetAst | null> {
  try {
    const ast = await fetchTweetAst(tweetId);
    return ast;
  } catch (error) {
    console.error(`Failed to fetch tweet AST for ID ${tweetId}:`, error);
    return null;
  }
}

/**
 * Batch fetch multiple tweets at build time
 */
export async function getTweetAstsMap(
  tweetIds: string[],
): Promise<Record<string, TweetAst | null>> {
  const results: Record<string, TweetAst | null> = {};

  for (const id of tweetIds) {
    results[id] = await getTweetAst(id);
  }

  return results;
}
