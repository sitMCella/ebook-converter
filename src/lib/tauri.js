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

export { isTauri, openFile, saveFile, openPdfFiles, validatePdf, getPdfMetadata, getFileSize };
