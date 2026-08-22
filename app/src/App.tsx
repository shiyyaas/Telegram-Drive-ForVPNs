import { useState, useEffect } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { AuthWizard } from "./components/AuthWizard";
import { Dashboard } from "./components/Dashboard";
import { ErrorBoundary } from "./components/ErrorBoundary";
import { UpdateBanner } from "./components/UpdateBanner";
import { useUpdateCheck } from "./hooks/useUpdateCheck";
import { api } from "./services/api";
import "./App.css";

import { Toaster } from "sonner";
import { ConfirmProvider } from "./context/ConfirmContext";
import { ThemeProvider, useTheme } from "./context/ThemeContext";
import { DropZoneProvider } from "./contexts/DropZoneContext";

const queryClient = new QueryClient();

function AppContent() {
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [isCheckingAuth, setIsCheckingAuth] = useState(true);
  const { theme } = useTheme();
  const { available, version, downloading, progress, downloadAndInstall, dismissUpdate } = useUpdateCheck();

  useEffect(() => {
    let isMounted = true;
    const checkStatus = async () => {
      try {
        const res = await api.getAuthStatus();
        if (isMounted) {
          setIsAuthenticated(res.authenticated);
        }
      } catch {
        if (isMounted) {
          setIsAuthenticated(false);
        }
      } finally {
        if (isMounted) {
          setIsCheckingAuth(false);
        }
      }
    };
    checkStatus();
    return () => {
      isMounted = false;
    };
  }, []);

  if (isCheckingAuth) {
    return (
      <div className="h-screen w-screen auth-gradient flex items-center justify-center p-6 relative">
        <div className="text-center space-y-4">
          <div className="w-20 h-20 mx-auto flex items-center justify-center animate-pulse">
            <img src="/logo.svg" alt="Logo" className="w-full h-full" />
          </div>
          <div className="flex items-center justify-center gap-2">
            <div className="w-2 h-2 rounded-full bg-blue-400 animate-bounce" style={{ animationDelay: '0ms' }} />
            <div className="w-2 h-2 rounded-full bg-blue-400 animate-bounce" style={{ animationDelay: '150ms' }} />
            <div className="w-2 h-2 rounded-full bg-blue-400 animate-bounce" style={{ animationDelay: '300ms' }} />
          </div>
          <p className="text-sm text-white/60 font-medium">Checking session...</p>
        </div>
      </div>
    );
  }

  return (
    <main className="h-screen w-screen text-telegram-text overflow-hidden selection:bg-telegram-primary/30 relative">
      <UpdateBanner
        available={available}
        version={version}
        downloading={downloading}
        progress={progress}
        onUpdate={downloadAndInstall}
        onDismiss={dismissUpdate}
      />
      <Toaster theme={theme} position="bottom-center" />
      {isAuthenticated ? (
        <Dashboard onLogout={() => setIsAuthenticated(false)} />
      ) : (
        <AuthWizard onLogin={() => setIsAuthenticated(true)} />
      )}
    </main>
  );
}


function App() {
  return (
    <ErrorBoundary>
      <ThemeProvider>
        <QueryClientProvider client={queryClient}>
          <ConfirmProvider>
            <DropZoneProvider>
              <AppContent />
            </DropZoneProvider>
          </ConfirmProvider>
        </QueryClientProvider>
      </ThemeProvider>
    </ErrorBoundary>
  );
}

export default App;
