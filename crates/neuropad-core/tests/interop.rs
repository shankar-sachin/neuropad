use neuropad_core::ipynb;
use neuropad_core::Notebook;
use tempfile::tempdir;

#[test]
fn npad_round_trip() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("demo.npad");

    let mut nb = Notebook::new("Demo");
    nb.add_markdown_cell("# Hello");
    nb.add_code_cell("go", "fmt.Println(\"ok\")");
    nb.save_npad(&path).expect("save");

    let loaded = Notebook::load_npad(&path).expect("load");
    assert_eq!(loaded.metadata.title, "Demo");
    assert_eq!(loaded.cells.len(), 2);
}

#[test]
fn ipynb_export_then_import() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("demo.ipynb");

    let mut nb = Notebook::new("Interop");
    nb.add_markdown_cell("hello");
    nb.add_code_cell("ruby", "1+1");
    ipynb::export_ipynb(&nb, &path).expect("export");

    let imported = ipynb::import_ipynb(&path).expect("import");
    assert_eq!(imported.cells.len(), 2);
    assert_eq!(
        imported.cells[1].language.as_deref(),
        Some("ruby")
    );
}
