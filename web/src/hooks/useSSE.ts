import { useEffect, useRef } from 'react';
import type { BroadcastEvent } from '../types';

export function useSSE(onEvent: (event: BroadcastEvent) => void) {
  const onEventRef = useRef(onEvent);
  onEventRef.current = onEvent;

  useEffect(() => {
    const eventSource = new EventSource('/api/stream');

    const handler = (type: string) => (e: MessageEvent) => {
      try {
        const data = JSON.parse(e.data);
        onEventRef.current({ type: type as BroadcastEvent['type'], data });
      } catch {
        // ignore parse errors
      }
    };

    eventSource.addEventListener('message', handler('message'));
    eventSource.addEventListener('conversation', handler('conversation'));
    eventSource.addEventListener('conversation_updated', handler('conversation_updated'));

    eventSource.onerror = () => {
      // EventSource will auto-reconnect
    };

    return () => eventSource.close();
  }, []);
}
