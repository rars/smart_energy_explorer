import { Injectable } from '@angular/core';

import { invoke } from '@tauri-apps/api/core';

import { ErrorService } from '../error/error.service';

@Injectable({
  providedIn: 'root',
})
export class AssistantService {
  public constructor(private readonly errorService: ErrorService) {}

  public async ask(message: string, prompt: string): Promise<string> {
    try {
      const response = await invoke<{ answer: string }>('ask_assistant', {
        message,
        prompt,
      });

      return response.answer;
    } catch (error) {
      this.errorService.showError(`${error}`, 'Error asking assistant');
      console.error(error);
      return 'Sorry, there was an error.';
    }
  }
}
