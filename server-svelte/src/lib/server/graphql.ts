import { env } from "$env/dynamic/private";
import { getCached, setCache } from "./cache";

const API_URL = env.API_URL || "http://localhost:4000/graphql";

interface CacheControlHint {
  path: string[];
  maxAge: number;
}

interface GraphQLResponse<T> {
  data?: T;
  errors?: Array<{ message: string }>;
  extensions?: {
    cacheControl?: {
      hints: CacheControlHint[];
    };
  };
}

interface QueryOptions {
  cacheKey?: string;
  skipCache?: boolean;
}

export async function gqlQuery<T>(
  query: string,
  variables?: Record<string, unknown>,
  token?: string,
  options?: QueryOptions
): Promise<T> {
  const { cacheKey, skipCache = false } = options ?? {};

  // Try cache first (only for queries with a cache key)
  if (cacheKey && !skipCache) {
    const cached = getCached<T>(cacheKey);
    if (cached) {
      return cached;
    }
  }

  const response = await fetch(API_URL, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      ...(token && { Authorization: `Bearer ${token}` }),
    },
    body: JSON.stringify({ query, variables }),
  });

  const result: GraphQLResponse<T> = await response.json();

  if (result.errors && result.errors.length > 0) {
    throw new Error(result.errors[0].message);
  }

  if (!result.data) {
    throw new Error("No data returned from GraphQL API");
  }

  // Calculate cache TTL from cacheControl hints
  if (cacheKey && !skipCache) {
    const hints = result.extensions?.cacheControl?.hints ?? [];
    const minMaxAge =
      hints.length > 0 ? Math.min(...hints.map((h) => h.maxAge)) : 0;

    // Use hint-based TTL if available, otherwise default
    const ttlMs = minMaxAge > 0 ? minMaxAge * 1000 : undefined;
    setCache(cacheKey, result.data, ttlMs);
  }

  return result.data;
}

export async function gqlMutate<T>(
  mutation: string,
  variables?: Record<string, unknown>,
  token?: string
): Promise<T> {
  return gqlQuery<T>(mutation, variables, token);
}

// GraphQL Queries
export const PROJECTS_QUERY = `
  query Projects {
    projects {
      id
      slug
      name
      description
      public
      createdAt
    }
  }
`;

export const PROJECT_QUERY = `
  query Project($slug: String!) {
    project(slug: $slug) {
      id
      slug
      name
      description
      public
      githubRepo
      githubPrComments
      githubStatusChecks
      hasGithubToken
      branches { id name }
      testbeds { id name }
      measures { id name units }
      benchmarks { id name }
      reports(limit: 10) {
        id
        gitHash
        prNumber
        createdAt
        branch { id name }
        testbed { id name }
      }
      thresholds {
        id
        measureId
        branchId
        testbedId
        upperBoundary
        lowerBoundary
        minSampleSize
      }
    }
  }
`;

export const PROJECT_SETTINGS_QUERY = `
  query ProjectSettings($slug: String!) {
    project(slug: $slug) {
      id
      slug
      name
      description
      public
      githubRepo
      githubPrComments
      githubStatusChecks
      hasGithubToken
    }
    apiTokens {
      id
      name
      lastUsedAt
      createdAt
    }
  }
`;

export const PROJECT_FOR_THRESHOLD_QUERY = `
  query ProjectForThreshold($slug: String!) {
    project(slug: $slug) {
      id
      slug
      name
      branches { id name }
      testbeds { id name }
      measures { id name units }
    }
  }
`;

export const ACTIVE_ALERTS_QUERY = `
  query ActiveAlerts($slug: String!) {
    project(slug: $slug) {
      alerts(status: ACTIVE) {
        id
        percentChange
        metric {
          benchmark { name }
          measure { name }
        }
      }
    }
  }
`;

// GraphQL Mutations
export const CREATE_PROJECT_MUTATION = `
  mutation CreateProject($input: CreateProjectInput!) {
    createProject(input: $input) {
      id
      slug
    }
  }
`;

export const UPDATE_PROJECT_MUTATION = `
  mutation UpdateProject($slug: String!, $input: UpdateProjectInput!) {
    updateProject(slug: $slug, input: $input) {
      id
      slug
      name
      description
      public
    }
  }
`;

export const DELETE_PROJECT_MUTATION = `
  mutation DeleteProject($slug: String!) {
    deleteProject(slug: $slug)
  }
`;

export const UPDATE_GITHUB_SETTINGS_MUTATION = `
  mutation UpdateGitHubSettings($slug: String!, $input: GitHubSettingsInput!) {
    updateGitHubSettings(slug: $slug, input: $input) {
      id
      githubRepo
      githubPrComments
      githubStatusChecks
      hasGithubToken
    }
  }
`;

export const CREATE_API_TOKEN_MUTATION = `
  mutation CreateApiToken($name: String!) {
    createApiToken(name: $name) {
      token {
        id
        name
        createdAt
      }
      secret
    }
  }
`;

export const REVOKE_API_TOKEN_MUTATION = `
  mutation RevokeApiToken($id: ID!) {
    revokeApiToken(id: $id)
  }
`;

export const CREATE_THRESHOLD_MUTATION = `
  mutation CreateThreshold($input: CreateThresholdInput!) {
    createThreshold(input: $input) {
      id
    }
  }
`;

export const DELETE_THRESHOLD_MUTATION = `
  mutation DeleteThreshold($id: ID!) {
    deleteThreshold(id: $id)
  }
`;
