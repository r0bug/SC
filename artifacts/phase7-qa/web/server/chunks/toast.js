import { w as writable } from "./index.js";
function createToastStore() {
  const { subscribe, update } = writable([]);
  let idCounter = 0;
  function addToast(toast) {
    const id = `toast-${++idCounter}`;
    const duration = toast.duration ?? 5e3;
    update((toasts2) => [...toasts2, { ...toast, id }]);
    if (duration > 0) {
      setTimeout(() => {
        removeToast(id);
      }, duration);
    }
    return id;
  }
  function removeToast(id) {
    update((toasts2) => toasts2.filter((t) => t.id !== id));
  }
  return {
    subscribe,
    success: (message, title, options) => addToast({ type: "success", message, title, ...options }),
    error: (message, title, options) => addToast({ type: "error", message, title, duration: options?.duration ?? 7e3, ...options }),
    warning: (message, title, options) => addToast({ type: "warning", message, title, ...options }),
    info: (message, title, options) => addToast({ type: "info", message, title, ...options }),
    remove: removeToast,
    clear: () => update(() => [])
  };
}
const toasts = createToastStore();
export {
  toasts as t
};
