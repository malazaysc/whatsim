import { MessageSquare } from 'lucide-react';

export default function EmptyState() {
  return (
    <div className="flex-1 flex flex-col items-center justify-center bg-stone-50 text-stone-400">
      <MessageSquare size={64} strokeWidth={1} className="mb-4 text-stone-300" />
      <h2 className="text-xl font-light text-stone-500 mb-1">Whatsim</h2>
      <p className="text-sm">Select a conversation or create a new one to get started.</p>
    </div>
  );
}
