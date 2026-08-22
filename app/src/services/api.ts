import { FileMetadata, FolderMetadata, FilePage, BandwidthStats } from '../types';

function getBaseUrl(): string {
  if (typeof window !== 'undefined') {
    const port = window.location.port;
    if (port === '1420' || port === '5173' || port === '3000') {
      return 'http://localhost:8550/api';
    }
  }
  return '/api';
}

async function fetchApi<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
  const baseUrl = getBaseUrl();
  const url = `${baseUrl}${endpoint.startsWith('/') ? endpoint : `/${endpoint}`}`;

  const headers = {
    'Content-Type': 'application/json',
    ...(options.headers || {}),
  };

  const response = await fetch(url, {
    ...options,
    headers,
  });

  if (!response.ok) {
    let errorMessage = `HTTP error ${response.status}: ${response.statusText}`;
    try {
      const errorText = await response.text();
      if (errorText) {
        errorMessage = errorText;
      }
    } catch {
      // Ignore text parsing errors
    }
    throw new Error(errorMessage);
  }

  // If status is 204 No Content or body is empty
  const contentType = response.headers.get('content-type');
  if (contentType && contentType.includes('application/json')) {
    return response.json() as Promise<T>;
  }

  const text = await response.text();
  try {
    return JSON.parse(text) as T;
  } catch {
    return text as unknown as T;
  }
}

export const api = {
  // Auth & Connection
  getAuthStatus: () =>
    fetchApi<{ authenticated: boolean }>('/auth/status', {
      method: 'GET',
    }),

  connect: (apiId: number) =>
    fetchApi<boolean>('/connect', {
      method: 'POST',
      body: JSON.stringify({ api_id: apiId }),
    }),

  checkConnection: () =>
    fetchApi<boolean>('/check-connection', {
      method: 'GET',
    }),

  logout: () =>
    fetchApi<boolean>('/logout', {
      method: 'POST',
    }),

  requestCode: (phone: string, apiId: number, apiHash: string) =>
    fetchApi<string>('/auth/request-code', {
      method: 'POST',
      body: JSON.stringify({ phone, api_id: apiId, api_hash: apiHash }),
    }),

  signIn: (code: string) =>
    fetchApi<{ success: boolean; next_step?: string }>('/auth/sign-in', {
      method: 'POST',
      body: JSON.stringify({ code }),
    }),

  checkPassword: (password: string) =>
    fetchApi<{ success: boolean; next_step?: string }>('/auth/check-password', {
      method: 'POST',
      body: JSON.stringify({ password }),
    }),

  setProxy: (proxyUrl: string | null) =>
    fetchApi<boolean>('/set-proxy', {
      method: 'POST',
      body: JSON.stringify({ proxy_url: proxyUrl }),
    }),

  // Network
  isNetworkAvailable: () =>
    fetchApi<boolean>('/is-network-available', {
      method: 'GET',
    }),

  // Folders
  scanFolders: () =>
    fetchApi<FolderMetadata[]>('/folders', {
      method: 'GET',
    }),

  createFolder: (name: string) =>
    fetchApi<FolderMetadata>('/folders', {
      method: 'POST',
      body: JSON.stringify({ name }),
    }),

  deleteFolder: (folderId: number) =>
    fetchApi<boolean>('/folders/delete', {
      method: 'POST',
      body: JSON.stringify({ folder_id: folderId }),
    }),

  // Files
  getFiles: (folderId: number | null, offset: number = 0, limit: number = 50) => {
    const params = new URLSearchParams();
    if (folderId !== null && folderId !== undefined) {
      params.append('folder_id', folderId.toString());
    }
    params.append('offset', offset.toString());
    params.append('limit', limit.toString());
    return fetchApi<FilePage>(`/files?${params.toString()}`, {
      method: 'GET',
    });
  },

  uploadFile: (path: string, folderId: number | null) =>
    fetchApi<string>('/files/upload', {
      method: 'POST',
      body: JSON.stringify({ path, folder_id: folderId }),
    }),

  deleteFile: (messageId: number, folderId: number | null) =>
    fetchApi<boolean>('/files/delete', {
      method: 'POST',
      body: JSON.stringify({ message_id: messageId, folder_id: folderId }),
    }),

  downloadFile: (messageId: number, savePath: string, folderId: number | null) =>
    fetchApi<string>('/files/download', {
      method: 'POST',
      body: JSON.stringify({ message_id: messageId, save_path: savePath, folder_id: folderId }),
    }),

  moveFiles: (messageIds: number[], sourceFolderId: number | null, targetFolderId: number | null) =>
    fetchApi<boolean>('/files/move', {
      method: 'POST',
      body: JSON.stringify({
        message_ids: messageIds,
        source_folder_id: sourceFolderId,
        target_folder_id: targetFolderId,
      }),
    }),

  searchGlobal: (query: string) => {
    const params = new URLSearchParams();
    params.append('query', query);
    return fetchApi<FileMetadata[]>(`/files/search?${params.toString()}`, {
      method: 'GET',
    });
  },

  getPreview: (messageId: number, folderId: number | null) => {
    const params = new URLSearchParams();
    params.append('message_id', messageId.toString());
    if (folderId !== null && folderId !== undefined) {
      params.append('folder_id', folderId.toString());
    }
    return fetchApi<string>(`/preview?${params.toString()}`, {
      method: 'GET',
    });
  },

  getThumbnail: (messageId: number, folderId: number | null) => {
    const params = new URLSearchParams();
    params.append('message_id', messageId.toString());
    if (folderId !== null && folderId !== undefined) {
      params.append('folder_id', folderId.toString());
    }
    return fetchApi<string>(`/thumbnail?${params.toString()}`, {
      method: 'GET',
    });
  },

  cleanCache: () =>
    fetchApi<void>('/clean-cache', {
      method: 'POST',
    }),

  getBandwidth: () =>
    fetchApi<BandwidthStats>('/bandwidth', {
      method: 'GET',
    }),

  getStreamPort: () =>
    fetchApi<number>('/stream-port', {
      method: 'GET',
    }),

  log: (message: string) =>
    fetchApi<void>('/log', {
      method: 'POST',
      body: JSON.stringify({ message }),
    }),
};
