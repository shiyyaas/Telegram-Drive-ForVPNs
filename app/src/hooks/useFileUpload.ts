import { useState, useEffect } from 'react';
import { api } from '../services/api';
import { useQueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';
import { QueueItem } from '../types';
import { useFileDrop } from './useFileDrop';
import type { Store } from '../utils/store';

export function useFileUpload(activeFolderId: number | null, store: Store | null, onRefresh?: () => void) {
    const queryClient = useQueryClient();
    const [uploadQueue, setUploadQueue] = useState<QueueItem[]>([]);
    const [processing, setProcessing] = useState(false);
    const [initialized, setInitialized] = useState(false);

    useEffect(() => {
        if (!store || initialized) return;
        store.get<QueueItem[]>('uploadQueue').then((saved) => {
            if (saved && saved.length > 0) {
                const pending = saved.filter(i => i.status === 'pending');
                if (pending.length > 0) {
                    setUploadQueue(pending);
                    toast.info(`Restored ${pending.length} pending uploads`);
                }
            }
            setInitialized(true);
        });
    }, [store, initialized]);

    useEffect(() => {
        if (!store || !initialized) return;
        const pending = uploadQueue.filter(i => i.status === 'pending');
        store.set('uploadQueue', pending).then(() => store.save());
    }, [store, uploadQueue, initialized]);

    useEffect(() => {
        if (processing) return;
        const nextItem = uploadQueue.find(i => i.status === 'pending');
        if (nextItem) {
            processItem(nextItem);
        }
    }, [uploadQueue, processing]);

    const processItem = async (item: QueueItem) => {
        setProcessing(true);
        setUploadQueue(q => q.map(i => i.id === item.id ? { ...i, status: 'uploading' } : i));
        try {
            await api.uploadFile(item.path, item.folderId);
            setUploadQueue(q => q.map(i => i.id === item.id ? { ...i, status: 'success' } : i));
            if (onRefresh) onRefresh(); else queryClient.invalidateQueries({ queryKey: ['files', item.folderId] });
        } catch (e) {
            setUploadQueue(q => q.map(i => i.id === item.id ? { ...i, status: 'error', error: String(e) } : i));
            toast.error(`Upload failed for ${item.path.split('/').pop()}: ${e}`);
        } finally {
            setProcessing(false);
        }
    };

    // Opens browser file picker for upload
    const handleManualUpload = async () => {
        try {
            const input = document.createElement('input');
            input.type = 'file';
            input.multiple = true;
            input.onchange = (e: Event) => {
                const target = e.target as HTMLInputElement;
                if (target.files && target.files.length > 0) {
                    const files = Array.from(target.files);
                    const newItems: QueueItem[] = files.map((file) => ({
                        id: Math.random().toString(36).substring(2, 9),
                        path: (file as any).path || file.name,
                        folderId: activeFolderId,
                        status: 'pending'
                    }));
                    setUploadQueue(prev => [...prev, ...newItems]);
                    toast.info(`Queued ${files.length} file(s) for upload`);
                }
            };
            input.click();
        } catch {
            toast.error("Failed to open file dialog");
        }
    };

    const { isDragging } = useFileDrop();

    return {
        uploadQueue,
        setUploadQueue,
        handleManualUpload,
        isDragging
    };
}
