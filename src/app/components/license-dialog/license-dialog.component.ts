import { Component, Inject } from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { MAT_DIALOG_DATA, MatDialogModule } from '@angular/material/dialog';

import { LicenseComponent } from '../license/license.component';

@Component({
  selector: 'app-license-dialog',
  standalone: true,
  imports: [LicenseComponent, MatDialogModule, MatButtonModule],
  templateUrl: './license-dialog.component.html',
  styleUrl: './license-dialog.component.scss',
})
export class LicenseDialogComponent {
  public readonly isReadonly: boolean;

  public constructor(@Inject(MAT_DIALOG_DATA) data: { isReadonly: boolean }) {
    this.isReadonly = data.isReadonly;
  }
}
