import { convertFileSrc } from "@tauri-apps/api/core";

/**
 * Safely converts a file path to a URL.
 * Detects if running in a Tauri environment. 
 * If not, returns the path as-is (assuming it's a web-compatible mock path).
 */
export function safeConvertFileSrc(path: string): string {
    const isTauri = typeof window !== "undefined" && (window as any).__TAURI__ !== undefined;

    if (isTauri) {
        try {
            return convertFileSrc(path);
        } catch (err) {
            console.warn("[TauriUtils] convertFileSrc failed, falling back to raw path:", err);
            return path;
        }
    }

    // In browser/mock mode, we assume the path is already a valid relative URL
    return path;
}
