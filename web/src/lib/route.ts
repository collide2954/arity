/** The pages of the website, as hash routes, for example `#/works/12?report=40`. */

export type Route =
  | { page: "home" }
  | { page: "works" }
  | { page: "work"; id: number; report: number | null }
  | { page: "report"; id: number }
  | { page: "properties" }
  | { page: "property"; key: string }
  | { page: "entities" }
  | { page: "entity"; id: number }
  | { page: "effects" }
  | { page: "about" }
  | { page: "missing"; path: string };

function integer(text: string | undefined): number | null {
  if (text === undefined || !/^[1-9]\d*$/.test(text)) return null;
  return Number(text);
}

export function parseRoute(hash: string): Route {
  const [path = "", query = ""] = hash.replace(/^#/, "").split("?", 2);
  const parts = path.split("/").filter((part) => part.length > 0);
  const parameters = new URLSearchParams(query);
  const [first, second, ...rest] = parts;
  const missing: Route = { page: "missing", path };
  if (rest.length > 0) return missing;
  switch (first) {
    case undefined:
      return { page: "home" };
    case "works": {
      if (second === undefined) return { page: "works" };
      const id = integer(second);
      return id === null
        ? missing
        : { page: "work", id, report: integer(parameters.get("report") ?? undefined) };
    }
    case "reports": {
      const id = integer(second);
      return id === null ? missing : { page: "report", id };
    }
    case "properties":
      return second === undefined
        ? { page: "properties" }
        : { page: "property", key: decodeURIComponent(second) };
    case "entities": {
      if (second === undefined) return { page: "entities" };
      const id = integer(second);
      return id === null ? missing : { page: "entity", id };
    }
    case "effects":
      return second === undefined ? { page: "effects" } : missing;
    case "about":
      return second === undefined ? { page: "about" } : missing;
    default:
      return missing;
  }
}

export function link(route: Route): string {
  switch (route.page) {
    case "home":
      return "#/";
    case "works":
      return "#/works";
    case "work":
      return route.report === null
        ? `#/works/${String(route.id)}`
        : `#/works/${String(route.id)}?report=${String(route.report)}`;
    case "report":
      return `#/reports/${String(route.id)}`;
    case "properties":
      return "#/properties";
    case "property":
      return `#/properties/${encodeURIComponent(route.key)}`;
    case "entities":
      return "#/entities";
    case "entity":
      return `#/entities/${String(route.id)}`;
    case "effects":
      return "#/effects";
    case "about":
      return "#/about";
    case "missing":
      return `#/${route.path}`;
  }
}
