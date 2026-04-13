import { transclusionSchema } from "./node";
import { transclusionInputRule } from "./inputRule";
import {
  transclusionNodeView,
  transclusionConfigCtx,
} from "./nodeView";
import { transclusionSuggest } from "./suggest";

export const transclusionPlugin = [
  transclusionConfigCtx,
  transclusionSchema,
  transclusionInputRule,
  transclusionNodeView,
  transclusionSuggest,
].flat();

export { transclusionConfigCtx } from "./nodeView";
export { configTransclusionSuggest } from "./suggest";
export { convertTextToTransclusions } from "./loadPass";
export type { TransclusionClickHandler } from "./nodeView";
export type { TransclusionSuggestion } from "./suggest";
