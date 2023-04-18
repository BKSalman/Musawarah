<script lang="ts">
    import { Hamburger } from "svelte-hamburgers";

    export let open = false;
</script>

<nav>
    <div class="left-side">
        <Hamburger bind:open />
        <a href="/"><div class="logo" /></a>
    </div>
    <div class="user-menu" />
</nav>

<style>
    nav {
        display: flex;
        justify-content: space-between;
        align-items: center;
        height: 3em;
        margin: auto;
        background-color: gray;
    }

    .user-menu {
        background-color: yellow;
        height: 2.5em;
        width: 2.5em;
    }

    .left-side {
        display: flex;
        align-items: center;
    }

    .logo {
        background-color: yellow;
        height: 2.5em;
        width: 2.5em;
    }
</style>

