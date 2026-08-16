export class WebStore {
  private prefix: string;

  constructor(filename: string = 'config.json') {
    this.prefix = `tdrive_${filename}_`;
  }

  static async load(filename: string): Promise<WebStore> {
    return new WebStore(filename);
  }

  async get<T>(key: string): Promise<T | null> {
    try {
      const item = localStorage.getItem(this.prefix + key);
      if (item === null) return null;
      return JSON.parse(item) as T;
    } catch {
      return null;
    }
  }

  async set(key: string, value: unknown): Promise<void> {
    try {
      localStorage.setItem(this.prefix + key, JSON.stringify(value));
    } catch (e) {
      console.error('LocalStorage error:', e);
    }
  }

  async delete(key: string): Promise<void> {
    localStorage.removeItem(this.prefix + key);
  }

  async save(): Promise<void> {
    // No-op for localStorage as set() persists synchronously
  }
}

export const load = WebStore.load;
export type Store = WebStore;
