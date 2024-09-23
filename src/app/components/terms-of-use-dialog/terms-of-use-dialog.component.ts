import { Component } from '@angular/core';
import { MatDialogModule, MatDialogRef } from '@angular/material/dialog';
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
  public constructor(
    private dialogRef: MatDialogRef<TermsOfUseDialogComponent>,
  ) {}
}
