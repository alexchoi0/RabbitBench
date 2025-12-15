<script lang="ts" module>
  export { default as Card } from "./card.svelte";
</script>

<script lang="ts">
  import { cn } from "$lib/utils";
  import type { HTMLAttributes } from "svelte/elements";
  import type { Snippet } from "svelte";

  interface Props extends HTMLAttributes<HTMLDivElement> {
    class?: string;
    children?: Snippet;
  }

  let { class: className, children, ...rest }: Props = $props();
</script>

<div class={cn("rounded-xl border bg-card text-card-foreground shadow", className)} {...rest}>
  {#if children}
    {@render children()}
  {/if}
</div>
