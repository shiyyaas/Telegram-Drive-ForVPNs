import { useState, useEffect, useRef } from 'react';
import { api } from '../services/api';

/**
 * Network detection optimized for high-latency VPN connections
 * 
 * Uses api.isNetworkAvailable which checks Telegram DCs.
 * Adaptive polling: 30s when online, 45s when offline to reduce VPN traffic.
 */
export function useNetworkStatus() {
    const [isOnline, setIsOnline] = useState(true);
    const isOnlineRef = useRef(true);

    useEffect(() => {
        const checkNetwork = async () => {
            try {
                const available = await api.isNetworkAvailable();
                setIsOnline(available);
                isOnlineRef.current = available;
            } catch {
                setIsOnline(false);
                isOnlineRef.current = false;
            }
        };

        // Initial check
        checkNetwork();

        // Adaptive polling: faster when online, slower when offline
        const getInterval = () => isOnlineRef.current ? 30000 : 45000;

        let timeoutId: ReturnType<typeof setTimeout>;
        const scheduleNext = () => {
            timeoutId = setTimeout(() => {
                checkNetwork().then(scheduleNext);
            }, getInterval());
        };
        scheduleNext();

        return () => clearTimeout(timeoutId);
    }, []);

    return isOnline;
}
