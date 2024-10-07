import { Injectable } from '@angular/core';

import { Observable, from } from 'rxjs';

import { invoke } from '@tauri-apps/api/core';

import { ErrorService } from '../error/error.service';

@Injectable({
  providedIn: 'root',
})
export class ApiKeyService {
  public constructor(private readonly errorService: ErrorService) {}

  public getApiKey(): Observable<string> {
    return from(invoke<string>('get_api_key', {}));
  }

  public async saveApiKey(apiKey: string): Promise<void> {
    try {
      await invoke('store_api_key', { apiKey });
    } catch (error) {
      this.errorService.showError(`${error}`, 'Error storing API key');
      console.error(error);
    }
  }

  public async testConnection(): Promise<{ active: boolean }> {
    return invoke<{ active: boolean }>('test_connection', {});
  }

  public closeWelcomeScreen(): Promise<void> {
    return invoke('close_welcome_screen', {});
  }
}
