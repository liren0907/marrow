import { wikiLinkSchema } from "./node";
import { wikiLinkInputRule } from "./inputRule";
import { wikiLinkNodeView, wikiLinkConfigCtx } from "./nodeView";
import { wikiLinkSuggest } from "./suggest";

export const wikiLinkPlugin = [
  wikiLinkConfigCtx,
  wikiLinkSchema,
  wikiLinkInputRule,
  wikiLinkNodeView,
  wikiLinkSuggest,
].flat();

export { wikiLinkConfigCtx } from "./nodeView";
export { configWikiLinkSuggest } from "./suggest";
export { convertTextToWikiLinks } from "./loadPass";
export type { WikiLinkClickHandler, WikiLinkConfig } from "./nodeView";
