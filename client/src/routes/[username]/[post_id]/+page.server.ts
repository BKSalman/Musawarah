import { error } from '@sveltejs/kit';
import type { PageServerData } from './$types';
 
export async function load({ fetch, params }) {
    const { post_id } = params;
    console.log(post_id);
    
//   throw error(404, 'Not found');
}