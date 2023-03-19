<script lang="ts">
    import { page } from '$app/stores';

    const errors = $page.error?.errors;
    const status = $page.status
</script>
  
<h1 class="container">
    <h1>{status}</h1>
    {#each errors || [] as error}
        <div>{error}</div>
    {/each}
</h1>

<style>
.container {
    margin: auto;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-direction: column;
}
</style>