import { CommonModule } from '@angular/common';
import { HttpClient } from '@angular/common/http';
import { Component, OnInit, inject } from '@angular/core';

import { ShellService } from '../../services/shell/shell.service';

@Component({
  selector: 'app-third-party-licensing',
  imports: [CommonModule],
  templateUrl: './third-party-licensing.component.html',
  styleUrl: './third-party-licensing.component.scss',
})
export class ThirdPartyLicensingComponent implements OnInit {
  private httpClient = inject(HttpClient);

  protected shellService = inject(ShellService);
  protected thirdPartyLicensesText = this.httpClient.get<string>(
    '3rdpartylicensetext.txt',
    { responseType: 'text' as 'json' },
  );

  public ngOnInit() {}
}
