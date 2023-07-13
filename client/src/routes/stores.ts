import type { UserResponseBrief } from 'bindings/UserResponseBrief';
import { writable } from 'svelte/store';

function createCurrentUser() {
  const { subscribe, set } = writable<UserResponseBrief | undefined>(undefined);

  return {
    subscribe,
    set,
    refresh: async () => {
      const res = await fetch(`http://localhost:6060/api/v1/users/me`, {
        credentials: "include",
      });
      const user: UserResponseBrief | undefined = res.status !== 200 ? undefined : await res.json();
      return set(user)
    }
  };
}

export const currentUser = createCurrentUser();
