import { isGameDataAdmin } from "./editorApi";

export type GameDataAdminState = "loading" | "allowed" | "denied";

/**
 * Checks if the current account has permission to edit maps.
 * Returns a reactive Svelte state.
 */
export function createGameDataAdmin(enabled = true) {
  let state = $state<GameDataAdminState>(enabled ? "loading" : "denied");

  if (enabled) {
    isGameDataAdmin().then((allowed) => {
      state = allowed ? "allowed" : "denied";
    });
  }

  return {
    get value() {
      return state;
    },
  };
}
