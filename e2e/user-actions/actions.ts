/**
 * Built-in Action Executors
 *
 * Each action corresponds to a "uses" value in the workflow YAML.
 */

import { execSync } from "child_process";
import type {
  ActionRegistry,
  NavigateAction,
  ClickAction,
  FillAction,
  TypeAction,
  SelectAction,
  CheckAction,
  WaitAction,
  PressAction,
  HoverAction,
  FocusAction,
  ScreenshotAction,
  StoreAction,
  ExecAction,
  ApiAction,
  WorkflowContext,
} from "./types";
import { resolveSelector, interpolateString } from "./selectors";

/**
 * Navigate to a URL
 */
async function navigate(
  params: NavigateAction,
  context: WorkflowContext
): Promise<void> {
  const url = interpolateString(params.url, context);
  await context.page.goto(url);
}

/**
 * Click on an element
 */
async function click(
  params: ClickAction,
  context: WorkflowContext
): Promise<void> {
  let locator = resolveSelector(context.page, params.selector);

  // Handle .first() and .nth() for multiple matches
  if (params.first) {
    locator = locator.first();
  } else if (params.nth !== undefined) {
    locator = locator.nth(params.nth);
  }

  await locator.click({
    button: params.button,
    clickCount: params.clickCount,
    delay: params.delay,
    force: params.force,
  });
}

/**
 * Fill an input field
 */
async function fill(
  params: FillAction,
  context: WorkflowContext
): Promise<void> {
  const locator = resolveSelector(context.page, params.selector);
  const value = interpolateString(params.value, context);

  if (params.clear) {
    await locator.clear();
  }
  await locator.fill(value);
}

/**
 * Type text character by character
 */
async function type(
  params: TypeAction,
  context: WorkflowContext
): Promise<void> {
  const locator = resolveSelector(context.page, params.selector);
  const text = interpolateString(params.text, context);
  await locator.pressSequentially(text, { delay: params.delay });
}

/**
 * Select an option from a dropdown
 */
async function select(
  params: SelectAction,
  context: WorkflowContext
): Promise<void> {
  const locator = resolveSelector(context.page, params.selector);
  const values = Array.isArray(params.value) ? params.value : [params.value];
  const interpolatedValues = values.map((v) => interpolateString(v, context));
  await locator.selectOption(interpolatedValues);
}

/**
 * Check or uncheck a checkbox/radio
 */
async function check(
  params: CheckAction,
  context: WorkflowContext
): Promise<void> {
  const locator = resolveSelector(context.page, params.selector);
  if (params.checked === false) {
    await locator.uncheck();
  } else {
    await locator.check();
  }
}

/**
 * Wait for element, URL, or timeout
 */
async function wait(
  params: WaitAction,
  context: WorkflowContext
): Promise<void> {
  const timeout = params.timeout || 30000;

  if (params.selector) {
    const locator = resolveSelector(context.page, params.selector);
    await locator.waitFor({ state: params.state || "visible", timeout });
  } else if (params.url) {
    const url = interpolateString(params.url, context);
    await context.page.waitForURL(url, { timeout });
  } else if (params.timeout) {
    await context.page.waitForTimeout(params.timeout);
  }
}

/**
 * Press a keyboard key
 */
async function press(
  params: PressAction,
  context: WorkflowContext
): Promise<void> {
  if (params.selector) {
    const locator = resolveSelector(context.page, params.selector);
    await locator.press(params.key);
  } else {
    await context.page.keyboard.press(params.key);
  }
}

/**
 * Hover over an element
 */
async function hover(
  params: HoverAction,
  context: WorkflowContext
): Promise<void> {
  const locator = resolveSelector(context.page, params.selector);
  await locator.hover();
}

/**
 * Focus an element
 */
async function focus(
  params: FocusAction,
  context: WorkflowContext
): Promise<void> {
  const locator = resolveSelector(context.page, params.selector);
  await locator.focus();
}

/**
 * Take a screenshot
 */
async function screenshot(
  params: ScreenshotAction,
  context: WorkflowContext
): Promise<void> {
  await context.page.screenshot({
    path: params.path ? interpolateString(params.path, context) : undefined,
    fullPage: params.fullPage,
  });
}

/**
 * Store a value from the page into context variables
 */
async function store(
  params: StoreAction,
  context: WorkflowContext
): Promise<void> {
  let value: string | null = null;

  if (params.value) {
    // Store a literal value (with interpolation)
    value = interpolateString(params.value, context);
  } else if (params.selector) {
    const locator = resolveSelector(context.page, params.selector);

    if (params.attribute) {
      value = await locator.getAttribute(params.attribute);
    } else if (params.property) {
      value = await locator.evaluate(
        (el, prop) => (el as unknown as Record<string, unknown>)[prop] as string,
        params.property
      );
    } else {
      value = await locator.textContent();
    }
  }

  // Apply regex extraction if specified
  if (value && params.regex) {
    const match = value.match(new RegExp(params.regex));
    value = match ? match[1] || match[0] : null;
  }

  context.vars[params.key] = value;
}

/**
 * Execute a shell command
 */
async function exec(
  params: ExecAction,
  context: WorkflowContext
): Promise<void> {
  const command = interpolateString(params.command, context);
  const args = params.args?.map((a) => interpolateString(a, context)) || [];

  const fullCommand = args.length ? `${command} ${args.join(" ")}` : command;

  const env = {
    ...process.env,
    ...context.env,
    ...(params.env || {}),
  };

  // Interpolate env values
  for (const [key, val] of Object.entries(env)) {
    if (typeof val === "string") {
      env[key] = interpolateString(val, context);
    }
  }

  const output = execSync(fullCommand, {
    encoding: "utf-8",
    timeout: params.timeout || 30000,
    cwd: params.cwd ? interpolateString(params.cwd, context) : undefined,
    env: env as NodeJS.ProcessEnv,
  });

  if (params.storeOutput) {
    context.vars[params.storeOutput] = output.trim();
  }
}

/**
 * Make an API request
 */
async function api(
  params: ApiAction,
  context: WorkflowContext
): Promise<void> {
  const url = interpolateString(params.url, context);

  const headers: Record<string, string> = {};
  if (params.headers) {
    for (const [key, val] of Object.entries(params.headers)) {
      headers[key] = interpolateString(val, context);
    }
  }

  let body: string | undefined;
  if (params.body) {
    if (typeof params.body === "string") {
      body = interpolateString(params.body, context);
    } else {
      body = JSON.stringify(params.body);
      if (!headers["Content-Type"]) {
        headers["Content-Type"] = "application/json";
      }
    }
  }

  const response = await context.page.request.fetch(url, {
    method: params.method,
    headers,
    data: body,
  });

  if (params.storeResponse) {
    const contentType = response.headers()["content-type"] || "";
    let responseData: unknown;

    if (contentType.includes("application/json")) {
      responseData = await response.json();
    } else {
      responseData = await response.text();
    }

    context.vars[params.storeResponse] = {
      status: response.status(),
      headers: response.headers(),
      body: responseData,
    };
  }
}

/**
 * Registry of all built-in actions
 */
export const actions: ActionRegistry = {
  navigate,
  click,
  fill,
  type,
  select,
  check,
  wait,
  press,
  hover,
  focus,
  screenshot,
  store,
  exec,
  api,
};

/**
 * Register a custom action
 */
export function registerAction<T>(
  name: string,
  executor: (params: T, context: WorkflowContext) => Promise<void>
): void {
  actions[name] = executor as ActionRegistry[string];
}
