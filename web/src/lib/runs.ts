import type { Agent, Run } from "./types";

/** The name of an agent, with its version when it has one. */
export function agentName(agent: Agent): string {
  return agent.version === null ? agent.name : `${agent.name} ${agent.version}`;
}

/** Whether a run is current, and the run that it corrects or that replaces it. */
export function runState(run: Run): string {
  if (run.replaced_by !== null) return `Replaced by run ${String(run.replaced_by)}`;
  if (run.replaces !== null) return `Current. It corrects run ${String(run.replaces)}.`;
  return "Current";
}
