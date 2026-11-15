import { describe, expect, it } from "vitest";

import { link, parseRoute, type Route } from "./route";

describe("parseRoute", () => {
  it.each<[string, Route]>([
    ["", { page: "home" }],
    ["#/", { page: "home" }],
    ["#/works", { page: "works" }],
    ["#/works?sort=year&order=desc", { page: "works", params: "sort=year&order=desc" }],
    ["#/works/12", { page: "work", id: 12, report: null }],
    ["#/works/12?report=40", { page: "work", id: 12, report: 40 }],
    ["#/reports", { page: "reports" }],
    ["#/reports?q=dox+mcf&property=ic50", { page: "reports", params: "q=dox+mcf&property=ic50" }],
    ["#/reports/40", { page: "report", id: 40 }],
    ["#/properties", { page: "properties" }],
    ["#/properties/ic50", { page: "property", key: "ic50" }],
    ["#/properties/ic50?entity=7", { page: "property", key: "ic50", params: "entity=7" }],
    ["#/entities", { page: "entities" }],
    ["#/entities/3", { page: "entity", id: 3 }],
    ["#/effects", { page: "effects" }],
    ["#/effects?direction=decrease", { page: "effects", params: "direction=decrease" }],
    ["#/about", { page: "about" }],
  ])("reads %s", (hash, route) => {
    expect(parseRoute(hash)).toEqual(route);
  });

  it.each(["#/works/0", "#/works/x", "#/reports/x", "#/nothing", "#/works/1/2", "#/effects/1"])(
    "gives the missing page for %s",
    (hash) => {
      expect(parseRoute(hash).page).toBe("missing");
    },
  );

  it("ignores a report number that is not valid", () => {
    expect(parseRoute("#/works/12?report=abc")).toEqual({ page: "work", id: 12, report: null });
  });

  it("keeps a question mark in a value", () => {
    expect(parseRoute("#/reports?q=why?")).toEqual({ page: "reports", params: "q=why%3F" });
  });
});

describe("link", () => {
  it.each<Route>([
    { page: "home" },
    { page: "works" },
    { page: "works", params: "q=breast&sort=year" },
    { page: "work", id: 12, report: null },
    { page: "work", id: 12, report: 40 },
    { page: "reports" },
    { page: "reports", params: "q=dox&entity=7&entity=8" },
    { page: "report", id: 40 },
    { page: "properties" },
    { page: "property", key: "melting point" },
    { page: "property", key: "ic50", params: "check=confirmed" },
    { page: "entities" },
    { page: "entity", id: 3 },
    { page: "effects" },
    { page: "about" },
  ])("round-trips %j", (route) => {
    expect(parseRoute(link(route))).toEqual(route);
  });

  it("leaves out empty parameters", () => {
    expect(link({ page: "reports", params: "" })).toBe("#/reports");
  });
});
