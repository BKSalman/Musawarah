<script lang="ts">
    import User from '$lib/components/User.svelte';
    import type { PageData } from './$types';
    
    export let data: PageData;

    const { posts } = data;
</script>


<div class="user-container">
    {#each posts || [] as post}
        <User username={post.user.username}/>
        <div>{post.title}</div>
        <div>{post.content}</div>
        <img src={`https://pub-57f47fc2081648efaa3f351802d15c5d.r2.dev/${post.image.path}`} alt="alo"/>
    {/each}
</div>

<style>

</style>