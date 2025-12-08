/**
 * User Actions Framework - TypeScript Types
 *
 * A declarative testing framework inspired by GitHub Actions YAML format.
 * Allows defining E2E user flows as step-by-step actions with assertions.
 */

import type { Page } from "@playwright/test";

// ============================================
// Core Workflow Types
// ============================================

export interface UserActionsWorkflow {
  name: string;
  description?: string;
  timeout?: number; // Overall workflow timeout in ms
  env?: Record<string, string>; // Environment variables
  jobs: Record<string, Job>;
}

export interface Job {
  name?: string;
  needs?: string | string[]; // Job dependencies
  env?: Record<string, string>;
  steps: Step[];
}

export interface Step {
  name: string;
  id?: string; // For referencing in later steps
  uses?: string; // Built-in action (e.g., "navigate", "click")
  with?: Record<string, unknown>; // Action parameters
  run?: string; // Custom TypeScript code to execute
  env?: Record<string, string>;
  if?: string; // Conditional execution
  timeout?: number; // Step timeout in ms
  assert?: Assertion | Assertion[]; // Assertions after step
}

// ============================================
// Assertion Types
// ============================================

export type Assertion =
  | UrlAssertion
  | VisibleAssertion
  | TextAssertion
  | CountAssertion
  | ValueAssertion
  | ContainsAssertion
  | CustomAssertion;

export interface UrlAssertion {
  type: "url";
  equals?: string;
  contains?: string;
  matches?: string; // Regex pattern
}

export interface VisibleAssertion {
  type: "visible";
  selector: Selector;
  visible?: boolean; // Default true
}

export interface TextAssertion {
  type: "text";
  selector: Selector;
  equals?: string;
  contains?: string;
  matches?: string;
}

export interface CountAssertion {
  type: "count";
  selector: Selector;
  equals?: number;
  greaterThan?: number;
  lessThan?: number;
}

export interface ValueAssertion {
  type: "value";
  selector: Selector;
  equals?: string;
  contains?: string;
}

export interface ContainsAssertion {
  type: "contains";
  selector: Selector;
  text: string;
}

export interface CustomAssertion {
  type: "custom";
  fn: string; // Function name from assertions registry
  args?: unknown[];
}

// ============================================
// Selector Types
// ============================================

export type Selector =
  | string // CSS selector shorthand
  | RoleSelector
  | TextSelector
  | LabelSelector
  | PlaceholderSelector
  | TestIdSelector;

export interface RoleSelector {
  role: string;
  name?: string;
  exact?: boolean;
}

export interface TextSelector {
  text: string;
  exact?: boolean;
}

export interface LabelSelector {
  label: string;
  exact?: boolean;
}

export interface PlaceholderSelector {
  placeholder: string;
}

export interface TestIdSelector {
  testId: string;
}

// ============================================
// Built-in Action Types
// ============================================

export interface NavigateAction {
  url: string;
}

export interface ClickAction {
  selector: Selector;
  button?: "left" | "right" | "middle";
  clickCount?: number;
  delay?: number;
  force?: boolean;
  first?: boolean; // Click the first matching element
  nth?: number; // Click the nth matching element (0-indexed)
}

export interface FillAction {
  selector: Selector;
  value: string;
  clear?: boolean;
}

export interface TypeAction {
  selector: Selector;
  text: string;
  delay?: number;
}

export interface SelectAction {
  selector: Selector;
  value: string | string[];
}

export interface CheckAction {
  selector: Selector;
  checked?: boolean;
}

export interface WaitAction {
  selector?: Selector;
  url?: string;
  timeout?: number;
  state?: "visible" | "hidden" | "attached" | "detached";
}

export interface PressAction {
  selector?: Selector;
  key: string;
}

export interface HoverAction {
  selector: Selector;
}

export interface FocusAction {
  selector: Selector;
}

export interface ScreenshotAction {
  path?: string;
  fullPage?: boolean;
}

export interface StoreAction {
  key: string;
  selector?: Selector;
  attribute?: string;
  property?: string;
  value?: string;
  regex?: string; // Extract with regex
}

export interface ExecAction {
  command: string;
  args?: string[];
  env?: Record<string, string>;
  cwd?: string;
  timeout?: number;
  storeOutput?: string; // Store stdout in variable
}

export interface ApiAction {
  method: "GET" | "POST" | "PUT" | "DELETE" | "PATCH";
  url: string;
  headers?: Record<string, string>;
  body?: unknown;
  storeResponse?: string;
}

// ============================================
// Context Types
// ============================================

export interface WorkflowContext {
  page: Page;
  env: Record<string, string>;
  vars: Record<string, unknown>; // Stored variables from steps
  outputs: Record<string, Record<string, unknown>>; // Step outputs by step id
}

export type ActionExecutor<T = unknown> = (
  params: T,
  context: WorkflowContext
) => Promise<void>;

export type AssertionExecutor<T extends Assertion = Assertion> = (
  assertion: T,
  context: WorkflowContext
) => Promise<void>;

// ============================================
// Registry Types
// ============================================

export interface ActionRegistry {
  navigate: ActionExecutor<NavigateAction>;
  click: ActionExecutor<ClickAction>;
  fill: ActionExecutor<FillAction>;
  type: ActionExecutor<TypeAction>;
  select: ActionExecutor<SelectAction>;
  check: ActionExecutor<CheckAction>;
  wait: ActionExecutor<WaitAction>;
  press: ActionExecutor<PressAction>;
  hover: ActionExecutor<HoverAction>;
  focus: ActionExecutor<FocusAction>;
  screenshot: ActionExecutor<ScreenshotAction>;
  store: ActionExecutor<StoreAction>;
  exec: ActionExecutor<ExecAction>;
  api: ActionExecutor<ApiAction>;
  [key: string]: ActionExecutor<unknown>;
}

export interface AssertionRegistry {
  url: AssertionExecutor<UrlAssertion>;
  visible: AssertionExecutor<VisibleAssertion>;
  text: AssertionExecutor<TextAssertion>;
  count: AssertionExecutor<CountAssertion>;
  value: AssertionExecutor<ValueAssertion>;
  contains: AssertionExecutor<ContainsAssertion>;
  custom: AssertionExecutor<CustomAssertion>;
  [key: string]: AssertionExecutor<Assertion>;
}
