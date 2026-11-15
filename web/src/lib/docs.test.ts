import { describe, expect, it } from "vitest";

import { docsLink } from "./docs";

describe("docsLink", () => {
  it("links to the book, to a chapter, and to a section of a chapter", () => {
    expect(docsLink()).toBe("/docs/");
    expect(docsLink("principles")).toBe("/docs/principles.html");
    expect(docsLink("website", "check-mode")).toBe("/docs/website.html#check-mode");
  });
});
