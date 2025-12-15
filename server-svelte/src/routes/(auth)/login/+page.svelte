<script lang="ts">
  import { goto } from "$app/navigation";
  import { authClient } from "$lib/auth-client";
  import { Button, Input, Label } from "$lib/components/ui";
  import Github from "lucide-svelte/icons/github";
  import LoaderCircle from "lucide-svelte/icons/loader-circle";

  let email = $state("");
  let password = $state("");
  let isLoading = $state(false);
  let isGitHubLoading = $state(false);
  let error = $state<string | null>(null);

  async function handleEmailLogin(e: SubmitEvent) {
    e.preventDefault();
    isLoading = true;
    error = null;

    const result = await authClient.signIn.email({
      email,
      password,
    });

    if (result.error) {
      error = result.error.message || "Invalid email or password";
      isLoading = false;
    } else {
      goto("/workspaces");
    }
  }

  function handleGitHubLogin() {
    isGitHubLoading = true;
    authClient.signIn.social({
      provider: "github",
      callbackURL: "/workspaces",
    });
  }
</script>

<svelte:head>
  <title>Sign In - Driftwatch</title>
</svelte:head>

<div class="container relative min-h-screen flex-col items-center justify-center grid lg:max-w-none lg:grid-cols-2 lg:px-0">
  <div class="relative hidden h-full flex-col bg-muted p-10 text-white dark:border-r lg:flex">
    <div class="absolute inset-0 bg-zinc-900"></div>
    <a href="/" class="relative z-20 flex items-center text-lg font-medium">
      <svg
        xmlns="http://www.w3.org/2000/svg"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        class="mr-2 h-6 w-6"
      >
        <path d="M15 6v12a3 3 0 1 0 3-3H6a3 3 0 1 0 3 3V6a3 3 0 1 0-3 3h12a3 3 0 1 0-3-3" />
      </svg>
      Driftwatch
    </a>
    <div class="relative z-20 mt-auto">
      <blockquote class="space-y-2">
        <p class="text-lg">
          "Driftwatch has transformed how we track performance in our Rust projects.
          Catching regressions early has saved us countless debugging hours."
        </p>
        <footer class="text-sm text-zinc-400">A Happy Developer</footer>
      </blockquote>
    </div>
  </div>
  <div class="lg:p-8">
    <div class="mx-auto flex w-full flex-col justify-center space-y-6 sm:w-[350px]">
      <div class="flex flex-col space-y-2 text-center">
        <h1 class="text-2xl font-semibold tracking-tight">
          Sign in to your account
        </h1>
        <p class="text-sm text-muted-foreground">
          Enter your email and password below to sign in
        </p>
      </div>

      <div class="grid gap-6">
        <form onsubmit={handleEmailLogin}>
          <div class="grid gap-4">
            <div class="grid gap-2">
              <Label for="email">Email</Label>
              <Input
                id="email"
                type="email"
                placeholder="name@example.com"
                autocapitalize="none"
                autocomplete="email"
                autocorrect="off"
                bind:value={email}
                required
                disabled={isLoading || isGitHubLoading}
              />
            </div>
            <div class="grid gap-2">
              <Label for="password">Password</Label>
              <Input
                id="password"
                type="password"
                placeholder="••••••••"
                autocomplete="current-password"
                bind:value={password}
                required
                disabled={isLoading || isGitHubLoading}
              />
            </div>
            {#if error}
              <p class="text-sm text-destructive">{error}</p>
            {/if}
            <Button type="submit" disabled={isLoading || isGitHubLoading} loading={isLoading}>
              {#if isLoading}
                Signing in...
              {:else}
                Sign in with Email
              {/if}
            </Button>
          </div>
        </form>

        <div class="relative">
          <div class="absolute inset-0 flex items-center">
            <span class="w-full border-t"></span>
          </div>
          <div class="relative flex justify-center text-xs uppercase">
            <span class="bg-background px-2 text-muted-foreground">
              Or continue with
            </span>
          </div>
        </div>

        <Button
          variant="outline"
          type="button"
          onclick={handleGitHubLogin}
          disabled={isLoading || isGitHubLoading}
        >
          {#if isGitHubLoading}
            <LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
          {:else}
            <Github class="mr-2 h-4 w-4" />
          {/if}
          GitHub
        </Button>

        <p class="text-center text-sm text-muted-foreground">
          Don't have an account?{" "}
          <a href="/register" class="underline underline-offset-4 hover:text-primary">
            Sign up
          </a>
        </p>
      </div>

      <p class="px-8 text-center text-sm text-muted-foreground">
        By clicking continue, you agree to our{" "}
        <a href="/terms" class="underline underline-offset-4 hover:text-primary">
          Terms of Service
        </a>{" "}
        and{" "}
        <a href="/privacy" class="underline underline-offset-4 hover:text-primary">
          Privacy Policy
        </a>.
      </p>
    </div>
  </div>
</div>
