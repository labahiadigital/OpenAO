<script lang="ts">
  import { goto } from "$app/navigation";

  type CharacterClass = "mago" | "clerigo" | "guerrero" | "asesino" | "bardo" | "druida" | "paladin" | "cazador";
  type Race = "humano" | "elfo" | "elfoDrow" | "enano" | "gnomo";
  type Gender = "male" | "female";

  const classes: { key: CharacterClass; label: string; desc: string }[] = [
    { key: "mago", label: "Mago", desc: "Maestro de las artes arcanas" },
    { key: "clerigo", label: "Clérigo", desc: "Sanador y protector divino" },
    { key: "guerrero", label: "Guerrero", desc: "Experto en combate cuerpo a cuerpo" },
    { key: "asesino", label: "Asesino", desc: "Golpes letales desde las sombras" },
    { key: "bardo", label: "Bardo", desc: "Versátil músico combatiente" },
    { key: "druida", label: "Druida", desc: "Conexión con la naturaleza" },
    { key: "paladin", label: "Paladín", desc: "Guerrero sagrado con magia divina" },
    { key: "cazador", label: "Cazador", desc: "Combatiente a distancia" },
  ];

  const races: { key: Race; label: string; bonus: string }[] = [
    { key: "humano", label: "Humano", bonus: "+1 Fue, +1 Agi, +2 Con" },
    { key: "elfo", label: "Elfo", bonus: "+2 Agi, +2 Int, +1 Car" },
    { key: "elfoDrow", label: "Elfo Drow", bonus: "+2 Fue, +1 Agi, +1 Con, +1 Int" },
    { key: "enano", label: "Enano", bonus: "+3 Fue, +3 Con, -3 Int" },
    { key: "gnomo", label: "Gnomo", bonus: "+3 Agi, +4 Int, +2 Car, -2 Fue" },
  ];

  let name = $state("");
  let selectedClass = $state<CharacterClass>("guerrero");
  let selectedRace = $state<Race>("humano");
  let selectedGender = $state<Gender>("male");
  let error = $state("");
  let loading = $state(false);

  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    error = "";
    const trimmedName = name.trim();

    if (!trimmedName) {
      error = "Necesitas escribir un nombre para tu personaje.";
      return;
    }

    if (trimmedName.length < 3) {
      error = "El nombre debe tener al menos 3 caracteres.";
      return;
    }

    if (trimmedName.length > 16) {
      error = "El nombre no puede tener más de 16 caracteres.";
      return;
    }

    if (!/^[a-zA-ZáéíóúÁÉÍÓÚñÑ\s]+$/.test(trimmedName)) {
      error = "El nombre solo puede contener letras y espacios.";
      return;
    }

    loading = true;

    try {
      const response = await fetch("/api/auth/create-character", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          name: trimmedName,
          class: selectedClass,
          race: selectedRace,
          gender: selectedGender,
        }),
      });

      if (!response.ok) {
        const data = await response.json();
        error = data.error || "Error al crear el personaje";
        return;
      }

      goto("/characters");
    } catch {
      error = "Error de conexión";
    } finally {
      loading = false;
    }
  }
</script>

<svelte:head>
  <title>Crear Personaje | OpenAO</title>
</svelte:head>

<main class="mx-auto flex min-h-[80vh] max-w-2xl items-center justify-center px-4 py-10">
  <div class="w-full rounded-2xl border border-white/8 bg-white/3 p-6 md:p-8">
    <h1 class="mb-6 text-center text-2xl font-bold text-stone-100">Crear Personaje</h1>

    {#if error}
      <div class="mb-4 rounded-lg bg-red-500/10 px-4 py-3 text-sm text-red-400">{error}</div>
    {/if}

    <form onsubmit={handleSubmit} class="flex flex-col gap-6">
      <div>
        <label for="charName" class="mb-1 block text-sm text-stone-400">Nombre del personaje</label>
        <input
          id="charName"
          type="text"
          bind:value={name}
          maxlength={16}
          required
          placeholder="Ej: Gandalf"
          class="w-full rounded-lg border border-white/10 bg-white/5 px-4 py-2.5 text-stone-100 placeholder-stone-500 focus:border-amber-300/50 focus:outline-none"
        />
      </div>

      <fieldset>
        <legend class="mb-2 block text-sm text-stone-400">Clase</legend>
        <div class="grid grid-cols-2 gap-2 sm:grid-cols-4">
          {#each classes as cls}
            <button
              type="button"
              onclick={() => selectedClass = cls.key}
              class="rounded-lg border px-3 py-2 text-left text-sm transition {selectedClass === cls.key ? 'border-amber-300/50 bg-amber-300/10 text-amber-200' : 'border-white/8 bg-white/3 text-stone-300 hover:border-white/15'}"
            >
              <div class="font-medium">{cls.label}</div>
              <div class="mt-0.5 text-[11px] text-stone-500">{cls.desc}</div>
            </button>
          {/each}
        </div>
      </fieldset>

      <fieldset>
        <legend class="mb-2 block text-sm text-stone-400">Raza</legend>
        <div class="grid grid-cols-2 gap-2 sm:grid-cols-3">
          {#each races as race}
            <button
              type="button"
              onclick={() => selectedRace = race.key}
              class="rounded-lg border px-3 py-2 text-left text-sm transition {selectedRace === race.key ? 'border-amber-300/50 bg-amber-300/10 text-amber-200' : 'border-white/8 bg-white/3 text-stone-300 hover:border-white/15'}"
            >
              <div class="font-medium">{race.label}</div>
              <div class="mt-0.5 text-[11px] text-stone-500">{race.bonus}</div>
            </button>
          {/each}
        </div>
      </fieldset>

      <fieldset>
        <legend class="mb-2 block text-sm text-stone-400">Género</legend>
        <div class="flex gap-2">
          <button
            type="button"
            onclick={() => selectedGender = "male"}
            class="flex-1 rounded-lg border px-4 py-2 text-sm transition {selectedGender === 'male' ? 'border-amber-300/50 bg-amber-300/10 text-amber-200' : 'border-white/8 bg-white/3 text-stone-300 hover:border-white/15'}"
          >
            Masculino
          </button>
          <button
            type="button"
            onclick={() => selectedGender = "female"}
            class="flex-1 rounded-lg border px-4 py-2 text-sm transition {selectedGender === 'female' ? 'border-amber-300/50 bg-amber-300/10 text-amber-200' : 'border-white/8 bg-white/3 text-stone-300 hover:border-white/15'}"
          >
            Femenino
          </button>
        </div>
      </fieldset>

      <button
        type="submit"
        disabled={loading}
        class="mt-2 rounded-lg bg-amber-300 px-4 py-3 font-semibold text-stone-950 transition hover:bg-amber-200 disabled:opacity-50"
      >
        {loading ? "Creando..." : "Crear Personaje"}
      </button>
    </form>

    <p class="mt-6 text-center text-sm text-stone-400">
      <a href="/characters" class="text-amber-300 hover:underline">Volver a mis personajes</a>
    </p>
  </div>
</main>
