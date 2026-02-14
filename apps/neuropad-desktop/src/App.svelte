<script>
  import { invoke } from "@tauri-apps/api/core";

  let notebook = null;
  let filePath = "";
  let title = "Untitled Notebook";
  let status = "Ready";

  async function createNotebook() {
    notebook = await invoke("notebook_new", { title });
    status = "Created new notebook";
  }

  async function addMarkdownCell() {
    notebook.cells.push({
      id: crypto.randomUUID(),
      type: "markdown",
      language: null,
      source: "",
      outputs: [],
      execution: { count: 0, status: "idle", duration_ms: 0 }
    });
    notebook = { ...notebook };
  }

  async function addCodeCell(language) {
    notebook.cells.push({
      id: crypto.randomUUID(),
      type: "code",
      language,
      source: "",
      outputs: [],
      execution: { count: 0, status: "idle", duration_ms: 0 }
    });
    notebook = { ...notebook };
  }

  async function runCell(cell) {
    const [ticket, outputs] = await invoke("cell_execute", {
      notebookId: "default-notebook",
      cellId: cell.id,
      language: cell.language,
      code: cell.source
    });
    cell.outputs = outputs;
    cell.execution.status = ticket.status;
    cell.execution.count += 1;
    notebook = { ...notebook };
  }

  async function saveNotebook() {
    if (!filePath) {
      status = "Set a path first";
      return;
    }
    await invoke("notebook_save", { path: filePath, notebook });
    status = `Saved ${filePath}`;
  }

  async function openNotebook() {
    if (!filePath) {
      status = "Set a path first";
      return;
    }
    notebook = await invoke("notebook_open", { path: filePath });
    status = `Opened ${filePath}`;
  }

  createNotebook();
</script>

<main>
  <header>
    <h1>NeuroPad</h1>
    <p>Desktop polyglot notebook for Go, Ruby, and Python</p>
  </header>

  <section class="toolbar">
    <input bind:value={title} placeholder="Notebook title" />
    <input bind:value={filePath} placeholder="C:\path\to\notebook.npad" />
    <button on:click={createNotebook}>New</button>
    <button on:click={openNotebook}>Open</button>
    <button on:click={saveNotebook}>Save</button>
    <button on:click={addMarkdownCell}>+ Markdown</button>
    <button on:click={() => addCodeCell("go")}>+ Go Cell</button>
    <button on:click={() => addCodeCell("ruby")}>+ Ruby Cell</button>
    <button on:click={() => addCodeCell("python")}>+ Python Cell</button>
  </section>

  {#if notebook}
    <section class="cells">
      {#each notebook.cells as cell}
        <article class="cell">
          <div class="cell-meta">
            <strong>{cell.type}</strong>
            {#if cell.language}<span>{cell.language}</span>{/if}
            {#if cell.type === "code"}
              <button on:click={() => runCell(cell)}>Run</button>
            {/if}
          </div>
          <textarea bind:value={cell.source} rows="6"></textarea>
          {#if cell.outputs?.length}
            <div class="outputs">
              {#each cell.outputs as output}
                <pre>{output.kind}: {output.data}</pre>
              {/each}
            </div>
          {/if}
        </article>
      {/each}
    </section>
  {/if}

  <footer>{status}</footer>
</main>

<style>
  :global(body) {
    margin: 0;
    font-family: "IBM Plex Sans", "Segoe UI", sans-serif;
    background: linear-gradient(180deg, #f4f8ff 0%, #eef5ec 100%);
    color: #13233a;
  }
  main {
    max-width: 1080px;
    margin: 0 auto;
    padding: 1rem;
  }
  header h1 {
    margin: 0;
    font-size: 2rem;
  }
  .toolbar {
    display: grid;
    grid-template-columns: repeat(7, minmax(0, 1fr));
    gap: 0.5rem;
    margin: 1rem 0;
  }
  .toolbar input,
  .toolbar button,
  textarea {
    border: 1px solid #8ea3be;
    border-radius: 8px;
    padding: 0.5rem;
    font-size: 0.95rem;
  }
  .toolbar button {
    background: #1f6feb;
    color: white;
    cursor: pointer;
  }
  .cells {
    display: grid;
    gap: 1rem;
  }
  .cell {
    background: white;
    border: 1px solid #d2dff0;
    border-radius: 10px;
    padding: 0.75rem;
  }
  .cell-meta {
    display: flex;
    gap: 0.6rem;
    align-items: center;
    margin-bottom: 0.4rem;
  }
  .outputs {
    margin-top: 0.5rem;
    border-top: 1px dashed #9bb2cd;
  }
  footer {
    margin-top: 1rem;
    color: #2e4863;
  }
  @media (max-width: 900px) {
    .toolbar {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }
</style>
