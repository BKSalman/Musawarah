<script lang="ts">
    import Navbar from "$lib/components/Navbar.svelte";
    import Sidebar from "$lib/components/Sidebar.svelte";
    import { currentUser } from "./stores";
    export let data;
    currentUser.set(data.user);
    let open = false;
</script>

<Navbar bind:open />
<Sidebar {open} />
<div class="layout-container">
    <slot />
</div>
<footer>footer</footer>

<style>
    .layout-container {
        margin-left: 60px;
        padding: 16px;
        flex: 1;
    }

    footer {
        background-color: darkgray;
        padding: 20px;
        text-align: center;
    }
</style>
