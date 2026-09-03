export type ToastType = "info" | "success" | "warning" | "error" | "levelup" | "death" | "item";

export interface Toast {
  id: number;
  message: string;
  type: ToastType;
  createdAt: number;
  durationMs: number;
}

let nextId = 1;
const DEFAULT_DURATION = 4000;

class ToastStore {
  toasts: Toast[] = $state([]);

  add(message: string, type: ToastType = "info", durationMs = DEFAULT_DURATION) {
    const toast: Toast = {
      id: nextId++,
      message,
      type,
      createdAt: Date.now(),
      durationMs,
    };
    this.toasts = [...this.toasts, toast];

    setTimeout(() => {
      this.remove(toast.id);
    }, durationMs);
  }

  remove(id: number) {
    this.toasts = this.toasts.filter((t) => t.id !== id);
  }

  info(message: string) {
    this.add(message, "info");
  }
  success(message: string) {
    this.add(message, "success");
  }
  warning(message: string) {
    this.add(message, "warning");
  }
  error(message: string) {
    this.add(message, "error");
  }
  levelUp(level: number) {
    this.add(`Has subido a nivel ${level}!`, "levelup", 6000);
  }
  death() {
    this.add("Has muerto", "death", 5000);
  }
  itemPickup(itemName: string, amount: number) {
    this.add(
      amount > 1 ? `+${amount} ${itemName}` : `+${itemName}`,
      "item",
      3000,
    );
  }
}

export const toastStore = new ToastStore();
