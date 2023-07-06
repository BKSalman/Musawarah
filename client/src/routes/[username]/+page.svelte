<script lang="ts">
    export let data;
    const { comics, user } = data;
</script>

<div class="container">
    <div class="sidebar">
        <h1>Profile</h1>
        <p><strong>Username: </strong>{user.username}</p>
        <p><strong>Display Name: </strong>{user.displayname}</p>
        <img src={user.profile_image.path} />
    </div>
    <div class="content">
        <h1>Comics</h1>
        {#each comics || [] as comic}
            <a href={`/${user.username}/${comic.id}`}>
                <div class="comic">
                    <h2 class="comic-title">{comic.title}</h2>
                    <p>{comic.description ?? "asdlfkjasdf"}</p>
                </div>
            </a>
        {/each}
    </div>
</div>

<style>
    .container {
        display: flex;
        justify-content: center;
        margin-top: 50px;
    }

    .sidebar {
        background-color: #f1f1f1;
        padding: 20px;
    }

    .content {
        flex: 1;
        padding: 20px;
        margin-left: 20px;
        margin-right: 20px;
        background-color: #ffffff;
        box-shadow: 0 0 5px rgba(0, 0, 0, 0.2);
    }

    h1,
    h2,
    p {
        margin-bottom: 10px;
    }

    .comic {
        margin-bottom: 30px;
    }

    .comic-title {
        font-weight: bold;
    }

    a {
        all: unset;
        cursor: pointer;
    }
</style>
