import { CommonModule } from '@angular/common';
import { Component, OnDestroy } from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatMenuModule } from '@angular/material/menu';
import { MatToolbarModule } from '@angular/material/toolbar';
import { RouterLink } from '@angular/router';

import { Observable, ReplaySubject } from 'rxjs';

import { UnlistenFn } from '@tauri-apps/api/event';

import { ThemeService } from '../../services/theme/theme.service';

@Component({
  selector: 'app-navigation-bar',
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
export class NavigationBarComponent implements OnDestroy {
  protected electricityPower$: Observable<string>;
  protected cumulativeDay$: Observable<string>;
  protected payload: any;

  protected isLightMode$: Observable<boolean>;

  private unlistenFn?: UnlistenFn;

  public constructor(private readonly themeService: ThemeService) {
    this.isLightMode$ = this.themeService.isLightMode();

    const electricityPower = new ReplaySubject<string>();
    const cumulativeDay = new ReplaySubject<string>();

    this.electricityPower$ = electricityPower.asObservable();
    this.cumulativeDay$ = cumulativeDay.asObservable();

    /*listen<any>('electricityUpdate', (message) => {
      const energy = message.payload.electricitymeter.energy;
      const dayMessage = `${energy.import.day} ${energy.import.units}`;

      const power = message.payload.electricitymeter.power;
      const powerMessage = `${power.value} ${power.units}`;
      electricityPower.next(powerMessage);

      cumulativeDay.next(dayMessage);
      this.payload = message.payload;
    }).then((unlisten) => {
      this.unlistenFn = unlisten;
    });*/
  }

  public ngOnDestroy(): void {
    this.unlistenFn?.();
  }

  public toggleTheme(): void {
    this.themeService.toggleTheme();
  }
}
