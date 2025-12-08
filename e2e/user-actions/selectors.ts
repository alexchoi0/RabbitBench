/**
 * Selector Resolution - Converts User Actions selectors to Playwright locators
 */

import type { Page, Locator } from "@playwright/test";
import type { Selector } from "./types";

/**
 * Resolve a User Actions selector to a Playwright Locator
 */
export function resolveSelector(page: Page, selector: Selector): Locator {
  // String shorthand - CSS selector or smart selector
  if (typeof selector === "string") {
    return resolveStringSelector(page, selector);
  }

  // Role selector
  if ("role" in selector) {
    const options: { name?: string | RegExp; exact?: boolean } = {};
    if (selector.name) {
      options.name = selector.exact ? selector.name : new RegExp(selector.name, "i");
    }
    if (selector.exact !== undefined) {
      options.exact = selector.exact;
    }
    return page.getByRole(selector.role as Parameters<Page["getByRole"]>[0], options);
  }

  // Text selector
  if ("text" in selector) {
    return page.getByText(
      selector.exact ? selector.text : new RegExp(selector.text, "i"),
      { exact: selector.exact }
    );
  }

  // Label selector
  if ("label" in selector) {
    return page.getByLabel(
      selector.exact ? selector.label : new RegExp(selector.label, "i"),
      { exact: selector.exact }
    );
  }

  // Placeholder selector
  if ("placeholder" in selector) {
    return page.getByPlaceholder(selector.placeholder);
  }

  // TestId selector
  if ("testId" in selector) {
    return page.getByTestId(selector.testId);
  }

  throw new Error(`Unknown selector type: ${JSON.stringify(selector)}`);
}

/**
 * Resolve string selector with smart parsing
 *
 * Supports:
 * - CSS selectors: "button.submit", "#myId", "[data-test]"
 * - Role shorthand: "@button:Submit" -> getByRole("button", { name: "Submit" })
 * - Text shorthand: ">>Submit" -> getByText("Submit")
 * - Label shorthand: "@@Email" -> getByLabel("Email")
 * - TestId shorthand: "$myTestId" -> getByTestId("myTestId")
 * - Placeholder shorthand: "%%Enter email" -> getByPlaceholder("Enter email")
 */
function resolveStringSelector(page: Page, selector: string): Locator {
  // Role shorthand: @role:name
  if (selector.startsWith("@") && !selector.startsWith("@@")) {
    const match = selector.match(/^@(\w+)(?::(.+))?$/);
    if (match) {
      const [, role, name] = match;
      const options = name ? { name: new RegExp(name, "i") } : undefined;
      return page.getByRole(role as Parameters<Page["getByRole"]>[0], options);
    }
  }

  // Label shorthand: @@label
  if (selector.startsWith("@@")) {
    return page.getByLabel(new RegExp(selector.slice(2), "i"));
  }

  // Text shorthand: >>text
  if (selector.startsWith(">>")) {
    return page.getByText(new RegExp(selector.slice(2), "i"));
  }

  // TestId shorthand: $testId
  if (selector.startsWith("$")) {
    return page.getByTestId(selector.slice(1));
  }

  // Placeholder shorthand: %%placeholder
  if (selector.startsWith("%%")) {
    return page.getByPlaceholder(new RegExp(selector.slice(2), "i"));
  }

  // Default: CSS selector
  return page.locator(selector);
}

/**
 * Interpolate variables in selector strings
 * Replaces ${{ vars.name }} with actual values
 */
export function interpolateSelector(
  selector: Selector,
  vars: Record<string, unknown>
): Selector {
  if (typeof selector === "string") {
    return interpolateString(selector, vars);
  }

  // Deep clone and interpolate all string values
  const interpolated = JSON.parse(JSON.stringify(selector));
  for (const [key, value] of Object.entries(interpolated)) {
    if (typeof value === "string") {
      interpolated[key] = interpolateString(value, vars);
    }
  }
  return interpolated;
}

/**
 * Interpolate variables in a string
 * Supports: ${{ vars.name }}, ${{ env.NAME }}, ${{ outputs.stepId.key }}
 */
export function interpolateString(
  str: string,
  context: Record<string, unknown>
): string {
  return str.replace(/\$\{\{\s*(\w+)\.(\w+(?:\.\w+)?)\s*\}\}/g, (match, namespace, path) => {
    const parts = path.split(".");
    let value: unknown = context[namespace];

    for (const part of parts) {
      if (value && typeof value === "object") {
        value = (value as Record<string, unknown>)[part];
      } else {
        return match; // Keep original if path doesn't resolve
      }
    }

    return value !== undefined ? String(value) : match;
  });
}
