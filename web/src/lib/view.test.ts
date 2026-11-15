import { describe, expect, it } from "vitest";

import {
  arrange,
  DEFAULT_VIEW,
  readView,
  shownColumns,
  sortedBy,
  writeView,
  type Column,
  type TableView,
} from "./view";

interface Row {
  name: string;
  size: number | null;
}

const COLUMNS: Column<Row>[] = [
  { id: "name", label: "Name", value: (row) => row.name },
  { id: "size", label: "Size", value: (row) => row.size, numeric: true },
];
const IDS = COLUMNS.map((column) => column.id);

function written(view: TableView): string {
  const params = new URLSearchParams();
  writeView(params, view);
  return params.toString();
}

describe("readView and writeView", () => {
  it("keep a view in the address", () => {
    const view: TableView = {
      sort: "size",
      order: "descending",
      page: 3,
      size: 100,
      columns: ["size", "name"],
    };
    const text = written(view);
    expect(text).toBe("sort=size&order=desc&page=3&size=100&columns=size%2Cname");
    expect(readView(new URLSearchParams(text), IDS)).toEqual(view);
  });

  it("ignore unknown columns, page numbers that are not valid, and other page sizes", () => {
    expect(
      readView(new URLSearchParams("sort=color&page=0&size=7&columns=color,name,name"), IDS),
    ).toEqual({ ...DEFAULT_VIEW, columns: ["name"] });
    expect(readView(new URLSearchParams("page=2.5&columns=color"), IDS)).toEqual(DEFAULT_VIEW);
  });

  it("write nothing for the default view", () => {
    expect(written(DEFAULT_VIEW)).toBe("");
    expect(written({ ...DEFAULT_VIEW, order: "descending" })).toBe("");
  });
});

describe("sortedBy", () => {
  it("sorts by a new column, or turns the order around, and goes to the first page", () => {
    const first = sortedBy({ ...DEFAULT_VIEW, page: 4 }, "size");
    expect(first).toMatchObject({ sort: "size", order: "ascending", page: 1 });
    expect(sortedBy(first, "size")).toMatchObject({ sort: "size", order: "descending" });
    expect(sortedBy({ ...first, order: "descending" }, "name")).toMatchObject({
      sort: "name",
      order: "ascending",
    });
  });
});

describe("arrange", () => {
  const rows: Row[] = [
    { name: "b", size: 2 },
    { name: "a", size: null },
    { name: "c", size: 1 },
  ];

  it("sorts by the column of the view and gives the rows of the page", () => {
    const page = arrange(rows, COLUMNS, { ...DEFAULT_VIEW, sort: "size", size: 25 });
    expect(page.rows.map((row) => row.name)).toEqual(["c", "b", "a"]);
    expect(arrange(rows, COLUMNS, DEFAULT_VIEW).rows).toEqual(rows);
  });
});

describe("shownColumns", () => {
  it("gives the columns of the view in order, or the default columns", () => {
    const ids = (view: TableView) =>
      shownColumns(COLUMNS, view, ["name"]).map((column) => column.id);
    expect(ids(DEFAULT_VIEW)).toEqual(["name"]);
    expect(ids({ ...DEFAULT_VIEW, columns: ["size", "name"] })).toEqual(["size", "name"]);
  });
});
