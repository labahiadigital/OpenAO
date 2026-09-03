/// <reference types="@sveltejs/kit" />

interface Hyperdrive {
  connectionString: string;
}

declare global {
  namespace App {
    interface Locals {
      session: import("$lib/auth").AuthSession | null;
    }

    interface Platform {
      env?: {
        HYPERDRIVE?: Hyperdrive;
      };
    }
  }
}

export {};
