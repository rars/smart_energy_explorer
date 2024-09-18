import { Component } from '@angular/core';
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
import {
  exactLengthValidator,
  noHyphenValidator,
} from '../../common/validators';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';

@Component({
  selector: 'app-api-key-form',
  standalone: true,
  imports: [
    ReactiveFormsModule,
    MatFormFieldModule,
    MatInputModule,
    MatButtonModule,
  ],
  templateUrl: './api-key-form.component.html',
  styleUrl: './api-key-form.component.scss',
})
export class ApiKeyFormComponent {
  public readonly apiKeyForm: FormGroup;

  public constructor(
    private readonly fb: FormBuilder,
    private readonly apiKeyService: ApiKeyService,
  ) {
    this.apiKeyForm = this.fb.group({
      apiKey: [
        '',
        [Validators.required, noHyphenValidator(), exactLengthValidator(16)],
      ],
    });

    this.apiKeyService
      .getApiKey()
      .pipe(takeUntilDestroyed())
      .subscribe((apiKey) => {
        this.apiKeyForm.get('apiKey')?.setValue(apiKey);
      });
  }

  public async onSubmit(): Promise<void> {
    if (this.apiKeyForm.valid) {
      const apiKey = this.apiKeyForm.get('apiKey')?.value;
      await this.apiKeyService.saveApiKey(apiKey);
    }
  }
}
