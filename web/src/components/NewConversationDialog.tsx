import { useState, useEffect, useRef } from 'react';
import { X } from 'lucide-react';

interface Props {
  open: boolean;
  onClose: () => void;
  onCreate: (data: { fromPhone: string; contactName?: string; organizationId?: string }) => void;
}

export default function NewConversationDialog({ open, onClose, onCreate }: Props) {
  const [fromPhone, setFromPhone] = useState('');
  const [contactName, setContactName] = useState('');
  const [organizationId, setOrganizationId] = useState('');
  const phoneRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (open) {
      setFromPhone('');
      setContactName('');
      setOrganizationId('');
      setTimeout(() => phoneRef.current?.focus(), 50);
    }
  }, [open]);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && open) onClose();
    };
    document.addEventListener('keydown', handler);
    return () => document.removeEventListener('keydown', handler);
  }, [open, onClose]);

  if (!open) return null;

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const phone = fromPhone.trim();
    if (!phone) return;
    onCreate({
      fromPhone: phone,
      contactName: contactName.trim() || undefined,
      organizationId: organizationId.trim() || undefined,
    });
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      {/* Backdrop */}
      <div className="absolute inset-0 bg-black/40" onClick={onClose} />
      {/* Dialog */}
      <div className="relative bg-white rounded-xl shadow-xl w-full max-w-md mx-4 p-6">
        <div className="flex items-center justify-between mb-5">
          <h2 className="text-lg font-semibold text-stone-800">New Conversation</h2>
          <button
            onClick={onClose}
            className="text-stone-400 hover:text-stone-600 cursor-pointer"
          >
            <X size={20} />
          </button>
        </div>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-stone-700 mb-1">
              Phone Number <span className="text-red-400">*</span>
            </label>
            <input
              ref={phoneRef}
              type="text"
              value={fromPhone}
              onChange={(e) => setFromPhone(e.target.value)}
              placeholder="+1234567890"
              className="w-full border border-stone-300 rounded-lg px-3 py-2 text-sm outline-none focus:border-emerald-400 focus:ring-1 focus:ring-emerald-400"
              required
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-stone-700 mb-1">
              Contact Name
            </label>
            <input
              type="text"
              value={contactName}
              onChange={(e) => setContactName(e.target.value)}
              placeholder="John Doe"
              className="w-full border border-stone-300 rounded-lg px-3 py-2 text-sm outline-none focus:border-emerald-400 focus:ring-1 focus:ring-emerald-400"
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-stone-700 mb-1">
              Organization ID
            </label>
            <input
              type="text"
              value={organizationId}
              onChange={(e) => setOrganizationId(e.target.value)}
              placeholder="org_123"
              className="w-full border border-stone-300 rounded-lg px-3 py-2 text-sm outline-none focus:border-emerald-400 focus:ring-1 focus:ring-emerald-400"
            />
          </div>
          <div className="flex justify-end gap-2 pt-2">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 text-sm text-stone-600 hover:text-stone-800 cursor-pointer"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={!fromPhone.trim()}
              className="px-4 py-2 text-sm bg-emerald-500 text-white rounded-lg hover:bg-emerald-600 disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer transition-colors"
            >
              Create
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
