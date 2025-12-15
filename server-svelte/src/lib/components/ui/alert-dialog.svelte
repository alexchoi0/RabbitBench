<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    open?: boolean;
    onOpenChange?: (open: boolean) => void;
    trigger?: Snippet;
    content?: Snippet;
  }

  let { open = $bindable(false), onOpenChange, trigger, content }: Props = $props();

  function handleBackdropClick() {
    open = false;
    onOpenChange?.(false);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape" && open) {
      open = false;
      onOpenChange?.(false);
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if trigger}
  {@render trigger()}
{/if}

{#if open}
  <div
    class="fixed inset-0 z-50 bg-black/80"
    onclick={handleBackdropClick}
    onkeydown={(e) => e.key === "Enter" && handleBackdropClick()}
    role="button"
    tabindex="-1"
    aria-label="Close dialog"
  ></div>
  <div
    class="fixed left-[50%] top-[50%] z-50 grid w-full max-w-lg translate-x-[-50%] translate-y-[-50%] gap-4 border bg-background p-6 shadow-lg sm:rounded-lg"
    role="alertdialog"
    aria-modal="true"
  >
    {#if content}
      {@render content()}
    {/if}
  </div>
{/if}
