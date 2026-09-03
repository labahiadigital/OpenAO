<script lang="ts">
  import { toastStore, type ToastType } from "$lib/game/state/toastStore.svelte";

  const typeStyles: Record<ToastType, { bg: string; border: string; text: string; icon: string }> = {
    info: { bg: "bg-slate-800/90", border: "border-slate-600/40", text: "text-slate-200", icon: "ℹ" },
    success: { bg: "bg-emerald-900/90", border: "border-emerald-500/40", text: "text-emerald-200", icon: "✓" },
    warning: { bg: "bg-amber-900/90", border: "border-amber-500/40", text: "text-amber-200", icon: "⚠" },
    error: { bg: "bg-red-900/90", border: "border-red-500/40", text: "text-red-200", icon: "✕" },
    levelup: { bg: "bg-amber-800/90", border: "border-amber-400/60", text: "text-amber-100", icon: "★" },
    death: { bg: "bg-red-950/90", border: "border-red-600/50", text: "text-red-300", icon: "☠" },
    item: { bg: "bg-cyan-900/90", border: "border-cyan-500/40", text: "text-cyan-200", icon: "+" },
  };
</script>

<div class="fixed top-16 left-1/2 -translate-x-1/2 z-[60] flex flex-col items-center gap-1.5 pointer-events-none">
  {#each toastStore.toasts as toast (toast.id)}
    {@const style = typeStyles[toast.type]}
    <div
      class="pointer-events-auto rounded-lg border px-4 py-1.5 backdrop-blur-sm shadow-lg flex items-center gap-2 animate-slide-in {style.bg} {style.border}"
    >
      <span class="text-sm {style.text}">{style.icon}</span>
      <span class="text-xs font-medium {style.text}">{toast.message}</span>
      <button
        onclick={() => toastStore.remove(toast.id)}
        class="ml-1 text-xs text-stone-500 hover:text-stone-300 transition"
      >×</button>
    </div>
  {/each}
</div>

<style>
  @keyframes slide-in {
    from {
      opacity: 0;
      transform: translateY(-12px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
  .animate-slide-in {
    animation: slide-in 0.25s ease-out;
  }
</style>
