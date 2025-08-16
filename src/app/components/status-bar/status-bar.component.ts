import { format } from 'date-fns';

import { CommonModule } from '@angular/common';
import { Component, OnDestroy, OnInit } from '@angular/core';
import { MatIconModule } from '@angular/material/icon';

import { BehaviorSubject, Observable, ReplaySubject, from } from 'rxjs';

import { invoke } from '@tauri-apps/api/core';
import { UnlistenFn, listen } from '@tauri-apps/api/event';

import { DataDownloadingComponent } from '../data-downloading/data-downloading.component';

@Component({
  selector: 'app-status-bar',
  imports: [
    CommonModule,
    MatIconModule,
    DataDownloadingComponent,
    DataDownloadingComponent,
  ],
  templateUrl: './status-bar.component.html',
  styleUrl: './status-bar.component.scss',
})
export class StatusBarComponent implements OnInit, OnDestroy {
  public isDownloading$ = new BehaviorSubject(false);

  protected electricityPower$: Observable<string>;
  protected cumulativeDay$: Observable<string>;
  protected electricityUpdateReceived$: Observable<boolean>;

  private unlistenFn?: UnlistenFn;
  private electricityUpdateUnlistenFn?: UnlistenFn;
  private clearCurrentUse?: any;

  private readonly electricityPowerSubject = new ReplaySubject<string>();
  private readonly cumulativeDaySubject = new ReplaySubject<string>();
  private readonly electricityUpdateReceivedSubject =
    new ReplaySubject<boolean>();

  public constructor() {
    this.electricityPower$ = this.electricityPowerSubject.asObservable();
    this.cumulativeDay$ = this.cumulativeDaySubject.asObservable();
    this.electricityUpdateReceived$ =
      this.electricityUpdateReceivedSubject.asObservable();
  }

  public ngOnInit(): void {
    listen<any>('electricityUpdate', (message) => {
      if (this.clearCurrentUse !== undefined) {
        clearTimeout(this.clearCurrentUse);
        this.clearCurrentUse = undefined;
      }

      this.electricityUpdateReceivedSubject.next(true);

      const lastUpdated = new Date(message.payload.electricitymeter.timestamp);
      const friendlyTimestamp = format(
        lastUpdated,
        "dd/MM/yyyy 'at' h:mm:ss a",
      );

      const energy = message.payload.electricitymeter.energy;
      const dayMessage = `${energy.import.day} ${energy.import.units} used today (last updated ${friendlyTimestamp})`;

      const power = message.payload.electricitymeter.power;
      const powerMessage = `${power.value} ${power.units}`;

      this.electricityPowerSubject.next(powerMessage);
      this.cumulativeDaySubject.next(dayMessage);

      if (this.clearCurrentUse !== undefined) {
        clearTimeout(this.clearCurrentUse);
        this.clearCurrentUse = undefined;
      }

      this.clearCurrentUse = setTimeout(
        () => this.electricityUpdateReceivedSubject.next(false),
        30000,
      );
    }).then((unlisten) => {
      this.electricityUpdateUnlistenFn = unlisten;
    });

    listen<{ isDownloading: boolean }>('appStatusUpdate', (event) => {
      // event.event is the event name (useful if you want to use a single callback fn for multiple event types)
      // event.payload is the payload object
      this.isDownloading$.next(event.payload.isDownloading);
    }).then((unlisten) => {
      this.unlistenFn = unlisten;
    });

    from(invoke<{ isDownloading: boolean }>('get_app_status', {})).subscribe(
      (status) => {
        this.isDownloading$.next(status.isDownloading);
      },
    );
  }

  public ngOnDestroy(): void {
    this.unlistenFn?.();
    this.isDownloading$.complete();

    this.electricityUpdateUnlistenFn?.();
    this.electricityPowerSubject.complete();
    this.cumulativeDaySubject.complete();
  }
}
