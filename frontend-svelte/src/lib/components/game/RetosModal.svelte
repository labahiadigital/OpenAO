<script lang="ts">
  import type { RetosState } from "$lib/game/lib/aowProtocol";
  import { createRetosActionPacket } from "$lib/game/lib/aowProtocol";
  import { gameSession } from "$lib/game/session/gameSession.svelte";
  import { gameState } from "$lib/game/state/gameState.svelte";

  let { retosState, onClose }: { retosState: RetosState; onClose: () => void } = $props();

  function sendRetos(action: "refresh" | "create" | "join" | "cancel", payload: Record<string, unknown> = {}) {
    gameSession.send(createRetosActionPacket(action, payload));
  }

  function refresh() { sendRetos("refresh"); }
  function createChallenge(teamSize: 1 | 2) { sendRetos("create", { teamSize }); }
  function joinChallenge(id: string) { sendRetos("join", { challengeId: id }); }
  function cancelChallenge(id: string) { sendRetos("cancel", { challengeId: id }); }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") onClose();
  }

  const challenges = $derived(retosState?.challenges ?? []);
  const myName = $derived(gameState.hud.name ?? "");
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="fixed inset-0 z-[92] flex items-center justify-center bg-black/60 px-4 py-6 backdrop-blur-sm">
  <div class="w-full max-w-2xl rounded-2xl border border-white/10 bg-stone-950/95 text-stone-100 shadow-2xl">
    <div class="flex items-center justify-between border-b border-white/8 px-4 py-3">
      <div>
        <p class="text-sm font-semibold text-white">Retos</p>
        <p class="text-xs text-stone-400">{myName}</p>
      </div>
      <div class="flex items-center gap-2">
        <button onclick={refresh}
          class="rounded-md border border-white/10 px-2.5 py-1 text-xs text-stone-300 hover:border-white/20 hover:text-white transition">
          Refrescar
        </button>
        <button onclick={onClose}
          class="rounded-md border border-white/10 px-2.5 py-1 text-xs text-stone-300 hover:border-white/20 hover:text-white transition">
          Cerrar
        </button>
      </div>
    </div>

    <div class="grid gap-3 p-4 md:grid-cols-[260px_1fr]">
      <section class="space-y-2">
        <div class="grid grid-cols-2 gap-2">
          <button onclick={() => createChallenge(1)}
            class="rounded-lg bg-amber-500/10 border border-amber-500/20 px-3 py-2 text-xs text-amber-300 hover:bg-amber-500/20 transition">
            Crear 1v1
          </button>
          <button onclick={() => createChallenge(2)}
            class="rounded-lg bg-blue-500/10 border border-blue-500/20 px-3 py-2 text-xs text-blue-300 hover:bg-blue-500/20 transition">
            Crear 2v2
          </button>
        </div>
        <p class="text-[10px] text-stone-500 px-1">{challenges.length} retos activos</p>
      </section>

      <section class="max-h-72 overflow-y-auto space-y-2">
        {#if challenges.length === 0}
          <div class="flex items-center justify-center h-24 text-xs text-stone-500">
            No hay retos activos.
          </div>
        {:else}
          {#each challenges as ch}
            <div class="rounded-lg border border-white/5 bg-white/[2%] p-3">
              <div class="flex items-center justify-between mb-1.5">
                <span class="text-xs font-medium text-stone-200">
                  {ch.teamSize === 1 ? "1v1" : "2v2"} &mdash; {ch.proposer.name}
                </span>
                <span class="text-[10px] text-stone-500">
                  {ch.participants.length}/{ch.teamSize * 2}
                </span>
              </div>
              <div class="flex flex-wrap gap-1 mb-2">
                {#each ch.participants as p}
                  <span class="rounded bg-white/5 px-2 py-0.5 text-[11px] text-stone-300">
                    {p.name} <span class="text-stone-500">Lv{p.level}</span>
                  </span>
                {/each}
              </div>
              <div class="flex gap-2">
                {#if ch.proposer.name === myName}
                  <button onclick={() => cancelChallenge(ch.id)}
                    class="rounded border border-red-500/20 bg-red-500/10 px-2 py-0.5 text-[11px] text-red-400 hover:bg-red-500/20 transition">
                    Cancelar
                  </button>
                {:else if ch.participants.length < ch.teamSize * 2}
                  <button onclick={() => joinChallenge(ch.id)}
                    class="rounded border border-green-500/20 bg-green-500/10 px-2 py-0.5 text-[11px] text-green-400 hover:bg-green-500/20 transition">
                    Unirse
                  </button>
                {/if}
              </div>
            </div>
          {/each}
        {/if}
      </section>
    </div>
  </div>
</div>
