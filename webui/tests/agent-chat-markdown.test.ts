import { describe, expect, test } from "bun:test";

import { normalizeThinkingMarkdown } from "../src/lib/agent-chat-markdown";

describe("normalizeThinkingMarkdown", () => {
  test("separates standalone reasoning summary headings from adjacent paragraphs", () => {
    const input = [
      "**Analyzing local changes**",
      "I need to review local changes before reporting.",
      "**Inspecting activity_event.rs**",
      "I'm checking the activity event contract.",
      "**Investigating the broken test**",
      "The failure points at a renderer mismatch.",
    ].join("\n");

    expect(normalizeThinkingMarkdown(input)).toBe(
      [
        "**Analyzing local changes**",
        "",
        "I need to review local changes before reporting.",
        "",
        "**Inspecting activity_event.rs**",
        "",
        "I'm checking the activity event contract.",
        "",
        "**Investigating the broken test**",
        "",
        "The failure points at a renderer mismatch.",
      ].join("\n"),
    );
  });

  test("does not split ordinary soft-wrapped prose", () => {
    const input = [
      "I need to review local changes before reporting.",
      "This continuation belongs to the same paragraph.",
    ].join("\n");

    expect(normalizeThinkingMarkdown(input)).toBe(input);
  });

  test("does not rewrite fenced code contents", () => {
    const input = ["```", "**Not a heading**", "```", "**Real heading**", "Body"].join(
      "\n",
    );

    expect(normalizeThinkingMarkdown(input)).toBe(
      ["```", "**Not a heading**", "```", "", "**Real heading**", "", "Body"].join(
        "\n",
      ),
    );
  });

  test("splits embedded bold headings from persisted thinking payloads", () => {
    const input = [
      "Everything needs to be careful with the path syntax.**Considering project operations**",
      "",
      "I’m thinking about whether to use the project tools.",
    ].join("\n");

    expect(normalizeThinkingMarkdown(input)).toBe(
      [
        "Everything needs to be careful with the path syntax.",
        "",
        "**Considering project operations**",
        "",
        "I’m thinking about whether to use the project tools.",
      ].join("\n"),
    );
  });
});
