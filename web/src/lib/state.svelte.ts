import type { Mode } from "./charts";
import { link, parseRoute, type Route } from "./route";

const query = window.matchMedia("(prefers-color-scheme: dark)");

function currentMode(): Mode {
  const stamped = document.documentElement.dataset.theme;
  if (stamped === "light" || stamped === "dark") return stamped;
  return query.matches ? "dark" : "light";
}

export const location: { route: Route } = $state({ route: parseRoute(window.location.hash) });
export const theme: { mode: Mode } = $state({ mode: currentMode() });

/** Shows a route and changes the address to it, without a new entry in the history of the browser. */
export function replaceRoute(route: Route): void {
  history.replaceState(null, "", link(route));
  location.route = route;
}

window.addEventListener("hashchange", () => {
  location.route = parseRoute(window.location.hash);
  window.scrollTo({ top: 0 });
});

query.addEventListener("change", () => {
  theme.mode = currentMode();
});
