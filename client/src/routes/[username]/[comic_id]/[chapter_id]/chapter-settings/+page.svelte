<script lang="ts">
    import Fa from "svelte-fa";
    import type { PageData } from "./$types";
    import type { UpdateChapter } from "bindings/UpdateChapter";
    import { faX } from "@fortawesome/free-solid-svg-icons";

    export let data: PageData;
    const { chapter, comic_id } = data;
    const acceptedFileTypes = ["image/png", "image/jpeg", "image/jpg"];

    let currentChapterNumber = chapter.number;

    const chapterPages = chapter.pages.map((p) => {
      return {
        page_id: p.id,
        path: `https://pub-26fa98a6ad0f4dd388ce1e8e1450be41.r2.dev/${p.image.path}`,
      }
    });

    let pages: any[] = [...chapterPages];

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

    async function deleteChapterPage(page: any) {
        if (!page.file) {
          const deleteRes = await fetch(`http://localhost:6060/api/v1/comics/chapters/pages/${page.page_id}`, {
            method: "DELETE",
            credentials: "include",
          });
          console.log(deleteRes.statusText);
        }
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
            number: i + 1,
            path: page.path,
          });

          return result;
        }, []);

        const updatePages = pages.reduce((result, page, i) => {
          if (page.file) return result;

          const chapterPageIndex = chapter.pages.findIndex((chapterPage) => `https://pub-26fa98a6ad0f4dd388ce1e8e1450be41.r2.dev/${chapterPage.image.path}` === page.path);
          if (i !== chapterPageIndex) {
            result.push({
              page_id: chapter.pages[chapterPageIndex].id,
              number: i + 1,
            });
          }
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
          console.log(createRes.statusText);
        }

        for (const page of updatePages) {
          const updateRes = await fetch(`http://localhost:6060/api/v1/comics/chapters/pages/${page.page_id}`, {
            headers: {
              "Content-type": "application/json",
            },
            method: "PUT",
            credentials: "include",
            body: JSON.stringify({
              number: page.number,
            }),
          });

          // TODO: handle error
          console.log(updateRes.statusText);
        }
    }
</script>

<!-- prevent page from changing when dropping image on document/window -->
<svelte:window on:dragover={(e) => e.preventDefault()} on:drop={(e) => e.preventDefault()} />

<div class="main-container">
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
        {#if pages.length < 1}
            <span class="drop-zone-text">Drop images to upload</span>
        {/if}
        <!-- TODO: also list already uploaded images -->
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
              <button class="drop-zone-image-x" on:click={() => deleteChapterPage(pages[other])} ><Fa size="1.5x" icon={faX}/></button>
              <img class="drop-zone-image" src={page.path} alt="">
            </div>
        {/each}
    </div>
    <input multiple on:input={addImagesFromInput} type="file" accept={acceptedFileTypes.join(",")}>
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
</style>
