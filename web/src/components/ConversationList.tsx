import type { Conversation } from '../types';

function timeAgo(dateStr: string): string {
  const now = Date.now();
  const then = new Date(dateStr).getTime();
  const diffSec = Math.floor((now - then) / 1000);
  if (diffSec < 60) return 'now';
  const diffMin = Math.floor(diffSec / 60);
  if (diffMin < 60) return `${diffMin}m`;
  const diffHr = Math.floor(diffMin / 60);
  if (diffHr < 24) return `${diffHr}h`;
  const diffDay = Math.floor(diffHr / 24);
  return `${diffDay}d`;
}

function truncate(str: string, max: number): string {
  return str.length > max ? str.slice(0, max) + '...' : str;
}

interface Props {
  conversations: Conversation[];
  activeId: string | null;
  onSelect: (id: string) => void;
}

export default function ConversationList({ conversations, activeId, onSelect }: Props) {
  const sorted = [...conversations].sort(
    (a, b) => new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime()
  );

  if (sorted.length === 0) {
    return (
      <div className="flex-1 flex items-center justify-center text-stone-400 text-sm px-4 text-center">
        No conversations yet. Click + to create one.
      </div>
    );
  }

  return (
    <div className="flex-1 overflow-y-auto">
      {sorted.map((conv) => {
        const isActive = conv.id === activeId;
        const displayName = conv.contactName || conv.fromPhone;
        const preview = conv.lastMessage?.text
          ? truncate(conv.lastMessage.text, 40)
          : 'No messages yet';
        const directionPrefix =
          conv.lastMessage?.direction === 'outbound' ? 'Bot: ' : '';

        return (
          <button
            key={conv.id}
            onClick={() => onSelect(conv.id)}
            className={`w-full text-left px-4 py-3 border-b border-stone-100 hover:bg-stone-50 transition-colors cursor-pointer ${
              isActive ? 'bg-emerald-50' : ''
            }`}
          >
            <div className="flex items-center justify-between mb-0.5">
              <span className="font-medium text-stone-800 text-sm truncate mr-2">
                {displayName}
              </span>
              <span className="text-xs text-stone-400 whitespace-nowrap">
                {timeAgo(conv.updatedAt)}
              </span>
            </div>
            <div className="flex items-center gap-1.5">
              <p className="text-xs text-stone-500 truncate flex-1">
                {directionPrefix}{preview}
              </p>
              {conv.organizationId && (
                <span className="text-[10px] bg-emerald-100 text-emerald-700 px-1.5 py-0.5 rounded-full whitespace-nowrap">
                  {conv.organizationId.length > 10
                    ? conv.organizationId.slice(0, 10) + '...'
                    : conv.organizationId}
                </span>
              )}
            </div>
          </button>
        );
      })}
    </div>
  );
}
