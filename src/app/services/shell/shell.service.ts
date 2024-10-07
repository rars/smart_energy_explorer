import { Injectable } from '@angular/core';

import { open } from '@tauri-apps/plugin-shell';

import { ErrorService } from '../error/error.service';

@Injectable({
  providedIn: 'root',
})
export class ShellService {
  public constructor(private readonly errorService: ErrorService) {}

  public async openLink(url: string): Promise<void> {
    try {
      await open(url);
    } catch (e) {
      this.errorService.showError(`Failed to open link: ${e}`);
    }
  }
}
