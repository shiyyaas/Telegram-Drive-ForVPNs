import { useState, useCallback } from 'react';

interface UpdateState {
    checking: boolean;
    available: boolean;
    downloading: boolean;
    progress: number;
    error: string | null;
    version: string | null;
}

export function useUpdateCheck() {
    const [state, setState] = useState<UpdateState>({
        checking: false,
        available: false,
        downloading: false,
        progress: 0,
        error: null,
        version: null,
    });

    const checkForUpdates = useCallback(async () => {
        setState(s => ({ ...s, checking: true, error: null }));
        // In web application, updates are served automatically via web server
        setState(s => ({ ...s, checking: false, available: false }));
    }, []);

    const downloadAndInstall = useCallback(async () => {
        // No-op for web
    }, []);

    const dismissUpdate = useCallback(() => {
        setState(s => ({ ...s, available: false }));
    }, []);

    return {
        ...state,
        checkForUpdates,
        downloadAndInstall,
        dismissUpdate,
    };
}
