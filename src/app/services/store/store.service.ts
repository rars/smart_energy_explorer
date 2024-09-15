import { Injectable } from '@angular/core';
import { Store } from '@tauri-apps/plugin-store';

@Injectable({
  providedIn: 'root',
})
export class StoreService {
  private readonly store: Store;

  public constructor() {
    this.store = new Store('app_settings.bin');
  }

  public set(key: string, value: unknown): Promise<void> {
    return this.store.set(key, value);
  }

  public get(key: string): Promise<unknown> {
    return this.store.get(key);
  }
}
