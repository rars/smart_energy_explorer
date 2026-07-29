import { inject, Service } from '@angular/core';

import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

import { ErrorService } from '../error/error.service';

type AssistantResponse = {
  promptHistory: string[];
  answerHistory: string[];
  answer: {text: string, chartConfiguration: unknown | null};
};

type AskResponse = {
  promptHistory: string[];
  answerHistory: string[];
  answer: {
    text: string;
    chartConfiguration: unknown | null;
  };
};

@Service()
export class AssistantService {
  private readonly listenerUnsubscribers: any[] = [];
  private readonly errorService = inject(ErrorService);

  public async getModels(): Promise<string[]> {
    try {
      return await invoke<string[]>('get_ollama_models');
    } catch (error) {
      this.errorService.showError(`${error}`, 'Error fetching Ollama models');
      console.error(error);
      return [];
    }
  }

  public async ask(
    message: string,
    prompt: string,
    model: string,
  ): Promise<AskResponse> {
    try {
      const response = await invoke<AssistantResponse>('ask_assistant', {
        message,
        prompt,
        model,
      });

      console.log(response.answer);

      return {
        promptHistory: response.promptHistory,
        answerHistory: response.answerHistory,
        answer: response.answer,
      };
    } catch (error) {
      this.errorService.showError(`${error}`, 'Error asking assistant');
      console.error(error);
      return {
        promptHistory: [],
        answerHistory: [],
        answer: {
          text: 'Sorry, there was an error.',
          chartConfiguration: null,
        },
      };
    }
  }

  public async addListener(
    eventName: string,
    listener: (event: unknown) => void,
  ) {
    this.listenerUnsubscribers.push(await listen(eventName, listener));
  }

  public unsubscribeAllListeners(): void {
    for (const unsubscribe of this.listenerUnsubscribers) {
      unsubscribe();
    }
  }
}
