const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__ != null;

async function openFile(filters = [{ name: 'EPUB Files', extensions: ['epub'] }]) {
  if (isTauri) {
    const { open } = await import('@tauri-apps/plugin-dialog');
    const { readFile } = await import('@tauri-apps/plugin-fs');

    const path = await open({
      multiple: false,
      filters,
    });

    if (!path) return null;

    const contents = await readFile(path);
    const name = path.split(/[\\/]/).pop();
    return { path, name, contents };
  }

  return new Promise((resolve) => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = filters.map((f) => f.extensions.map((e) => `.${e}`).join(',')).join(',');
    input.onchange = async () => {
      const file = input.files?.[0];
      if (!file) return resolve(null);
      const contents = new Uint8Array(await file.arrayBuffer());
      resolve({ path: file.name, name: file.name, contents });
    };
    input.click();
  });
}

async function saveFile(data, defaultName = 'output.epub', filters = [{ name: 'EPUB Files', extensions: ['epub'] }]) {
  if (isTauri) {
    const { save } = await import('@tauri-apps/plugin-dialog');
    const { writeFile } = await import('@tauri-apps/plugin-fs');

    const path = await save({
      defaultPath: defaultName,
      filters,
    });

    if (!path) return null;

    await writeFile(path, data instanceof Uint8Array ? data : new Uint8Array(data));
    return path;
  }

  const blob = new Blob([data]);
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = defaultName;
  a.click();
  URL.revokeObjectURL(url);
  return defaultName;
}

async function openPdfFiles() {
  if (isTauri) {
    const { open } = await import('@tauri-apps/plugin-dialog');
    const paths = await open({
      multiple: true,
      filters: [{ name: 'PDF Files', extensions: ['pdf'] }],
    });
    if (!paths) return null;
    return Array.isArray(paths) ? paths : [paths];
  }

  return new Promise((resolve) => {
    const input = document.createElement('input');
    input.type = 'file';
    input.multiple = true;
    input.accept = '.pdf';
    input.onchange = () => {
      const files = Array.from(input.files || []);
      if (files.length === 0) return resolve(null);
      resolve(files.map((f) => f.name));
    };
    input.click();
  });
}

async function validatePdf(path) {
  if (isTauri) {
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke('validate_pdf', { path });
  }
  return { status: 'valid' };
}

async function getPdfMetadata(path) {
  if (isTauri) {
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke('get_pdf_metadata', { path });
  }
  return {
    title: null,
    author: null,
    pageCount: 0,
    pdfVersion: '1.7',
    createdDate: null,
    modifiedDate: null,
    producer: null,
    fileSize: 0,
  };
}

async function getFileSize(path) {
  if (isTauri) {
    const { stat } = await import('@tauri-apps/plugin-fs');
    const info = await stat(path);
    return info.size;
  }
  return 0;
}

async function convertPdfToEpub(path, options) {
  if (isTauri) {
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke('convert_pdf', { path, options });
  }
  return Promise.reject(new Error('Conversion requires the desktop app'));
}

async function importPdf(sourcePath) {
  if (isTauri) {
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke('import_pdf', { sourcePath });
  }
  return { bookId: null, storedPdfPath: sourcePath };
}

async function deleteBook(bookId) {
  if (isTauri) {
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke('delete_book', { bookId });
  }
}

async function getBooksDir() {
  if (isTauri) {
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke('get_books_dir');
  }
  return '';
}

async function saveBookMetadata(metadata) {
  if (isTauri) {
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke('save_book_metadata', { metadata });
  }
}

async function listBooks() {
  if (isTauri) {
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke('list_books');
  }
  return [];
}

async function cancelConversion(path) {
  if (isTauri) {
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke('cancel_conversion', { path });
  }
}

async function onConversionProgress(callback) {
  if (isTauri) {
    const { listen } = await import('@tauri-apps/api/event');
    return listen('conversion-progress', (event) => callback(event.payload));
  }
  return () => {};
}

async function readEpubPreview(path) {
  if (isTauri) {
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke('read_epub_preview', { path });
  }
  return { coverImage: null };
}

async function openFileWithSystem(path) {
  if (isTauri) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke('open_path', { path });
    } catch (err) {
      console.error('openFileWithSystem failed:', err);
    }
  }
}

async function openFolder(path) {
  if (isTauri) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke('open_path', { path });
    } catch (err) {
      console.error('openFolder failed:', err);
    }
  }
}

export {
  isTauri,
  openFile,
  saveFile,
  openPdfFiles,
  validatePdf,
  getPdfMetadata,
  getFileSize,
  importPdf,
  deleteBook,
  getBooksDir,
  saveBookMetadata,
  listBooks,
  convertPdfToEpub,
  cancelConversion,
  onConversionProgress,
  readEpubPreview,
  openFileWithSystem,
  openFolder,
};
