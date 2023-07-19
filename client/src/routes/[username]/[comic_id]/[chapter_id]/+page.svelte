<script lang="ts">
    import { page } from '$app/stores';
    import type { PageServerData } from './$types';

    export let data: PageServerData;

    const { chapter } = data;
</script>

<div class="main-container">
<div class="title-bar">
    <div class="title"><strong>Title: </strong>{chapter.title}</div>

    {#if chapter.description}
        <div class="description"><strong>Description: </strong>{chapter.description}</div>
    {/if}
    <a href={`${$page.url.href}/chapter-settings`}>Settings</a>
</div>
<br/>
<div class="pages">
    {#each chapter.pages as page}
      <div class="">{page.number}</div>
      {#if page.description}
          <div class="">{page.description}</div>
      {/if}
      <img src={`https://pub-26fa98a6ad0f4dd388ce1e8e1450be41.r2.dev/${page.image.path}`} alt="page"/>
      <div class="">{page.number}</div>
    {/each}
</div>
</div>

<style>
.main-container {
    display: flex;
    flex-direction: column;
    width: 80%;
}
.main-container > * {
    margin-bottom: 10px;
}
.pages {
    flex: 1;
    padding: 20px;
    margin-left: 20px;
    margin-right: 20px;
    background-color: #ffffff;
    box-shadow: 0 0 5px rgba(0, 0, 0, 0.2);
}
</style>
