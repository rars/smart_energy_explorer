import { CommonModule } from '@angular/common';
import { HttpClient } from '@angular/common/http';
import { Component, inject } from '@angular/core';

import { ShellService } from '../../services/shell/shell.service';
import { ThirdPartyLicensingComponent } from '../third-party-licensing/third-party-licensing.component';

@Component({
  selector: 'app-license',
  standalone: true,
  imports: [ThirdPartyLicensingComponent, CommonModule],
  templateUrl: './license.component.html',
  styleUrl: './license.component.scss',
})
export class LicenseComponent {
  private readonly httpClient = inject(HttpClient);

  protected readonly shellService = inject(ShellService);
  protected readonly gplaffero = this.httpClient.get<string>('gplaffero.txt', {
    responseType: 'text' as 'json',
  });
}
