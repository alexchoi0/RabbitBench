<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { Button, Card, CardHeader, CardTitle, CardDescription, CardContent } from "$lib/components/ui";
  import Settings from "lucide-svelte/icons/settings";
  import Plus from "lucide-svelte/icons/plus";

  let { data } = $props();

  let currentTab = $derived($page.url.searchParams.get("tab") || "perf");

  function setTab(tab: string) {
    const url = new URL($page.url);
    url.searchParams.set("tab", tab);
    goto(url.toString(), { replaceState: true, noScroll: true });
  }
</script>

<svelte:head>
  <title>{data.project.name} - Driftwatch</title>
</svelte:head>

<div class="space-y-6">
  <div class="flex items-center justify-between">
    <div>
      <div class="flex items-center gap-2 text-sm text-muted-foreground mb-1">
        <a href="/workspaces" class="hover:underline">Workspaces</a>
        <span>/</span>
        <span>{data.project.slug}</span>
      </div>
      <h1 class="text-3xl font-bold">{data.project.name}</h1>
      {#if data.project.description}
        <p class="text-muted-foreground mt-1">{data.project.description}</p>
      {/if}
    </div>
    <div class="flex gap-2">
      <a href={`/workspaces/${data.project.slug}/settings`}>
        <Button variant="outline">
          <Settings class="mr-2 h-4 w-4" />
          Settings
        </Button>
      </a>
    </div>
  </div>

  {#if data.alerts.length > 0}
    <Card class="border-destructive">
      <CardHeader class="pb-3">
        <CardTitle class="text-destructive flex items-center gap-2">
          <span class="h-2 w-2 rounded-full bg-destructive animate-pulse"></span>
          {data.alerts.length} Active Alert{data.alerts.length > 1 ? "s" : ""}
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div class="space-y-2">
          {#each data.alerts.slice(0, 3) as alert (alert.id)}
            <div class="flex items-center justify-between text-sm">
              <span>
                {alert.metric.benchmark.name} / {alert.metric.measure.name}
              </span>
              <span class="text-destructive font-mono">
                {alert.percentChange > 0 ? "+" : ""}{alert.percentChange.toFixed(1)}%
              </span>
            </div>
          {/each}
          {#if data.alerts.length > 3}
            <p class="text-xs text-muted-foreground">
              and {data.alerts.length - 3} more...
            </p>
          {/if}
        </div>
      </CardContent>
    </Card>
  {/if}

  <div class="space-y-4">
    <div class="inline-flex h-9 items-center justify-center rounded-lg bg-muted p-1 text-muted-foreground">
      <button
        class="inline-flex items-center justify-center whitespace-nowrap rounded-md px-3 py-1 text-sm font-medium ring-offset-background transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 {currentTab === 'perf' ? 'bg-background text-foreground shadow' : ''}"
        onclick={() => setTab("perf")}
      >
        Performance
      </button>
      <button
        class="inline-flex items-center justify-center whitespace-nowrap rounded-md px-3 py-1 text-sm font-medium ring-offset-background transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 {currentTab === 'reports' ? 'bg-background text-foreground shadow' : ''}"
        onclick={() => setTab("reports")}
      >
        Reports
      </button>
      <button
        class="inline-flex items-center justify-center whitespace-nowrap rounded-md px-3 py-1 text-sm font-medium ring-offset-background transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 {currentTab === 'thresholds' ? 'bg-background text-foreground shadow' : ''}"
        onclick={() => setTab("thresholds")}
      >
        Thresholds
      </button>
    </div>

    {#if currentTab === "perf"}
      <Card class="min-h-[400px]">
        <CardHeader>
          <CardTitle>Performance Over Time</CardTitle>
        </CardHeader>
        <CardContent>
          {#if data.project.benchmarks.length === 0}
            <p class="text-sm text-muted-foreground py-8 text-center">
              No benchmark data yet. Submit your first report to see performance charts.
            </p>
          {:else}
            <p class="text-sm text-muted-foreground py-8 text-center">
              Chart component will be implemented with LayerChart.
              <br />
              {data.project.benchmarks.length} benchmarks, {data.project.reports.length} reports available.
            </p>
          {/if}
        </CardContent>
      </Card>
    {/if}

    {#if currentTab === "reports"}
      <Card>
        <CardHeader>
          <CardTitle>Recent Reports</CardTitle>
        </CardHeader>
        <CardContent>
          {#if data.project.reports.length === 0}
            <p class="text-sm text-muted-foreground py-4">
              No reports yet. Reports will appear here once you submit benchmark data.
            </p>
          {:else}
            <div class="space-y-4">
              {#each data.project.reports as report (report.id)}
                <div class="flex items-center justify-between border-b pb-4 last:border-0 last:pb-0">
                  <div>
                    <p class="font-medium">{report.branch.name}</p>
                    <p class="text-sm text-muted-foreground">
                      {report.testbed.name}
                      {#if report.gitHash}
                        · <span class="font-mono">{report.gitHash.slice(0, 7)}</span>
                      {/if}
                    </p>
                  </div>
                  <div class="text-sm text-muted-foreground">
                    {new Date(report.createdAt).toLocaleDateString()}
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </CardContent>
      </Card>
    {/if}

    {#if currentTab === "thresholds"}
      <div class="flex justify-between items-center">
        <div>
          <h3 class="text-lg font-semibold">Thresholds</h3>
          <p class="text-sm text-muted-foreground">
            Configure regression detection rules
          </p>
        </div>
        <a href={`/workspaces/${data.project.slug}/thresholds/new`}>
          <Button variant="outline">
            <Plus class="mr-2 h-4 w-4" />
            Add Threshold
          </Button>
        </a>
      </div>

      {#if data.project.thresholds.length === 0}
        <p class="text-sm text-muted-foreground py-4">
          No thresholds configured. Add thresholds to automatically detect performance regressions.
        </p>
      {:else}
        <div class="grid gap-4">
          {#each data.project.thresholds as threshold (threshold.id)}
            {@const measure = data.project.measures.find((m) => m.id === threshold.measureId)}
            {@const branch = threshold.branchId ? data.project.branches.find((b) => b.id === threshold.branchId) : null}
            {@const testbed = threshold.testbedId ? data.project.testbeds.find((t) => t.id === threshold.testbedId) : null}
            <Card>
              <CardHeader class="pb-2">
                <CardTitle class="text-base">
                  {measure?.name || "Unknown measure"}
                </CardTitle>
                <CardDescription>
                  {branch?.name || "All branches"} / {testbed?.name || "All testbeds"}
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div class="flex gap-4 text-sm">
                  {#if threshold.upperBoundary}
                    <div>
                      <span class="text-muted-foreground">Upper: </span>
                      <span class="font-mono">+{threshold.upperBoundary}%</span>
                    </div>
                  {/if}
                  {#if threshold.lowerBoundary}
                    <div>
                      <span class="text-muted-foreground">Lower: </span>
                      <span class="font-mono">-{threshold.lowerBoundary}%</span>
                    </div>
                  {/if}
                  <div>
                    <span class="text-muted-foreground">Min samples: </span>
                    <span class="font-mono">{threshold.minSampleSize}</span>
                  </div>
                </div>
              </CardContent>
            </Card>
          {/each}
        </div>
      {/if}
    {/if}
  </div>
</div>
