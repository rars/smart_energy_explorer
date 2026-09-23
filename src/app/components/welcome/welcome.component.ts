import { animate, style, transition, trigger } from '@angular/animations';
import { Component, inject, signal, viewChild } from '@angular/core';
import { FormField, form, required } from '@angular/forms/signals';
import { MatButtonModule } from '@angular/material/button';
import { MatCheckboxModule } from '@angular/material/checkbox';
import { MatDialog } from '@angular/material/dialog';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatIconModule } from '@angular/material/icon';
import { MatInputModule } from '@angular/material/input';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { MatStepper, MatStepperModule } from '@angular/material/stepper';

import { ApiKeyService } from '../../services/api-key/api-key.service';
import { ShellService } from '../../services/shell/shell.service';
import { LicenseDialogComponent } from '../license-dialog/license-dialog.component';
import { UsageGuidanceDialogComponent } from '../usage-guidance-dialog/usage-guidance-dialog.component';

@Component({
  selector: 'app-welcome',
  imports: [
    FormField,
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
  protected readonly shellService = inject(ShellService);
  private readonly apiKeyService = inject(ApiKeyService);
  private readonly dialog = inject(MatDialog);

  public stepper = viewChild<MatStepper>('stepper');

  protected showElement = signal(false);
  protected active = signal(false);
  protected isTestingConnection = signal(false);

  protected readonly firstFormValues = signal({
    agreement: false,
  });
  protected readonly secondFormValues = signal({
    glowmarktUsername: '',
    glowmarktPassword: '',
  });

  protected readonly firstForm = form(this.firstFormValues, (tree) => {
    required(tree.agreement);
  });
  protected readonly secondForm = form(this.secondFormValues, (tree) => {
    required(tree.glowmarktUsername);
    required(tree.glowmarktPassword);
  });

  public constructor() {
    setTimeout(() => this.showElement.set(true), 100);
  }

  public showUsageGuidance(): void {
    const dialogRef = this.dialog.open(UsageGuidanceDialogComponent, {
      width: '90%',
      maxWidth: '90vw',
      maxHeight: '90vh',
      data: { isReadonly: false },
    });

    dialogRef.afterClosed().subscribe((result) => {
      if (result?.accept !== undefined) {
        this.firstFormValues.update((value) => {
          return { ...value, agreement: result.accept };
        });
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
        this.firstFormValues.update((value) => {
          return { ...value, agreement: result.accept };
        });
      }
    });
  }

  public async saveApiKey(): Promise<void> {
    this.isTestingConnection.set(true);
    this.active.set(false);

    try {
      const { glowmarktUsername, glowmarktPassword } = this.secondFormValues();

      await this.apiKeyService.saveGlowmarktCredentials(
        glowmarktUsername,
        glowmarktPassword,
      );

      const testResponse = await this.apiKeyService.testGlowmarktConnection();
      this.active.set(testResponse.active);
    } finally {
      this.isTestingConnection.set(false);
    }
  }

  public complete(): void {
    this.stepper()?.reset();
    this.apiKeyService.closeWelcomeScreen();
  }

  public reset(): void {
    this.firstFormValues.set({ agreement: false });
    this.secondFormValues.set({ glowmarktUsername: '', glowmarktPassword: '' });
    this.active.set(false);
    this.stepper()?.reset();
  }
}
