import { createContext, useContext, useState, useEffect, useCallback, ReactNode } from 'react';

interface ConfirmOptions {
    title: string;
    message: string;
    confirmText?: string;
    cancelText?: string;
    variant?: 'danger' | 'info';
}

interface ConfirmContextType {
    confirm: (options: ConfirmOptions) => Promise<boolean>;
}

const ConfirmContext = createContext<ConfirmContextType | undefined>(undefined);

export function ConfirmProvider({ children }: { children: ReactNode }) {
    const [isOpen, setIsOpen] = useState(false);
    const [options, setOptions] = useState<ConfirmOptions>({ title: '', message: '' });
    const [resolveRef, setResolveRef] = useState<((value: boolean) => void) | null>(null);

    const confirm = (opts: ConfirmOptions) => {
        setOptions(opts);
        setIsOpen(true);
        return new Promise<boolean>((resolve) => {
            setResolveRef(() => resolve);
        });
    };

    const handleConfirm = useCallback(() => {
        setIsOpen(false);
        if (resolveRef) resolveRef(true);
    }, [resolveRef]);

    const handleCancel = useCallback(() => {
        setIsOpen(false);
        if (resolveRef) resolveRef(false);
    }, [resolveRef]);

    useEffect(() => {
        if (!isOpen) return;

        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                e.preventDefault();
                e.stopPropagation();
                handleCancel();
            }
        };

        window.addEventListener('keydown', handleKeyDown, true);
        return () => window.removeEventListener('keydown', handleKeyDown, true);
    }, [isOpen, handleCancel]);

    const isDanger = options.variant === 'danger';

    return (
        <ConfirmContext.Provider value={{ confirm }}>
            {children}
            {isOpen && (
                <div
                    className="fixed inset-0 z-[200] flex items-center justify-center bg-black/50 backdrop-blur-sm"
                    onClick={handleCancel}
                >
                    <div
                        role="alertdialog"
                        aria-modal="true"
                        aria-labelledby="confirm-dialog-title"
                        aria-describedby="confirm-dialog-message"
                        className="bg-[#1c1c1c] border border-white/10 rounded-xl p-6 w-96 shadow-2xl animate-in zoom-in-95"
                        onClick={e => e.stopPropagation()}
                    >
                        <h3 id="confirm-dialog-title" className="text-lg font-medium text-white mb-2">
                            {options.title}
                        </h3>
                        <p id="confirm-dialog-message" className="text-telegram-subtext text-sm mb-6 whitespace-pre-line">
                            {options.message}
                        </p>
                        <div className="flex justify-end gap-3">
                            <button
                                autoFocus={isDanger}
                                onClick={handleCancel}
                                className="px-4 py-2 rounded-lg text-sm font-medium hover:bg-white/5 text-telegram-subtext transition focus-visible:ring-2 focus-visible:ring-telegram-primary focus-visible:outline-none"
                            >
                                {options.cancelText || 'Cancel'}
                            </button>
                            <button
                                autoFocus={!isDanger}
                                onClick={handleConfirm}
                                className={`px-4 py-2 rounded-lg text-sm font-medium transition focus-visible:ring-2 focus-visible:outline-none ${isDanger ? 'bg-red-500/10 text-red-400 hover:bg-red-500/20 focus-visible:ring-red-400' : 'bg-telegram-primary text-white hover:bg-telegram-primary/90 focus-visible:ring-telegram-primary'}`}
                            >
                                {options.confirmText || 'Confirm'}
                            </button>
                        </div>
                    </div>
                </div>
            )}
        </ConfirmContext.Provider>
    );
}

export const useConfirm = () => {
    const context = useContext(ConfirmContext);
    if (!context) throw new Error('useConfirm must be used within a ConfirmProvider');
    return context;
};
