<script lang="ts">
  import { goto } from "$app/navigation";
  import { Button, Card, CardHeader, CardTitle, CardDescription, CardContent, Input, Label } from "$lib/components/ui";

  let isLoading = $state(false);
  let error = $state<string | null>(null);

  let slug = $state("");
  let name = $state("");
  let description = $state("");

  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    isLoading = true;
    error = null;

    const formData = new FormData();
    formData.set("slug", slug);
    formData.set("name", name);
    formData.set("description", description);

    const response = await fetch("?/create", {
      method: "POST",
      body: formData,
    });

    const result = await response.json();

    if (result.type === "failure") {
      error = result.data?.error || "Failed to create workspace";
      isLoading = false;
      return;
    }

    if (result.type === "redirect") {
      goto(result.location);
      return;
    }

    if (result.data?.slug) {
      goto(`/workspaces/${result.data.slug}`);
    } else {
      error = "Failed to create workspace";
      isLoading = false;
    }
  }
</script>

<svelte:head>
  <title>New Workspace - Driftwatch</title>
</svelte:head>

<div class="max-w-2xl">
  <h1 class="text-3xl font-bold">New Workspace</h1>
  <p class="text-muted-foreground">
    Create a new workspace for your Cargo workspace benchmarks
  </p>

  <Card class="mt-8">
    <CardHeader>
      <CardTitle>Workspace Details</CardTitle>
      <CardDescription>
        Enter the details for your Cargo workspace
      </CardDescription>
    </CardHeader>
    <CardContent>
      <form onsubmit={handleSubmit} class="space-y-4">
        <div class="space-y-2">
          <Label for="slug">Slug</Label>
          <Input
            id="slug"
            name="slug"
            placeholder="my-workspace"
            pattern="[a-z0-9-]+"
            required
            bind:value={slug}
          />
          <p class="text-xs text-muted-foreground">
            URL-friendly identifier (lowercase letters, numbers, hyphens)
          </p>
        </div>

        <div class="space-y-2">
          <Label for="name">Name</Label>
          <Input
            id="name"
            name="name"
            placeholder="My Workspace"
            required
            bind:value={name}
          />
        </div>

        <div class="space-y-2">
          <Label for="description">Description (optional)</Label>
          <Input
            id="description"
            name="description"
            placeholder="A brief description of your workspace"
            bind:value={description}
          />
        </div>

        {#if error}
          <p class="text-sm text-destructive">{error}</p>
        {/if}

        <div class="flex gap-4">
          <Button type="submit" loading={isLoading}>
            Create Workspace
          </Button>
          <Button
            type="button"
            variant="outline"
            onclick={() => history.back()}
          >
            Cancel
          </Button>
        </div>
      </form>
    </CardContent>
  </Card>
</div>
