import { Injectable } from '@angular/core';

import { Store, createStore } from '@tauri-apps/plugin-store';

@Injectable({
  providedIn: 'root',
})
export class StoreService {
  private readonly storePromise: Promise<Store>;

  public constructor() {
    this.storePromise = createStore('app_settings.bin');
  }

  public async safe_set(key: string, value: unknown): Promise<void> {
    const store = await this.storePromise;

    await store.set(key, value);

    await store.save();
  }

  public async get(key: string): Promise<unknown> {
    const store = await this.storePromise;

    return await store.get(key);
  }
}
