import { Component, Inject, Input } from '@angular/core';
import { MAT_DIALOG_DATA, MatDialogModule } from '@angular/material/dialog';
import { TermsOfUseComponent } from '../terms-of-use/terms-of-use.component';
import { MatButtonModule } from '@angular/material/button';

@Component({
  selector: 'app-terms-of-use-dialog',
  standalone: true,
  imports: [TermsOfUseComponent, MatDialogModule, MatButtonModule],
  templateUrl: './terms-of-use-dialog.component.html',
  styleUrl: './terms-of-use-dialog.component.scss',
})
export class TermsOfUseDialogComponent {
  public readonly isReadonly: boolean;

  public constructor(
    @Inject(MAT_DIALOG_DATA) private data: { isReadonly: boolean },
  ) {
    this.isReadonly = data.isReadonly;
  }
}
