<script lang="ts">
  import { Button } from "$lib/components/ui";
  import Github from "lucide-svelte/icons/github";
  import Upload from "lucide-svelte/icons/upload";
  import BarChart3 from "lucide-svelte/icons/bar-chart-3";
  import Bell from "lucide-svelte/icons/bell";
  import Check from "lucide-svelte/icons/check";

  const BRAND_COLOR = "#ff5533";

  let { data } = $props();
  let isSignedIn = $derived(!!data.session);
</script>

<div class="min-h-screen bg-background">
  <header class="sticky top-0 z-50 border-b bg-background/80 backdrop-blur-sm">
    <div class="mx-auto flex h-14 max-w-5xl items-center justify-between px-4 sm:h-16 sm:px-6">
      <a href="/" class="text-lg font-semibold tracking-tight sm:text-xl">
        Driftwatch
      </a>
      <nav class="flex items-center gap-6">
        <a
          href="/docs"
          class="text-sm text-muted-foreground transition-colors hover:text-foreground"
        >
          Docs
        </a>
        {#if isSignedIn}
          <a href="/workspaces">
            <Button variant="ghost" size="sm">Dashboard</Button>
          </a>
        {:else}
          <a href="/login">
            <Button variant="ghost" size="sm">Sign In</Button>
          </a>
        {/if}
      </nav>
    </div>
  </header>

  <main>
    <!-- Hero Section -->
    <section class="relative overflow-hidden">
      <div class="absolute inset-0 dot-background opacity-50"></div>
      <div class="relative mx-auto max-w-5xl px-4 py-16 sm:px-6 sm:py-24 lg:py-32">
        <div class="flex flex-col items-center text-center">
          <h1 class="max-w-4xl text-2xl font-bold tracking-tight sm:text-4xl lg:text-5xl">
            Guard Your
            <span style="color: {BRAND_COLOR}">Performance</span>
          </h1>
          <p class="mt-4 max-w-2xl text-sm text-muted-foreground sm:mt-6 sm:text-base lg:text-lg">
            Automatically track and visualize your Criterion benchmarks.
            Get alerted when performance regresses.
          </p>
          <div class="mt-8 flex flex-col gap-3 sm:mt-10 sm:flex-row sm:gap-4">
            <a href={isSignedIn ? "/workspaces" : "/register"}>
              <Button size="lg" class="h-11 px-6 sm:h-12 sm:px-8">
                Get Started
              </Button>
            </a>
            <a
              href="https://github.com/alexchoi0/driftwatch"
              target="_blank"
              rel="noopener noreferrer"
            >
              <Button variant="outline" size="lg" class="h-11 px-6 sm:h-12 sm:px-8">
                <Github class="mr-2 h-5 w-5" />
                View on GitHub
              </Button>
            </a>
          </div>
        </div>
      </div>
    </section>

    <!-- Features Section -->
    <section class="border-t bg-muted/30">
      <div class="mx-auto max-w-5xl px-4 py-16 sm:px-6 sm:py-24">
        <div class="text-center mb-10 sm:mb-16">
          <h2 class="text-xl font-bold tracking-tight sm:text-2xl lg:text-3xl">
            Everything you need
          </h2>
          <p class="mt-3 text-sm text-muted-foreground sm:mt-4 sm:text-base">
            Built for performance-critical Rust programs
          </p>
        </div>
        <div class="grid gap-4 sm:gap-8 sm:grid-cols-3">
          <div class="group border bg-card p-6 sm:p-8 transition-all hover:border-foreground/20 hover:shadow-lg">
            <div class="mb-4 inline-flex h-10 w-10 sm:h-12 sm:w-12 items-center justify-center bg-primary/10">
              <Upload class="h-5 w-5 sm:h-6 sm:w-6 text-primary" />
            </div>
            <h3 class="text-base sm:text-lg font-semibold">Collect</h3>
            <p class="mt-2 text-sm sm:text-base text-muted-foreground">
              Run your Criterion benchmarks with our CLI and automatically
              submit results to track over time.
            </p>
          </div>
          <div class="group border bg-card p-6 sm:p-8 transition-all hover:border-foreground/20 hover:shadow-lg">
            <div class="mb-4 inline-flex h-10 w-10 sm:h-12 sm:w-12 items-center justify-center bg-primary/10">
              <BarChart3 class="h-5 w-5 sm:h-6 sm:w-6 text-primary" />
            </div>
            <h3 class="text-base sm:text-lg font-semibold">Visualize</h3>
            <p class="mt-2 text-sm sm:text-base text-muted-foreground">
              Interactive charts show performance trends across branches,
              commits, and testbeds.
            </p>
          </div>
          <div class="group border bg-card p-6 sm:p-8 transition-all hover:border-foreground/20 hover:shadow-lg">
            <div class="mb-4 inline-flex h-10 w-10 sm:h-12 sm:w-12 items-center justify-center bg-primary/10">
              <Bell class="h-5 w-5 sm:h-6 sm:w-6 text-primary" />
            </div>
            <h3 class="text-base sm:text-lg font-semibold">Alert</h3>
            <p class="mt-2 text-sm sm:text-base text-muted-foreground">
              Configure thresholds and get notified when benchmarks regress
              beyond acceptable limits.
            </p>
          </div>
        </div>
      </div>
    </section>

    <!-- CLI Section -->
    <section class="border-t">
      <div class="mx-auto max-w-5xl px-4 py-16 sm:px-6 sm:py-24">
        <div class="grid gap-8 lg:grid-cols-2 lg:gap-12 items-center">
          <div>
            <h2 class="text-xl font-bold tracking-tight sm:text-2xl lg:text-3xl">
              Simple CLI interface
            </h2>
            <p class="mt-3 text-sm text-muted-foreground sm:mt-4 sm:text-base">
              Get started in minutes. Install our CLI, authenticate, and
              start tracking your benchmarks with a single command.
            </p>
            <ul class="mt-6 space-y-3 sm:mt-8 sm:space-y-4">
              <li class="flex items-start gap-3">
                <Check class="h-5 w-5 sm:h-6 sm:w-6 text-success flex-shrink-0" />
                <span class="text-sm sm:text-base text-muted-foreground">
                  Works with existing Criterion benchmarks
                </span>
              </li>
              <li class="flex items-start gap-3">
                <Check class="h-5 w-5 sm:h-6 sm:w-6 text-success flex-shrink-0" />
                <span class="text-sm sm:text-base text-muted-foreground">
                  CI/CD integration with GitHub Actions
                </span>
              </li>
              <li class="flex items-start gap-3">
                <Check class="h-5 w-5 sm:h-6 sm:w-6 text-success flex-shrink-0" />
                <span class="text-sm sm:text-base text-muted-foreground">
                  Automatic branch and commit detection
                </span>
              </li>
            </ul>
          </div>
          <div class="relative">
            <div class="absolute -inset-4 blur-xl" style="background: linear-gradient(to right, {BRAND_COLOR}33, {BRAND_COLOR}1a)"></div>
            <div class="relative rounded-xl border bg-[#0a0a0a] p-4 sm:p-6 text-[#ededed] shadow-2xl overflow-hidden">
              <div class="flex items-center gap-2 mb-3 sm:mb-4">
                <div class="h-2.5 w-2.5 sm:h-3 sm:w-3 rounded-full bg-[#ff5f56]"></div>
                <div class="h-2.5 w-2.5 sm:h-3 sm:w-3 rounded-full bg-[#ffbd2e]"></div>
                <div class="h-2.5 w-2.5 sm:h-3 sm:w-3 rounded-full bg-[#27c93f]"></div>
              </div>
              <pre class="text-xs sm:text-sm overflow-x-auto font-mono"><code><span class="text-[#a1a1a1]"># Install the CLI</span>
<span style="color: {BRAND_COLOR}">$</span> cargo install driftwatch

<span class="text-[#a1a1a1]"># Authenticate</span>
<span style="color: {BRAND_COLOR}">$</span> driftwatch auth login

<span class="text-[#a1a1a1]"># Run benchmarks and submit</span>
<span style="color: {BRAND_COLOR}">$</span> driftwatch run \
    --project my-lib \
    --branch main \
    "cargo bench"</code></pre>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- CTA Section -->
    <section class="border-t bg-muted/30">
      <div class="mx-auto max-w-5xl px-4 py-16 sm:px-6 sm:py-24 text-center">
        <h2 class="text-xl font-bold tracking-tight sm:text-2xl lg:text-3xl">
          Ready to track your benchmarks?
        </h2>
        <p class="mt-3 text-sm text-muted-foreground sm:mt-4 sm:text-base">
          Start for free. No credit card required.
        </p>
        <div class="mt-8 sm:mt-10">
          <a href={isSignedIn ? "/workspaces" : "/register"}>
            <Button size="lg" class="h-11 px-6 sm:h-12 sm:px-8">
              Get Started Free
            </Button>
          </a>
        </div>
      </div>
    </section>
  </main>

  <footer class="border-t">
    <div class="mx-auto max-w-5xl px-4 py-6 sm:px-6 sm:py-8">
      <div class="flex flex-col sm:flex-row items-center justify-between gap-4">
        <div class="text-xs sm:text-sm text-muted-foreground">
          Built for the Rust community
        </div>
        <div class="flex items-center gap-4 sm:gap-6 text-xs sm:text-sm text-muted-foreground">
          <a href="/docs" class="hover:text-foreground transition-colors">
            Docs
          </a>
          <a href="/privacy" class="hover:text-foreground transition-colors">
            Privacy
          </a>
          <a href="/terms" class="hover:text-foreground transition-colors">
            Terms
          </a>
        </div>
      </div>
    </div>
  </footer>
</div>
