/**
 * Assertion Executors
 *
 * Assertions verify the state of the page after actions.
 */

import { expect } from "@playwright/test";
import type {
  AssertionRegistry,
  UrlAssertion,
  VisibleAssertion,
  TextAssertion,
  CountAssertion,
  ValueAssertion,
  ContainsAssertion,
  CustomAssertion,
  WorkflowContext,
  Assertion,
} from "./types";
import { resolveSelector, interpolateString } from "./selectors";

/**
 * Assert URL matches expected value
 */
async function url(
  assertion: UrlAssertion,
  context: WorkflowContext
): Promise<void> {
  const page = context.page;

  if (assertion.equals) {
    const expected = interpolateString(assertion.equals, context);
    await expect(page).toHaveURL(expected, { timeout: 15000 });
  } else if (assertion.contains) {
    const expected = interpolateString(assertion.contains, context);
    await expect(page).toHaveURL(new RegExp(escapeRegex(expected)), { timeout: 15000 });
  } else if (assertion.matches) {
    const pattern = interpolateString(assertion.matches, context);
    await expect(page).toHaveURL(new RegExp(pattern), { timeout: 15000 });
  }
}

/**
 * Assert element visibility
 */
async function visible(
  assertion: VisibleAssertion,
  context: WorkflowContext
): Promise<void> {
  const locator = resolveSelector(context.page, assertion.selector);

  if (assertion.visible === false) {
    await expect(locator).not.toBeVisible({ timeout: 10000 });
  } else {
    await expect(locator).toBeVisible({ timeout: 10000 });
  }
}

/**
 * Assert element text content
 */
async function text(
  assertion: TextAssertion,
  context: WorkflowContext
): Promise<void> {
  const locator = resolveSelector(context.page, assertion.selector);

  if (assertion.equals) {
    const expected = interpolateString(assertion.equals, context);
    await expect(locator).toHaveText(expected, { timeout: 10000 });
  } else if (assertion.contains) {
    const expected = interpolateString(assertion.contains, context);
    await expect(locator).toContainText(expected, { timeout: 10000 });
  } else if (assertion.matches) {
    const pattern = interpolateString(assertion.matches, context);
    await expect(locator).toHaveText(new RegExp(pattern), { timeout: 10000 });
  }
}

/**
 * Assert element count
 */
async function count(
  assertion: CountAssertion,
  context: WorkflowContext
): Promise<void> {
  const locator = resolveSelector(context.page, assertion.selector);

  if (assertion.equals !== undefined) {
    await expect(locator).toHaveCount(assertion.equals, { timeout: 10000 });
  } else if (assertion.greaterThan !== undefined) {
    const actualCount = await locator.count();
    expect(actualCount).toBeGreaterThan(assertion.greaterThan);
  } else if (assertion.lessThan !== undefined) {
    const actualCount = await locator.count();
    expect(actualCount).toBeLessThan(assertion.lessThan);
  }
}

/**
 * Assert input value
 */
async function value(
  assertion: ValueAssertion,
  context: WorkflowContext
): Promise<void> {
  const locator = resolveSelector(context.page, assertion.selector);

  if (assertion.equals) {
    const expected = interpolateString(assertion.equals, context);
    await expect(locator).toHaveValue(expected, { timeout: 10000 });
  } else if (assertion.contains) {
    const expected = interpolateString(assertion.contains, context);
    const actualValue = await locator.inputValue();
    expect(actualValue).toContain(expected);
  }
}

/**
 * Assert element contains text
 */
async function contains(
  assertion: ContainsAssertion,
  context: WorkflowContext
): Promise<void> {
  const locator = resolveSelector(context.page, assertion.selector);
  const expected = interpolateString(assertion.text, context);
  await expect(locator).toContainText(expected, { timeout: 10000 });
}

/**
 * Registry of custom assertion functions
 */
const customAssertions: Record<
  string,
  (args: unknown[], context: WorkflowContext) => Promise<void>
> = {};

/**
 * Execute a custom assertion function
 */
async function custom(
  assertion: CustomAssertion,
  context: WorkflowContext
): Promise<void> {
  const fn = customAssertions[assertion.fn];
  if (!fn) {
    throw new Error(`Unknown custom assertion: ${assertion.fn}`);
  }
  await fn(assertion.args || [], context);
}

/**
 * Registry of all assertion executors
 */
export const assertions: AssertionRegistry = {
  url,
  visible,
  text,
  count,
  value,
  contains,
  custom,
};

/**
 * Register a custom assertion function
 */
export function registerAssertion(
  name: string,
  fn: (args: unknown[], context: WorkflowContext) => Promise<void>
): void {
  customAssertions[name] = fn;
  assertions[name] = ((assertion: CustomAssertion, context: WorkflowContext) =>
    fn(assertion.args || [], context)) as AssertionRegistry[string];
}

/**
 * Execute an assertion
 */
export async function executeAssertion(
  assertion: Assertion,
  context: WorkflowContext
): Promise<void> {
  const executor = assertions[assertion.type];
  if (!executor) {
    throw new Error(`Unknown assertion type: ${assertion.type}`);
  }
  await executor(assertion, context);
}

/**
 * Execute multiple assertions
 */
export async function executeAssertions(
  assertionOrAssertions: Assertion | Assertion[],
  context: WorkflowContext
): Promise<void> {
  const assertionList = Array.isArray(assertionOrAssertions)
    ? assertionOrAssertions
    : [assertionOrAssertions];

  for (const assertion of assertionList) {
    await executeAssertion(assertion, context);
  }
}

/**
 * Escape special regex characters
 */
function escapeRegex(str: string): string {
  return str.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
