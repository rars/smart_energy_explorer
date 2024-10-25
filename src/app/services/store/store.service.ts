import { Injectable } from '@angular/core';

import { LazyStore } from '@tauri-apps/plugin-store';

@Injectable({
  providedIn: 'root',
})
export class StoreService {
  private readonly store: LazyStore;

  public constructor() {
    this.store = new LazyStore('app_settings.bin');
  }

  public async safe_set(key: string, value: unknown): Promise<void> {
    await this.store.set(key, value);

    await this.store.save();
  }

  public async get(key: string): Promise<unknown> {
    return await this.store.get(key);
  }
}
