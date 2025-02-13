import { Component } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import {
  FormBuilder,
  FormGroup,
  ReactiveFormsModule,
  Validators,
} from '@angular/forms';
import { MatButtonModule } from '@angular/material/button';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';

import { ApiKeyService } from '../../services/api-key/api-key.service';

@Component({
  selector: 'app-bright-credentials-form',
  imports: [
    ReactiveFormsModule,
    MatFormFieldModule,
    MatInputModule,
    MatButtonModule,
  ],
  templateUrl: './bright-credentials-form.component.html',
  styleUrl: './bright-credentials-form.component.scss',
})
export class BrightCredentialsFormComponent {
  public readonly glowmarktCredentialsForm: FormGroup;

  public constructor(
    private readonly fb: FormBuilder,
    private readonly apiKeyService: ApiKeyService,
  ) {
    this.glowmarktCredentialsForm = this.fb.group({
      glowmarktUsernameCtrl: ['', [Validators.required]],
      glowmarktPasswordCtrl: ['', [Validators.required]],
    });

    this.apiKeyService
      .getGlowmarktCredentials()
      .pipe(takeUntilDestroyed())
      .subscribe(({ username, password }) => {
        this.glowmarktCredentialsForm
          .get('glowmarktUsernameCtrl')
          ?.setValue(username);
        this.glowmarktCredentialsForm
          .get('glowmarktPasswordCtrl')
          ?.setValue(password);
      });
  }

  public async onSubmit(): Promise<void> {
    if (this.glowmarktCredentialsForm.valid) {
      const glowmarktUsername = this.glowmarktCredentialsForm.get(
        'glowmarktUsernameCtrl',
      )?.value;
      const glowmarktPassword = this.glowmarktCredentialsForm.get(
        'glowmarktPasswordCtrl',
      )?.value;
      await this.apiKeyService.saveGlowmarktCredentials(
        glowmarktUsername,
        glowmarktPassword,
      );
    }
  }
}
