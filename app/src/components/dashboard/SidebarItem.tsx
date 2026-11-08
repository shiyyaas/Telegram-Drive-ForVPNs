import { useState, memo } from 'react';
import { Plus } from 'lucide-react';

interface SidebarItemProps {
    icon: React.ElementType;
    label: string;
    active: boolean;
    onClick: (folderId: number | null) => void;
    onDrop: (e: React.DragEvent, folderId: number | null) => void;
    onDelete?: (id: number, name: string) => void;
    folderId: number | null;
}

/**
 * SidebarItem - Pure DOM event-based drop handling
 * 
 * Wrapped in React.memo to prevent unnecessary sidebar item re-renders during periodic bandwidth polling and selection updates.
 * Callbacks receive folderId and label to allow parents to pass stable callback references.
 */
export const SidebarItem = memo(function SidebarItem({ icon: Icon, label, active = false, onClick, onDrop, onDelete, folderId }: SidebarItemProps) {
    const [isOver, setIsOver] = useState(false);

    return (
        <div className="relative group">
        <button
            onClick={() => onClick(folderId)}
            onDragEnter={(e) => {
                e.preventDefault();
                e.stopPropagation();
                setIsOver(true);
            }}
            onDragOver={(e) => {
                e.preventDefault();
                e.stopPropagation();
                e.dataTransfer.dropEffect = 'move';
            }}
            onDragLeave={(e) => {
                e.preventDefault();
                e.stopPropagation();
                // Only clear if truly leaving (not entering a child element)
                const rect = e.currentTarget.getBoundingClientRect();
                const x = e.clientX;
                const y = e.clientY;
                if (x < rect.left || x > rect.right || y < rect.top || y > rect.bottom) {
                    setIsOver(false);
                }
            }}
            onDrop={(e) => {
                e.preventDefault();
                e.stopPropagation();
                setIsOver(false);
                if (onDrop) onDrop(e, folderId);
            }}
            onContextMenu={(e) => {
                if (onDelete && folderId !== null) {
                    e.preventDefault();
                    onDelete(folderId, label);
                }
            }}
            className={`w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium transition-all duration-150 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-telegram-primary ${active
                ? 'bg-telegram-primary/10 text-telegram-primary'
                : isOver
                    ? 'bg-telegram-primary/30 text-telegram-text ring-2 ring-telegram-primary scale-[1.02] shadow-lg'
                    : 'text-telegram-subtext hover:bg-telegram-hover hover:text-telegram-text'
                } ${onDelete ? 'pr-8' : ''}`}
        >
            <Icon className={`w-4 h-4 ${isOver ? 'text-telegram-primary' : ''}`} />
            <span className="flex-1 text-left truncate">{label}</span>
        </button>
        {onDelete && folderId !== null && (
            <button
                type="button"
                onClick={(e) => { e.stopPropagation(); onDelete(folderId, label); }}
                aria-label={`Delete folder ${label}`}
                title={`Delete folder ${label}`}
                className="absolute right-2 top-1/2 -translate-y-1/2 opacity-0 group-hover:opacity-100 group-focus-within:opacity-100 p-1 rounded hover:text-red-400 text-telegram-subtext hover:bg-white/5 transition-opacity focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-red-400"
            >
                <Plus className="w-3 h-3 rotate-45" />
            </button>
        )}
        </div>
    )
});
