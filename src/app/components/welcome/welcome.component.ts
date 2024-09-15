import { Component } from '@angular/core';
import { ApiKeyFormComponent } from '../api-key-form/api-key-form.component';

@Component({
  selector: 'app-welcome',
  standalone: true,
  imports: [ApiKeyFormComponent, ApiKeyFormComponent],
  templateUrl: './welcome.component.html',
  styleUrl: './welcome.component.scss',
})
export class WelcomeComponent {}
