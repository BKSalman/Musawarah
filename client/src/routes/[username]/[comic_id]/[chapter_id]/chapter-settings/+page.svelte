<script lang="ts">
    import type { PageData } from "./$types";

    export let data: PageData;
    const { chapter } = data;

    let pages: any[] = [];
</script>

<!-- prevent page from changing when dropping image on document/window -->
<svelte:window on:dragover={(e) => e.preventDefault()} on:drop={(e) => e.preventDefault()} />

<div class="main-container">
  <form action="" method="post">
    <label for="chapter_title">Chapter Title:</label>
    <span>{chapter.title}</span>
    <label for="chapter_number">Chapter Number:</label>
    <input type="number" id="chapter_number" name="chapter_number" required bind:value={chapter.number}>
    <label for="chapter_pages">Chapter Pages:</label>
    <div role="region" class="drop-zone" on:dragover={(e) => {
      e.preventDefault();
      if (!e.dataTransfer) return;
      e.dataTransfer.dropEffect = "move";
    }} on:drop={(e) => {
        e.preventDefault();

        if (e.dataTransfer && e.dataTransfer.files) {
            for (const file of e.dataTransfer?.files) {
                const reader = new FileReader();
                reader.addEventListener("load", () => {
                    console.log("adding image");
                    pages.push(reader.result);
                    pages = pages;
                });
                reader.readAsDataURL(file);
            }
        }
    }}>
        {#if pages.length < 1}
            <span class="drop-zone-text">Drop images to upload</span>
        {/if}
        {#each pages as page, other (other)}
            <img on:dragover={(e) => {
                e.preventDefault();
                if (!e.dataTransfer) return;
                e.dataTransfer.dropEffect = "move";
            }} on:dragstart={(e) => {
                if (!e.dataTransfer) return;
                e.dataTransfer.setData("text/plain", other.toString());
            }} on:drop={(e) => {
              e.preventDefault();
              if (!e.dataTransfer) return;
              const currentDraggingIndex = parseInt(e.dataTransfer.getData('text/plain'));
              if (!(e.target instanceof HTMLElement)) return;
              // remove and return the dragged element from the list
              const currentDraggingPage = pages.splice(currentDraggingIndex, 1)[0];
              // replace the other element with the dragged element
              pages.splice(other, 0, currentDraggingPage);
              pages = pages;
            }} class="drop-zone-image" src={page} alt="">
        {/each}
    </div>
    <label for="publish">Publish Chapter:</label>
    <input type="checkbox" id="publish" name="publish">
    <input type="submit" value="Submit">
  </form>
</div>

<style>
    * {
      box-sizing: border-box;
      margin: 0;
      padding: 0;
    }
    .drop-zone {
      display: flex;
      justify-content: center;
      align-items: center;
      min-height: 50px;
      background-color: #f2f2f2;
    }
    .drop-zone-text {
      color: rgba(0, 0, 0, 0.4);
    }
    .drop-zone-image {
      cursor: grab;
      width: 10%;
      height: 10%;
    }
    /* Set background color, font and link styles */
    body {
      background-color: #f2f2f2;
      font-family: 'Roboto', sans-serif;
      font-size: 16px;
      color: #444;
      line-height: 1.6;
    }
    a {
      color: #009688;
      text-decoration: none;
    }
    a:hover {
      text-decoration: underline;
    }
    /* Layout styles for form */
    .container {
      max-width: 500px;
      margin: 0 auto;
      padding: 20px;
      background-color: #fff;
      border-radius: 4px;
      box-shadow: 0 0 10px rgba(0, 0, 0, 0.1);
    }
    .form-group {
      margin-bottom: 20px;
    }
    label {
      display: block;
      margin-bottom: 5px;
      font-weight: bold;
      font-size: 18px;
    }
    input[type="text"],
    input[type="number"],
    input[type="file"],
    input[type="checkbox"] {
      display: block;
      width: 100%;
      padding: 10px;
      border: none;
      border-radius: 4px;
      font-size: 16px;
      background-color: #f2f2f2;
      transition: all 0.3s ease;
    }
    input[type="text"]:focus,
    input[type="number"]:focus,
    input[type="file"]:focus {
      outline: none;
      background-color: #e0e0e0;
    }
    input[type="file"] {
      padding: 5px;
      font-size: 14px;
      cursor: pointer;
    }
    input[type="submit"] {
      background-color: #009688;
      color: #fff;
      padding: 10px 20px;
      border: none;
      border-radius: 4px;
      font-size: 18px;
      cursor: pointer;
      transition: all 0.3s ease;
    }
    input[type="submit"]:hover {
      background-color: #00796b;
    }
    input[type="checkbox"] {
      display: inline-block;
      margin-right: 10px;
      cursor: pointer;
    }
    /* Styling for error messages */
    .error {
      color: #f44336;
      font-size: 14px;
      margin-top: 5px;
    }
    /* Media query for small screens */
    @media screen and (max-width: 480px) {
      .container {
        padding: 10px;
      }
      label {
        font-size: 16px;
      }
      input[type="text"],
      input[type="number"],
      input[type="file"],
      input[type="checkbox"] {
        font-size: 14px;
        padding: 8px;
      }
      input[type="submit"] {
        font-size: 16px;
        padding: 8px 16px;
      }
    }
</style>
