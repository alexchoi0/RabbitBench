/**
 * User Actions Workflow Runner
 *
 * Parses YAML workflow files and executes them as Playwright tests.
 */

import * as fs from "fs";
import * as path from "path";
import * as yaml from "yaml";
import { test } from "@playwright/test";
import type { Page } from "@playwright/test";
import type {
  UserActionsWorkflow,
  Job,
  Step,
  WorkflowContext,
} from "./types";
import { actions } from "./actions";
import { executeAssertions } from "./assertions";
import { interpolateString } from "./selectors";

/**
 * Load and parse a workflow YAML file
 */
export function loadWorkflow(filePath: string): UserActionsWorkflow {
  const content = fs.readFileSync(filePath, "utf-8");
  return yaml.parse(content) as UserActionsWorkflow;
}

/**
 * Create initial workflow context
 */
function createContext(
  page: Page,
  workflow: UserActionsWorkflow,
  jobEnv?: Record<string, string>
): WorkflowContext {
  return {
    page,
    env: {
      ...process.env as Record<string, string>,
      ...workflow.env,
      ...jobEnv,
    },
    vars: {
      // Built-in variables
      timestamp: Date.now().toString(),
      random: Math.random().toString(36).substring(7),
    },
    outputs: {},
  };
}

/**
 * Execute a single step
 */
async function executeStep(
  step: Step,
  context: WorkflowContext
): Promise<void> {
  // Check conditional
  if (step.if) {
    const condition = interpolateString(step.if, context);
    // Simple truthy check - could be expanded to expression evaluation
    if (!condition || condition === "false" || condition === "0") {
      console.log(`  Skipping step "${step.name}" (condition not met)`);
      return;
    }
  }

  console.log(`  Running step: ${step.name}`);

  // Merge step env into context
  const stepContext: WorkflowContext = step.env
    ? {
        ...context,
        env: { ...context.env, ...step.env },
      }
    : context;

  // Execute action or custom run
  if (step.uses) {
    const action = actions[step.uses];
    if (!action) {
      throw new Error(`Unknown action: ${step.uses}`);
    }

    // Interpolate "with" parameters
    const params = step.with ? interpolateParams(step.with, stepContext) : {};
    await action(params, stepContext);
  } else if (step.run) {
    // Execute custom TypeScript code
    // This is a simplified version - in production you'd want sandboxing
    const code = interpolateString(step.run, stepContext);
    const AsyncFunction = Object.getPrototypeOf(async function () {}).constructor;
    const fn = new AsyncFunction("context", "page", code);
    await fn(stepContext, stepContext.page);
  }

  // Store step outputs if step has an id
  if (step.id) {
    context.outputs[step.id] = { ...context.vars };
  }

  // Execute assertions
  if (step.assert) {
    await executeAssertions(step.assert, stepContext);
  }
}

/**
 * Execute a job
 */
async function executeJob(
  jobName: string,
  job: Job,
  page: Page,
  workflow: UserActionsWorkflow,
  completedJobs: Set<string>
): Promise<void> {
  // Check dependencies
  if (job.needs) {
    const deps = Array.isArray(job.needs) ? job.needs : [job.needs];
    for (const dep of deps) {
      if (!completedJobs.has(dep)) {
        throw new Error(`Job "${jobName}" depends on "${dep}" which has not completed`);
      }
    }
  }

  console.log(`Running job: ${job.name || jobName}`);

  const context = createContext(page, workflow, job.env);

  for (const step of job.steps) {
    await executeStep(step, context);
  }

  completedJobs.add(jobName);
}

/**
 * Execute a complete workflow
 */
export async function executeWorkflow(
  workflow: UserActionsWorkflow,
  page: Page
): Promise<void> {
  console.log(`\nExecuting workflow: ${workflow.name}`);
  if (workflow.description) {
    console.log(`Description: ${workflow.description}`);
  }

  const completedJobs = new Set<string>();
  const jobNames = Object.keys(workflow.jobs);

  // Simple topological sort based on needs
  const sortedJobs = topologicalSort(workflow.jobs);

  for (const jobName of sortedJobs) {
    await executeJob(
      jobName,
      workflow.jobs[jobName],
      page,
      workflow,
      completedJobs
    );
  }

  console.log(`\nWorkflow "${workflow.name}" completed successfully`);
}

/**
 * Topological sort jobs based on dependencies
 */
function topologicalSort(jobs: Record<string, Job>): string[] {
  const result: string[] = [];
  const visited = new Set<string>();
  const visiting = new Set<string>();

  function visit(name: string) {
    if (visited.has(name)) return;
    if (visiting.has(name)) {
      throw new Error(`Circular dependency detected involving job "${name}"`);
    }

    visiting.add(name);

    const job = jobs[name];
    if (job.needs) {
      const deps = Array.isArray(job.needs) ? job.needs : [job.needs];
      for (const dep of deps) {
        if (!jobs[dep]) {
          throw new Error(`Job "${name}" depends on unknown job "${dep}"`);
        }
        visit(dep);
      }
    }

    visiting.delete(name);
    visited.add(name);
    result.push(name);
  }

  for (const name of Object.keys(jobs)) {
    visit(name);
  }

  return result;
}

/**
 * Interpolate all string values in params object
 */
function interpolateParams(
  params: Record<string, unknown>,
  context: WorkflowContext
): Record<string, unknown> {
  const result: Record<string, unknown> = {};

  for (const [key, value] of Object.entries(params)) {
    if (typeof value === "string") {
      result[key] = interpolateString(value, context);
    } else if (Array.isArray(value)) {
      result[key] = value.map((v) =>
        typeof v === "string" ? interpolateString(v, context) : v
      );
    } else if (value && typeof value === "object") {
      result[key] = interpolateParams(
        value as Record<string, unknown>,
        context
      );
    } else {
      result[key] = value;
    }
  }

  return result;
}

/**
 * Create Playwright tests from a workflow file
 */
export function createTestsFromWorkflow(workflowPath: string): void {
  const workflow = loadWorkflow(workflowPath);

  test.describe(workflow.name, () => {
    if (workflow.timeout) {
      test.setTimeout(workflow.timeout);
    }

    for (const [jobName, job] of Object.entries(workflow.jobs)) {
      test(job.name || jobName, async ({ page }) => {
        const context = createContext(page, workflow, job.env);

        for (const step of job.steps) {
          await test.step(step.name, async () => {
            await executeStep(step, context);
          });
        }
      });
    }
  });
}

/**
 * Create Playwright tests from all workflow files in a directory
 */
export function createTestsFromDirectory(dirPath: string): void {
  const files = fs.readdirSync(dirPath);

  for (const file of files) {
    if (file.endsWith(".yaml") || file.endsWith(".yml")) {
      const workflowPath = path.join(dirPath, file);
      createTestsFromWorkflow(workflowPath);
    }
  }
}

/**
 * Run a single workflow file (for programmatic use)
 */
export async function runWorkflowFile(
  workflowPath: string,
  page: Page
): Promise<void> {
  const workflow = loadWorkflow(workflowPath);
  await executeWorkflow(workflow, page);
}
