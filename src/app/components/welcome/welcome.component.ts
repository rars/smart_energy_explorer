import { Component, inject } from '@angular/core';
import { ApiKeyFormComponent } from '../api-key-form/api-key-form.component';
import { MatStepperModule } from '@angular/material/stepper';
import {
  FormBuilder,
  FormsModule,
  ReactiveFormsModule,
  Validators,
} from '@angular/forms';
import { MatCheckboxModule } from '@angular/material/checkbox';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { ApiKeyService } from '../../services/api-key/api-key.service';
import { EMPTY, map, Observable } from 'rxjs';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { CommonModule } from '@angular/common';
import {
  exactLengthValidator,
  noHyphenValidator,
} from '../../common/validators';

@Component({
  selector: 'app-welcome',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    ReactiveFormsModule,
    MatFormFieldModule,
    MatCheckboxModule,
    MatIconModule,
    MatInputModule,
    MatButtonModule,
    MatStepperModule,
    ApiKeyFormComponent,
    ApiKeyFormComponent,
  ],
  templateUrl: './welcome.component.html',
  styleUrl: './welcome.component.scss',
})
export class WelcomeComponent {
  protected active$: Observable<boolean>;

  private readonly formBuilder = inject(FormBuilder);

  protected readonly firstFormGroup = this.formBuilder.group({
    agreement: [false, Validators.requiredTrue],
  });
  protected readonly secondFormGroup = this.formBuilder.group({
    apiKeyCtrl: [
      '',
      [Validators.required, noHyphenValidator(), exactLengthValidator(16)],
    ],
  });

  public constructor(private readonly apiKeyService: ApiKeyService) {
    this.apiKeyService
      .getApiKey()
      .pipe(takeUntilDestroyed())
      .subscribe((apiKey) => {
        this.secondFormGroup.get('apiKeyCtrl')?.setValue(apiKey);
      });

    this.active$ = EMPTY;
  }

  public async saveApiKey(): Promise<void> {
    const apiKey = this.secondFormGroup.get('apiKeyCtrl')?.value || '';
    await this.apiKeyService.saveApiKey(apiKey);
    this.active$ = this.apiKeyService.testConnection().pipe(
      map((status) => {
        return status.active;
      }),
    );
  }

  public complete(): void {
    this.apiKeyService.closeWelcomeScreen();
  }
}
