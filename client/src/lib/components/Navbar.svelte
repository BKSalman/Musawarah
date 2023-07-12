<script lang="ts">
    import type { UserResponseBrief } from "bindings/UserResponseBrief";
    import { Hamburger } from "svelte-hamburgers";

    export let open = false;
    export let user: UserResponseBrief | undefined;
</script>

<nav>
    <div class="left-side">
        <Hamburger bind:open />
        <a href="/"><div class="logo" /></a>
    </div>
    <div>
        {#if user}
            <a href={`/${user.username}`}><div class="user-menu" /></a>
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
        margin: auto;
        background-color: gray;
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
