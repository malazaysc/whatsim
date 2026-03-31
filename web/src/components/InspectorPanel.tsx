import { useState, useEffect } from 'react';
import { ChevronRight } from 'lucide-react';
import type { Event } from '../types';
import { listEvents } from '../api';

interface Props {
  conversationId: string;
}

function formatTimestamp(dateStr: string): string {
  const d = new Date(dateStr);
  return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });
}

function EventTypeBadge({ type }: { type: string }) {
  const colors: Record<string, string> = {
    'message.inbound': 'bg-blue-100 text-blue-700',
    'message.outbound': 'bg-emerald-100 text-emerald-700',
    'webhook.sent': 'bg-purple-100 text-purple-700',
    'webhook.delivered': 'bg-indigo-100 text-indigo-700',
    'delivery.status': 'bg-amber-100 text-amber-700',
  };
  const cls = colors[type] || 'bg-stone-100 text-stone-600';
  return (
    <span className={`text-[10px] px-1.5 py-0.5 rounded font-mono ${cls}`}>{type}</span>
  );
}

function EventItem({ event }: { event: Event }) {
  const [expanded, setExpanded] = useState(false);
  const hasPayload = event.payload && Object.keys(event.payload).length > 0;

  return (
    <div className="border-b border-stone-100 last:border-b-0">
      <button
        onClick={() => hasPayload && setExpanded(!expanded)}
        className={`w-full text-left px-3 py-2 flex items-start gap-2 text-xs ${
          hasPayload ? 'hover:bg-stone-50 cursor-pointer' : ''
        }`}
      >
        {hasPayload && (
          <ChevronRight
            size={12}
            className={`mt-0.5 text-stone-400 transition-transform shrink-0 ${
              expanded ? 'rotate-90' : ''
            }`}
          />
        )}
        {!hasPayload && <div className="w-3 shrink-0" />}
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-0.5">
            <EventTypeBadge type={event.eventType} />
            <span className="text-stone-400 text-[10px]">{formatTimestamp(event.timestamp)}</span>
          </div>
        </div>
      </button>
      {expanded && hasPayload && (
        <div className="px-3 pb-2 pl-8">
          <pre className="text-[10px] bg-stone-50 border border-stone-200 rounded p-2 overflow-x-auto text-stone-600 leading-relaxed">
            {JSON.stringify(event.payload, null, 2)}
          </pre>
        </div>
      )}
    </div>
  );
}

export default function InspectorPanel({ conversationId }: Props) {
  const [events, setEvents] = useState<Event[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    listEvents(conversationId).then((evts) => {
      if (!cancelled) {
        setEvents(evts);
        setLoading(false);
      }
    }).catch(() => {
      if (!cancelled) setLoading(false);
    });
    return () => {
      cancelled = true;
    };
  }, [conversationId]);

  return (
    <div className="w-80 border-l border-stone-200 bg-white flex flex-col shrink-0">
      <div className="h-10 border-b border-stone-200 flex items-center px-3">
        <span className="text-xs font-medium text-stone-600">Event Inspector</span>
        <span className="ml-auto text-[10px] text-stone-400">{events.length} events</span>
      </div>
      <div className="flex-1 overflow-y-auto">
        {loading ? (
          <div className="flex items-center justify-center h-20 text-stone-400 text-xs">
            Loading...
          </div>
        ) : events.length === 0 ? (
          <div className="flex items-center justify-center h-20 text-stone-400 text-xs">
            No events yet
          </div>
        ) : (
          events.map((event) => <EventItem key={event.id} event={event} />)
        )}
      </div>
    </div>
  );
}
