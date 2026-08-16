import { api } from '../services/api';
import { useQueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';
import { useConfirm } from '../context/ConfirmContext';
import { TelegramFile } from '../types';
import { formatBytes } from '../utils';

export function useFileOperations(
    activeFolderId: number | null,
    selectedIds: number[],
    setSelectedIds: (ids: number[]) => void,
    displayedFiles: TelegramFile[],
    onRefresh?: () => void,
) {
    const queryClient = useQueryClient();
    const { confirm } = useConfirm();

    const handleDelete = async (id: number) => {
        if (!await confirm({ title: "Delete File", message: "Are you sure you want to delete this file?", confirmText: "Delete", variant: 'danger' })) return;
        try {
            await api.deleteFile(id, activeFolderId);
            if (onRefresh) onRefresh(); else queryClient.invalidateQueries({ queryKey: ['files', activeFolderId] });
            toast.success("File deleted");
        } catch (e) {
            toast.error(`Delete failed: ${e}`);
        }
    }

    const handleBulkDelete = async () => {
        if (selectedIds.length === 0) return;
        if (!await confirm({ title: "Delete Files", message: `Are you sure you want to delete ${selectedIds.length} files?`, confirmText: "Delete All", variant: 'danger' })) return;

        let success = 0;
        let fail = 0;
        for (const id of selectedIds) {
            try {
                await api.deleteFile(id, activeFolderId);
                success++;
            } catch {
                fail++;
            }
        }
        setSelectedIds([]);
        if (onRefresh) onRefresh(); else queryClient.invalidateQueries({ queryKey: ['files', activeFolderId] });
        if (success > 0) toast.success(`Deleted ${success} files.`);
        if (fail > 0) toast.error(`Failed to delete ${fail} files.`);
    }

    const handleDownload = async (id: number, name: string) => {
        try {
            toast.info(`Download started: ${name}`);
            await api.downloadFile(id, name, activeFolderId);
            toast.success(`Download complete: ${name}`);
        } catch (e) {
            toast.error(`Download failed: ${e}`);
        }
    }

    const handleBulkDownload = async () => {
        if (selectedIds.length === 0) return;
        try {
            let successCount = 0;
            const targetFiles = displayedFiles.filter((f) => selectedIds.includes(f.id));
            toast.info(`Starting batch download of ${targetFiles.length} files...`);

            for (const file of targetFiles) {
                try {
                    await api.downloadFile(file.id, file.name, activeFolderId);
                    successCount++;
                } catch { }
            }
            toast.success(`Downloaded ${successCount} files.`);
            setSelectedIds([]);
        } catch (e) {
            toast.error(`Bulk download failed: ${e}`);
        }
    }

    const handleBulkMove = async (targetFolderId: number | null, onSuccess?: () => void) => {
        if (selectedIds.length === 0) return;
        try {
            await api.moveFiles(selectedIds, activeFolderId, targetFolderId);
            toast.success(`Moved ${selectedIds.length} files.`);
            if (onRefresh) onRefresh(); else queryClient.invalidateQueries({ queryKey: ['files', activeFolderId] });
            setSelectedIds([]);
            if (onSuccess) onSuccess();
        } catch {
            toast.error('Failed to move files');
        }
    };

    const handleDownloadFolder = async () => {
        if (displayedFiles.length === 0) {
            toast.info("Folder is empty.");
            return;
        }
        try {
            let successCount = 0;
            toast.info(`Downloading folder contents (${displayedFiles.length} files)...`);
            for (const file of displayedFiles) {
                try {
                    await api.downloadFile(file.id, file.name, activeFolderId);
                    successCount++;
                } catch { }
            }
            toast.success(`Folder Download Complete: ${successCount} files.`);
        } catch (e) {
            toast.error("Error: " + e);
        }
    }

    return {
        handleDelete,
        handleBulkDelete,
        handleDownload,
        handleBulkDownload,
        handleBulkMove,
        handleDownloadFolder,
        handleGlobalSearch: async (query: string): Promise<TelegramFile[]> => {
            try {
                const results = await api.searchGlobal(query);
                return results.map(f => ({
                    ...f,
                    sizeStr: formatBytes(f.size),
                    type: f.icon_type === 'folder' ? 'folder' : 'file',
                }));
            } catch {
                return [];
            }
        }
    };
}
