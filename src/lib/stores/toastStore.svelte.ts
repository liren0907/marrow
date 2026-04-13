export interface Toast {
    id: string;
    message: string;
    variant: 'success' | 'error' | 'info' | 'warning';
    duration?: number;
}

export const toasts = $state<Toast[]>([]);

function addToast(message: string, variant: Toast['variant'], duration: number = 5000): string {
    const id = crypto.randomUUID();
    const toast: Toast = { id, message, variant, duration };
    toasts.push(toast);

    if (duration > 0) {
        setTimeout(() => {
            dismissToast(id);
        }, duration);
    }

    return id;
}

export function showToast(message: string, variant: Toast['variant'] = 'info', duration: number = 5000): string {
    return addToast(message, variant, duration);
}

export function showSuccess(message: string, duration: number = 5000): string {
    return addToast(message, 'success', duration);
}

export function showError(message: string, duration: number = 5000): string {
    return addToast(message, 'error', duration);
}

export function showWarning(message: string, duration: number = 5000): string {
    return addToast(message, 'warning', duration);
}

export function dismissToast(id: string) {
    const idx = toasts.findIndex(t => t.id === id);
    if (idx !== -1) toasts.splice(idx, 1);
}

export function clearToasts() {
    toasts.length = 0;
}

// Backward-compatible object for existing consumers that use toastStore.show() / toastStore.success() etc.
export const toastStore = {
    show: showToast,
    success: showSuccess,
    error: showError,
    dismiss: dismissToast,
    clear: clearToasts,
};
