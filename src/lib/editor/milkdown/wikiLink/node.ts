import { $nodeSchema } from "@milkdown/utils";

export const wikiLinkSchema = $nodeSchema("wikiLink", () => ({
  inline: true,
  group: "inline",
  atom: true,
  selectable: true,
  draggable: false,
  attrs: {
    target: { default: "" },
    display: { default: "" },
  },
  parseDOM: [
    {
      tag: "span[data-wiki-link]",
      getAttrs: (dom) => {
        if (!(dom instanceof HTMLElement)) return false;
        return {
          target: dom.getAttribute("data-target") ?? dom.textContent ?? "",
          display: dom.getAttribute("data-display") ?? "",
        };
      },
    },
  ],
  toDOM: (node) => {
    const target = node.attrs.target as string;
    const display = node.attrs.display as string;
    return [
      "span",
      {
        "data-wiki-link": "",
        "data-target": target,
        "data-display": display,
        class: "wiki-link",
      },
      display || target,
    ];
  },
  parseMarkdown: {
    // Remark does not emit wikiLink nodes; the load-time pass handles
    // converting `[[target]]` text into wikiLink nodes after parse.
    match: () => false,
    runner: () => {},
  },
  toMarkdown: {
    match: (node) => node.type.name === "wikiLink",
    runner: (state, node) => {
      const target = node.attrs.target as string;
      const display = node.attrs.display as string;
      const inner = display && display !== target ? `${target}|${display}` : target;
      state.addNode("text", undefined, `[[${inner}]]`);
    },
  },
}));
