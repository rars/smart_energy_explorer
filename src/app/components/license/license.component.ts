import { Component, inject } from '@angular/core';

import { ShellService } from '../../services/shell/shell.service';

@Component({
  selector: 'app-license',
  standalone: true,
  imports: [],
  templateUrl: './license.component.html',
  styleUrl: './license.component.scss',
})
export class LicenseComponent {
  protected readonly shellService = inject(ShellService);
}
