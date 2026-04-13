import { $nodeSchema } from "@milkdown/utils";

export const transclusionSchema = $nodeSchema("transclusion", () => ({
  inline: false,
  group: "block",
  atom: true,
  selectable: true,
  draggable: false,
  attrs: {
    target: { default: "" },
  },
  parseDOM: [
    {
      tag: "div[data-transclusion]",
      getAttrs: (dom) => {
        if (!(dom instanceof HTMLElement)) return false;
        return { target: dom.getAttribute("data-target") ?? "" };
      },
    },
  ],
  toDOM: (node) => [
    "div",
    {
      "data-transclusion": "",
      "data-target": node.attrs.target as string,
      class: "transclusion-embed",
    },
    node.attrs.target as string,
  ],
  parseMarkdown: {
    // Remark doesn't emit transclusion nodes; load-pass converts text after parse.
    match: () => false,
    runner: () => {},
  },
  toMarkdown: {
    match: (node) => node.type.name === "transclusion",
    runner: (state, node) => {
      state.addNode("text", undefined, `![[${node.attrs.target}]]`);
    },
  },
}));
