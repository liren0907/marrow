// Layout algorithm configurations + cytoscape extension registration.
//
// fcose runs once for the initial pass — it gives nice deterministic
// positions cheaply. cola then takes over with continuous force simulation
// for the live, draggable feel.

import cytoscape from "cytoscape";
import fcose from "cytoscape-fcose";
import cola from "cytoscape-cola";

// Idempotent guard — HMR / multiple GraphTab mounts won't double-register.
let registered = false;
export function ensureCytoscapeExtensions(): void {
  if (registered) return;
  cytoscape.use(fcose);
  cytoscape.use(cola);
  registered = true;
}

// First-paint layout: fcose gives nice initial positions cheap.
export const fcoseInitOptions = {
  name: "fcose",
  quality: "default",
  randomize: true,
  animate: false,
  nodeRepulsion: () => 4500,
  idealEdgeLength: () => 80,
  nodeSeparation: 75,
  numIter: 1500,
  tile: true,
} as unknown as cytoscape.LayoutOptions;

// cola in `infinite: true` mode runs continuously — drag a node and the
// rest of the graph reacts. This is the "live force" feel.
//
// `handleDisconnected: false` is intentional. When `true`, every cola
// restart (which happens on tab visibility change) repacks disconnected
// components in a grid — visually identical to the graph "jumping" even
// though the camera hasn't moved. fcose already places disconnected
// components reasonably during the initial pre-layout pass, so cola
// doesn't need to re-do that work on every resume.
export const colaLayoutOptions = {
  name: "cola",
  infinite: true,
  fit: false,
  animate: true,
  refresh: 1,
  maxSimulationTime: 4000,
  ungrabifyWhileSimulating: false,
  nodeSpacing: () => 16,
  edgeLength: () => 80,
  randomize: false,
  avoidOverlap: true,
  handleDisconnected: false,
} as unknown as cytoscape.LayoutOptions;
