<script lang="ts">
  import { goto, invalidateAll } from "$app/navigation";
  import {
    Button,
    Card,
    CardHeader,
    CardTitle,
    CardDescription,
    CardContent,
    Input,
    Label,
    Switch,
  } from "$lib/components/ui";
  import Copy from "lucide-svelte/icons/copy";
  import Eye from "lucide-svelte/icons/eye";
  import EyeOff from "lucide-svelte/icons/eye-off";
  import Trash2 from "lucide-svelte/icons/trash-2";
  import Plus from "lucide-svelte/icons/plus";
  import Github from "lucide-svelte/icons/github";

  let { data } = $props();

  let name = $state("");
  let description = $state("");
  let isPublic = $state(false);
  let isSaving = $state(false);
  let saveError = $state<string | null>(null);
  let saveSuccess = $state(false);

  let isDeleting = $state(false);
  let deleteConfirmation = $state("");
  let showDeleteDialog = $state(false);

  let newTokenName = $state("");
  let isCreatingToken = $state(false);
  let newTokenSecret = $state<string | null>(null);
  let showSecret = $state(false);
  let tokenError = $state<string | null>(null);

  let githubRepo = $state("");
  let githubToken = $state("");
  let githubPrComments = $state(false);
  let githubStatusChecks = $state(false);
  let hasGithubToken = $state(false);
  let isSavingGithub = $state(false);
  let githubError = $state<string | null>(null);
  let githubSuccess = $state(false);

  let tokenToRevoke = $state<{ id: string; name: string } | null>(null);

  $effect(() => {
    name = data.project.name;
    description = data.project.description || "";
    isPublic = data.project.public;
    githubRepo = data.project.githubRepo || "";
    githubPrComments = data.project.githubPrComments;
    githubStatusChecks = data.project.githubStatusChecks;
    hasGithubToken = data.project.hasGithubToken;
  });

  async function handleSave(e: SubmitEvent) {
    e.preventDefault();
    isSaving = true;
    saveError = null;
    saveSuccess = false;

    const formData = new FormData();
    formData.set("name", name);
    formData.set("description", description);
    formData.set("public", isPublic.toString());

    const response = await fetch("?/updateProject", {
      method: "POST",
      body: formData,
    });

    const result = await response.json();
    isSaving = false;

    if (result.type === "failure") {
      saveError = result.data?.error || "Failed to save";
      return;
    }

    saveSuccess = true;
    setTimeout(() => (saveSuccess = false), 3000);
  }

  async function handleDelete() {
    if (deleteConfirmation !== data.project.slug) return;

    isDeleting = true;

    const response = await fetch("?/deleteProject", {
      method: "POST",
    });

    const result = await response.json();

    if (result.type === "redirect") {
      goto(result.location);
      return;
    }

    isDeleting = false;
  }

  async function handleCreateToken(e: SubmitEvent) {
    e.preventDefault();
    if (!newTokenName.trim()) return;

    isCreatingToken = true;
    tokenError = null;

    const formData = new FormData();
    formData.set("name", newTokenName.trim());

    const response = await fetch("?/createToken", {
      method: "POST",
      body: formData,
    });

    const result = await response.json();
    isCreatingToken = false;

    if (result.type === "failure") {
      tokenError = result.data?.error || "Failed to create token";
      return;
    }

    if (result.data?.secret) {
      newTokenSecret = result.data.secret;
      newTokenName = "";
      await invalidateAll();
    }
  }

  async function handleRevokeToken(tokenId: string) {
    const formData = new FormData();
    formData.set("tokenId", tokenId);

    await fetch("?/revokeToken", {
      method: "POST",
      body: formData,
    });

    tokenToRevoke = null;
    await invalidateAll();
  }

  function copyToClipboard(text: string) {
    navigator.clipboard.writeText(text);
  }

  async function handleSaveGithub(e: SubmitEvent) {
    e.preventDefault();
    isSavingGithub = true;
    githubError = null;
    githubSuccess = false;

    const formData = new FormData();
    formData.set("githubRepo", githubRepo);
    formData.set("githubToken", githubToken);
    formData.set("githubPrComments", githubPrComments.toString());
    formData.set("githubStatusChecks", githubStatusChecks.toString());

    const response = await fetch("?/updateGitHub", {
      method: "POST",
      body: formData,
    });

    const result = await response.json();
    isSavingGithub = false;

    if (result.type === "failure") {
      githubError = result.data?.error || "Failed to save GitHub settings";
      return;
    }

    if (githubToken) {
      hasGithubToken = true;
      githubToken = "";
    }

    githubSuccess = true;
    setTimeout(() => (githubSuccess = false), 3000);
  }
</script>

<svelte:head>
  <title>Settings - {data.project.name} - Driftwatch</title>
</svelte:head>

<div class="max-w-3xl space-y-6">
  <div class="flex items-center gap-2 text-sm text-muted-foreground mb-1">
    <a href="/workspaces" class="hover:underline">Workspaces</a>
    <span>/</span>
    <a href={`/workspaces/${data.project.slug}`} class="hover:underline">
      {data.project.slug}
    </a>
    <span>/</span>
    <span>Settings</span>
  </div>

  <h1 class="text-3xl font-bold">Settings</h1>

  <Card>
    <CardHeader>
      <CardTitle>General</CardTitle>
      <CardDescription>Update your workspace name and description</CardDescription>
    </CardHeader>
    <CardContent>
      <form onsubmit={handleSave} class="space-y-4">
        <div class="space-y-2">
          <Label for="name">Name</Label>
          <Input id="name" bind:value={name} required />
        </div>

        <div class="space-y-2">
          <Label for="description">Description</Label>
          <Input id="description" bind:value={description} placeholder="Optional description" />
        </div>

        <div class="flex items-center justify-between rounded-lg border p-4">
          <div class="space-y-0.5">
            <Label for="public">Public Workspace</Label>
            <p class="text-sm text-muted-foreground">Make this workspace visible to everyone</p>
          </div>
          <Switch id="public" bind:checked={isPublic} />
        </div>

        {#if saveError}
          <p class="text-sm text-destructive">{saveError}</p>
        {/if}
        {#if saveSuccess}
          <p class="text-sm text-green-600">Settings saved!</p>
        {/if}

        <Button type="submit" loading={isSaving}>Save Changes</Button>
      </form>
    </CardContent>
  </Card>

  <Card>
    <CardHeader>
      <CardTitle>API Tokens</CardTitle>
      <CardDescription>Create tokens for CLI authentication</CardDescription>
    </CardHeader>
    <CardContent class="space-y-4">
      {#if newTokenSecret}
        <div class="rounded-lg border border-green-500 bg-green-50 dark:bg-green-950/20 p-4">
          <p class="text-sm font-medium text-green-800 dark:text-green-200 mb-2">
            Token created! Copy it now - you won't see it again.
          </p>
          <div class="flex items-center gap-2">
            <code class="flex-1 rounded bg-background px-2 py-1 font-mono text-sm">
              {showSecret ? newTokenSecret : "•".repeat(40)}
            </code>
            <Button variant="ghost" size="icon-sm" onclick={() => (showSecret = !showSecret)}>
              {#if showSecret}
                <EyeOff class="h-4 w-4" />
              {:else}
                <Eye class="h-4 w-4" />
              {/if}
            </Button>
            <Button variant="ghost" size="icon-sm" onclick={() => copyToClipboard(newTokenSecret!)}>
              <Copy class="h-4 w-4" />
            </Button>
          </div>
          <Button
            variant="outline"
            size="sm"
            class="mt-2"
            onclick={() => (newTokenSecret = null)}
          >
            Done
          </Button>
        </div>
      {/if}

      <form onsubmit={handleCreateToken} class="flex gap-2">
        <Input
          placeholder="Token name (e.g., CI/CD)"
          bind:value={newTokenName}
          class="flex-1"
        />
        <Button type="submit" loading={isCreatingToken} disabled={!newTokenName.trim()}>
          <Plus class="h-4 w-4 mr-1" />
          Create Token
        </Button>
      </form>

      {#if tokenError}
        <p class="text-sm text-destructive">{tokenError}</p>
      {/if}

      {#if data.tokens.length > 0}
        <div class="space-y-2">
          {#each data.tokens as token (token.id)}
            <div class="flex items-center justify-between rounded-lg border p-3">
              <div>
                <p class="font-medium">{token.name}</p>
                <p class="text-xs text-muted-foreground">
                  Created {new Date(token.createdAt).toLocaleDateString()}
                  {#if token.lastUsedAt}
                    · Last used {new Date(token.lastUsedAt).toLocaleDateString()}
                  {/if}
                </p>
              </div>
              <Button
                variant="ghost"
                size="icon-sm"
                onclick={() => (tokenToRevoke = { id: token.id, name: token.name })}
              >
                <Trash2 class="h-4 w-4 text-destructive" />
              </Button>
            </div>
          {/each}
        </div>
      {:else}
        <p class="text-sm text-muted-foreground py-4">
          No API tokens yet. Create one to use with the CLI.
        </p>
      {/if}
    </CardContent>
  </Card>

  <Card>
    <CardHeader>
      <CardTitle class="flex items-center gap-2">
        <Github class="h-5 w-5" />
        GitHub Integration
      </CardTitle>
      <CardDescription>Connect to GitHub for PR comments and status checks</CardDescription>
    </CardHeader>
    <CardContent>
      <form onsubmit={handleSaveGithub} class="space-y-4">
        <div class="space-y-2">
          <Label for="githubRepo">Repository</Label>
          <Input
            id="githubRepo"
            bind:value={githubRepo}
            placeholder="owner/repo"
          />
          <p class="text-xs text-muted-foreground">
            The GitHub repository to post comments to (e.g., owner/repo)
          </p>
        </div>

        <div class="space-y-2">
          <Label for="githubToken">Personal Access Token</Label>
          <Input
            id="githubToken"
            type="password"
            bind:value={githubToken}
            placeholder={hasGithubToken ? "••••••••••••••••" : "ghp_..."}
          />
          <p class="text-xs text-muted-foreground">
            {#if hasGithubToken}
              A token is configured. Enter a new token to replace it.
            {:else}
              Create a token with 'repo' scope at GitHub Settings > Developer settings > Personal
              access tokens
            {/if}
          </p>
        </div>

        <div class="space-y-3">
          <div class="flex items-center justify-between rounded-lg border p-4">
            <div class="space-y-0.5">
              <Label for="prComments">PR Comments</Label>
              <p class="text-sm text-muted-foreground">Post benchmark results as PR comments</p>
            </div>
            <Switch id="prComments" bind:checked={githubPrComments} disabled={!githubRepo} />
          </div>

          <div class="flex items-center justify-between rounded-lg border p-4">
            <div class="space-y-0.5">
              <Label for="statusChecks">Status Checks</Label>
              <p class="text-sm text-muted-foreground">
                Report performance regressions as commit status
              </p>
            </div>
            <Switch id="statusChecks" bind:checked={githubStatusChecks} disabled={!githubRepo} />
          </div>
        </div>

        {#if githubError}
          <p class="text-sm text-destructive">{githubError}</p>
        {/if}
        {#if githubSuccess}
          <p class="text-sm text-green-600">GitHub settings saved!</p>
        {/if}

        <Button type="submit" loading={isSavingGithub}>Save GitHub Settings</Button>
      </form>
    </CardContent>
  </Card>

  <Card class="border-destructive">
    <CardHeader>
      <CardTitle class="text-destructive">Danger Zone</CardTitle>
      <CardDescription>Irreversible actions for this workspace</CardDescription>
    </CardHeader>
    <CardContent>
      <Button variant="destructive" onclick={() => (showDeleteDialog = true)}>
        Delete Workspace
      </Button>
    </CardContent>
  </Card>
</div>

{#if showDeleteDialog}
  <button
    type="button"
    class="fixed inset-0 z-50 bg-black/80 cursor-default"
    onclick={() => (showDeleteDialog = false)}
    aria-label="Close dialog"
  ></button>
  <div
    class="fixed left-[50%] top-[50%] z-50 grid w-full max-w-lg translate-x-[-50%] translate-y-[-50%] gap-4 border bg-background p-6 shadow-lg sm:rounded-lg"
    role="alertdialog"
  >
    <div class="flex flex-col space-y-2 text-center sm:text-left">
      <h2 class="text-lg font-semibold">Delete Workspace</h2>
      <p class="text-sm text-muted-foreground">
        This will permanently delete the workspace "{data.project.name}" and all associated data
        including benchmarks, reports, and alerts. This action cannot be undone.
      </p>
    </div>
    <div class="py-4">
      <Label for="confirm">
        Type <span class="font-mono font-bold">{data.project.slug}</span> to confirm
      </Label>
      <Input
        id="confirm"
        bind:value={deleteConfirmation}
        class="mt-2"
        placeholder={data.project.slug}
      />
    </div>
    <div class="flex flex-col-reverse sm:flex-row sm:justify-end sm:space-x-2">
      <Button
        variant="outline"
        onclick={() => {
          showDeleteDialog = false;
          deleteConfirmation = "";
        }}
      >
        Cancel
      </Button>
      <Button
        variant="destructive"
        onclick={handleDelete}
        disabled={deleteConfirmation !== data.project.slug || isDeleting}
      >
        {isDeleting ? "Deleting..." : "Delete Workspace"}
      </Button>
    </div>
  </div>
{/if}

{#if tokenToRevoke}
  <button
    type="button"
    class="fixed inset-0 z-50 bg-black/80 cursor-default"
    onclick={() => (tokenToRevoke = null)}
    aria-label="Close dialog"
  ></button>
  <div
    class="fixed left-[50%] top-[50%] z-50 grid w-full max-w-lg translate-x-[-50%] translate-y-[-50%] gap-4 border bg-background p-6 shadow-lg sm:rounded-lg"
    role="alertdialog"
  >
    <div class="flex flex-col space-y-2 text-center sm:text-left">
      <h2 class="text-lg font-semibold">Revoke Token</h2>
      <p class="text-sm text-muted-foreground">
        Are you sure you want to revoke "{tokenToRevoke.name}"? Any applications using this token
        will lose access.
      </p>
    </div>
    <div class="flex flex-col-reverse sm:flex-row sm:justify-end sm:space-x-2">
      <Button variant="outline" onclick={() => (tokenToRevoke = null)}>Cancel</Button>
      <Button
        variant="destructive"
        onclick={() => handleRevokeToken(tokenToRevoke!.id)}
      >
        Revoke
      </Button>
    </div>
  </div>
{/if}
