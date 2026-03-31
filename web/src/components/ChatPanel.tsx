import { useState, useEffect, useRef } from 'react';
import { Send, Bug, Phone } from 'lucide-react';
import type { Conversation, Message } from '../types';
import { listMessages, sendInboundText } from '../api';
import InspectorPanel from './InspectorPanel';

interface Props {
  conversation: Conversation;
  realtimeMessages: Message[];
}

function formatTime(dateStr: string): string {
  const d = new Date(dateStr);
  return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
}

export default function ChatPanel({ conversation, realtimeMessages }: Props) {
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState('');
  const [sending, setSending] = useState(false);
  const [showInspector, setShowInspector] = useState(false);
  const bottomRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  // Load messages when conversation changes
  useEffect(() => {
    let cancelled = false;
    listMessages(conversation.id).then((msgs) => {
      if (!cancelled) setMessages(msgs);
    });
    return () => {
      cancelled = true;
    };
  }, [conversation.id]);

  // Merge realtime messages
  useEffect(() => {
    if (realtimeMessages.length === 0) return;
    setMessages((prev) => {
      const existingIds = new Set(prev.map((m) => m.id));
      const newMsgs = realtimeMessages.filter((m) => !existingIds.has(m.id));
      return newMsgs.length > 0 ? [...prev, ...newMsgs] : prev;
    });
  }, [realtimeMessages]);

  // Auto-scroll to bottom
  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  // Focus input when conversation changes
  useEffect(() => {
    inputRef.current?.focus();
  }, [conversation.id]);

  const handleSend = async () => {
    const text = input.trim();
    if (!text || sending) return;
    setSending(true);
    setInput('');
    try {
      const msg = await sendInboundText(conversation.id, text);
      setMessages((prev) => {
        const exists = prev.some((m) => m.id === msg.id);
        return exists ? prev : [...prev, msg];
      });
    } catch (err) {
      console.error('Failed to send message:', err);
    } finally {
      setSending(false);
      inputRef.current?.focus();
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const displayName = conversation.contactName || conversation.fromPhone;

  return (
    <div className="flex-1 flex flex-col min-w-0">
      {/* Header */}
      <div className="h-14 bg-stone-100 border-b border-stone-200 flex items-center justify-between px-4 shrink-0">
        <div className="flex items-center gap-3 min-w-0">
          <div className="w-9 h-9 rounded-full bg-emerald-500 flex items-center justify-center text-white text-sm font-medium shrink-0">
            {displayName.charAt(0).toUpperCase()}
          </div>
          <div className="min-w-0">
            <div className="font-medium text-stone-800 text-sm truncate">{displayName}</div>
            <div className="flex items-center gap-1.5 text-xs text-stone-500">
              <Phone size={10} />
              <span className="truncate">{conversation.fromPhone}</span>
              {conversation.organizationId && (
                <span className="bg-emerald-100 text-emerald-700 px-1.5 py-0.5 rounded-full text-[10px]">
                  {conversation.organizationId}
                </span>
              )}
            </div>
          </div>
        </div>
        <button
          onClick={() => setShowInspector(!showInspector)}
          className={`p-2 rounded-lg transition-colors cursor-pointer ${
            showInspector
              ? 'bg-emerald-100 text-emerald-700'
              : 'text-stone-400 hover:text-stone-600 hover:bg-stone-200'
          }`}
          title="Toggle event inspector"
        >
          <Bug size={18} />
        </button>
      </div>

      <div className="flex-1 flex min-h-0">
        {/* Chat area */}
        <div className="flex-1 flex flex-col min-w-0">
          {/* Messages */}
          <div
            className="flex-1 overflow-y-auto px-4 py-3"
            style={{
              backgroundImage:
                'url("data:image/svg+xml,%3Csvg width=\'60\' height=\'60\' viewBox=\'0 0 60 60\' xmlns=\'http://www.w3.org/2000/svg\'%3E%3Cg fill=\'none\' fill-rule=\'evenodd\'%3E%3Cg fill=\'%23e7e5e4\' fill-opacity=\'0.3\'%3E%3Ccircle cx=\'30\' cy=\'30\' r=\'1\'/%3E%3C/g%3E%3C/g%3E%3C/svg%3E")',
              backgroundColor: '#f5f5f4',
            }}
          >
            {messages.length === 0 && (
              <div className="flex items-center justify-center h-full text-stone-400 text-sm">
                No messages yet. Send the first one!
              </div>
            )}
            <div className="max-w-2xl mx-auto space-y-1.5">
              {messages.map((msg) => {
                const isOutbound = msg.direction === 'outbound';
                const isSystem = msg.kind === 'system';

                if (isSystem) {
                  return (
                    <div key={msg.id} className="flex justify-center my-2">
                      <span className="bg-amber-100 text-amber-800 text-xs px-3 py-1 rounded-full">
                        {msg.text || 'System event'}
                      </span>
                    </div>
                  );
                }

                return (
                  <div
                    key={msg.id}
                    className={`flex ${isOutbound ? 'justify-end' : 'justify-start'}`}
                  >
                    <div
                      className={`max-w-[75%] px-3 py-1.5 rounded-lg text-sm shadow-sm ${
                        isOutbound
                          ? 'bg-emerald-100 text-stone-800 rounded-tr-none'
                          : 'bg-white text-stone-800 rounded-tl-none'
                      }`}
                    >
                      <p className="whitespace-pre-wrap break-words">{msg.text}</p>
                      <div
                        className={`flex items-center gap-1 mt-0.5 ${
                          isOutbound ? 'justify-end' : 'justify-start'
                        }`}
                      >
                        <span className="text-[10px] text-stone-400">
                          {formatTime(msg.timestamp)}
                        </span>
                        {msg.deliveryStatus && (
                          <span className="text-[10px] text-stone-400">
                            {msg.deliveryStatus}
                          </span>
                        )}
                      </div>
                    </div>
                  </div>
                );
              })}
              <div ref={bottomRef} />
            </div>
          </div>

          {/* Input area */}
          <div className="border-t border-stone-200 bg-stone-100 px-4 py-3 shrink-0">
            <div className="max-w-2xl mx-auto flex items-center gap-2">
              <input
                ref={inputRef}
                type="text"
                value={input}
                onChange={(e) => setInput(e.target.value)}
                onKeyDown={handleKeyDown}
                placeholder="Type a message as the user..."
                className="flex-1 bg-white border border-stone-300 rounded-full px-4 py-2 text-sm outline-none focus:border-emerald-400 focus:ring-1 focus:ring-emerald-400 placeholder:text-stone-400"
                disabled={sending}
              />
              <button
                onClick={handleSend}
                disabled={!input.trim() || sending}
                className="w-10 h-10 rounded-full bg-emerald-500 text-white flex items-center justify-center hover:bg-emerald-600 disabled:opacity-40 disabled:cursor-not-allowed transition-colors cursor-pointer shrink-0"
              >
                <Send size={18} />
              </button>
            </div>
          </div>
        </div>

        {/* Inspector panel */}
        {showInspector && (
          <InspectorPanel conversationId={conversation.id} />
        )}
      </div>
    </div>
  );
}
