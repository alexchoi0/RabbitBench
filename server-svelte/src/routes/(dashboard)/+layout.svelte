<script lang="ts">
  import type { Snippet } from "svelte";
  import UserMenu from "$lib/components/user-menu.svelte";

  interface Props {
    data: {
      user: {
        id: string;
        name: string | null;
        email: string;
        image: string | null;
      };
    };
    children: Snippet;
  }

  let { data, children }: Props = $props();
</script>

<div class="min-h-screen bg-background">
  <header class="sticky top-0 z-50 border-b bg-background/95 backdrop-blur">
    <div class="container mx-auto flex h-16 items-center justify-between px-4">
      <div class="flex items-center gap-6">
        <a href="/" class="text-xl font-bold">
          Driftwatch
        </a>
        <nav class="flex gap-4 text-sm">
          <a
            href="/workspaces"
            class="text-muted-foreground hover:text-foreground transition-colors"
          >
            Workspaces
          </a>
        </nav>
      </div>
      <UserMenu user={data.user} />
    </div>
  </header>
  <main class="container mx-auto px-4 py-8">
    {@render children()}
  </main>
</div>
