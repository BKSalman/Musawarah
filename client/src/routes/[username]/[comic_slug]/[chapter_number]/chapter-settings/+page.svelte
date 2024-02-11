<script lang="ts">
    import Fa from "svelte-fa";
    import { page } from "$app/stores";
    import type { UpdateChapter } from "bindings/UpdateChapter";
    import { faX } from "@fortawesome/free-solid-svg-icons";
    import { invalidate } from "$app/navigation";
    import { faArrowLeft } from "@fortawesome/free-solid-svg-icons";

    export let data;

    $: ({chapter, comic_id} = data);

    const acceptedFileTypes = ["image/png", "image/jpeg", "image/jpg"];

    $: currentChapterNumber = chapter.number;

    let pages: any[] = [];

    function addImagesFromInput(e: Event) {
        const input = e.target as HTMLInputElement;
        for (const file of input.files || []) {
            addPage(file)
        }
    }

    function addPage(file: File) {
        // TODO: limit file size
        if (!acceptedFileTypes.includes(file.type)) return;
        const reader = new FileReader();
        reader.addEventListener("load", () => {
            pages.push({
              file,
              path: reader.result,
            });
            pages = pages;
        });
        reader.readAsDataURL(file);
    }

    async function deleteServerChapterPage(page_id: string) {
        const deleteRes = await fetch(`http://localhost:6060/api/v1/comics/chapters/pages/${page_id}`, {
          method: "DELETE",
          credentials: "include",
        });
        chapter.pages.splice(chapter.pages.findIndex((p) => p.id === page_id), 1);
        chapter.pages = chapter.pages;
        chapter = chapter;
    }

    async function deleteClientChapterPage(page: any) {
        pages.splice(pages.findIndex((p) => p.path === page.path), 1);
        pages = pages;
    }
    
    async function updateChapter(e: SubmitEvent) {
        const form = new FormData(e.target as HTMLFormElement);
        let updateChapter: UpdateChapter = {
          title: null,
          description: null,
          number: null,
        };

        const newPages = pages.reduce((result, page, i) => {
          if (!page.file) return result;

           result.push({
            file: page.file,
            number: chapter.pages.length + i + 1,
            path: page.path,
          });

          return result;
        }, []);

        // TODO: add option to change desc and title
        
        if (currentChapterNumber !== chapter.number) {
            updateChapter.number = currentChapterNumber;
        }

        for (const page of newPages) {
          const createForm = new FormData();

          createForm.append("number", page.number);

          createForm.append("image", page.file);
          
          const createRes = await fetch(`http://localhost:6060/api/v1/comics/${comic_id}/chapters/${chapter.id}/pages`, {
            method: "POST",
            credentials: "include",
            body: createForm,
          });

          // TODO: handle error

          await invalidate("chapter-info");
          pages.splice(pages.findIndex((p) => p.path === page.path) - 1, 1);
          pages = pages;
        }
    }
</script>

<!-- prevent page from changing when dropping image on document/window -->
<svelte:window on:dragover={(e) => e.preventDefault()} on:drop={(e) => e.preventDefault()} />

<div class="main-container">
    <div>
        <a
            class="back-button"
            href={`/${$page.params.username}/${$page.params.comic_slug}/${$page.params.chapter_number}`}
            ><Fa size="1.5x" icon={faArrowLeft} /></a
        >
    </div>
  <form action="" on:submit|preventDefault={updateChapter}>
    <label for="chapter_title">Chapter Title:</label>
    <span>{chapter.title}</span>
    {#if chapter.description}
        <label for="chapter_desc">Chapter Description:</label>
        <span>{chapter.description}</span>
    {/if}
    <label for="chapter_number">Chapter Number:</label>
    <input type="number" id="chapter_number" min="1" name="chapter_number" required bind:value={currentChapterNumber}>
    <label for="chapter_pages">Chapter Pages:</label>
    <!-- files drop zone -->
    <div role="region" class="drop-zone" on:dragover={(e) => {
      e.preventDefault();
      if (!e.dataTransfer) return;
      e.dataTransfer.dropEffect = "move";
    }} on:drop={(e) => {
        e.preventDefault();

        if (e.dataTransfer) {
            for (const file of e.dataTransfer.files) {
                addPage(file);
            }
        }
    }}>
        {#if pages.length < 1 && chapter.pages.length < 1}
            <span class="drop-zone-text">Drop images to upload</span>
        {/if}
        <!-- already uploaded images -->
        {#each chapter.pages as page}
            <div role="region" class="drop-zone-image-container" >
              <button class="drop-zone-image-x" on:click={() => deleteServerChapterPage(page.id)} ><Fa size="1.5x" icon={faX}/></button>
              <img class="server-image" src={`http://localhost:6060/api/v1/images/${page.image.path}`} alt="">
            </div>
        {/each}
        <!-- client side images (not yet uploaded) -->
        {#each pages as page, other (other)}
            <div role="region" class="drop-zone-image-container" on:dragover={(e) => {
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
            }}>
              <button class="drop-zone-image-x" on:click={() => deleteClientChapterPage(pages[other])} ><Fa size="1.5x" icon={faX}/></button>
              <img class="drop-zone-image" src={page.path} alt="">
            </div>
        {/each}
    </div>
    <input multiple on:input={addImagesFromInput} type="file" accept={acceptedFileTypes.join(",")}>
    <div class="submit-container">
        <label for="publish">Publish Chapter:</label>
        <input type="checkbox" id="publish" name="publish">
        <input type="submit" value="Submit">
    </div>
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
      position: relative;
      justify-content: center;
      align-items: center;
      min-height: 50px;
      background-color: #f2f2f2;
    }
    .drop-zone-text {
      color: rgba(0, 0, 0, 0.4);
    }
    .drop-zone-image-container {
      width: 10%;
      height: 10%;
      padding-bottom: 1.5rem;
    }
    .drop-zone-image {
      cursor: grab;
      width: 100%;
      height: 100%;
    }
    .server-image {
      width: 100%;
      height: 100%;
    }
    .drop-zone-image-x {
        background: none;
        border: none;
        position: absolute;
        cursor: pointer;
        bottom: 0;
    }
    a {
      color: #009688;
      text-decoration: none;
    }
    label {
      display: block;
      margin-bottom: 5px;
      font-weight: bold;
      font-size: 18px;
    }
    input[type="text"],
    input[type="number"],
    input[type="file"] {
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
    .submit-container {
      margin-top: 1.5rem;
      display: flex;
      justify-content: center;
      align-items: center;
      gap: 1.5rem;
    }
    .back-button {
        background: none;
        border: none;
        cursor: pointer;
        color: black;
    }
</style>
