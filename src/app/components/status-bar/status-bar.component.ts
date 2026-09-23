import { CommonModule } from '@angular/common';
import {
  ChangeDetectionStrategy,
  Component,
  OnDestroy,
  OnInit,
  inject,
  signal,
} from '@angular/core';
import { MatIconModule } from '@angular/material/icon';

import { from } from 'rxjs';

import { invoke } from '@tauri-apps/api/core';
import { UnlistenFn, listen } from '@tauri-apps/api/event';

import { DateService } from '../../services/date/date.service';
import { DataDownloadingComponent } from '../data-downloading/data-downloading.component';
import {
  ElectricityMeterMessage,
  GasMeterMessage,
} from './status-bar-messages';

@Component({
  selector: 'app-status-bar',
  imports: [CommonModule, MatIconModule, DataDownloadingComponent],
  templateUrl: './status-bar.component.html',
  styleUrl: './status-bar.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class StatusBarComponent implements OnInit, OnDestroy {
  private readonly dateService = inject(DateService);

  public readonly isDownloading = signal(false);
  protected readonly electricityUpdateReceived = signal(false);
  protected readonly electricityPower = signal<string | undefined>(undefined);
  protected readonly cumulativeElectricityDay = signal<string | undefined>(
    undefined,
  );
  protected readonly gasUpdateReceived = signal(false);
  protected readonly cumulativeGasDay = signal<string | undefined>(undefined);

  private unlistenFn?: UnlistenFn;
  private electricityUpdateUnlistenFn?: UnlistenFn;
  private gasUpdateUnlistenFn?: UnlistenFn;
  private clearCurrentUse?: ReturnType<typeof setTimeout>;
  private clearCurrentGasUse?: ReturnType<typeof setTimeout>;
  private isDestroyed = false;

  public ngOnInit(): void {
    listen<ElectricityMeterMessage>('electricityUpdate', (message) => {
      if (this.clearCurrentUse !== undefined) {
        clearTimeout(this.clearCurrentUse);
        this.clearCurrentUse = undefined;
      }

      this.electricityUpdateReceived.set(true);

      const lastUpdated = new Date(message.payload.electricitymeter.timestamp);
      const friendlyTimestamp = this.dateService.format(
        lastUpdated,
        "dd/MM/yyyy 'at' h:mm:ss a",
      );

      const energy = message.payload.electricitymeter.energy;
      const dayMessage = `${energy.import.day} ${energy.import.units} used today (last updated ${friendlyTimestamp})`;

      const power = message.payload.electricitymeter.power;
      const powerMessage = `${power.value} ${power.units}`;

      this.electricityPower.set(powerMessage);
      this.cumulativeElectricityDay.set(dayMessage);

      this.clearCurrentUse = setTimeout(
        () => this.electricityUpdateReceived.set(false),
        30000,
      );
    }).then((unlisten) => {
      if (!this.isDestroyed) {
        this.electricityUpdateUnlistenFn = unlisten;
      } else {
        unlisten();
      }
    });

    listen<GasMeterMessage>('gasUpdate', (message) => {
      if (this.clearCurrentGasUse !== undefined) {
        clearTimeout(this.clearCurrentGasUse);
        this.clearCurrentGasUse = undefined;
      }

      this.gasUpdateReceived.set(true);

      const lastUpdated = new Date(message.payload.gasmeter.timestamp);
      const friendlyTimestamp = this.dateService.format(
        lastUpdated,
        "dd/MM/yyyy 'at' h:mm:ss a",
      );

      const energy = message.payload.gasmeter.energy;
      const dayMessage = `${energy.import.day} ${energy.import.units} used today (last updated ${friendlyTimestamp})`;

      this.cumulativeGasDay.set(dayMessage);

      this.clearCurrentGasUse = setTimeout(
        () => this.gasUpdateReceived.set(false),
        30000,
      );
    }).then((unlisten) => {
      if (!this.isDestroyed) {
        this.gasUpdateUnlistenFn = unlisten;
      } else {
        unlisten();
      }
    });

    listen<{ isDownloading: boolean }>('appStatusUpdate', (event) => {
      // event.event is the event name (useful if you want to use a single callback fn for multiple event types)
      // event.payload is the payload object
      this.isDownloading.set(event.payload.isDownloading);
    }).then((unlisten) => {
      if (!this.isDestroyed) {
        this.unlistenFn = unlisten;
      } else {
        unlisten();
      }
    });

    from(invoke<{ isDownloading: boolean }>('get_app_status', {})).subscribe(
      (status) => {
        this.isDownloading.set(status.isDownloading);
      },
    );
  }

  public ngOnDestroy(): void {
    this.isDestroyed = true;
    this.unlistenFn?.();
    this.electricityUpdateUnlistenFn?.();
    this.gasUpdateUnlistenFn?.();

    if (this.clearCurrentUse !== undefined) {
      clearTimeout(this.clearCurrentUse);
      this.clearCurrentUse = undefined;
    }

    if (this.clearCurrentGasUse !== undefined) {
      clearTimeout(this.clearCurrentGasUse);
      this.clearCurrentGasUse = undefined;
    }
  }
}
