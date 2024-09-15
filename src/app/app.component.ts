import { Component, OnInit } from '@angular/core';
import { RouterOutlet } from '@angular/router';
import { ApiModule } from './core/modules/n3rgyapi';
import { CommonModule } from '@angular/common';
import { NavigationBarComponent } from './components/navigation-bar/navigation-bar.component';
import { MatIconRegistry } from '@angular/material/icon';

import { StatusBarComponent } from './status-bar/status-bar.component';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [
    RouterOutlet,
    ApiModule,
    CommonModule,
    NavigationBarComponent,
    StatusBarComponent,
  ],
  templateUrl: './app.component.html',
  styleUrl: './app.component.scss',
})
export class AppComponent implements OnInit {
  public constructor(private readonly matIconRegistry: MatIconRegistry) {
    this.matIconRegistry.setDefaultFontSetClass('material-symbols-outlined');
  }

  public ngOnInit(): void {}
}
