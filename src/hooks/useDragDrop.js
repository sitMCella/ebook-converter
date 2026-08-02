import { useEffect, useRef } from 'react';
import { isTauri } from '../lib/tauri';

export function useDragDrop(onDrop) {
  const onDropRef = useRef(onDrop);
  onDropRef.current = onDrop;

  useEffect(() => {
    if (!isTauri) return;

    let unlisten;

    (async () => {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      unlisten = await getCurrentWindow().onDragDropEvent((event) => {
        if (event.payload.type === 'drop' && event.payload.paths) {
          const pdfPaths = event.payload.paths.filter((p) =>
            p.toLowerCase().endsWith('.pdf')
          );
          if (pdfPaths.length > 0) {
            onDropRef.current(pdfPaths);
          }
        }
      });
    })();

    return () => {
      if (unlisten) unlisten();
    };
  }, []);
}
