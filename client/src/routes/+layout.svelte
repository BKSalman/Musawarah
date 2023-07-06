<script lang="ts">
    import Navbar from "$lib/components/Navbar.svelte";
    import Sidebar from "$lib/components/Sidebar.svelte";
    import type { LayoutData } from "./$types";
    export let data: LayoutData;
    let open = false;
    $: user = data.user;
</script>

<Navbar bind:open {user} />
<div class="layout-container">
    <Sidebar {open} />
    <slot />
</div>

<br />

<footer>footer</footer>

<style>
    .layout-container {
        display: grid;
        height: 100%;
        grid-template-columns: 1fr 7fr;
    }
</style>
