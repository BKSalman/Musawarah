<script lang="ts">
    import { page } from '$app/stores';

    let errors = $page.error?.errors;
</script>
  
<h1>
    {#each errors || [] as error}
        <div>{error}</div>
    {/each}
</h1>
