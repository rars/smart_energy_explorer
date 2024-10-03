import { Component, Inject } from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { MAT_DIALOG_DATA, MatDialogModule } from '@angular/material/dialog';

import { UsageGuidanceComponent } from '../usage-guidance/usage-guidance.component';

@Component({
  selector: 'app-usage-guidance-dialog',
  standalone: true,
  imports: [UsageGuidanceComponent, MatDialogModule, MatButtonModule],
  templateUrl: './usage-guidance-dialog.component.html',
  styleUrl: './usage-guidance-dialog.component.scss',
})
export class UsageGuidanceDialogComponent {
  public readonly isReadonly: boolean;

  public constructor(@Inject(MAT_DIALOG_DATA) data: { isReadonly: boolean }) {
    this.isReadonly = data.isReadonly;
  }
}
