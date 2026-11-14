import { describe, expect, it } from "vitest";

import { link, parseRoute, type Route } from "./route";

describe("parseRoute", () => {
  it.each<[string, Route]>([
    ["", { page: "home" }],
    ["#/", { page: "home" }],
    ["#/works", { page: "works" }],
    ["#/works/12", { page: "work", id: 12, report: null }],
    ["#/works/12?report=40", { page: "work", id: 12, report: 40 }],
    ["#/reports/40", { page: "report", id: 40 }],
    ["#/properties", { page: "properties" }],
    ["#/properties/ic50", { page: "property", key: "ic50" }],
    ["#/entities", { page: "entities" }],
    ["#/entities/3", { page: "entity", id: 3 }],
    ["#/effects", { page: "effects" }],
    ["#/about", { page: "about" }],
  ])("reads %s", (hash, route) => {
    expect(parseRoute(hash)).toEqual(route);
  });

  it.each(["#/works/0", "#/works/x", "#/reports", "#/nothing", "#/works/1/2", "#/effects/1"])(
    "gives the missing page for %s",
    (hash) => {
      expect(parseRoute(hash).page).toBe("missing");
    },
  );

  it("ignores a report number that is not valid", () => {
    expect(parseRoute("#/works/12?report=abc")).toEqual({ page: "work", id: 12, report: null });
  });
});

describe("link", () => {
  it.each<Route>([
    { page: "home" },
    { page: "works" },
    { page: "work", id: 12, report: null },
    { page: "work", id: 12, report: 40 },
    { page: "report", id: 40 },
    { page: "properties" },
    { page: "property", key: "melting point" },
    { page: "entities" },
    { page: "entity", id: 3 },
    { page: "effects" },
    { page: "about" },
  ])("round-trips %j", (route) => {
    expect(parseRoute(link(route))).toEqual(route);
  });
});
