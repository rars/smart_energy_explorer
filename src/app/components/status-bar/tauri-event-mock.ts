import { vi } from 'vitest';

export const listenHandlers: Record<string, Function> = {};
export const unlistenSpy = vi.fn();

// Change these to use the exact names 'listen' and 'invoke' as exports
export const listen = vi
  .fn()
  .mockImplementation((eventName: string, callback: Function) => {
    listenHandlers[eventName] = callback;
    return Promise.resolve(unlistenSpy);
  });

export const invoke = vi.fn().mockResolvedValue({ isDownloading: false });
