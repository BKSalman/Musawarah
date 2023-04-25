import type { ComicResponseBrief } from '../../../bindings/ComicResponseBrief';
import type { PageServerData } from './$types';

export async function load({fetch}) {
    const res = await fetch("http://127.0.0.1:6060/api/v1/comics");

    if (res.status != 200) {
        return {
            status: res.status,
            error: await res.json()
        };
    }

    const data: ComicResponseBrief[] = await res.json();

    return {
        comics: data
    };
}