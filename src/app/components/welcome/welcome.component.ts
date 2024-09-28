import { Component, inject } from '@angular/core';
import { trigger, transition, style, animate } from '@angular/animations';
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
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { MatIconModule } from '@angular/material/icon';
import { ApiKeyService } from '../../services/api-key/api-key.service';
import { BehaviorSubject, Observable, ReplaySubject } from 'rxjs';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { CommonModule } from '@angular/common';
import {
  exactLengthValidator,
  noHyphenValidator,
} from '../../common/validators';
import { MatDialog } from '@angular/material/dialog';
import { TermsOfUseDialogComponent } from '../terms-of-use-dialog/terms-of-use-dialog.component';
import { ShellService } from '../../services/shell/shell.service';

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
    MatProgressSpinnerModule,
    MatStepperModule,
    ApiKeyFormComponent,
    ApiKeyFormComponent,
  ],
  templateUrl: './welcome.component.html',
  styleUrl: './welcome.component.scss',
  animations: [
    trigger('fadeIn', [
      transition(':enter', [
        style({ opacity: 0 }),
        animate('500ms ease-in', style({ opacity: 1 })),
      ]),
    ]),
  ],
})
export class WelcomeComponent {
  protected showElement = false;
  protected active$: Observable<boolean>;
  protected isTestingConnection$: Observable<boolean>;

  private readonly isActiveSubject = new ReplaySubject<boolean>(1);
  private readonly isTestingConnectionSubject = new BehaviorSubject(false);
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

  public constructor(
    protected readonly shellService: ShellService,
    private readonly apiKeyService: ApiKeyService,
    private readonly dialog: MatDialog,
  ) {
    this.active$ = this.isActiveSubject.asObservable();
    this.isTestingConnection$ = this.isTestingConnectionSubject.asObservable();

    this.apiKeyService
      .getApiKey()
      .pipe(takeUntilDestroyed())
      .subscribe((apiKey) => {
        this.secondFormGroup.get('apiKeyCtrl')?.setValue(apiKey);
      });

    setTimeout(() => (this.showElement = true), 100);
  }

  public showTermsOfUse() {
    const dialogRef = this.dialog.open(TermsOfUseDialogComponent, {
      data: { isReadonly: false },
    });

    dialogRef.afterClosed().subscribe((result) => {
      if (result?.accept !== undefined) {
        this.firstFormGroup.get('agreement')?.setValue(result.accept);
      }
    });
  }

  public async saveApiKey(): Promise<void> {
    const apiKey = this.secondFormGroup.get('apiKeyCtrl')?.value || '';
    await this.apiKeyService.saveApiKey(apiKey);

    this.isTestingConnectionSubject.next(true);
    const testResponse = await this.apiKeyService.testConnection();
    this.isActiveSubject.next(testResponse.active);
    this.isTestingConnectionSubject.next(false);
  }

  public complete(): void {
    this.apiKeyService.closeWelcomeScreen();
  }
}
