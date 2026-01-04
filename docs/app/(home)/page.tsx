import Link from 'next/link';
import { 
  GitBranch, 
  Database, 
  WifiOff, 
  Network, 
  Terminal, 
  BookOpen, 
  Map, 
  Mail,
  ArrowRight,
  Github
} from 'lucide-react';

export default function HomePage() {
  return (
    <div className="flex flex-col min-h-screen">
      <section className="relative flex flex-col items-center justify-center py-24 text-center lg:py-32 overflow-hidden">
        <div className="absolute inset-0 -z-10 bg-[radial-gradient(ellipse_at_top,_var(--tw-gradient-stops))] from-sky-900/20 via-slate-900/0 to-slate-900/0"></div>
        
        <div className="inline-flex items-center gap-2 px-3 py-1 mb-6 text-sm font-medium rounded-full bg-sky-500/10 text-sky-400 border border-sky-500/20 animate-in fade-in slide-in-from-bottom-4 duration-500">
          <span className="relative flex h-2 w-2">
            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-sky-400 opacity-75"></span>
            <span className="relative inline-flex rounded-full h-2 w-2 bg-sky-500"></span>
          </span>
          AI-Native Developer Tools
        </div>

        <h1 className="text-4xl font-extrabold tracking-tight sm:text-6xl md:text-7xl mb-6 max-w-4xl mx-auto animate-in fade-in slide-in-from-bottom-8 duration-700 delay-100">
          Tools that <span className="text-gradient">Remember</span>
          <br /> for the Agentic Era
        </h1>
        
        <p className="max-w-2xl mx-auto text-lg text-slate-400 mb-10 animate-in fade-in slide-in-from-bottom-8 duration-700 delay-200">
          A monorepo of CLI tools powered by HelixDB. Give your AI agents persistent, structured memory with graph traversal and vector search.
        </p>
        
        <div className="flex flex-col sm:flex-row gap-4 animate-in fade-in slide-in-from-bottom-8 duration-700 delay-300">
          <Link 
            href="/docs" 
            className="inline-flex items-center justify-center px-8 py-3 text-base font-medium text-white transition-all bg-sky-500 rounded-lg hover:bg-sky-400 hover:shadow-[0_0_20px_-5px_rgba(14,165,233,0.5)] active:scale-95"
          >
            Read Documentation
            <ArrowRight className="ml-2 h-4 w-4" />
          </Link>
          <a 
            href="https://github.com/kevinmichaelchen/helix-tools" 
            target="_blank" 
            rel="noopener noreferrer"
            className="inline-flex items-center justify-center px-8 py-3 text-base font-medium text-slate-300 transition-all bg-white/5 border border-white/10 rounded-lg hover:bg-white/10 hover:text-white active:scale-95"
          >
            <Github className="mr-2 h-4 w-4" />
            View on GitHub
          </a>
        </div>
      </section>

      <section className="px-4 py-16 mx-auto max-w-7xl sm:px-6 lg:px-8">
        <h2 className="text-2xl font-bold text-center mb-12 text-slate-200">Built on Core Principles</h2>
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

      <section className="px-4 py-16 mx-auto max-w-7xl sm:px-6 lg:px-8">
        <div className="flex flex-col md:flex-row items-end justify-between mb-8 gap-4">
          <div>
            <h2 className="text-3xl font-bold text-slate-100">The Helix Suite</h2>
            <p className="mt-2 text-slate-400"> specialized tools composing via standard interfaces</p>
          </div>
        </div>
        
        <div className="grid grid-cols-1 gap-6 md:grid-cols-2">
          <ToolCard 
            icon={<Terminal className="h-8 w-8" />}
            name="hbd"
            tag="In Development"
            tagColor="bg-yellow-500/10 text-yellow-400 border-yellow-500/20"
            description="Git-first issue tracker for AI-supervised workflows. Bridge the gap between conceptual specs and tasks."
          />
          <ToolCard 
            icon={<BookOpen className="h-8 w-8" />}
            name="helix-docs"
            tag="Planned"
            tagColor="bg-slate-500/10 text-slate-400 border-slate-500/20"
            description="Local cache for fetched documentation. Give your agents instant access to library docs during research."
          />
          <ToolCard 
            icon={<Map className="h-8 w-8" />}
            name="helix-map"
            tag="Planned"
            tagColor="bg-slate-500/10 text-slate-400 border-slate-500/20"
            description="Codebase structure cache for fast exploration. Helps agents understand project architecture."
          />
          <ToolCard 
            icon={<Mail className="h-8 w-8" />}
            name="helix-mail"
            tag="Planned"
            tagColor="bg-slate-500/10 text-slate-400 border-slate-500/20"
            description="Agent-to-agent messaging and coordination system for multi-agent workflows."
          />
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

function FeatureCard({ icon, title, description }: { icon: React.ReactNode, title: string, description: string }) {
  return (
    <div className="glow-card h-full flex flex-col">
      <div className="mb-4 p-3 bg-white/5 rounded-lg w-fit border border-white/5">
        {icon}
      </div>
      <h3 className="text-lg font-semibold text-slate-100 mb-2">{title}</h3>
      <p className="text-slate-400 text-sm leading-relaxed">{description}</p>
    </div>
  );
}

function ToolCard({ icon, name, description, tag, tagColor }: { icon: React.ReactNode, name: string, description: string, tag: string, tagColor: string }) {
  return (
    <div className="glow-card group">
      <div className="flex items-start justify-between mb-4">
        <div className="p-3 bg-sky-500/10 text-sky-400 rounded-lg group-hover:bg-sky-500 group-hover:text-white transition-colors duration-300">
          {icon}
        </div>
        <span className={`px-2.5 py-0.5 rounded-full text-xs font-medium border ${tagColor}`}>
          {tag}
        </span>
      </div>
      <h3 className="text-xl font-bold text-slate-100 mb-2 group-hover:text-sky-400 transition-colors">{name}</h3>
      <p className="text-slate-400">{description}</p>
    </div>
  );
}
