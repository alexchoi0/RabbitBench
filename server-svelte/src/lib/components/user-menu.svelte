<script lang="ts">
  import { authClient } from "$lib/auth-client";
  import { Button } from "$lib/components/ui";
  import ChevronDown from "lucide-svelte/icons/chevron-down";
  import LogOut from "lucide-svelte/icons/log-out";
  import Settings from "lucide-svelte/icons/settings";
  import LayoutGrid from "lucide-svelte/icons/layout-grid";

  interface Props {
    user: {
      name: string | null;
      email: string;
      image: string | null;
    };
  }

  let { user }: Props = $props();

  let isOpen = $state(false);

  let initials = $derived(
    user.name
      ?.split(" ")
      .map((n) => n[0])
      .join("")
      .toUpperCase() || user.email[0].toUpperCase()
  );

  function handleSignOut() {
    authClient.signOut({
      fetchOptions: {
        onSuccess: () => {
          window.location.href = "/";
        },
      },
    });
  }

  function handleClickOutside(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (!target.closest(".user-menu")) {
      isOpen = false;
    }
  }
</script>

<svelte:window onclick={handleClickOutside} />

<div class="relative user-menu">
  <Button
    variant="ghost"
    class="relative h-8 w-8 rounded-full p-0"
    onclick={() => (isOpen = !isOpen)}
  >
    {#if user.image}
      <img
        src={user.image}
        alt={user.name || ""}
        class="h-8 w-8 rounded-full object-cover"
      />
    {:else}
      <div class="flex h-8 w-8 items-center justify-center rounded-full bg-muted text-sm font-medium">
        {initials}
      </div>
    {/if}
  </Button>

  {#if isOpen}
    <div class="absolute right-0 mt-2 w-56 rounded-md border bg-popover p-1 shadow-lg z-50">
      <div class="px-2 py-1.5">
        <p class="text-sm font-medium">{user.name}</p>
        <p class="text-xs text-muted-foreground">{user.email}</p>
      </div>
      <div class="my-1 h-px bg-border"></div>
      <a
        href="/workspaces"
        class="flex items-center gap-2 rounded-sm px-2 py-1.5 text-sm hover:bg-accent"
        onclick={() => (isOpen = false)}
      >
        <LayoutGrid class="h-4 w-4" />
        Workspaces
      </a>
      <a
        href="/settings"
        class="flex items-center gap-2 rounded-sm px-2 py-1.5 text-sm hover:bg-accent"
        onclick={() => (isOpen = false)}
      >
        <Settings class="h-4 w-4" />
        Settings
      </a>
      <div class="my-1 h-px bg-border"></div>
      <button
        class="flex w-full items-center gap-2 rounded-sm px-2 py-1.5 text-sm hover:bg-accent"
        onclick={handleSignOut}
      >
        <LogOut class="h-4 w-4" />
        Sign out
      </button>
    </div>
  {/if}
</div>
