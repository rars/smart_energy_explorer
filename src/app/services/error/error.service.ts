import { Injectable } from '@angular/core';
import { MatSnackBar } from '@angular/material/snack-bar';

@Injectable({
  providedIn: 'root',
})
export class ErrorService {
  public constructor(private snackBar: MatSnackBar) {}

  public showError(
    message: string,
    action: string = 'Close',
    duration: number = 5000,
  ): void {
    this.snackBar.open(message, action, {
      duration: duration,
      panelClass: ['error-snackbar'],
    });
  }
}
