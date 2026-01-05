import { Contributors, type Contributor } from './Contributors';

interface ContributorsLoaderProps {
  contributors: Contributor[];
}

export function ContributorsLoader({ contributors }: ContributorsLoaderProps) {
  return <Contributors contributors={contributors} />;
}
