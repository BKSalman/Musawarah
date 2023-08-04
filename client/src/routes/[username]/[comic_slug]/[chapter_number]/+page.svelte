<script lang="ts">
    import { page } from '$app/stores';
    import Fa from "svelte-fa";
    import { faArrowLeft } from "@fortawesome/free-solid-svg-icons";
    import type { PageServerData } from './$types';

    export let data: PageServerData;

    const { chapter, comic_slug, username } = data;
</script>

<div class="main-container">
    <div class="back">
        <a class="back-button" href={`/${username}/${comic_slug}`}><Fa size="1.5x" icon={faArrowLeft}/></a>
    </div>
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
          {#if page.description}
              <div class="">{page.description}</div>
          {/if}
          <img class="page-image" src={`https://pub-26fa98a6ad0f4dd388ce1e8e1450be41.r2.dev/${page.image.path}`} alt="page"/>
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
.page-image {
    width: 100%;
}
.back-button {
    background: none;
    border: none;
    cursor: pointer;
    color: black;
}
</style>
