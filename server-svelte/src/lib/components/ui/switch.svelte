<script lang="ts">
  import { cn } from "$lib/utils";

  interface Props {
    id?: string;
    checked?: boolean;
    disabled?: boolean;
    onchange?: (checked: boolean) => void;
    class?: string;
  }

  let { id, checked = $bindable(false), disabled = false, onchange, class: className }: Props =
    $props();

  function handleClick() {
    if (disabled) return;
    checked = !checked;
    onchange?.(checked);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === " " || e.key === "Enter") {
      e.preventDefault();
      handleClick();
    }
  }
</script>

<button
  type="button"
  role="switch"
  aria-checked={checked}
  {id}
  {disabled}
  onclick={handleClick}
  onkeydown={handleKeydown}
  class={cn(
    "peer inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:cursor-not-allowed disabled:opacity-50",
    checked ? "bg-primary" : "bg-input",
    className
  )}
>
  <span
    class={cn(
      "pointer-events-none block h-4 w-4 rounded-full bg-background shadow-lg ring-0 transition-transform",
      checked ? "translate-x-4" : "translate-x-0"
    )}
  ></span>
  <span class="sr-only">{checked ? "On" : "Off"}</span>
</button>
