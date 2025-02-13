import { Injectable } from '@angular/core';

import { Observable, from } from 'rxjs';

import { invoke } from '@tauri-apps/api/core';

import { ErrorService } from '../error/error.service';

@Injectable({ providedIn: 'root' })
export class ApiKeyService {
  public constructor(private readonly errorService: ErrorService) {}

  public getGlowmarktCredentials(): Observable<{
    username: string;
    password: string;
  }> {
    return from(
      invoke<{ username: string; password: string }>(
        'get_glowmarkt_credentials',
        {},
      ),
    );
  }

  public async saveGlowmarktCredentials(
    username: string,
    password: string,
  ): Promise<void> {
    try {
      await invoke('store_glowmarkt_credentials', { username, password });
    } catch (error) {
      this.errorService.showError(
        `${error}`,
        'Error storing Glowmarkt credentials',
      );
      console.error(error);
    }
  }

  public async testGlowmarktConnection(): Promise<{ active: boolean }> {
    return invoke<{ active: boolean }>('test_glowmarkt_connection', {});
  }

  public closeWelcomeScreen(): Promise<void> {
    return invoke('close_welcome_screen', {});
  }
}
