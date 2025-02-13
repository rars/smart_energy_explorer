import { animate, style, transition, trigger } from '@angular/animations';
import { CommonModule } from '@angular/common';
import { Component, ViewChild, inject } from '@angular/core';
import {
  FormBuilder,
  FormsModule,
  ReactiveFormsModule,
  Validators,
} from '@angular/forms';
import { MatButtonModule } from '@angular/material/button';
import { MatCheckboxModule } from '@angular/material/checkbox';
import { MatDialog } from '@angular/material/dialog';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatIconModule } from '@angular/material/icon';
import { MatInputModule } from '@angular/material/input';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { MatStepper, MatStepperModule } from '@angular/material/stepper';

import { BehaviorSubject, Observable, ReplaySubject } from 'rxjs';

import { ApiKeyService } from '../../services/api-key/api-key.service';
import { ShellService } from '../../services/shell/shell.service';
import { LicenseDialogComponent } from '../license-dialog/license-dialog.component';
import { UsageGuidanceDialogComponent } from '../usage-guidance-dialog/usage-guidance-dialog.component';

@Component({
  selector: 'app-welcome',
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
  @ViewChild('stepper') public stepper?: MatStepper;

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
    glowmarktUsernameCtrl: ['', [Validators.required]],
    glowmarktPasswordCtrl: ['', [Validators.required]],
  });

  public constructor(
    protected readonly shellService: ShellService,
    private readonly apiKeyService: ApiKeyService,
    private readonly dialog: MatDialog,
  ) {
    this.active$ = this.isActiveSubject.asObservable();
    this.isTestingConnection$ = this.isTestingConnectionSubject.asObservable();

    setTimeout(() => (this.showElement = true), 100);
  }

  public showUsageGuidance() {
    const dialogRef = this.dialog.open(UsageGuidanceDialogComponent, {
      width: '90%',
      maxWidth: '90vw',
      maxHeight: '90vh',
      data: { isReadonly: false },
    });

    dialogRef.afterClosed().subscribe((result) => {
      if (result?.accept !== undefined) {
        this.firstFormGroup.get('agreement')?.setValue(result.accept);
      }
    });
  }

  public showLicensing(): void {
    const dialogRef = this.dialog.open(LicenseDialogComponent, {
      width: '90%',
      maxWidth: '90vw',
      maxHeight: '90vh',
      data: { isReadonly: false },
    });

    dialogRef.afterClosed().subscribe((result) => {
      if (result?.accept !== undefined) {
        this.firstFormGroup.get('agreement')?.setValue(result.accept);
      }
    });
  }

  public async saveApiKey(): Promise<void> {
    this.isTestingConnectionSubject.next(true);

    const glowmarktUsername =
      this.secondFormGroup.get('glowmarktUsernameCtrl')?.value || '';
    const glowmarktPassword =
      this.secondFormGroup.get('glowmarktPasswordCtrl')?.value || '';

    await this.apiKeyService.saveGlowmarktCredentials(
      glowmarktUsername,
      glowmarktPassword,
    );

    const testResponse = await this.apiKeyService.testGlowmarktConnection();
    this.isActiveSubject.next(testResponse.active);

    this.isTestingConnectionSubject.next(false);
  }

  public complete(): void {
    this.stepper?.reset();
    this.apiKeyService.closeWelcomeScreen();
  }
}
