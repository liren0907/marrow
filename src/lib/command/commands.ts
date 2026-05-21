// Command Palette command list — a thin adapter over the action registry
// (actions.ts). Every palette command derives from an ActionDef, so the
// displayed shortcut is always in sync with the real binding. To add a
// command, add an action in actions.ts (`inPalette` defaults to true).

import { getActions, comboToDisplay } from "./actions";
import { resolveBinding } from "$lib/settings/keybindingSettings.svelte";

export interface Command {
  id: string;
  title: string;
  category: string;
  shortcut?: string;
  action: () => void;
}

export function getCommands(): Command[] {
  return getActions()
    .filter((a) => a.inPalette !== false)
    .map((a) => {
      const binds = resolveBinding(a.id);
      return {
        id: a.id,
        title: a.title,
        category: a.category,
        shortcut: binds.length
          ? binds.map(comboToDisplay).join(" / ")
          : undefined,
        action: a.run,
      };
    });
}
