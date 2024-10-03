import { CommonModule } from '@angular/common';
import { Component, OnInit } from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule, MatIconRegistry } from '@angular/material/icon';
import { NavigationEnd, Router, RouterOutlet } from '@angular/router';

import { Observable, filter, map, startWith } from 'rxjs';

import { NavigationBarComponent } from './components/navigation-bar/navigation-bar.component';
import { StatusBarComponent } from './components/status-bar/status-bar.component';
import { ThemeService } from './services/theme/theme.service';

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
