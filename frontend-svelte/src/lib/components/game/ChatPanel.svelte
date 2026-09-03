<script lang="ts">
  import type { ChatMessage, ConsoleMessage } from "$lib/game/state/gameState.svelte";

  let {
    messages,
    consoleMessages,
    onSendChat,
  }: {
    messages: ChatMessage[];
    consoleMessages: ConsoleMessage[];
    onSendChat: (msg: string) => void;
  } = $props();

  let input = $state("");
  let activeTab = $state<"consola" | "global" | "party" | "clan" | "privado">("consola");
  let messagesEl: HTMLDivElement | undefined = $state();

  type Tab = typeof activeTab;
  const TABS: { id: Tab; label: string }[] = [
    { id: "consola", label: "Consola" },
    { id: "global", label: "Global" },
    { id: "party", label: "Party" },
    { id: "clan", label: "Clan" },
    { id: "privado", label: "Privado" },
  ];

  let filteredMessages = $derived(() => {
    if (activeTab === "consola") return [];
    return messages.filter((msg) => {
      switch (activeTab) {
        case "global":
          return msg.from === "GLOBAL" || msg.color === "#00ff00";
        case "party":
          return msg.from === "PARTY" || msg.color === "#00bfff";
        case "clan":
          return msg.from === "CLAN" || msg.color === "#ff8800";
        case "privado":
          return msg.from === "PRIVADO" || msg.color === "#ff00ff";
        default:
          return true;
      }
    });
  });

  function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    if (!input.trim()) return;
    onSendChat(input.trim());
    input = "";
  }

  $effect(() => {
    if (messagesEl) {
      const _ = activeTab === "consola" ? consoleMessages.length : messages.length;
      messagesEl.scrollTop = messagesEl.scrollHeight;
    }
  });
</script>

<div class="flex flex-col rounded-lg border border-white/10 bg-white/[2%] overflow-hidden">
  <!-- Tabs row -->
  <div class="flex border-b border-white/10">
    {#each TABS as tab}
      <button
        onclick={() => { activeTab = tab.id; }}
        class="flex-1 px-1 py-1.5 text-[9px] font-medium transition
          {activeTab === tab.id
            ? 'bg-white/5 text-white border-b-2 border-amber-400'
            : 'text-stone-500 hover:text-stone-300'}"
      >
        {tab.label}
      </button>
    {/each}
  </div>

  <!-- Messages -->
  <div bind:this={messagesEl} class="h-28 overflow-y-auto p-2 space-y-0.5 text-[10px] scrollbar-thin">
    {#if activeTab === "consola"}
      {#each consoleMessages as msg}
        <div>
          <span style="color: {msg.color}">{msg.text}</span>
        </div>
      {/each}
      {#if consoleMessages.length === 0}
        <p class="text-stone-600 text-center mt-6 text-[10px]">Sin mensajes en consola</p>
      {/if}
    {:else}
      {#each filteredMessages() as msg}
        <div>
          <span class="font-semibold" style="color: {msg.color}">{msg.from}:</span>
          <span class="text-stone-400 ml-1">{msg.text}</span>
        </div>
      {/each}
      {#if filteredMessages().length === 0}
        <p class="text-stone-600 text-center mt-6 text-[10px]">Sin mensajes</p>
      {/if}
    {/if}
  </div>

  <!-- Input -->
  <form onsubmit={handleSubmit} class="border-t border-white/10 flex">
    <input
      type="text"
      bind:value={input}
      placeholder="Escribe un mensaje..."
      class="flex-1 bg-transparent px-2 py-1.5 text-[10px] text-stone-200 placeholder:text-stone-600 focus:outline-none"
    />
    <button
      type="submit"
      class="px-3 py-1.5 text-[10px] font-medium text-stone-300 hover:text-white transition"
    >
      Enviar
    </button>
  </form>
</div>
