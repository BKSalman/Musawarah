import { error } from '@sveltejs/kit';
import { writable } from 'svelte/store';
import type { PageServerData } from './$types';
import type { PostResponse } from '../../../../bindings/PostResponse';

export async function load({ fetch, params, cookies }) {

  const { username } = params;

  const authKey = cookies.get("auth_key");

  const res = await fetch(`http://127.0.0.1:6060/api/users/${username}`, {
    headers: {
      "Authorization": `Bearer ${authKey}`
    }
  });

  if (res.status != 200) {
    const errorMessages = await res.json();
    throw error(res.status, errorMessages);
  }

  const data: Array<PostResponse> = await res.json();

  return {
    posts: data
  };
}
