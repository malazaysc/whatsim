export interface Conversation {
  id: string;
  organizationId?: string;
  contactName?: string;
  fromPhone: string;
  toPhone: string;
  createdAt: string;
  updatedAt: string;
  metadata?: Record<string, unknown>;
  lastMessage?: Message;
}

export interface Message {
  id: string;
  conversationId: string;
  direction: 'inbound' | 'outbound';
  kind: 'text' | 'system';
  text?: string;
  externalMessageId?: string;
  timestamp: string;
  rawPayloadId?: string;
  provider: 'metaSimulated' | 'mockMetaOutbound';
  deliveryStatus?: string;
  metadata?: Record<string, unknown>;
}

export interface Event {
  id: string;
  conversationId: string;
  eventType: string;
  timestamp: string;
  payload?: Record<string, unknown>;
}

export interface BroadcastEvent {
  type: 'message' | 'conversation' | 'conversation_updated';
  data: Message | Conversation;
}
