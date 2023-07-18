<script lang="ts">
    import { Hamburger } from "svelte-hamburgers";
    import { currentUser } from "../../routes/stores";

    export let open = false;
</script>

<nav>
    <div class="left-side">
        <Hamburger bind:open />
        <a href="/"><div class="logo" /></a>
    </div>
    <div>
        {#if $currentUser}
            <a href={`/${$currentUser.username}`}><div class="user-menu" /></a>
        {:else}
            <a href="/login" class="button">Log In</a>
            <a href="/register" class="button">Register</a>
        {/if}
    </div>
</nav>

<style>
    nav {
        display: flex;
        justify-content: space-between;
        align-items: center;
        height: 3em;
        width: 100%;
        background-color: gray;
        overflow: hidden;
        position: sticky;
        top: 0;
    }

    .user-menu {
        background-color: yellow;
        height: 2.5em;
        width: 2.5em;
        margin-right: 5px;
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

    .button {
        all: unset;
        background-color: yellow;
        border: none;
        padding: 8px 16px;
        text-align: center;
        text-decoration: none;
        display: inline-block;
        cursor: pointer;
        border-radius: 5px;
        margin-right: 5px;
    }

    .button:hover {
        background-color: #d9c700;
    }
</style>
