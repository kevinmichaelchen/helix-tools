'use client';

import Link from 'next/link';
import { useState, useEffect, useCallback } from 'react';
import {
  GitBranch,
  Database,
  WifiOff,
  Network,
  BookOpen,
  Map,
  ArrowRight,
  Github,
  Bot,
  Code2,
  Users,
  Terminal,
  Search,
  Share2,
  Sparkles,
  FileText,
  Cpu,
  ExternalLink,
  RotateCcw
} from 'lucide-react';

export default function HomePage() {
  return (
    <div className="flex flex-col min-h-screen">
      {/* Hero Section */}
      <section className="relative flex flex-col items-center justify-center py-20 text-center lg:py-28 overflow-hidden">
        {/* Enhanced background effects */}
        <div className="absolute inset-0 -z-10">
          <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_top,_var(--tw-gradient-stops))] from-sky-900/30 via-transparent to-transparent"></div>
          <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_bottom_right,_var(--tw-gradient-stops))] from-indigo-900/20 via-transparent to-transparent"></div>
          {/* Animated grid */}
          <div className="absolute inset-0 bg-[linear-gradient(rgba(14,165,233,0.03)_1px,transparent_1px),linear-gradient(90deg,rgba(14,165,233,0.03)_1px,transparent_1px)] bg-[size:60px_60px] [mask-image:radial-gradient(ellipse_at_center,black_20%,transparent_70%)]"></div>
        </div>

        <div className="inline-flex items-center gap-2 px-4 py-1.5 mb-6 text-sm font-medium rounded-full bg-sky-500/10 text-sky-400 border border-sky-500/20 animate-in fade-in slide-in-from-bottom-4 duration-500">
          <span className="relative flex h-2 w-2">
            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-sky-400 opacity-75"></span>
            <span className="relative inline-flex rounded-full h-2 w-2 bg-sky-500"></span>
          </span>
          AI-Native Developer Tools
        </div>

        <h1 className="text-4xl font-extrabold tracking-tight sm:text-6xl md:text-7xl mb-6 max-w-4xl mx-auto px-4 animate-in fade-in slide-in-from-bottom-8 duration-700 delay-100">
          Tools that <span className="text-gradient">Remember</span>
          <br /> for the Agentic Era
        </h1>

        <p className="max-w-2xl mx-auto text-lg text-slate-400 mb-10 px-4 animate-in fade-in slide-in-from-bottom-8 duration-700 delay-200">
          Give your AI agents persistent, structured memory. Graph traversal meets vector search, powered by HelixDB.
        </p>

        <div className="flex flex-col sm:flex-row gap-4 mb-16 animate-in fade-in slide-in-from-bottom-8 duration-700 delay-300">
          <Link
            href="/docs/getting-started"
            className="inline-flex items-center justify-center px-8 py-3 text-base font-medium text-white transition-all bg-sky-500 rounded-lg hover:bg-sky-400 hover:shadow-[0_0_30px_-5px_rgba(14,165,233,0.5)] active:scale-95"
          >
            Get Started
            <ArrowRight className="ml-2 h-4 w-4" />
          </Link>
          <a
            href="https://github.com/kevinmichaelchen/ixchel"
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center justify-center px-8 py-3 text-base font-medium text-slate-700 dark:text-slate-300 transition-all bg-slate-100 dark:bg-white/5 border border-slate-200 dark:border-white/10 rounded-lg hover:bg-slate-200 dark:hover:bg-white/10 hover:text-slate-900 dark:hover:text-white active:scale-95"
          >
            <Github className="mr-2 h-4 w-4" />
            View on GitHub
          </a>
        </div>

        {/* Terminal Preview */}
        <div className="w-full max-w-3xl mx-auto px-4 animate-in fade-in slide-in-from-bottom-12 duration-1000 delay-500">
          <TerminalPreview />
        </div>
      </section>

      {/* Use Case Tabs Section */}
      <section className="px-4 py-20 mx-auto max-w-6xl sm:px-6 lg:px-8">
        <div className="text-center mb-12">
          <h2 className="text-3xl font-bold text-slate-100 mb-4">Built for Everyone</h2>
          <p className="text-slate-400 max-w-2xl mx-auto">Whether you're an AI agent, a developer, or part of a team—Ixchel adapts to your workflow.</p>
        </div>
        <UseCaseTabs />
      </section>

      {/* Architecture Diagram */}
      <section className="px-4 py-20 mx-auto max-w-6xl sm:px-6 lg:px-8">
        <div className="text-center mb-12">
          <h2 className="text-3xl font-bold text-slate-100 mb-4">How It Works</h2>
          <p className="text-slate-400 max-w-2xl mx-auto">A simple architecture that keeps your data in git while enabling powerful queries.</p>
        </div>
        <ArchitectureDiagram />
      </section>

      {/* Core Principles */}
      <section className="px-4 py-20 mx-auto max-w-7xl sm:px-6 lg:px-8">
        <h2 className="text-2xl font-bold text-center mb-12 text-slate-200">Core Principles</h2>
        <div className="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-4">
          <FeatureCard
            icon={<GitBranch className="h-6 w-6 text-sky-400" />}
            title="Git-First"
            description="Data stored in human-readable Markdown, YAML, and JSONL. Your repo is the source of truth."
          />
          <FeatureCard
            icon={<WifiOff className="h-6 w-6 text-indigo-400" />}
            title="Offline-First"
            description="Full functionality without network. Local embeddings via fastembed means no cloud dependencies."
          />
          <FeatureCard
            icon={<Network className="h-6 w-6 text-purple-400" />}
            title="Graph Intelligence"
            description="Traverse relationships between entities. Navigate your codebase structure naturally."
          />
          <FeatureCard
            icon={<Database className="h-6 w-6 text-emerald-400" />}
            title="Hybrid Search"
            description="Combine vector semantic search with traditional BM25 keyword matching for best results."
          />
        </div>
      </section>

      {/* Quick Start */}
      <section className="px-4 py-20 mx-auto max-w-4xl sm:px-6 lg:px-8">
        <div className="glow-card">
          <div className="flex items-center gap-3 mb-6">
            <div className="p-2 bg-sky-500/10 rounded-lg">
              <Terminal className="h-5 w-5 text-sky-400" />
            </div>
            <h2 className="text-2xl font-bold text-slate-100">Quick Start</h2>
          </div>
          <div className="bg-slate-950 rounded-lg border border-white/10 overflow-hidden">
            <div className="flex items-center gap-2 px-4 py-2 bg-white/5 border-b border-white/10">
              <div className="w-3 h-3 rounded-full bg-red-500/80"></div>
              <div className="w-3 h-3 rounded-full bg-yellow-500/80"></div>
              <div className="w-3 h-3 rounded-full bg-green-500/80"></div>
              <span className="ml-2 text-xs text-slate-500 font-mono">terminal</span>
            </div>
            <pre className="p-4 text-sm font-mono overflow-x-auto">
              <code>
                <span className="text-slate-500"># Install</span>{'\n'}
                <span className="text-emerald-400">cargo</span> <span className="text-slate-300">install --path apps/ix-cli</span>{'\n\n'}
                <span className="text-slate-500"># Initialize in your project</span>{'\n'}
                <span className="text-emerald-400">ixchel</span> <span className="text-slate-300">init</span>{'\n\n'}
                <span className="text-slate-500"># Create an issue</span>{'\n'}
                <span className="text-emerald-400">ixchel</span> <span className="text-slate-300">create issue</span> <span className="text-amber-300">"Add user authentication"</span>{'\n\n'}
                <span className="text-slate-500"># Search semantically</span>{'\n'}
                <span className="text-emerald-400">ixchel</span> <span className="text-slate-300">search</span> <span className="text-amber-300">"how to handle auth"</span>
              </code>
            </pre>
          </div>
          <div className="mt-6 flex justify-end">
            <Link
              href="/docs/getting-started"
              className="inline-flex items-center text-sm text-sky-400 hover:text-sky-300 transition-colors"
            >
              Full installation guide
              <ArrowRight className="ml-1 h-4 w-4" />
            </Link>
          </div>
        </div>
      </section>

      {/* The Toolkit */}
      <section className="px-4 py-20 mx-auto max-w-7xl sm:px-6 lg:px-8">
        <div className="flex flex-col md:flex-row items-end justify-between mb-8 gap-4">
          <div>
            <h2 className="text-3xl font-bold text-slate-100">The Toolkit</h2>
            <p className="mt-2 text-slate-400">Specialized tools composing via standard interfaces</p>
          </div>
        </div>

        <div className="grid grid-cols-1 gap-6 md:grid-cols-2">
          <ToolCard
            icon={<BookOpen className="h-8 w-8" />}
            name="ixchel"
            tag="MVP"
            tagColor="bg-sky-500/10 text-sky-400 border-sky-500/20"
            description="Git-first knowledge weaving system. Canonical Markdown artifacts + semantic search over your project memory."
            href="/docs/tools/ixchel"
          />
          <ToolCard
            icon={<Map className="h-8 w-8" />}
            name="ideas"
            tag="Planned"
            tagColor="bg-slate-500/10 text-slate-400 border-slate-500/20"
            description="Future tools and experiments are tracked as Ixchel ideas under .ixchel/ideas/ (git-first Markdown)."
          />
        </div>
      </section>

      {/* Inspirations Section */}
      <section className="px-4 py-20 mx-auto max-w-6xl sm:px-6 lg:px-8 border-t border-white/5">
        <div className="text-center mb-12">
          <h2 className="text-2xl font-bold text-slate-100 mb-4">Standing on Giants' Shoulders</h2>
          <p className="text-slate-400">Inspired by brilliant projects in the AI tooling ecosystem</p>
        </div>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          <InspirationCard name="beads" author="Steve Yegge" color="#ff6b35" />
          <InspirationCard name="swarm-tools" author="Joel Hooks" color="#f7931e" />
          <InspirationCard name="dots" author="Joel Reymont" color="#fdb833" />
          <InspirationCard name=".context" author="Andre Figueira" color="#c1272d" />
        </div>
        <div className="text-center mt-8">
          <Link
            href="/docs/acknowledgments"
            className="inline-flex items-center text-sm text-slate-400 hover:text-sky-400 transition-colors"
          >
            See all acknowledgments
            <ArrowRight className="ml-1 h-4 w-4" />
          </Link>
        </div>
      </section>

      <footer className="mt-auto py-12 border-t border-white/5 bg-black/20 text-center">
        <p className="text-slate-500 text-sm">
          Built with <span className="text-sky-500">HelixDB</span> & Next.js
        </p>
      </footer>
    </div>
  );
}

// Typing animation hook
function useTypewriter(text: string, speed: number = 40, startDelay: number = 0, enabled: boolean = true) {
  const [displayedText, setDisplayedText] = useState('');
  const [isComplete, setIsComplete] = useState(false);

  useEffect(() => {
    if (!enabled) {
      setDisplayedText('');
      setIsComplete(false);
      return;
    }

    setDisplayedText('');
    setIsComplete(false);

    const startTimeout = setTimeout(() => {
      let index = 0;
      const interval = setInterval(() => {
        if (index < text.length) {
          setDisplayedText(text.slice(0, index + 1));
          index++;
        } else {
          setIsComplete(true);
          clearInterval(interval);
        }
      }, speed);

      return () => clearInterval(interval);
    }, startDelay);

    return () => clearTimeout(startTimeout);
  }, [text, speed, startDelay, enabled]);

  return { displayedText, isComplete };
}

type TerminalScenario = 'search' | 'create' | 'context';

const scenarioTabs: { id: TerminalScenario; label: string; icon: typeof Search }[] = [
  { id: 'search', label: 'Search', icon: Search },
  { id: 'create', label: 'Create', icon: FileText },
  { id: 'context', label: 'Context', icon: Bot },
];

function TerminalPreview() {
  const [scenario, setScenario] = useState<TerminalScenario>('search');
  const [step, setStep] = useState(0);
  const [animationKey, setAnimationKey] = useState(0);

  // Reset animation when scenario changes
  const switchScenario = useCallback((newScenario: TerminalScenario) => {
    setScenario(newScenario);
    setStep(0);
    setAnimationKey(k => k + 1);
  }, []);

  const restart = useCallback(() => {
    setStep(0);
    setAnimationKey(k => k + 1);
  }, []);

  return (
    <div className="bg-slate-950/80 backdrop-blur-sm rounded-xl border border-white/10 shadow-2xl shadow-sky-500/10 overflow-hidden">
      {/* Scenario tabs */}
      <div className="flex items-center gap-1 px-4 py-2 bg-white/5 border-b border-white/10">
        {scenarioTabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => switchScenario(tab.id)}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs font-medium transition-all ${
              scenario === tab.id
                ? 'bg-sky-500/20 text-sky-400'
                : 'text-slate-500 hover:text-slate-300 hover:bg-white/5'
            }`}
          >
            <tab.icon className="h-3 w-3" />
            {tab.label}
          </button>
        ))}
        <div className="flex-1" />
        <button
          onClick={restart}
          className="p-1.5 rounded hover:bg-white/10 transition-colors text-slate-500 hover:text-slate-300"
          title="Replay"
        >
          <RotateCcw className="h-3.5 w-3.5" />
        </button>
      </div>

      {/* Terminal header */}
      <div className="flex items-center justify-between px-4 py-2 bg-black/20 border-b border-white/5">
        <div className="flex items-center gap-2">
          <div className="w-2.5 h-2.5 rounded-full bg-red-500/80"></div>
          <div className="w-2.5 h-2.5 rounded-full bg-yellow-500/80"></div>
          <div className="w-2.5 h-2.5 rounded-full bg-green-500/80"></div>
        </div>
        <span className="text-xs text-slate-600 font-mono">~/my-project</span>
        <div className="w-12" />
      </div>

      {/* Terminal content */}
      <div className="p-4 font-mono text-sm min-h-[260px]">
        {scenario === 'search' && (
          <SearchScenario key={`search-${animationKey}`} step={step} setStep={setStep} />
        )}
        {scenario === 'create' && (
          <CreateScenario key={`create-${animationKey}`} step={step} setStep={setStep} />
        )}
        {scenario === 'context' && (
          <ContextScenario key={`context-${animationKey}`} step={step} setStep={setStep} />
        )}
      </div>
    </div>
  );
}

function SearchScenario({ step, setStep }: { step: number; setStep: (s: number) => void }) {
  const command1 = 'ixchel search "authentication flow"';
  const command2 = 'ixchel graph iss-a1b2c3';

  const { displayedText: typed1, isComplete: cmd1Done } = useTypewriter(command1, 35, 400, step >= 0);
  const { displayedText: typed2, isComplete: cmd2Done } = useTypewriter(command2, 35, 0, step >= 2);

  useEffect(() => {
    if (cmd1Done && step === 0) {
      const timer = setTimeout(() => setStep(1), 300);
      return () => clearTimeout(timer);
    }
  }, [cmd1Done, step, setStep]);

  useEffect(() => {
    if (step === 1) {
      const timer = setTimeout(() => setStep(2), 1600);
      return () => clearTimeout(timer);
    }
  }, [step, setStep]);

  useEffect(() => {
    if (cmd2Done && step === 2) {
      const timer = setTimeout(() => setStep(3), 300);
      return () => clearTimeout(timer);
    }
  }, [cmd2Done, step, setStep]);

  useEffect(() => {
    if (step === 3) {
      const timer = setTimeout(() => setStep(4), 1200);
      return () => clearTimeout(timer);
    }
  }, [step, setStep]);

  return (
    <div className="space-y-3">
      <TerminalCommand text={typed1} isComplete={cmd1Done} highlightQuotes />

      {step >= 1 && (
        <div className="pl-4 text-slate-400 text-xs space-y-2 border-l-2 border-sky-500/30 ml-1 animate-in fade-in slide-in-from-bottom-2 duration-300">
          <div className="flex items-center gap-2">
            <Search className="h-3 w-3 text-sky-400" />
            <span>Searching with hybrid (vector + BM25)...</span>
          </div>
          <div className="space-y-1">
            {[
              { id: 'iss-a1b2c3', title: 'Add OAuth2 authentication', score: '0.94' },
              { id: 'iss-d4e5f6', title: 'Implement JWT refresh tokens', score: '0.87' },
              { id: 'iss-g7h8i9', title: 'Session management refactor', score: '0.82' },
            ].map((result, i) => (
              <div
                key={result.id}
                className="flex items-center gap-2 text-slate-300 animate-in fade-in slide-in-from-left-2 duration-300"
                style={{ animationDelay: `${(i + 1) * 100}ms` }}
              >
                <FileText className="h-3 w-3 text-emerald-400" />
                <span className="text-emerald-400">{result.id}</span>
                <span className="text-slate-500">•</span>
                <span>{result.title}</span>
                <span className="text-sky-400 ml-auto">{result.score}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {step >= 2 && (
        <div className="pt-2 animate-in fade-in duration-200">
          <TerminalCommand text={typed2} isComplete={cmd2Done} highlightIds={['iss-a1b2c3']} />
        </div>
      )}

      {step >= 3 && (
        <div className="pl-4 text-slate-400 text-xs border-l-2 border-purple-500/30 ml-1 animate-in fade-in slide-in-from-bottom-2 duration-300">
          <div className="flex items-center gap-2 mb-1">
            <Share2 className="h-3 w-3 text-purple-400" />
            <span>Dependency graph for iss-a1b2c3</span>
          </div>
          <pre className="text-[10px] leading-relaxed text-slate-500">
{`iss-a1b2c3 (Add OAuth2 authentication)
   ├── blocks → iss-d4e5f6 (JWT refresh tokens)
     └── related  → iss-g7h8i9 (Session management)`}
          </pre>
        </div>
      )}

      {step >= 4 && <TerminalPrompt />}
    </div>
  );
}

function CreateScenario({ step, setStep }: { step: number; setStep: (s: number) => void }) {
  const command1 = 'ixchel create issue "Add caching layer"';
  const command2 = 'ixchel link iss-x1y2z3 blocks iss-a1b2c3';

  const { displayedText: typed1, isComplete: cmd1Done } = useTypewriter(command1, 35, 400, step >= 0);
  const { displayedText: typed2, isComplete: cmd2Done } = useTypewriter(command2, 35, 0, step >= 2);

  useEffect(() => {
    if (cmd1Done && step === 0) {
      const timer = setTimeout(() => setStep(1), 300);
      return () => clearTimeout(timer);
    }
  }, [cmd1Done, step, setStep]);

  useEffect(() => {
    if (step === 1) {
      const timer = setTimeout(() => setStep(2), 1200);
      return () => clearTimeout(timer);
    }
  }, [step, setStep]);

  useEffect(() => {
    if (cmd2Done && step === 2) {
      const timer = setTimeout(() => setStep(3), 300);
      return () => clearTimeout(timer);
    }
  }, [cmd2Done, step, setStep]);

  useEffect(() => {
    if (step === 3) {
      const timer = setTimeout(() => setStep(4), 1000);
      return () => clearTimeout(timer);
    }
  }, [step, setStep]);

  return (
    <div className="space-y-3">
      <TerminalCommand text={typed1} isComplete={cmd1Done} highlightQuotes />

      {step >= 1 && (
        <div className="pl-4 text-xs border-l-2 border-emerald-500/30 ml-1 animate-in fade-in slide-in-from-bottom-2 duration-300 space-y-1">
          <div className="flex items-center gap-2 text-emerald-400">
            <span>✓</span>
            <span>Created issue</span>
            <span className="text-slate-400 font-semibold">iss-x1y2z3</span>
          </div>
          <div className="text-slate-500">
            → .ixchel/issues/iss-x1y2z3.md
          </div>
        </div>
      )}

      {step >= 2 && (
        <div className="pt-2 animate-in fade-in duration-200">
          <TerminalCommand text={typed2} isComplete={cmd2Done} highlightIds={['iss-x1y2z3', 'iss-a1b2c3']} />
        </div>
      )}

      {step >= 3 && (
        <div className="pl-4 text-xs border-l-2 border-purple-500/30 ml-1 animate-in fade-in slide-in-from-bottom-2 duration-300 space-y-1">
          <div className="flex items-center gap-2 text-purple-400">
            <Share2 className="h-3 w-3" />
            <span>Linked</span>
            <span className="text-emerald-400">iss-x1y2z3</span>
            <span className="text-slate-500">blocks</span>
            <span className="text-emerald-400">iss-a1b2c3</span>
          </div>
        </div>
      )}

      {step >= 4 && <TerminalPrompt />}
    </div>
  );
}

function ContextScenario({ step, setStep }: { step: number; setStep: (s: number) => void }) {
  const command1 = 'ixchel context iss-a1b2c3 --json';

  const { displayedText: typed1, isComplete: cmd1Done } = useTypewriter(command1, 35, 400, step >= 0);

  useEffect(() => {
    if (cmd1Done && step === 0) {
      const timer = setTimeout(() => setStep(1), 300);
      return () => clearTimeout(timer);
    }
  }, [cmd1Done, step, setStep]);

  useEffect(() => {
    if (step === 1) {
      const timer = setTimeout(() => setStep(2), 2000);
      return () => clearTimeout(timer);
    }
  }, [step, setStep]);

  return (
    <div className="space-y-3">
      <TerminalCommand text={typed1} isComplete={cmd1Done} highlightIds={['iss-a1b2c3']} highlightFlags={['--json']} />

      {step >= 1 && (
        <div className="pl-4 text-xs border-l-2 border-amber-500/30 ml-1 animate-in fade-in slide-in-from-bottom-2 duration-300">
          <pre className="font-mono text-[11px] leading-relaxed text-left whitespace-pre">{`{
  "id": "iss-a1b2c3",
  "title": "Add OAuth2 authentication",
  "status": "in_progress",
  "blocks": ["iss-d4e5f6"],
  "related": ["iss-g7h8i9"],
  "similarity": 0.94
}`}</pre>
        </div>
      )}

      {step >= 2 && <TerminalPrompt />}
    </div>
  );
}

function TerminalCommand({
  text,
  isComplete,
  highlightQuotes = false,
  highlightIds = [],
  highlightFlags = [],
}: {
  text: string;
  isComplete: boolean;
  highlightQuotes?: boolean;
  highlightIds?: string[];
  highlightFlags?: string[];
}) {
  const renderText = () => {
    if (highlightQuotes && text.includes('"')) {
      const parts = text.split('"');
      return (
        <>
          {parts[0]}
          {parts.length > 1 && <span className="text-amber-300">"{parts[1]}{parts.length > 2 ? '"' : ''}</span>}
          {parts.slice(2).join('"')}
        </>
      );
    }

    return text.split(' ').map((word, i) => {
      const isId = highlightIds.includes(word);
      const isFlag = highlightFlags.includes(word);
      return (
        <span key={i}>
          {i > 0 && ' '}
          {isId ? (
            <span className="text-emerald-400">{word}</span>
          ) : isFlag ? (
            <span className="text-purple-400">{word}</span>
          ) : (
            word
          )}
        </span>
      );
    });
  };

  return (
    <div className="flex items-start gap-2">
      <span className="text-sky-400">$</span>
      <span className="text-slate-300">
        {renderText()}
        {!isComplete && <span className="animate-pulse text-slate-400">▊</span>}
      </span>
    </div>
  );
}

function TerminalPrompt() {
  return (
    <div className="flex items-start gap-2 pt-2 animate-in fade-in duration-200">
      <span className="text-sky-400">$</span>
      <span className="text-slate-400 animate-pulse">▊</span>
    </div>
  );
}

function UseCaseTabs() {
  const [activeTab, setActiveTab] = useState<'agents' | 'developers' | 'teams'>('agents');

  const tabs = [
    { id: 'agents' as const, label: 'AI Agents', icon: Bot },
    { id: 'developers' as const, label: 'Developers', icon: Code2 },
    { id: 'teams' as const, label: 'Teams', icon: Users },
  ];

  const content = {
    agents: {
      title: 'Context-aware AI assistants',
      description: 'Give your AI agents the memory they deserve. Structured JSON output, semantic search, and full context retrieval.',
      features: [
        { icon: Terminal, text: '--json output on all commands' },
        { icon: Search, text: 'Semantic search for intelligent retrieval' },
        { icon: Sparkles, text: 'Token-aware context with ixchel context <id>' },
      ],
      code: `$ ixchel context iss-a1b2c3 --json
{
  "id": "iss-a1b2c3",
  "title": "Add OAuth2 authentication",
  "status": "in_progress",
  "blocks": ["iss-d4e5f6"],
  "related": ["iss-g7h8i9"],
  "embedding_similarity": 0.94
}`,
    },
    developers: {
      title: 'CLI-first workflow',
      description: 'Fast, scriptable commands that integrate with your existing tools. Pipes, scripts, and UNIX philosophy.',
      features: [
        { icon: Terminal, text: 'Composable CLI commands' },
        { icon: GitBranch, text: 'Git-native storage (Markdown files)' },
        { icon: Database, text: 'Local HelixDB for fast queries' },
      ],
      code: `$ ixchel create issue "Add caching layer" \\
    --status open --priority high

Created: iss-x1y2z3

$ ixchel link iss-x1y2z3 blocks iss-a1b2c3
Linked: iss-x1y2z3 blocks iss-a1b2c3`,
    },
    teams: {
      title: 'Collaborate via git',
      description: 'No separate service to sync. Issues merge like code. Branch, PR, and resolve conflicts naturally.',
      features: [
        { icon: GitBranch, text: 'Issues travel with your code' },
        { icon: Users, text: 'Standard git merge workflow' },
        { icon: WifiOff, text: 'Works offline, syncs when ready' },
      ],
      code: `$ git pull origin main
Updating issues...

$ ixchel sync
Indexed 3 new issues
Updated 2 existing issues
Regenerated 5 embeddings`,
    },
  };

  const current = content[activeTab];

  return (
    <div className="glow-card">
      {/* Tab buttons */}
      <div className="flex flex-wrap gap-2 mb-8 p-1 bg-white/5 rounded-lg w-fit">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={`flex items-center gap-2 px-4 py-2 rounded-md text-sm font-medium transition-all ${
              activeTab === tab.id
                ? 'bg-sky-500 text-white shadow-lg shadow-sky-500/25'
                : 'text-slate-400 hover:text-slate-200 hover:bg-white/5'
            }`}
          >
            <tab.icon className="h-4 w-4" />
            {tab.label}
          </button>
        ))}
      </div>

      {/* Tab content */}
      <div className="grid md:grid-cols-2 gap-8">
        <div>
          <h3 className="text-xl font-bold text-slate-900 dark:text-slate-100 mb-3">{current.title}</h3>
          <p className="text-slate-600 dark:text-slate-400 mb-6">{current.description}</p>
          <ul className="space-y-3">
            {current.features.map((feature, i) => (
              <li key={i} className="flex items-center gap-3 text-slate-700 dark:text-slate-300">
                <div className="p-1.5 bg-sky-500/10 rounded">
                  <feature.icon className="h-4 w-4 text-sky-500 dark:text-sky-400" />
                </div>
                {feature.text}
              </li>
            ))}
          </ul>
        </div>
        <div className="bg-slate-100 dark:bg-slate-950 rounded-lg border border-slate-200 dark:border-white/10 overflow-hidden">
          <div className="flex items-center gap-2 px-4 py-2 bg-slate-200/50 dark:bg-white/5 border-b border-slate-200 dark:border-white/10">
            <div className="w-2.5 h-2.5 rounded-full bg-red-500/80"></div>
            <div className="w-2.5 h-2.5 rounded-full bg-yellow-500/80"></div>
            <div className="w-2.5 h-2.5 rounded-full bg-green-500/80"></div>
          </div>
          <pre className="p-4 text-xs font-mono text-slate-700 dark:text-slate-300 overflow-x-auto">
            {current.code}
          </pre>
        </div>
      </div>
    </div>
  );
}

function ArchitectureDiagram() {
  return (
    <div className="glow-card">
      <div className="flex flex-col lg:flex-row items-center justify-between gap-8">
        {/* Source */}
        <div className="flex-1 text-center">
          <div className="inline-flex p-4 bg-emerald-500/10 rounded-2xl mb-4 border border-emerald-500/20">
            <FileText className="h-10 w-10 text-emerald-600 dark:text-emerald-400" />
          </div>
          <h3 className="text-lg font-semibold text-slate-900 dark:text-slate-100 mb-2">Markdown Files</h3>
          <p className="text-sm text-slate-600 dark:text-slate-400">Human-readable, git-tracked</p>
          <code className="text-xs text-emerald-600 dark:text-emerald-400 mt-2 block">.ixchel/issues/*.md</code>
        </div>

        {/* Arrow */}
        <div className="flex items-center text-slate-600">
          <div className="hidden lg:block w-16 h-px bg-gradient-to-r from-emerald-500/50 to-sky-500/50"></div>
          <ArrowRight className="h-6 w-6 text-slate-400 dark:text-slate-500 mx-2" />
          <div className="hidden lg:block w-16 h-px bg-gradient-to-r from-sky-500/50 to-purple-500/50"></div>
        </div>

        {/* Index */}
        <div className="flex-1 text-center">
          <div className="inline-flex p-4 bg-sky-500/10 rounded-2xl mb-4 border border-sky-500/20">
            <Cpu className="h-10 w-10 text-sky-600 dark:text-sky-400" />
          </div>
          <h3 className="text-lg font-semibold text-slate-900 dark:text-slate-100 mb-2">HelixDB Index</h3>
          <p className="text-sm text-slate-600 dark:text-slate-400">Graph + Vector + BM25</p>
          <code className="text-xs text-sky-600 dark:text-sky-400 mt-2 block">Rebuilds from source</code>
        </div>

        {/* Arrow */}
        <div className="flex items-center text-slate-600">
          <div className="hidden lg:block w-16 h-px bg-gradient-to-r from-sky-500/50 to-purple-500/50"></div>
          <ArrowRight className="h-6 w-6 text-slate-400 dark:text-slate-500 mx-2" />
          <div className="hidden lg:block w-16 h-px bg-gradient-to-r from-purple-500/50 to-amber-500/50"></div>
        </div>

        {/* Query */}
        <div className="flex-1 text-center">
          <div className="inline-flex p-4 bg-purple-500/10 rounded-2xl mb-4 border border-purple-500/20">
            <Search className="h-10 w-10 text-purple-600 dark:text-purple-400" />
          </div>
          <h3 className="text-lg font-semibold text-slate-900 dark:text-slate-100 mb-2">Smart Queries</h3>
          <p className="text-sm text-slate-600 dark:text-slate-400">Semantic + keyword search</p>
          <code className="text-xs text-purple-600 dark:text-purple-400 mt-2 block">ixchel search "..."</code>
        </div>
      </div>

      <div className="mt-8 pt-6 border-t border-slate-200 dark:border-white/10 text-center">
        <p className="text-sm text-slate-600 dark:text-slate-500">
          <span className="text-sky-600 dark:text-sky-400">Delete the database?</span> No problem. Run <code className="text-emerald-600 dark:text-emerald-400">ixchel sync</code> and it rebuilds from your Markdown files.
        </p>
      </div>
    </div>
  );
}

function FeatureCard({ icon, title, description }: { icon: React.ReactNode, title: string, description: string }) {
  return (
    <div className="glow-card h-full flex flex-col">
      <div className="mb-4 p-3 bg-slate-100 dark:bg-white/5 rounded-lg w-fit border border-slate-200 dark:border-white/5">
        {icon}
      </div>
      <h3 className="text-lg font-semibold text-slate-900 dark:text-slate-100 mb-2">{title}</h3>
      <p className="text-slate-600 dark:text-slate-400 text-sm leading-relaxed">{description}</p>
    </div>
  );
}

function ToolCard({ icon, name, description, tag, tagColor, href }: { icon: React.ReactNode, name: string, description: string, tag: string, tagColor: string, href?: string }) {
  const content = (
    <>
      <div className="flex items-start justify-between mb-4">
        <div className="p-3 bg-sky-500/10 text-sky-600 dark:text-sky-400 rounded-lg group-hover:bg-sky-500 group-hover:text-white transition-colors duration-300">
          {icon}
        </div>
        <span className={`px-2.5 py-0.5 rounded-full text-xs font-medium border ${tagColor}`}>
          {tag}
        </span>
      </div>
      <h3 className="text-xl font-bold text-slate-900 dark:text-slate-100 mb-2 group-hover:text-sky-500 dark:group-hover:text-sky-400 transition-colors">{name}</h3>
      <p className="text-slate-600 dark:text-slate-400">{description}</p>
    </>
  );

  if (href) {
    return (
      <Link href={href} className="glow-card group block">
        {content}
      </Link>
    );
  }

  return (
    <div className="glow-card group">
      {content}
    </div>
  );
}

function InspirationCard({ name, author, color }: { name: string, author: string, color: string }) {
  return (
    <div
      className="p-4 rounded-xl border border-slate-200 dark:border-white/10 bg-slate-50 dark:bg-white/5 hover:bg-slate-100 dark:hover:bg-white/10 transition-all hover:border-slate-300 dark:hover:border-opacity-50 group"
      style={{ '--accent': color } as React.CSSProperties}
    >
      <div
        className="w-2 h-2 rounded-full mb-3 group-hover:scale-125 transition-transform"
        style={{ backgroundColor: color }}
      />
      <p className="font-mono text-sm text-slate-800 dark:text-slate-200 mb-1">{name}</p>
      <p className="text-xs text-slate-500">{author}</p>
    </div>
  );
}
