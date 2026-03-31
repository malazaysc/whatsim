import type { Conversation, Message, Event } from './types';

const BASE = '';

export async function listConversations(organizationId?: string): Promise<Conversation[]> {
  const params = organizationId ? `?organizationId=${organizationId}` : '';
  const res = await fetch(`${BASE}/api/conversations${params}`);
  return res.json();
}

export async function getConversation(id: string): Promise<Conversation> {
  const res = await fetch(`${BASE}/api/conversations/${id}`);
  return res.json();
}

export async function createConversation(data: { fromPhone: string; contactName?: string; organizationId?: string }): Promise<Conversation> {
  const res = await fetch(`${BASE}/api/conversations`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
  return res.json();
}

export async function listMessages(conversationId: string): Promise<Message[]> {
  const res = await fetch(`${BASE}/api/conversations/${conversationId}/messages`);
  return res.json();
}

export async function sendInboundText(conversationId: string, text: string): Promise<Message> {
  const res = await fetch(`${BASE}/api/messages/inbound-text`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ conversationId, text }),
  });
  return res.json();
}

export async function listEvents(conversationId: string): Promise<Event[]> {
  const res = await fetch(`${BASE}/api/conversations/${conversationId}/events`);
  return res.json();
}
