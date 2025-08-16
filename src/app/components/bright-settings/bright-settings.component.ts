import { Component } from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { RouterLink } from '@angular/router';

import { BrightCredentialsFormComponent } from '../bright-credentials-form/bright-credentials-form.component';

@Component({
  selector: 'app-bright-settings',
  imports: [
    BrightCredentialsFormComponent,
    MatButtonModule,
    MatIconModule,
    RouterLink,
  ],
  templateUrl: './bright-settings.component.html',
  styleUrl: './bright-settings.component.scss',
})
export class BrightSettingsComponent {}
