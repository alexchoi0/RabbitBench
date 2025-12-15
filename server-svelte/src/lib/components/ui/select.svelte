<script lang="ts">
  import { cn } from "$lib/utils";
  import ChevronDown from "lucide-svelte/icons/chevron-down";

  interface Option {
    value: string;
    label: string;
  }

  interface Props {
    id?: string;
    value?: string;
    options?: Option[];
    placeholder?: string;
    disabled?: boolean;
    onchange?: (value: string) => void;
    class?: string;
  }

  let {
    id,
    value = $bindable(""),
    options = [],
    placeholder = "Select...",
    disabled = false,
    onchange,
    class: className,
  }: Props = $props();

  function handleChange(e: Event) {
    const target = e.target as HTMLSelectElement;
    value = target.value;
    onchange?.(value);
  }
</script>

<div class={cn("relative", className)}>
  <select
    {id}
    {disabled}
    bind:value
    onchange={handleChange}
    class={cn(
      "flex h-9 w-full appearance-none items-center justify-between whitespace-nowrap rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm ring-offset-background placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring disabled:cursor-not-allowed disabled:opacity-50",
      !value && "text-muted-foreground"
    )}
  >
    {#if placeholder}
      <option value="" disabled selected={!value}>{placeholder}</option>
    {/if}
    {#each options as option (option.value)}
      <option value={option.value}>{option.label}</option>
    {/each}
  </select>
  <ChevronDown
    class="pointer-events-none absolute right-3 top-1/2 h-4 w-4 -translate-y-1/2 opacity-50"
  />
</div>
