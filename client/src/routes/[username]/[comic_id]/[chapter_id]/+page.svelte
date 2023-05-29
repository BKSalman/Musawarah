<script lang="ts">
  import type { PageServerData } from './$types';

  export let data: PageServerData;

  const { chapter } = data;
  console.log(chapter);
</script>

<div class="chapter-main-container">
<div class="">{chapter.description}</div>
<br/>
{#each chapter.pages as page}
  <div class="">{page.number}</div>
  <img src={`https://pub-26fa98a6ad0f4dd388ce1e8e1450be41.r2.dev/${page.image.path}`}/>
{/each}
</div>

<style>
</style>
