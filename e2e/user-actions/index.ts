/**
 * User Actions Framework
 *
 * A declarative E2E testing framework inspired by GitHub Actions.
 *
 * @example
 * ```yaml
 * name: Create Workspace Flow
 * description: Test creating a new workspace
 *
 * jobs:
 *   create-workspace:
 *     name: Create workspace via UI
 *     steps:
 *       - name: Navigate to home
 *         uses: navigate
 *         with:
 *           url: /
 *         assert:
 *           type: visible
 *           selector:
 *             role: heading
 *             name: Benchmark Performance
 *
 *       - name: Click Get Started
 *         uses: click
 *         with:
 *           selector:
 *             role: button
 *             name: Get Started
 *         assert:
 *           type: url
 *           equals: /workspaces
 * ```
 */

// Types
export type {
  UserActionsWorkflow,
  Job,
  Step,
  Assertion,
  Selector,
  WorkflowContext,
  ActionRegistry,
  AssertionRegistry,
} from "./types";

// Actions
export { actions, registerAction } from "./actions";

// Assertions
export {
  assertions,
  registerAssertion,
  executeAssertion,
  executeAssertions,
} from "./assertions";

// Selectors
export {
  resolveSelector,
  interpolateSelector,
  interpolateString,
} from "./selectors";

// Runner
export {
  loadWorkflow,
  executeWorkflow,
  createTestsFromWorkflow,
  createTestsFromDirectory,
  runWorkflowFile,
} from "./runner";
