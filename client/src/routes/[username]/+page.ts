import { error } from '@sveltejs/kit';
import { writable } from 'svelte/store';
import { browser } from '$app/environment';
import type { PageData } from './$types';
import type { PostResponse } from '../../../../bindings/PostResponse';

export async function load({ fetch, params }) {

  const { username } = params;

  if (browser) {
    localStorage.setItem("auth_key", "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpYXQiOjE2NzkxODAxODUsImV4cCI6MTY3OTE4MTM4NSwibmJmIjoxNjc5MTgwMTg1LCJ1c2VyIjp7ImlkIjoiMDE4NmY2NzMtNzkxNi03NTM1LTk2YmItNzA1NDIzZTg0MDAyIiwidXNlcm5hbWUiOiJsbWZhbyIsImVtYWlsIjoibG1hb0BsbWFvLmNvbSJ9fQ.PtcrZl5Vq6DFTsUDv5DLxm8RNqZl_nm0DLHM3rISjmw");

    const authKey = localStorage.getItem("auth_key");

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

    console.log(data);

      return {
        posts: data
      };
  }
}