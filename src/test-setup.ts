import { enGB } from 'date-fns/locale';
import { beforeEach, vi } from 'vitest';

import { provideHttpClient } from '@angular/common/http';
import { provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import { MAT_DATE_LOCALE } from '@angular/material/core';
import { MAT_DIALOG_DATA, MatDialogRef } from '@angular/material/dialog';
import { provideDateFnsAdapter } from '@angular/material-date-fns-adapter';
import { provideNoopAnimations } from '@angular/platform-browser/animations';
import { provideRouter } from '@angular/router';

const tauriMocks = vi.hoisted(() => {
  const invoke = vi.fn((command: string) => {
    switch (command) {
      case 'get_app_status':
        return Promise.resolve({ isDownloading: false });
      case 'get_app_version':
        return Promise.resolve('0.0.0-test');
      case 'get_glowmarkt_credentials':
        return Promise.resolve({ username: '', password: '' });
      case 'test_glowmarkt_connection':
        return Promise.resolve({ active: false });
      case 'get_electricity_tariff_history':
      case 'get_gas_tariff_history':
        return Promise.resolve({ standingCharges: [], unitPrices: [] });
      default:
        return Promise.resolve([]);
    }
  });

  const listenHandlers: Record<string, Function> = {};
  const unlisten = vi.fn();
  const listen = vi.fn((eventName: string, callback: Function) => {
    listenHandlers[eventName] = callback;
    return Promise.resolve(unlisten);
  });

  return { invoke, listen, listenHandlers, unlisten };
});

vi.mock('@tauri-apps/api/core', () => ({
  invoke: tauriMocks.invoke,
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: tauriMocks.listen,
}));

vi.mock('@tauri-apps/plugin-dialog', () => ({
  confirm: vi.fn().mockResolvedValue(false),
  save: vi.fn().mockResolvedValue(null),
}));

vi.mock('@tauri-apps/plugin-fs', () => ({
  writeFile: vi.fn().mockResolvedValue(undefined),
}));

vi.mock('@tauri-apps/plugin-store', () =>
  import('./testing/tauri-plugin-store.mock'),
);

vi.mock('@tauri-apps/plugin-shell', () => ({
  open: vi.fn().mockResolvedValue(undefined),
}));

Object.defineProperty(window, 'alert', {
  configurable: true,
  value: vi.fn(),
});

Object.defineProperty(window, 'matchMedia', {
  configurable: true,
  value: vi.fn().mockImplementation((query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    addListener: vi.fn(),
    removeListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
});

Object.defineProperty(HTMLCanvasElement.prototype, 'getContext', {
  configurable: true,
  value: vi.fn(() => null),
});

beforeEach(() => {
  TestBed.configureTestingModule({
    providers: [
      provideRouter([]),
      provideHttpClient(),
      provideHttpClientTesting(),
      provideNoopAnimations(),
      { provide: MAT_DATE_LOCALE, useValue: enGB },
      provideDateFnsAdapter(),
      { provide: MAT_DIALOG_DATA, useValue: { isReadonly: false } },
      { provide: MatDialogRef, useValue: { close: vi.fn() } },
    ],
  });
});
