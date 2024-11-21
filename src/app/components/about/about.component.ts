import { Component } from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { MatDialog } from '@angular/material/dialog';

import { invoke } from '@tauri-apps/api/core';

import { ShellService } from '../../services/shell/shell.service';
import { LicenseDialogComponent } from '../license-dialog/license-dialog.component';
import { UsageGuidanceDialogComponent } from '../usage-guidance-dialog/usage-guidance-dialog.component';

@Component({
    selector: 'app-about',
    imports: [MatButtonModule],
    templateUrl: './about.component.html',
    styleUrl: './about.component.scss'
})
export class AboutComponent {
  protected version: string = '';

  public constructor(
    protected readonly shellService: ShellService,
    private readonly dialog: MatDialog,
  ) {
    invoke<string>('get_app_version', {}).then((version) => {
      this.version = version;
    });
  }

  public showUsageGuidance(): void {
    this.dialog.open(UsageGuidanceDialogComponent, {
      width: '90%',
      maxWidth: '90vw',
      maxHeight: '90vh',
      data: { isReadonly: true },
    });
  }

  public showLicensing(): void {
    this.dialog.open(LicenseDialogComponent, {
      width: '90%',
      maxWidth: '90vw',
      maxHeight: '90vh',
      data: {
        isReadonly: true,
      },
    });
  }
}
