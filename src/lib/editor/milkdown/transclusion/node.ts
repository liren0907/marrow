import { $nodeSchema } from "@milkdown/utils";

export const transclusionSchema = $nodeSchema("transclusion", () => ({
  inline: false,
  group: "block",
  atom: true,
  selectable: true,
  draggable: false,
  attrs: {
    target: { default: "" },
    section: { default: "" },
  },
  parseDOM: [
    {
      tag: "div[data-transclusion]",
      getAttrs: (dom) => {
        if (!(dom instanceof HTMLElement)) return false;
        return {
          target: dom.getAttribute("data-target") ?? "",
          section: dom.getAttribute("data-section") ?? "",
        };
      },
    },
  ],
  toDOM: (node) => [
    "div",
    {
      "data-transclusion": "",
      "data-target": node.attrs.target as string,
      "data-section": node.attrs.section as string,
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
      const target = node.attrs.target as string;
      const section = node.attrs.section as string;
      const inner = section ? `${target}#${section}` : target;
      state.addNode("text", undefined, `![[${inner}]]`);
    },
  },
}));
