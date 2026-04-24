import { getDocument, type PDFDocumentProxy } from "pdfjs-dist";
import { readBinaryFile } from "$lib/workspace/tauri";
import { ensurePdfWorker } from "$lib/viewers/PdfViewer/usePdfWorker";

export async function loadPdfDoc(path: string): Promise<PDFDocumentProxy> {
  ensurePdfWorker();
  const bytes = await readBinaryFile(path);
  return await getDocument({ data: bytes }).promise;
}
