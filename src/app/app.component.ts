import { Component, OnInit } from '@angular/core';
import { NavigationEnd, Router, RouterOutlet } from '@angular/router';
import { ApiModule } from './core/modules/n3rgyapi';
import { CommonModule } from '@angular/common';
import { NavigationBarComponent } from './components/navigation-bar/navigation-bar.component';
import { MatIconRegistry } from '@angular/material/icon';

import { StatusBarComponent } from './status-bar/status-bar.component';
import { filter, map, Observable, startWith } from 'rxjs';

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
  public isWelcomeActive$: Observable<boolean>;

  public constructor(
    private readonly matIconRegistry: MatIconRegistry,
    private readonly router: Router,
  ) {
    this.matIconRegistry.setDefaultFontSetClass('material-symbols-outlined');

    this.isWelcomeActive$ = this.router.events.pipe(
      filter((event) => event instanceof NavigationEnd),
      map((event: NavigationEnd) => event.urlAfterRedirects === '/welcome'),
      startWith(true),
    );
  }

  public ngOnInit(): void {}
}
