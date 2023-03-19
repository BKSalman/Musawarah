<script>
    export let data;

    const { post } = data;
</script>


<div class="post-container">
    <h1>{post.title}</h1>
    <div>{post.content}</div>
    <img src={`https://pub-57f47fc2081648efaa3f351802d15c5d.r2.dev/${post.image.path}`} alt="alo"/>
</div>

<style>

</style>
