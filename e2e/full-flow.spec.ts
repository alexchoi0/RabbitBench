import { test, expect } from "@playwright/test";
import { execSync } from "child_process";
import { writeFileSync, unlinkSync, mkdirSync } from "fs";
import { join } from "path";
import { tmpdir } from "os";

/**
 * Full E2E test covering the complete user journey:
 * 1. Land on home page
 * 2. Sign in (dev mode auto-auth)
 * 3. Create a workspace
 * 4. Get API token from settings
 * 5. Run CLI with benchmark data
 * 6. Verify results appear in UI
 */

const CRITERION_OUTPUT = `
Benchmarking e2e_test/operation1
e2e_test/operation1     time:   [100.00 ns 110.00 ns 120.00 ns]
                        change: [-0.5% +0.8% +2.3%] (p = 0.27 > 0.05)
                        No change in performance detected.

Benchmarking e2e_test/operation2
e2e_test/operation2     time:   [1.5000 µs 1.6000 µs 1.7000 µs]
                        change: [+1.0% +2.0% +3.0%] (p = 0.01 < 0.05)
                        Performance has regressed.
`;

test.describe("Full User Flow E2E", () => {
  test.setTimeout(120000); // 2 minutes for full flow

  test("complete journey: home -> workspace -> API token -> CLI -> results in UI", async ({
    page,
  }) => {
    const uniqueSlug = `e2e-flow-${Date.now()}`;
    const workspaceName = `E2E Flow Test ${Date.now()}`;

    // ============================================
    // STEP 1: Land on home page
    // ============================================
    await test.step("Visit home page", async () => {
      await page.goto("/");
      await expect(page).toHaveTitle(/RabbitBench/i);
      await expect(
        page.getByRole("heading", { name: /Benchmark Performance/i })
      ).toBeVisible();
    });

    // ============================================
    // STEP 2: Navigate to workspaces (auto-authenticated in dev mode)
    // ============================================
    await test.step("Navigate to workspaces via Get Started", async () => {
      await page.getByRole("button", { name: "Get Started" }).first().click();
      await expect(page).toHaveURL("/workspaces");
      await expect(
        page.getByRole("heading", { name: /Workspaces/i })
      ).toBeVisible();
    });

    // ============================================
    // STEP 3: Create a new workspace
    // ============================================
    await test.step("Create new workspace", async () => {
      await page.getByRole("button", { name: /New Workspace/i }).click();
      await expect(page).toHaveURL("/workspaces/new");

      // Fill in workspace form
      await page.getByLabel(/Slug/i).fill(uniqueSlug);
      await page.getByLabel(/Name/i).fill(workspaceName);
      await page.getByLabel(/Description/i).fill("E2E test workspace");

      // Submit form
      await page.getByRole("button", { name: /Create Workspace/i }).click();

      // Should redirect to the new workspace page
      await expect(page).toHaveURL(`/workspaces/${uniqueSlug}`, { timeout: 15000 });
      await expect(
        page.getByRole("heading", { name: workspaceName })
      ).toBeVisible({ timeout: 10000 });
    });

    // ============================================
    // STEP 4: Navigate to settings and create API token
    // ============================================
    let apiToken: string;
    await test.step("Get API token from settings", async () => {
      // Navigate to settings (LinkButton renders as a <button>, not <a>)
      await page.getByRole("button", { name: /Settings/i }).click();
      await expect(page).toHaveURL(`/workspaces/${uniqueSlug}/settings`, { timeout: 15000 });

      // Wait for page to load
      await expect(page.getByRole("heading", { name: /Settings/i })).toBeVisible();

      // Find the API Tokens section and create a new token
      const tokenInput = page.getByPlaceholder(/Token name/i);
      await expect(tokenInput).toBeVisible({ timeout: 10000 });

      await tokenInput.fill(`E2E CLI Token ${Date.now()}`);
      await page.getByRole("button", { name: /Create Token/i }).click();

      // Wait for the token to appear in the green success box
      await page.waitForSelector("code", { timeout: 10000 });

      // Click the eye icon to show the token
      await page.getByRole("button").filter({ has: page.locator("svg") }).first().click();

      // Get the token from the code element
      const codeElement = page.locator("code").first();
      const tokenText = await codeElement.textContent();

      // Token format: rb_{32 hex chars}
      const tokenMatch = tokenText?.match(/rb_[a-f0-9]{32}/);
      apiToken = tokenMatch ? tokenMatch[0] : "";

      expect(apiToken).toMatch(/^rb_[a-f0-9]{32}$/);

      // Click Done to dismiss the token display
      await page.getByRole("button", { name: /Done/i }).click();
    });

    // ============================================
    // STEP 5: Run CLI with benchmark data
    // ============================================
    await test.step("Run CLI with benchmark data", async () => {
      // Create temporary script that outputs Criterion-like results
      const tempDir = join(tmpdir(), `rabbitbench-e2e-${Date.now()}`);
      mkdirSync(tempDir, { recursive: true });

      const scriptPath = join(tempDir, "mock-benchmark.sh");
      writeFileSync(
        scriptPath,
        `#!/bin/sh
cat << 'CRITERION_EOF'
${CRITERION_OUTPUT}
CRITERION_EOF
`,
        { mode: 0o755 }
      );

      // Path to CLI binary - go up from server to workspace root
      const cliBinary = join(process.cwd(), "..", "target", "release", "rabbitbench");

      // Get the API URL - CLI appends /graphql, so we need /api as base
      const apiUrl = "http://localhost:3000/api";

      try {
        // Run the CLI
        const result = execSync(
          `"${cliBinary}" run -p ${uniqueSlug} -b main --hash e2e123abc "${scriptPath}"`,
          {
            env: {
              ...process.env,
              RABBITBENCH_TOKEN: apiToken,
              RABBITBENCH_API_URL: apiUrl,
            },
            encoding: "utf-8",
            timeout: 30000,
          }
        );

        // Verify CLI output
        expect(result).toContain("Found 2 benchmark results");
        expect(result).toContain("e2e_test/operation1");
        expect(result).toContain("e2e_test/operation2");
        expect(result).toContain("Report submitted");
      } finally {
        // Cleanup
        try {
          unlinkSync(scriptPath);
        } catch {
          // Ignore cleanup errors
        }
      }
    });

    // ============================================
    // STEP 6: Verify results appear in UI
    // ============================================
    await test.step("Verify benchmark results in UI", async () => {
      // Navigate back to workspace
      await page.goto(`/workspaces/${uniqueSlug}`);

      // Wait for page to load
      await expect(
        page.getByRole("heading", { name: workspaceName })
      ).toBeVisible();

      // Click Reports tab
      await page.getByRole("tab", { name: /Reports/i }).click();

      // Wait for reports to load
      await page.waitForTimeout(1000);

      // Should see the report with git hash
      await expect(page.getByText(/e2e123/)).toBeVisible({ timeout: 10000 });

      // Should see branch info
      await expect(page.getByText(/main/)).toBeVisible();
    });
  });

  test("complete flow with multiple reports and GraphQL API", async ({
    page,
    request,
  }) => {
    const uniqueSlug = `e2e-multi-${Date.now()}`;
    const workspaceName = `E2E Multi Report Test`;

    // ============================================
    // STEP 1: Create workspace via GraphQL API (faster)
    // ============================================
    await test.step("Create workspace via API", async () => {
      const response = await request.post("/api/graphql", {
        headers: { "Content-Type": "application/json" },
        data: {
          query: `
            mutation CreateProject($input: CreateProjectInput!) {
              createProject(input: $input) {
                id
                slug
              }
            }
          `,
          variables: {
            input: {
              slug: uniqueSlug,
              name: workspaceName,
            },
          },
        },
      });

      const result = await response.json();
      expect(result.errors).toBeUndefined();
      expect(result.data?.createProject.slug).toBe(uniqueSlug);
    });

    // ============================================
    // STEP 2: Create API token via GraphQL
    // ============================================
    let apiToken: string;
    await test.step("Create API token via API", async () => {
      const response = await request.post("/api/graphql", {
        headers: { "Content-Type": "application/json" },
        data: {
          query: `
            mutation CreateApiToken($name: String!) {
              createApiToken(name: $name) {
                secret
              }
            }
          `,
          variables: {
            name: `E2E Multi Token ${Date.now()}`,
          },
        },
      });

      const result = await response.json();
      expect(result.errors).toBeUndefined();
      apiToken = result.data?.createApiToken.secret;
      expect(apiToken).toMatch(/^rb_[a-f0-9]{32}$/);
    });

    // ============================================
    // STEP 3: Submit multiple reports via CLI
    // ============================================
    await test.step("Submit multiple reports via CLI", async () => {
      const benchmarkOutputs = [
        {
          hash: "hash1abc",
          output: `multi_bench    time:   [100.00 ns 105.00 ns 110.00 ns]`,
        },
        {
          hash: "hash2def",
          output: `multi_bench    time:   [95.00 ns 100.00 ns 105.00 ns]`,
        },
        {
          hash: "hash3ghi",
          output: `multi_bench    time:   [110.00 ns 115.00 ns 120.00 ns]`,
        },
      ];

      const cliBinary = join(process.cwd(), "..", "target", "release", "rabbitbench");
      const apiUrl = "http://localhost:3000/api";

      for (const { hash, output } of benchmarkOutputs) {
        const tempDir = join(tmpdir(), `rabbitbench-multi-${Date.now()}-${hash}`);
        mkdirSync(tempDir, { recursive: true });

        const scriptPath = join(tempDir, "mock.sh");
        writeFileSync(
          scriptPath,
          `#!/bin/sh\ncat << 'EOF'\n${output}\nEOF\n`,
          { mode: 0o755 }
        );

        try {
          const result = execSync(
            `"${cliBinary}" run -p ${uniqueSlug} -b main --hash ${hash} "${scriptPath}"`,
            {
              env: {
                ...process.env,
                RABBITBENCH_TOKEN: apiToken,
                RABBITBENCH_API_URL: apiUrl,
              },
              encoding: "utf-8",
              timeout: 30000,
            }
          );

          expect(result).toContain("Report submitted");
        } finally {
          try {
            unlinkSync(scriptPath);
          } catch {
            // Ignore
          }
        }

        // Small delay between reports
        await new Promise((resolve) => setTimeout(resolve, 200));
      }
    });

    // ============================================
    // STEP 4: Verify all reports in UI
    // ============================================
    await test.step("Verify all reports in UI", async () => {
      await page.goto(`/workspaces/${uniqueSlug}`);

      // Click Reports tab
      await page.getByRole("tab", { name: /Reports/i }).click();

      // Wait for reports to load
      await page.waitForTimeout(1000);

      // Should see all three reports
      await expect(page.getByText(/hash1/)).toBeVisible({ timeout: 10000 });
      await expect(page.getByText(/hash2/)).toBeVisible();
      await expect(page.getByText(/hash3/)).toBeVisible();
    });

    // ============================================
    // STEP 5: Verify Performance chart has data
    // ============================================
    await test.step("Verify Performance chart", async () => {
      // Click Performance tab
      await page.getByRole("tab", { name: /Performance/i }).click();

      // Wait for chart to load
      await page.waitForTimeout(2000);

      // The chart area should be visible
      await expect(page.locator("canvas, svg, [class*='chart']").first()).toBeVisible({
        timeout: 10000,
      });
    });
  });

  test("complete flow with PR number for GitHub integration", async ({
    page,
    request,
  }) => {
    const uniqueSlug = `e2e-pr-${Date.now()}`;
    const workspaceName = `E2E PR Test`;

    // Create workspace via GraphQL (faster than UI)
    await test.step("Create workspace via API", async () => {
      const response = await request.post("/api/graphql", {
        headers: { "Content-Type": "application/json" },
        data: {
          query: `
            mutation CreateProject($input: CreateProjectInput!) {
              createProject(input: $input) {
                id
                slug
              }
            }
          `,
          variables: {
            input: {
              slug: uniqueSlug,
              name: workspaceName,
            },
          },
        },
      });

      const result = await response.json();
      expect(result.errors).toBeUndefined();
    });

    // Create API token via GraphQL
    let apiToken: string;
    await test.step("Create API token via API", async () => {
      const response = await request.post("/api/graphql", {
        headers: { "Content-Type": "application/json" },
        data: {
          query: `
            mutation CreateApiToken($name: String!) {
              createApiToken(name: $name) {
                secret
              }
            }
          `,
          variables: {
            name: `E2E PR Token ${Date.now()}`,
          },
        },
      });

      const result = await response.json();
      apiToken = result.data?.createApiToken.secret;
      expect(apiToken).toMatch(/^rb_[a-f0-9]{32}$/);
    });

    // Run CLI with PR number
    await test.step("Run CLI with PR number", async () => {
      const tempDir = join(tmpdir(), `rabbitbench-e2e-pr-${Date.now()}`);
      mkdirSync(tempDir, { recursive: true });

      const scriptPath = join(tempDir, "mock-benchmark.sh");
      writeFileSync(
        scriptPath,
        `#!/bin/sh
cat << 'CRITERION_EOF'
pr_benchmark            time:   [50.00 ns 55.00 ns 60.00 ns]
CRITERION_EOF
`,
        { mode: 0o755 }
      );

      const cliBinary = join(process.cwd(), "..", "target", "release", "rabbitbench");
      const apiUrl = "http://localhost:3000/api";

      try {
        const result = execSync(
          `"${cliBinary}" run -p ${uniqueSlug} -b feature-pr --pr 42 --hash pr123abc "${scriptPath}"`,
          {
            env: {
              ...process.env,
              RABBITBENCH_TOKEN: apiToken,
              RABBITBENCH_API_URL: apiUrl,
            },
            encoding: "utf-8",
            timeout: 30000,
          }
        );

        expect(result).toContain("PR: #42");
        expect(result).toContain("Branch: feature-pr");
        expect(result).toContain("Report submitted");
      } finally {
        try {
          unlinkSync(scriptPath);
        } catch {
          // Ignore
        }
      }
    });

    // Verify in UI
    await test.step("Verify PR report in UI", async () => {
      await page.goto(`/workspaces/${uniqueSlug}`);
      await page.getByRole("tab", { name: /Reports/i }).click();

      // Wait for content
      await page.waitForTimeout(1000);

      // Should see the report
      await expect(page.getByText(/pr123/)).toBeVisible({ timeout: 10000 });
      await expect(page.getByText(/feature-pr/)).toBeVisible();
    });
  });
});
