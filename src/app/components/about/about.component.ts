import { Component } from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { invoke } from '@tauri-apps/api/core';
import { TermsOfUseDialogComponent } from '../terms-of-use-dialog/terms-of-use-dialog.component';
import { MatDialog } from '@angular/material/dialog';
import { LicenseComponent } from '../license/license.component';
import { LicenseDialogComponent } from '../license-dialog/license-dialog.component';

@Component({
  selector: 'app-about',
  standalone: true,
  imports: [MatButtonModule],
  templateUrl: './about.component.html',
  styleUrl: './about.component.scss',
})
export class AboutComponent {
  protected version: string = '';

  public constructor(private readonly dialog: MatDialog) {
    invoke<string>('get_app_version', {}).then((version) => {
      this.version = version;
    });
  }

  public showTermsOfUse(): void {
    this.dialog.open(TermsOfUseDialogComponent, { data: { isReadonly: true } });
  }

  public showLicensing(): void {
    this.dialog.open(LicenseDialogComponent);
  }
}
