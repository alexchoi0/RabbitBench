import { APIRequestContext } from "@playwright/test";
import crypto from "crypto";

const GRAPHQL_URL = "/api/graphql";

export interface GraphQLResponse<T = unknown> {
  data?: T;
  errors?: Array<{ message: string }>;
}

/**
 * Execute a GraphQL query/mutation
 */
export async function graphql<T = unknown>(
  request: APIRequestContext,
  query: string,
  variables?: Record<string, unknown>,
  token?: string
): Promise<GraphQLResponse<T>> {
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
  };

  if (token) {
    headers["Authorization"] = `Bearer ${token}`;
  }

  const response = await request.post(GRAPHQL_URL, {
    headers,
    data: { query, variables },
  });

  return response.json();
}

/**
 * Convert a string ID to a deterministic UUID (v5-like)
 * This ensures the same ID always produces the same UUID
 */
function stringToUuid(str: string): string {
  const hash = crypto.createHash("sha256").update(str).digest("hex");
  // Format as UUID v4 format (xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx)
  return [
    hash.substring(0, 8),
    hash.substring(8, 12),
    "4" + hash.substring(13, 16),
    ((parseInt(hash.substring(16, 17), 16) & 0x3) | 0x8).toString(16) + hash.substring(17, 20),
    hash.substring(20, 32),
  ].join("-");
}

/**
 * Create a JWT token for testing (mimics createGraphQLToken from auth.ts)
 */
export function createTestToken(user: {
  id: string;
  email: string;
  name?: string;
}): string {

  // Convert string ID to valid UUID
  const userId = stringToUuid(user.id);

  const header = { alg: "HS256", typ: "JWT" };
  const now = Math.floor(Date.now() / 1000);
  const payload = {
    sub: userId,
    email: user.email,
    name: user.name,
    iat: now,
    exp: now + 3600,
  };

  const base64UrlEncode = (data: string) =>
    Buffer.from(data).toString("base64url");

  const secret = process.env.AUTH_SECRET || "e2e-test-secret-that-is-at-least-32-characters-long";

  const headerEncoded = base64UrlEncode(JSON.stringify(header));
  const payloadEncoded = base64UrlEncode(JSON.stringify(payload));
  const signatureInput = `${headerEncoded}.${payloadEncoded}`;

  const signature = crypto
    .createHmac("sha256", secret)
    .update(signatureInput)
    .digest("base64url");

  return `${signatureInput}.${signature}`;
}

/**
 * Generate a unique slug for testing
 */
export function uniqueSlug(prefix: string = "test"): string {
  return `${prefix}-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
}

// GraphQL Queries
export const queries = {
  me: `
    query Me {
      me {
        id
        email
        name
        avatarUrl
      }
    }
  `,

  projects: `
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
  `,

  project: `
    query Project($slug: String!) {
      project(slug: $slug) {
        id
        slug
        name
        description
        public
        createdAt
        githubRepo
        githubPrComments
        githubStatusChecks
        hasGithubToken
        branches {
          id
          name
        }
        testbeds {
          id
          name
        }
        measures {
          id
          name
          units
        }
        benchmarks {
          id
          name
        }
        thresholds {
          id
          branchId
          testbedId
          measureId
          upperBoundary
          lowerBoundary
          minSampleSize
        }
        recentReports(limit: 10) {
          id
          gitHash
          createdAt
          branch {
            id
            name
          }
          testbed {
            id
            name
          }
        }
      }
    }
  `,

  perf: `
    query Perf(
      $projectSlug: String!
      $benchmarks: [ID!]!
      $branches: [ID!]!
      $measures: [ID!]!
      $testbeds: [ID!]!
    ) {
      perf(
        projectSlug: $projectSlug
        benchmarks: $benchmarks
        branches: $branches
        measures: $measures
        testbeds: $testbeds
      ) {
        series {
          benchmark {
            id
            name
          }
          branch {
            id
            name
          }
          testbed {
            id
            name
          }
          measure {
            id
            name
          }
          data {
            x
            y
            lower
            upper
            gitHash
          }
        }
      }
    }
  `,

  alerts: `
    query Alerts($projectSlug: String!, $status: AlertStatus) {
      alerts(projectSlug: $projectSlug, status: $status) {
        id
        baselineValue
        percentChange
        status
        createdAt
        metric {
          id
          value
          benchmark {
            id
            name
          }
          measure {
            id
            name
          }
        }
      }
    }
  `,
};

// GraphQL Mutations
export const mutations = {
  createProject: `
    mutation CreateProject($input: CreateProjectInput!) {
      createProject(input: $input) {
        id
        slug
        name
        description
        public
      }
    }
  `,

  updateProject: `
    mutation UpdateProject($slug: String!, $input: UpdateProjectInput!) {
      updateProject(slug: $slug, input: $input) {
        id
        slug
        name
        description
        public
      }
    }
  `,

  deleteProject: `
    mutation DeleteProject($slug: String!) {
      deleteProject(slug: $slug)
    }
  `,

  createReport: `
    mutation CreateReport($input: CreateReportInput!) {
      createReport(input: $input) {
        id
        gitHash
        createdAt
        branch {
          id
          name
        }
        testbed {
          id
          name
        }
        metrics {
          id
          value
          lowerValue
          upperValue
          benchmark {
            id
            name
          }
          measure {
            id
            name
          }
        }
      }
    }
  `,

  createThreshold: `
    mutation CreateThreshold($input: CreateThresholdInput!) {
      createThreshold(input: $input) {
        id
        branchId
        testbedId
        measureId
        upperBoundary
        lowerBoundary
        minSampleSize
      }
    }
  `,

  updateThreshold: `
    mutation UpdateThreshold($id: ID!, $input: UpdateThresholdInput!) {
      updateThreshold(id: $id, input: $input) {
        id
        upperBoundary
        lowerBoundary
        minSampleSize
      }
    }
  `,

  deleteThreshold: `
    mutation DeleteThreshold($id: ID!) {
      deleteThreshold(id: $id)
    }
  `,

  dismissAlert: `
    mutation DismissAlert($id: ID!) {
      dismissAlert(id: $id) {
        id
        status
      }
    }
  `,

  createApiToken: `
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
  `,

  revokeApiToken: `
    mutation RevokeApiToken($id: ID!) {
      revokeApiToken(id: $id)
    }
  `,

  updateGitHubSettings: `
    mutation UpdateGitHubSettings($slug: String!, $input: UpdateGitHubSettingsInput!) {
      updateGitHubSettings(slug: $slug, input: $input) {
        id
        githubRepo
        githubPrComments
        githubStatusChecks
        hasGithubToken
      }
    }
  `,
};
