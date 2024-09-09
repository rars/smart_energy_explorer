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
import { invoke } from '@tauri-apps/api/core';
import { ErrorService } from '../../services/error/error.service';

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
    private readonly errorService: ErrorService,
  ) {
    this.apiKeyForm = this.fb.group({
      apiKey: ['', Validators.required],
    });
  }

  public async onSubmit(): Promise<void> {
    if (this.apiKeyForm.valid) {
      const apiKey = this.apiKeyForm.get('apiKey')?.value;
      try {
        await invoke('store_api_key', { apiKey });
      } catch (error) {
        this.errorService.showError(`${error}`, 'Error storing API key');
        console.error(error);
      }
    }
  }
}
