import { error } from '@sveltejs/kit';
import type { PostResponse } from '../../../../../bindings/PostResponse';
import type { PageServerData } from './$types';

export async function load({ fetch, params, cookies }) {
    const { post_id, username } = params;

    const authKey = cookies.get("auth_key");

    const res = await fetch(`http://127.0.0.1:6060/api/posts/${username}/${post_id}`, {
        headers: {
            "Authorization": `Bearer ${authKey}`
        }
    });

    if (res.status != 200) {
        const errorMessages = await res.json();
        throw error(res.status, errorMessages);
    }

    const data: PostResponse = await res.json();

    return {
        post: data
    };
}