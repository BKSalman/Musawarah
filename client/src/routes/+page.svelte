<script lang="ts">
  // import Gallery from "$lib/components/Gallery.svelte";
  // import Navbar from "$lib/components/Navbar.svelte";
  import type { PageServerData } from './$types';

  export let data: PageServerData;
  let { comics } = data;
  
</script>

<div class="main-container">
  <h1>Home page</h1>
  {#each comics || [] as comic}
    <a href={`/${comic.author.username}/${comic.id}`}><div>{comic.title}</div></a>
    <div>Genres:</div>
    {#each comic.genres as genre}
      <div>{genre.name}</div>
    {/each}
    <br/>
  {/each}
</div>

<style>
  .main-container {
    max-width: 1250px;
    height: 100%;
    margin: 0px auto;
  }
</style>
