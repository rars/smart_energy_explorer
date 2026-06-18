import { beforeEach, describe, expect, it, vi } from 'vitest';

import { TestBed } from '@angular/core/testing';

import {
  mockLazyStore,
  mockStoreConstructor,
  mockStoreGet,
  mockStoreSave,
  mockStoreSet,
  resetStoreMocks,
} from '../../../testing/tauri-plugin-store.mock';
import { StoreService } from './store.service';

vi.mock('@tauri-apps/plugin-store', () => ({
  LazyStore: mockLazyStore,
}));

describe('StoreService', () => {
  let service: StoreService;

  beforeEach(() => {
    resetStoreMocks();

    TestBed.configureTestingModule({
      providers: [StoreService],
    });

    service = TestBed.inject(StoreService);
  });

  it('should be created and initialize LazyStore with "app_settings.bin"', () => {
    expect(service).toBeTruthy();

    expect(mockStoreConstructor).toHaveBeenCalledWith('app_settings.bin');
  });

  describe('safe_set', () => {
    it('should call store.set and follow up immediately with store.save', async () => {
      const key = 'theme';
      const value = 'dark';

      await service.safe_set(key, value);

      expect(mockStoreSet).toHaveBeenCalledWith(key, value);
      expect(mockStoreSave).toHaveBeenCalled();
    });
  });

  describe('get', () => {
    it('should retrieve a value from the store by its key', async () => {
      const key = 'user_id';

      mockStoreGet.mockResolvedValue(12345);

      const result = await service.get(key);

      expect(mockStoreGet).toHaveBeenCalledWith(key);
      expect(result).toBe(12345);
    });
  });
});
