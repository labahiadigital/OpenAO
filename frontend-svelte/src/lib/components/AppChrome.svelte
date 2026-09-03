<script lang="ts">
  import {
    Home,
    Swords,
    Trophy,
    UserRound,
    ScrollText,
    LogIn,
    LogOut,
    MessageCircle,
    Hammer,
    Gamepad2,
    Newspaper,
    BarChart3,
  } from "lucide-svelte";
  import { page } from "$app/state";
  import { goto } from "$app/navigation";
  import type { AuthSession, AuthErrorResponse } from "$lib/auth";
  import { browser } from "$app/environment";
  import type { Snippet } from "svelte";

  let { children }: { children: Snippet } = $props();

  let session = $state<AuthSession | null>(null);
  let isGameDataAdmin = $state(false);

  type NavItem = {
    href: string;
    label: string;
    icon: typeof Home;
    external?: boolean;
  };

  const navItems: NavItem[] = [
    { href: "/", label: "Inicio", icon: Home },
    { href: "/play", label: "Jugar", icon: Gamepad2 },
    { href: "/arenas", label: "Arenas", icon: Swords },
    { href: "/ranking", label: "Ranking", icon: Trophy },
    { href: "/updates", label: "Novedades", icon: Newspaper },
    { href: "/wiki/equipment", label: "Wiki", icon: ScrollText },
    { href: "/users-online-stats", label: "Stats", icon: BarChart3 },
    {
      href: "https://discord.gg/sf8rWAvgxs",
      label: "Discord",
      icon: MessageCircle,
      external: true,
    },
  ];

  function isActivePath(pathname: string, href: string): boolean {
    if (href === "/") return pathname === "/";
    if (href === "/wiki/equipment")
      return pathname === "/wiki" || pathname.startsWith("/wiki/");
    return pathname === href || pathname.startsWith(`${href}/`);
  }

  $effect(() => {
    if (!browser) return;

    let cancelled = false;

    fetch("/api/auth/me", { cache: "no-store" })
      .then(async (response) => {
        if (!response.ok) return null;
        const result = (await response.json()) as AuthSession | AuthErrorResponse;
        if ("error" in result) return null;
        return result;
      })
      .then((result) => {
        if (!cancelled) session = result;
      })
      .catch(() => {
        if (!cancelled) session = null;
      });

    return () => {
      cancelled = true;
    };
  });

  async function handleSignOut() {
    await fetch("/api/auth/signout", { method: "POST" });
    session = null;
    goto("/login");
  }

  let pathname = $derived(page.url.pathname);
  let isPlayPage = $derived(pathname === "/play");
</script>

{#if isPlayPage}
  {@render children()}
{:else}
  <header
    class="sticky top-0 z-50 border-b border-white/8 bg-[#05080d]/92 backdrop-blur-xl"
  >
    <div
      class="mx-auto flex max-w-7xl items-center justify-between gap-4 px-4 py-4"
    >
      <a href="/" class="flex items-center gap-3">
        <div
          class="flex h-8 w-8 items-center justify-center rounded-xl bg-amber-300 text-sm font-black text-stone-950"
        >
          AO
        </div>
        <span class="text-3xl font-semibold tracking-wide text-stone-100">
          AOWeb
        </span>
      </a>

      <nav
        class="hidden items-center gap-2 rounded-2xl border border-white/6 bg-black/20 p-1 md:flex"
      >
        {#each navItems as item}
          {@const active = item.external
            ? false
            : isActivePath(pathname, item.href)}
          <a
            href={item.href}
            target={item.external ? "_blank" : undefined}
            rel={item.external ? "noreferrer" : undefined}
            class="inline-flex items-center gap-2 rounded-xl px-4 py-2 text-sm transition {active
              ? 'bg-amber-300/12 text-amber-300'
              : 'text-stone-400 hover:bg-white/5 hover:text-stone-100'}"
          >
            <item.icon class="h-4 w-4" />
            {item.label}
          </a>
        {/each}
        {#if isGameDataAdmin}
          <a
            href="/construccion"
            class="inline-flex items-center gap-2 rounded-xl px-4 py-2 text-sm transition {isActivePath(
              pathname,
              '/construccion',
            )
              ? 'bg-amber-300/12 text-amber-300'
              : 'text-stone-400 hover:bg-white/5 hover:text-stone-100'}"
          >
            <Hammer class="h-4 w-4" />
            Construccion
          </a>
        {/if}
      </nav>

      <div class="flex items-center gap-3">
        {#if session}
          <span class="hidden text-sm text-stone-200 sm:inline">
            {session.account.name}
          </span>
          <button
            type="button"
            onclick={handleSignOut}
            class="inline-flex items-center justify-center rounded-full p-2 text-stone-400 transition hover:bg-white/5 hover:text-stone-100"
            aria-label="Cerrar sesion"
          >
            <LogOut class="h-4 w-4" />
          </button>
        {:else}
          <a
            href="/login"
            class="inline-flex items-center gap-2 rounded-xl border border-white/8 px-4 py-2 text-sm text-stone-200 transition hover:bg-white/5"
          >
            <LogIn class="h-4 w-4" />
            Ingresar
          </a>
        {/if}
      </div>
    </div>
  </header>

  <div
    class="border-b border-white/8 bg-[#05080d]/92 px-4 py-2 backdrop-blur-xl md:hidden"
  >
    <nav
      class="mx-auto flex max-w-7xl items-center gap-2 overflow-x-auto"
    >
      {#each navItems as item}
        {@const active = item.external
          ? false
          : isActivePath(pathname, item.href)}
        <a
          href={item.href}
          target={item.external ? "_blank" : undefined}
          rel={item.external ? "noreferrer" : undefined}
          class="inline-flex shrink-0 items-center gap-2 rounded-xl px-3 py-2 text-sm transition {active
            ? 'bg-amber-300/12 text-amber-300'
            : 'text-stone-400 hover:bg-white/5 hover:text-stone-100'}"
        >
          <item.icon class="h-4 w-4" />
          {item.label}
        </a>
      {/each}
    </nav>
  </div>

  {@render children()}
{/if}
