import { vi } from 'vitest';

export const mockStoreSet = vi.fn().mockResolvedValue(undefined);
export const mockStoreSave = vi.fn().mockResolvedValue(undefined);
export const mockStoreGet = vi.fn().mockResolvedValue(undefined);
export const mockStoreConstructor = vi.fn();

export class LazyStore {
  public readonly filename: string;
  public readonly set = mockStoreSet;
  public readonly save = mockStoreSave;
  public readonly get = mockStoreGet;

  public constructor(filename: string) {
    this.filename = filename;
    mockStoreConstructor(filename);
  }
}

export const resetStoreMocks = () => {
  mockStoreSet.mockClear();
  mockStoreSave.mockClear();
  mockStoreGet.mockClear();
  mockStoreGet.mockResolvedValue(undefined);
  mockStoreConstructor.mockClear();
};
