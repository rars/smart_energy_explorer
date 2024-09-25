import { Component, OnInit } from '@angular/core';
import { NavigationEnd, Router, RouterOutlet } from '@angular/router';
import { CommonModule } from '@angular/common';
import { NavigationBarComponent } from './components/navigation-bar/navigation-bar.component';
import { MatIconModule, MatIconRegistry } from '@angular/material/icon';

import { StatusBarComponent } from './status-bar/status-bar.component';
import { filter, map, Observable, startWith } from 'rxjs';
import { ThemeService } from './services/theme/theme.service';
import { MatButtonModule } from '@angular/material/button';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [
    RouterOutlet,
    CommonModule,
    NavigationBarComponent,
    StatusBarComponent,
    MatIconModule,
    MatButtonModule,
  ],
  templateUrl: './app.component.html',
  styleUrl: './app.component.scss',
})
export class AppComponent implements OnInit {
  public isWelcomeActive$: Observable<boolean>;
  public isLightMode$: Observable<boolean>;

  public constructor(
    private readonly matIconRegistry: MatIconRegistry,
    private readonly router: Router,
    private readonly themeService: ThemeService,
  ) {
    this.matIconRegistry.setDefaultFontSetClass('material-symbols-outlined');
    this.isLightMode$ = this.themeService.isLightMode();
    this.isWelcomeActive$ = this.router.events.pipe(
      filter((event) => event instanceof NavigationEnd),
      map((event: NavigationEnd) => event.urlAfterRedirects === '/welcome'),
      startWith(true),
    );
  }

  public ngOnInit(): void {}

  public async toggleTheme(): Promise<void> {
    await this.themeService.toggleTheme();
  }
}
