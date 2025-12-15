<script lang="ts">
  import { Button, Card, CardHeader, CardTitle, CardDescription, CardContent } from "$lib/components/ui";
  import Plus from "lucide-svelte/icons/plus";

  let { data } = $props();
</script>

<svelte:head>
  <title>Workspaces - Driftwatch</title>
</svelte:head>

<div>
  <div class="flex items-center justify-between">
    <div>
      <h1 class="text-3xl font-bold">Workspaces</h1>
      <p class="text-muted-foreground">
        Manage your Cargo workspaces for benchmark tracking
      </p>
    </div>
    <a href="/workspaces/new">
      <Button>
        <Plus class="mr-2 h-4 w-4" />
        New Workspace
      </Button>
    </a>
  </div>

  {#if data.projects.length === 0}
    <Card class="mt-8">
      <CardHeader>
        <CardTitle>No workspaces yet</CardTitle>
        <CardDescription>
          Create your first workspace to start tracking your Cargo workspace benchmarks
        </CardDescription>
      </CardHeader>
      <CardContent>
        <a href="/workspaces/new">
          <Button>Create Workspace</Button>
        </a>
      </CardContent>
    </Card>
  {:else}
    <div class="mt-8 grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
      {#each data.projects as project (project.id)}
        <a href={`/workspaces/${project.slug}`}>
          <Card class="h-full transition-colors hover:bg-muted/50">
            <CardHeader>
              <CardTitle class="flex items-center justify-between">
                {project.name}
                {#if project.public}
                  <span class="text-xs font-normal text-muted-foreground">
                    Public
                  </span>
                {/if}
              </CardTitle>
              <CardDescription>{project.slug}</CardDescription>
            </CardHeader>
            {#if project.description}
              <CardContent>
                <p class="text-sm text-muted-foreground line-clamp-2">
                  {project.description}
                </p>
              </CardContent>
            {/if}
          </Card>
        </a>
      {/each}
    </div>
  {/if}
</div>
