import Link from "next/link";
import { Button } from "@/components/ui/button";
import { UserMenu } from "@/components/auth/user-menu";
import { MobileNav } from "@/components/mobile-nav";
import { GitHubStars } from "@/components/github-stars";
import { GetStartedButton, GetStartedFreeButton } from "@/components/home-buttons";
import { getSession } from "@/lib/auth";

const BRAND_COLOR = "#ff5533";

export default async function Home() {
  const session = await getSession();
  return (
    <div className="min-h-screen bg-background">
      <header className="sticky top-0 z-50 border-b bg-background/80 backdrop-blur-sm">
        <div className="mx-auto flex h-14 max-w-5xl items-center justify-between px-4 sm:h-16 sm:px-6">
          <Link href="/" className="text-lg font-semibold tracking-tight sm:text-xl">
            RabbitBench
          </Link>
          {/* Desktop nav */}
          <nav className="hidden items-center gap-6 sm:flex">
            <GitHubStars />
            <Link
              href="/docs"
              className="text-sm text-muted-foreground transition-colors hover:text-foreground"
            >
              Docs
            </Link>
            <UserMenu />
          </nav>
          {/* Mobile nav */}
          <div className="sm:hidden">
            <MobileNav />
          </div>
        </div>
      </header>

      <main>
        {/* Hero Section */}
        <section className="relative overflow-hidden">
          <div className="absolute inset-0 dot-background opacity-50" />
          <div className="relative mx-auto max-w-5xl px-4 py-16 sm:px-6 sm:py-24 lg:py-32">
            <div className="flex flex-col items-center text-center">
              <h1 className="max-w-4xl text-2xl font-bold tracking-tight sm:text-4xl lg:text-5xl">
                Benchmark{" "}
                <span style={{ color: BRAND_COLOR }}>
                  Performance
                </span>
              </h1>
              <p className="mt-4 max-w-2xl text-sm text-muted-foreground sm:mt-6 sm:text-base lg:text-lg">
                Continuous benchmarking for Rust programs. Collect, visualize,
                and catch performance regressions before they ship.
              </p>
              <div className="mt-8 flex flex-col gap-3 sm:mt-10 sm:flex-row sm:gap-4">
                <GetStartedButton isSignedIn={!!session} />
                <Button asChild variant="outline" size="lg" className="h-11 px-6 sm:h-12 sm:px-8">
                  <a
                    href="https://github.com/alexchoi0/rabbitbench"
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    <svg
                      className="mr-2 h-5 w-5"
                      fill="currentColor"
                      viewBox="0 0 24 24"
                    >
                      <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z" />
                    </svg>
                    View on GitHub
                  </a>
                </Button>
              </div>
            </div>
          </div>
        </section>

        {/* Features Section */}
        <section className="border-t bg-muted/30">
          <div className="mx-auto max-w-5xl px-4 py-16 sm:px-6 sm:py-24">
            <div className="text-center mb-10 sm:mb-16">
              <h2 className="text-xl font-bold tracking-tight sm:text-2xl lg:text-3xl">
                Everything you need
              </h2>
              <p className="mt-3 text-sm text-muted-foreground sm:mt-4 sm:text-base">
                Built for performance-critical Rust programs
              </p>
            </div>
            <div className="grid gap-4 sm:gap-8 sm:grid-cols-3">
              <div className="group border bg-card p-6 sm:p-8 transition-all hover:border-foreground/20 hover:shadow-lg">
                <div className="mb-4 inline-flex h-10 w-10 sm:h-12 sm:w-12 items-center justify-center bg-primary/10">
                  <svg
                    className="h-5 w-5 sm:h-6 sm:w-6 text-primary"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"
                    />
                  </svg>
                </div>
                <h3 className="text-base sm:text-lg font-semibold">Collect</h3>
                <p className="mt-2 text-sm sm:text-base text-muted-foreground">
                  Run your Criterion benchmarks with our CLI and automatically
                  submit results to track over time.
                </p>
              </div>
              <div className="group border bg-card p-6 sm:p-8 transition-all hover:border-foreground/20 hover:shadow-lg">
                <div className="mb-4 inline-flex h-10 w-10 sm:h-12 sm:w-12 items-center justify-center bg-primary/10">
                  <svg
                    className="h-5 w-5 sm:h-6 sm:w-6 text-primary"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"
                    />
                  </svg>
                </div>
                <h3 className="text-base sm:text-lg font-semibold">Visualize</h3>
                <p className="mt-2 text-sm sm:text-base text-muted-foreground">
                  Interactive charts show performance trends across branches,
                  commits, and testbeds.
                </p>
              </div>
              <div className="group border bg-card p-6 sm:p-8 transition-all hover:border-foreground/20 hover:shadow-lg">
                <div className="mb-4 inline-flex h-10 w-10 sm:h-12 sm:w-12 items-center justify-center bg-primary/10">
                  <svg
                    className="h-5 w-5 sm:h-6 sm:w-6 text-primary"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9"
                    />
                  </svg>
                </div>
                <h3 className="text-base sm:text-lg font-semibold">Alert</h3>
                <p className="mt-2 text-sm sm:text-base text-muted-foreground">
                  Configure thresholds and get notified when benchmarks regress
                  beyond acceptable limits.
                </p>
              </div>
            </div>
          </div>
        </section>

        {/* CLI Section */}
        <section className="border-t">
          <div className="mx-auto max-w-5xl px-4 py-16 sm:px-6 sm:py-24">
            <div className="grid gap-8 lg:grid-cols-2 lg:gap-12 items-center">
              <div>
                <h2 className="text-xl font-bold tracking-tight sm:text-2xl lg:text-3xl">
                  Simple CLI interface
                </h2>
                <p className="mt-3 text-sm text-muted-foreground sm:mt-4 sm:text-base">
                  Get started in minutes. Install our CLI, authenticate, and
                  start tracking your benchmarks with a single command.
                </p>
                <ul className="mt-6 space-y-3 sm:mt-8 sm:space-y-4">
                  <li className="flex items-start gap-3">
                    <svg
                      className="h-5 w-5 sm:h-6 sm:w-6 text-success flex-shrink-0"
                      fill="none"
                      viewBox="0 0 24 24"
                      stroke="currentColor"
                    >
                      <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth={2}
                        d="M5 13l4 4L19 7"
                      />
                    </svg>
                    <span className="text-sm sm:text-base text-muted-foreground">
                      Works with existing Criterion benchmarks
                    </span>
                  </li>
                  <li className="flex items-start gap-3">
                    <svg
                      className="h-5 w-5 sm:h-6 sm:w-6 text-success flex-shrink-0"
                      fill="none"
                      viewBox="0 0 24 24"
                      stroke="currentColor"
                    >
                      <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth={2}
                        d="M5 13l4 4L19 7"
                      />
                    </svg>
                    <span className="text-sm sm:text-base text-muted-foreground">
                      CI/CD integration with GitHub Actions
                    </span>
                  </li>
                  <li className="flex items-start gap-3">
                    <svg
                      className="h-5 w-5 sm:h-6 sm:w-6 text-success flex-shrink-0"
                      fill="none"
                      viewBox="0 0 24 24"
                      stroke="currentColor"
                    >
                      <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth={2}
                        d="M5 13l4 4L19 7"
                      />
                    </svg>
                    <span className="text-sm sm:text-base text-muted-foreground">
                      Automatic branch and commit detection
                    </span>
                  </li>
                </ul>
              </div>
              <div className="relative">
                <div className="absolute -inset-4 blur-xl" style={{ background: `linear-gradient(to right, ${BRAND_COLOR}33, ${BRAND_COLOR}1a)` }} />
                <div className="relative rounded-xl border bg-[#0a0a0a] p-4 sm:p-6 text-[#ededed] shadow-2xl dark:shadow-2xl shadow-black/20 dark:shadow-black/50 overflow-hidden">
                  <div className="flex items-center gap-2 mb-3 sm:mb-4">
                    <div className="h-2.5 w-2.5 sm:h-3 sm:w-3 rounded-full bg-[#ff5f56]" />
                    <div className="h-2.5 w-2.5 sm:h-3 sm:w-3 rounded-full bg-[#ffbd2e]" />
                    <div className="h-2.5 w-2.5 sm:h-3 sm:w-3 rounded-full bg-[#27c93f]" />
                  </div>
                  <pre className="text-xs sm:text-sm overflow-x-auto font-mono">
                    <code>
                      <span className="text-[#a1a1a1]"># Install the CLI</span>
                      {"\n"}
                      <span style={{ color: BRAND_COLOR }}>$</span> cargo install
                      rabbitbench
                      {"\n\n"}
                      <span className="text-[#a1a1a1]"># Authenticate</span>
                      {"\n"}
                      <span style={{ color: BRAND_COLOR }}>$</span> rabbitbench auth
                      login
                      {"\n\n"}
                      <span className="text-[#a1a1a1]">
                        # Run benchmarks and submit
                      </span>
                      {"\n"}
                      <span style={{ color: BRAND_COLOR }}>$</span> rabbitbench run \
                      {"\n"}
                      {"    "}--project my-lib \{"\n"}
                      {"    "}--branch main \{"\n"}
                      {"    "}&quot;cargo bench&quot;
                    </code>
                  </pre>
                </div>
              </div>
            </div>
          </div>
        </section>

        {/* CTA Section */}
        <section className="border-t bg-muted/30">
          <div className="mx-auto max-w-5xl px-4 py-16 sm:px-6 sm:py-24 text-center">
            <h2 className="text-xl font-bold tracking-tight sm:text-2xl lg:text-3xl">
              Ready to track your benchmarks?
            </h2>
            <p className="mt-3 text-sm text-muted-foreground sm:mt-4 sm:text-base">
              Start for free. No credit card required.
            </p>
            <div className="mt-8 sm:mt-10">
              <GetStartedFreeButton isSignedIn={!!session} />
            </div>
          </div>
        </section>
      </main>

      <footer className="border-t">
        <div className="mx-auto max-w-5xl px-4 py-6 sm:px-6 sm:py-8">
          <div className="flex flex-col sm:flex-row items-center justify-between gap-4">
            <div className="text-xs sm:text-sm text-muted-foreground">
              Built for the Rust community
            </div>
            <div className="flex items-center gap-4 sm:gap-6 text-xs sm:text-sm text-muted-foreground">
              <Link href="/docs" className="hover:text-foreground transition-colors">
                Docs
              </Link>
              <Link href="/privacy" className="hover:text-foreground transition-colors">
                Privacy
              </Link>
              <Link href="/terms" className="hover:text-foreground transition-colors">
                Terms
              </Link>
            </div>
          </div>
        </div>
      </footer>
    </div>
  );
}
