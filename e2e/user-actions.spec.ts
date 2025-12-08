/**
 * User Actions Framework E2E Tests
 *
 * This file loads and executes YAML workflow definitions.
 */

import { test } from "@playwright/test";
import * as path from "path";
import { runWorkflowFile } from "./user-actions";

const WORKFLOWS_DIR = path.join(__dirname, "workflows");

test.describe("User Actions Workflows", () => {
  test.setTimeout(120000); // 2 minutes for full flows

  test("complete user flow", async ({ page }) => {
    const workflowPath = path.join(WORKFLOWS_DIR, "complete-user-flow.yaml");
    await runWorkflowFile(workflowPath, page);
  });
});
