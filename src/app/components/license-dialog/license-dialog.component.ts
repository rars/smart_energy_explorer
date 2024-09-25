import { Component } from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { MatDialogModule } from '@angular/material/dialog';
import { LicenseComponent } from '../license/license.component';

@Component({
  selector: 'app-license-dialog',
  standalone: true,
  imports: [LicenseComponent, MatDialogModule, MatButtonModule],
  templateUrl: './license-dialog.component.html',
  styleUrl: './license-dialog.component.scss',
})
export class LicenseDialogComponent {}
