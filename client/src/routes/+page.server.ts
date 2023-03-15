/** @type {import('./$types').PageServerData} */
export async function load({fetch}) {
    const res = await fetch("http://127.0.0.1:6060/api/posts");

    if (res.status != 200) {
        return {
            status: res.status,
            error: await res.json()
        };
    }

    const data = await res.json();

    return {
        posts: data
    };
}