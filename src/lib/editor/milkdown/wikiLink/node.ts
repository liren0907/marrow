import { $nodeSchema } from "@milkdown/utils";

export const wikiLinkSchema = $nodeSchema("wikiLink", () => ({
  inline: true,
  group: "inline",
  atom: true,
  selectable: true,
  draggable: false,
  attrs: {
    target: { default: "" },
  },
  parseDOM: [
    {
      tag: "span[data-wiki-link]",
      getAttrs: (dom) => {
        if (!(dom instanceof HTMLElement)) return false;
        return { target: dom.getAttribute("data-target") ?? dom.textContent ?? "" };
      },
    },
  ],
  toDOM: (node) => [
    "span",
    {
      "data-wiki-link": "",
      "data-target": node.attrs.target as string,
      class: "wiki-link",
    },
    node.attrs.target as string,
  ],
  parseMarkdown: {
    // Remark does not emit wikiLink nodes; the load-time pass handles
    // converting `[[target]]` text into wikiLink nodes after parse.
    match: () => false,
    runner: () => {},
  },
  toMarkdown: {
    match: (node) => node.type.name === "wikiLink",
    runner: (state, node) => {
      state.addNode("text", undefined, `[[${node.attrs.target}]]`);
    },
  },
}));
