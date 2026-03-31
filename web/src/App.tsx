import { useState, useEffect, useCallback } from 'react';
import { Plus, MessageSquare } from 'lucide-react';
import type { Conversation, Message, BroadcastEvent } from './types';
import { listConversations, createConversation } from './api';
import { useSSE } from './hooks/useSSE';
import ConversationList from './components/ConversationList';
import ChatPanel from './components/ChatPanel';
import EmptyState from './components/EmptyState';
import NewConversationDialog from './components/NewConversationDialog';

export default function App() {
  const [conversations, setConversations] = useState<Conversation[]>([]);
  const [activeId, setActiveId] = useState<string | null>(null);
  const [showNewDialog, setShowNewDialog] = useState(false);
  const [realtimeMessages, setRealtimeMessages] = useState<Record<string, Message[]>>({});

  // Load conversations on mount
  useEffect(() => {
    listConversations().then(setConversations).catch(console.error);
  }, []);

  // Handle SSE events
  const handleSSE = useCallback((event: BroadcastEvent) => {
    if (event.type === 'message') {
      const msg = event.data as Message;
      // Append to realtime messages for the conversation
      setRealtimeMessages((prev) => ({
        ...prev,
        [msg.conversationId]: [...(prev[msg.conversationId] || []), msg],
      }));
      // Update last message in conversation list
      setConversations((prev) =>
        prev.map((c) =>
          c.id === msg.conversationId
            ? { ...c, lastMessage: msg, updatedAt: msg.timestamp }
            : c
        )
      );
    } else if (event.type === 'conversation') {
      const conv = event.data as Conversation;
      setConversations((prev) => {
        const exists = prev.some((c) => c.id === conv.id);
        return exists ? prev.map((c) => (c.id === conv.id ? conv : c)) : [conv, ...prev];
      });
    } else if (event.type === 'conversation_updated') {
      const conv = event.data as Conversation;
      setConversations((prev) =>
        prev.map((c) => (c.id === conv.id ? { ...c, ...conv } : c))
      );
    }
  }, []);

  useSSE(handleSSE);

  const activeConversation = conversations.find((c) => c.id === activeId) || null;

  const handleCreate = async (data: { fromPhone: string; contactName?: string; organizationId?: string }) => {
    try {
      const conv = await createConversation(data);
      setConversations((prev) => {
        const exists = prev.some((c) => c.id === conv.id);
        return exists ? prev : [conv, ...prev];
      });
      setActiveId(conv.id);
      setShowNewDialog(false);
    } catch (err) {
      console.error('Failed to create conversation:', err);
    }
  };

  return (
    <div className="h-screen flex bg-white text-stone-800">
      {/* Sidebar */}
      <div className="w-80 border-r border-stone-200 flex flex-col shrink-0 bg-white">
        {/* Sidebar header */}
        <div className="h-14 bg-stone-100 border-b border-stone-200 flex items-center justify-between px-4 shrink-0">
          <div className="flex items-center gap-2">
            <MessageSquare size={20} className="text-emerald-600" />
            <span className="font-semibold text-stone-700 text-base">Whatsim</span>
          </div>
          <button
            onClick={() => setShowNewDialog(true)}
            className="w-8 h-8 rounded-full bg-emerald-500 text-white flex items-center justify-center hover:bg-emerald-600 transition-colors cursor-pointer"
            title="New conversation"
          >
            <Plus size={18} />
          </button>
        </div>
        {/* Conversation list */}
        <ConversationList
          conversations={conversations}
          activeId={activeId}
          onSelect={setActiveId}
        />
      </div>

      {/* Main area */}
      {activeConversation ? (
        <ChatPanel
          conversation={activeConversation}
          realtimeMessages={realtimeMessages[activeConversation.id] || []}
        />
      ) : (
        <EmptyState />
      )}

      {/* New conversation dialog */}
      <NewConversationDialog
        open={showNewDialog}
        onClose={() => setShowNewDialog(false)}
        onCreate={handleCreate}
      />
    </div>
  );
}
