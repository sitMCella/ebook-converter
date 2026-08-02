# 05 — Storage Location: Tasks

## Phase 1: Rust Storage Module

### T1: Add UUID Dependency
- [ ] Add `uuid = { version = "1", features = ["v4"] }` to `src-tauri/Cargo.toml` under `[dependencies]`
- [ ] Verify `cargo check` succeeds

### T2: Create Storage Module
- [ ] Create `src-tauri/src/storage.rs`
- [ ] Implement `get_books_dir(app: &AppHandle) -> Result<PathBuf, String>` — resolves `<app_data_dir>/books/`, creates it if it does not exist
- [ ] Implement `validate_book_id(book_id: &str) -> Result<(), String>` — validates the string is a valid UUID v4 to prevent path traversal
- [ ] Implement `create_book_dir(app: &AppHandle) -> Result<(String, PathBuf), String>` — generates a UUID v4, creates `books/<uuid>/`, returns `(uuid, path)`
- [ ] Implement `copy_pdf_to_storage(app: &AppHandle, source_path: &str) -> Result<StoredBook, String>` — creates a book directory, copies the PDF as `source.pdf`, returns `StoredBook`
- [ ] Implement `get_epub_output_path(app: &AppHandle, book_id: &str) -> Result<PathBuf, String>` — returns `books/<uuid>/output.epub`
- [ ] Implement `delete_book_dir(app: &AppHandle, book_id: &str) -> Result<(), String>` — validates book_id, removes the entire directory
- [ ] Define the `StoredBook` struct with `book_id` and `stored_pdf_path` fields, serialized as camelCase
- [ ] Handle cleanup on copy failure: if `std::fs::copy` fails, remove the created directory

### T3: Register IPC Commands
- [ ] Implement `import_pdf` IPC command in `storage.rs` — wraps `copy_pdf_to_storage`
- [ ] Implement `delete_book` IPC command in `storage.rs` — wraps `delete_book_dir`
- [ ] Implement `get_books_dir` IPC command in `storage.rs` — wraps `get_books_dir` and returns the path as a string
- [ ] Add `mod storage;` to `src-tauri/src/lib.rs`
- [ ] Register all three commands in the `invoke_handler` macro in `lib.rs`

### T4: Rust Unit Tests for Storage
- [ ] Test `get_books_dir` creates the `books/` directory when it does not exist
- [ ] Test `create_book_dir` generates a valid UUID directory
- [ ] Test `copy_pdf_to_storage` copies a file and returns correct `StoredBook`
- [ ] Test `copy_pdf_to_storage` cleans up on copy failure (source does not exist)
- [ ] Test `get_epub_output_path` returns the correct path
- [ ] Test `validate_book_id` accepts valid UUIDs and rejects invalid strings
- [ ] Test `delete_book_dir` removes the directory and contents
- [ ] Test `delete_book_dir` rejects non-UUID book IDs

## Phase 2: Conversion Pipeline Update

### T5: Add book_id to ConversionOptions
- [ ] Add `pub book_id: Option<String>` field to `ConversionOptions` in `src-tauri/src/conversion/mod.rs`
- [ ] Ensure the field deserializes correctly (defaults to `None` when absent from JSON)

### T6: Update Pipeline Output Path Resolution
- [ ] Modify `run_conversion` in `pipeline.rs` to check `options.book_id`
- [ ] When `book_id` is `Some`, call `storage::get_epub_output_path` to resolve the output path
- [ ] When `book_id` is `None`, fall back to the existing `resolve_output_path` logic
- [ ] Import the `storage` module in `pipeline.rs`

### T7: Pipeline Unit Tests
- [ ] Test that conversion with `book_id` writes to the book directory
- [ ] Test that conversion without `book_id` falls back to `output_folder`

## Phase 3: Frontend Tauri Bridge

### T8: Add New Functions to tauri.js
- [ ] Add `importPdf(sourcePath)` — invokes `import_pdf`, browser fallback returns `{ bookId: null, storedPdfPath: sourcePath }`
- [ ] Add `deleteBook(bookId)` — invokes `delete_book`, browser fallback is a no-op
- [ ] Add `getBooksDir()` — invokes `get_books_dir`, browser fallback returns `''`
- [ ] Export all three functions

## Phase 4: Import Flow Update

### T9: Add SET_STORAGE_INFO Action to ImportContext
- [ ] Add `SET_STORAGE_INFO` case to the reducer in `ImportContext.jsx`
- [ ] The action sets `bookId` and `storedPdfPath` on the file entry identified by `action.path`

### T10: Update useImport Hook
- [ ] After successful validation and metadata extraction, call `importPdf(path)`
- [ ] On success, dispatch `SET_STORAGE_INFO` with the returned `bookId` and `storedPdfPath`
- [ ] On failure, dispatch `UPDATE_STATUS` with status `error` and the error message

### T11: Update File Removal
- [ ] In the remove files flow, call `deleteBook(bookId)` for each file that has a `bookId` before dispatching `REMOVE_FILES`
- [ ] Handle deletion errors gracefully (log warning, still remove from UI state)

## Phase 5: Conversion Flow Update

### T12: Update useConversion Hook
- [ ] In `convertFile`, read `bookId` and `storedPdfPath` from the import state file entry
- [ ] Pass `bookId` to `settingsToConversionOptions`
- [ ] Use `storedPdfPath` (when available) as the PDF path for `convertPdfToEpub`, falling back to the original path

### T13: Update settingsToConversionOptions
- [ ] Change the function signature to accept an options object: `settingsToConversionOptions(settings, { outputFolder, bookId } = {})`
- [ ] Include `bookId` in the returned conversion options
- [ ] Update all call sites

## Phase 6: UI Updates

### T14: Remove Output Location Setting
- [ ] Remove the "Output location" setting group from the Settings screen component
- [ ] Rebalance the two-column layout (keep "Page handling" in the right column)

### T15: Update "Open Folder" Button
- [ ] In the Converted screen header, update the "Open folder" button to call `getBooksDir()` and open the result in the OS file manager
- [ ] Use Tauri's shell plugin or `open` command to open the folder

## Phase 7: Testing

### T16: Update Existing JS Tests
- [ ] Update `useImport` tests to mock the new `importPdf` call
- [ ] Update `useConversion` tests to expect `bookId` in conversion options
- [ ] Update `settingsToConversionOptions` tests for the new signature

### T17: E2E Tests
- [ ] Test that importing a PDF creates a book directory in app data
- [ ] Test that converting a PDF writes the EPUB to the book directory
- [ ] Test that removing a file deletes the book directory
- [ ] Test that the Settings screen does not show the "Output location" group

## Acceptance Criteria

- [ ] Importing a PDF copies it to `<app_data_dir>/books/<uuid>/source.pdf`
- [ ] Converting a PDF writes the EPUB to `<app_data_dir>/books/<uuid>/output.epub`
- [ ] Removing a file from the import list deletes the book directory
- [ ] The Settings screen no longer shows "Output location"
- [ ] The "Open folder" button on the Converted screen opens the `books/` directory
- [ ] The "Save as..." button on the Converted screen copies the EPUB from managed storage to a user-chosen location
- [ ] All existing unit tests pass
- [ ] All existing e2e tests pass
- [ ] Book IDs are validated as UUID v4 before constructing paths
