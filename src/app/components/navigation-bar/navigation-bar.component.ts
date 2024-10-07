import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatMenuModule } from '@angular/material/menu';
import { MatToolbarModule } from '@angular/material/toolbar';
import { RouterLink } from '@angular/router';

import { Observable } from 'rxjs';

import { ThemeService } from '../../services/theme/theme.service';

@Component({
  selector: 'app-navigation-bar',
  standalone: true,
  imports: [
    RouterLink,
    CommonModule,
    MatToolbarModule,
    MatButtonModule,
    MatMenuModule,
    MatIconModule,
  ],
  templateUrl: './navigation-bar.component.html',
  styleUrl: './navigation-bar.component.scss',
})
export class NavigationBarComponent {
  protected isLightMode$: Observable<boolean>;

  public constructor(private readonly themeService: ThemeService) {
    this.isLightMode$ = this.themeService.isLightMode();
  }

  public toggleTheme(): void {
    this.themeService.toggleTheme();
  }
}
