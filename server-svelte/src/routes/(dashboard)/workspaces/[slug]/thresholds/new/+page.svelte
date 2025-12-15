<script lang="ts">
  import { goto } from "$app/navigation";
  import {
    Button,
    Card,
    CardHeader,
    CardTitle,
    CardDescription,
    CardContent,
    Input,
    Label,
    Select,
  } from "$lib/components/ui";

  let { data } = $props();

  let measureId = $state("");
  let branchId = $state("all");
  let testbedId = $state("all");
  let upperBoundary = $state("");
  let lowerBoundary = $state("");
  let minSampleSize = $state("2");

  let isLoading = $state(false);
  let error = $state<string | null>(null);

  $effect(() => {
    if (data.project.measures.length > 0 && !measureId) {
      measureId = data.project.measures[0].id;
    }
  });

  let measureOptions = $derived([
    ...data.project.measures.map((m) => ({
      value: m.id,
      label: m.units ? `${m.name} (${m.units})` : m.name,
    })),
  ]);

  let branchOptions = $derived([
    { value: "all", label: "All branches" },
    ...data.project.branches.map((b) => ({ value: b.id, label: b.name })),
  ]);

  let testbedOptions = $derived([
    { value: "all", label: "All testbeds" },
    ...data.project.testbeds.map((t) => ({ value: t.id, label: t.name })),
  ]);

  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    isLoading = true;
    error = null;

    if (!measureId) {
      error = "Please select a measure";
      isLoading = false;
      return;
    }

    if (!upperBoundary && !lowerBoundary) {
      error = "Please set at least one boundary (upper or lower)";
      isLoading = false;
      return;
    }

    const formData = new FormData();
    formData.set("measureId", measureId);
    formData.set("branchId", branchId);
    formData.set("testbedId", testbedId);
    formData.set("upperBoundary", upperBoundary);
    formData.set("lowerBoundary", lowerBoundary);
    formData.set("minSampleSize", minSampleSize);

    const response = await fetch("?/create", {
      method: "POST",
      body: formData,
    });

    const result = await response.json();

    if (result.type === "failure") {
      error = result.data?.error || "Failed to create threshold";
      isLoading = false;
      return;
    }

    if (result.type === "redirect") {
      goto(result.location);
    }
  }
</script>

<svelte:head>
  <title>New Threshold - {data.project.name} - Driftwatch</title>
</svelte:head>

<div class="max-w-2xl">
  <div class="flex items-center gap-2 text-sm text-muted-foreground mb-1">
    <a href="/workspaces" class="hover:underline">Workspaces</a>
    <span>/</span>
    <a href={`/workspaces/${data.project.slug}`} class="hover:underline">
      {data.project.slug}
    </a>
    <span>/</span>
    <span>New Threshold</span>
  </div>
  <h1 class="text-3xl font-bold">New Threshold</h1>
  <p class="text-muted-foreground">
    Configure a threshold to detect performance regressions automatically
  </p>

  <Card class="mt-8">
    <CardHeader>
      <CardTitle>Threshold Configuration</CardTitle>
      <CardDescription>Set boundaries to trigger alerts when metrics exceed limits</CardDescription>
    </CardHeader>
    <CardContent>
      <form onsubmit={handleSubmit} class="space-y-6">
        <div class="space-y-2">
          <Label for="measure">Measure *</Label>
          {#if data.project.measures.length === 0}
            <p class="text-sm text-muted-foreground py-2">
              No measures available. Submit benchmark data first to create measures.
            </p>
          {:else}
            <Select
              id="measure"
              bind:value={measureId}
              options={measureOptions}
              placeholder="Select a measure"
            />
          {/if}
          <p class="text-xs text-muted-foreground">The metric to monitor for regressions</p>
        </div>

        <div class="grid gap-4 sm:grid-cols-2">
          <div class="space-y-2">
            <Label for="branch">Branch</Label>
            <Select id="branch" bind:value={branchId} options={branchOptions} />
          </div>

          <div class="space-y-2">
            <Label for="testbed">Testbed</Label>
            <Select id="testbed" bind:value={testbedId} options={testbedOptions} />
          </div>
        </div>

        <div class="grid gap-4 sm:grid-cols-2">
          <div class="space-y-2">
            <Label for="upperBoundary">Upper Boundary (%)</Label>
            <Input
              id="upperBoundary"
              type="number"
              step="0.1"
              min="0"
              placeholder="e.g., 10"
              bind:value={upperBoundary}
            />
            <p class="text-xs text-muted-foreground">
              Alert if metric increases by more than this %
            </p>
          </div>

          <div class="space-y-2">
            <Label for="lowerBoundary">Lower Boundary (%)</Label>
            <Input
              id="lowerBoundary"
              type="number"
              step="0.1"
              min="0"
              placeholder="e.g., 10"
              bind:value={lowerBoundary}
            />
            <p class="text-xs text-muted-foreground">
              Alert if metric decreases by more than this %
            </p>
          </div>
        </div>

        <div class="space-y-2">
          <Label for="minSampleSize">Minimum Sample Size</Label>
          <Input id="minSampleSize" type="number" min="1" bind:value={minSampleSize} />
          <p class="text-xs text-muted-foreground">
            Number of previous data points to compare against (default: 2)
          </p>
        </div>

        {#if error}
          <p class="text-sm text-destructive">{error}</p>
        {/if}

        <div class="flex gap-4">
          <Button type="submit" loading={isLoading} disabled={data.project.measures.length === 0}>
            Create Threshold
          </Button>
          <Button type="button" variant="outline" onclick={() => history.back()}>Cancel</Button>
        </div>
      </form>
    </CardContent>
  </Card>
</div>
