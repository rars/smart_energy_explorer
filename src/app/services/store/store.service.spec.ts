import { beforeEach, describe, expect, it, vi } from 'vitest';

import { TestBed } from '@angular/core/testing';

import { StoreService } from './store.service';

const mockset = vi.fn().mockResolvedValue(undefined);
const mocksave = vi.fn().mockResolvedValue(undefined);
const mockget = vi.fn().mockResolvedValue('mocked_value');

const mockConstructor = vi.fn();

vi.mock('@tauri-apps/plugin-store', () => {
  return {
    LazyStore: class {
      filename: string;
      set = mockset;
      save = mocksave;
      get = mockget;

      constructor(filename: string) {
        this.filename = filename;
        mockConstructor(filename);
      }
    },
  };
});

describe('StoreService', () => {
  let service: StoreService;

  beforeEach(() => {
    vi.clearAllMocks();

    TestBed.configureTestingModule({
      providers: [StoreService],
    });

    service = TestBed.inject(StoreService);
  });

  it('should be created and initialize LazyStore with "app_settings.bin"', () => {
    expect(service).toBeTruthy();

    expect(mockConstructor).toHaveBeenCalledWith('app_settings.bin');
  });

  describe('safe_set', () => {
    it('should call store.set and follow up immediately with store.save', async () => {
      const key = 'theme';
      const value = 'dark';

      await service.safe_set(key, value);

      expect(mockset).toHaveBeenCalledWith(key, value);
      expect(mocksave).toHaveBeenCalled();
    });
  });

  describe('get', () => {
    it('should retrieve a value from the store by its key', async () => {
      const key = 'user_id';

      mockget.mockResolvedValue(12345);

      const result = await service.get(key);

      expect(mockget).toHaveBeenCalledWith(key);
      expect(result).toBe(12345);
    });
  });
});
