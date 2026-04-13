import { GlobalWorkerOptions } from "pdfjs-dist";
import workerSrc from "pdfjs-dist/build/pdf.worker.mjs?url";

let initialized = false;

export function ensurePdfWorker(): void {
  if (initialized) return;
  GlobalWorkerOptions.workerSrc = workerSrc;
  initialized = true;
}
