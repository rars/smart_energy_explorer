import { Component, signal } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { FormField, FormRoot, form, required } from '@angular/forms/signals';
import { MatButtonModule } from '@angular/material/button';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';

import { ApiKeyService } from '../../services/api-key/api-key.service';

interface GlowmarktCredentials {
  username: string;
  password: string;
}

@Component({
  selector: 'app-bright-credentials-form',
  imports: [
    FormRoot,
    FormField,
    MatFormFieldModule,
    MatInputModule,
    MatButtonModule,
  ],
  templateUrl: './bright-credentials-form.component.html',
  styleUrl: './bright-credentials-form.component.scss',
})
export class BrightCredentialsFormComponent {
  protected readonly glowmarktCredentials = signal<GlowmarktCredentials>({
    username: '',
    password: '',
  });
  protected readonly form = form(
    this.glowmarktCredentials,
    (path) => {
      required(path.username, { message: 'Username is required' });
      required(path.password, { message: 'Password is required' });
    },
    {
      submission: {
        action: async (f) => {
          const currentFormValues = f().value();
          try {
            if (await this.submit()) {
              f().reset(currentFormValues);
            }
            return;
          } catch (err) {
            console.error(err);
            return {
              kind: 'runtimeError',
              message: `Failed to submit form: ${err}`,
            };
          }
        },
      },
    },
  );

  public constructor(private readonly apiKeyService: ApiKeyService) {
    this.apiKeyService
      .getGlowmarktCredentials()
      .pipe(takeUntilDestroyed())
      .subscribe(({ username, password }) => {
        this.glowmarktCredentials.set({ username, password });
      });
  }

  public async submit(): Promise<boolean> {
    const { username, password } = this.form().value();

    return await this.apiKeyService.saveGlowmarktCredentials(
      username,
      password,
    );
  }
}
