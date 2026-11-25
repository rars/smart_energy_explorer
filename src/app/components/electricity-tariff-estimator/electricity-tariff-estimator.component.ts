import { CommonModule } from '@angular/common';
import {
  ChangeDetectionStrategy,
  Component,
  inject,
  signal,
} from '@angular/core';
import { FormField, form } from '@angular/forms/signals';
import { MatButtonModule } from '@angular/material/button';
import { MatDatepickerModule } from '@angular/material/datepicker';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatIconModule } from '@angular/material/icon';
import { MatInputModule } from '@angular/material/input';
import { MatSelectModule } from '@angular/material/select';

import { take } from 'rxjs';

import { invoke } from '@tauri-apps/api/core';

import { DateService } from '../../services/date/date.service';
import { ErrorService } from '../../services/error/error.service';
import { FormControlService } from '../../services/form-control/form-control.service';

interface DateRange {
  startDate: Date;
  endDate: Date;
}

interface TariffRate {
  startMinutes: number;
  endMinutes: number;
  unitPricePence: number;
  dayOfWeek?: number;
}

interface TariffEstimate {
  totalConsumptionKwh: number;
  totalCostPence: number;
  averageUnitPricePence: number;
}

const timeToMinutes = (time: string): number => {
  const [hours, minutes] = time.split(':').map(Number);
  return hours * 60 + minutes;
};

const minutesToTime = (minutes: number): string =>
  `${Math.floor(minutes / 60)
    .toString()
    .padStart(2, '0')}:${(minutes % 60).toString().padStart(2, '0')}`;

@Component({
  selector: 'app-electricity-tariff-estimator',
  imports: [
    CommonModule,
    FormField,
    MatButtonModule,
    MatDatepickerModule,
    MatFormFieldModule,
    MatIconModule,
    MatInputModule,
    MatSelectModule,
  ],
  templateUrl: './electricity-tariff-estimator.component.html',
  styleUrl: './electricity-tariff-estimator.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ElectricityTariffEstimatorComponent {
  private readonly dateService = inject(DateService);
  private readonly errorService = inject(ErrorService);
  private readonly formControlService = inject(FormControlService);

  protected readonly dates = signal<DateRange>({
    startDate: this.dateService.addDays(this.dateService.startOfToday(), -30),
    endDate: this.dateService.startOfToday(),
  });
  protected readonly dateForm = form(this.dates);
  protected readonly fallbackUnitPricePence = signal(33.68);
  protected readonly rates = signal<TariffRate[]>([
    {
      startMinutes: timeToMinutes('12:00'),
      endMinutes: timeToMinutes('15:00'),
      unitPricePence: 15,
    },
    {
      startMinutes: timeToMinutes('19:30'),
      endMinutes: timeToMinutes('22:30'),
      unitPricePence: 15,
    },
    {
      startMinutes: timeToMinutes('03:00'),
      endMinutes: timeToMinutes('06:00'),
      unitPricePence: 15,
    },
    {
      startMinutes: timeToMinutes('12:00'),
      endMinutes: timeToMinutes('14:00'),
      unitPricePence: 0,
      dayOfWeek: 0,
    },
  ]);
  protected readonly result = signal<TariffEstimate | undefined>(undefined);
  protected readonly isCalculating = signal(false);

  public constructor() {
    this.formControlService
      .getDateRange()
      .pipe(take(1))
      .subscribe(([startDate, endDate]) => {
        this.dates.set({ startDate, endDate });
      });
  }

  protected timeFor(minutes: number): string {
    return minutesToTime(minutes);
  }

  protected updateRate(
    index: number,
    field: keyof TariffRate,
    value: string,
  ): void {
    this.rates.update((rates) =>
      rates.map((rate, rateIndex) => {
        if (rateIndex !== index) return rate;
        if (field === 'dayOfWeek') {
          return {
            ...rate,
            dayOfWeek: value === '' ? undefined : Number(value),
          };
        }
        return {
          ...rate,
          [field]:
            field === 'startMinutes' || field === 'endMinutes'
              ? timeToMinutes(value)
              : Number(value),
        };
      }),
    );
  }

  protected addRate(): void {
    this.rates.update((rates) => [
      ...rates,
      {
        startMinutes: timeToMinutes('00:00'),
        endMinutes: timeToMinutes('01:00'),
        unitPricePence: 0,
      },
    ]);
  }

  protected removeRate(index: number): void {
    this.rates.update((rates) =>
      rates.filter((_, rateIndex) => rateIndex !== index),
    );
  }

  protected setFallbackPrice(value: string): void {
    this.fallbackUnitPricePence.set(Number(value));
  }

  protected async calculate(): Promise<void> {
    const { startDate, endDate } = this.dates();
    if (!startDate || !endDate) return;

    this.isCalculating.set(true);
    try {
      const result = await invoke<TariffEstimate>(
        'estimate_electricity_tariff_cost',
        {
          startDate: this.dateService.formatISODate(startDate),
          endDate: this.dateService.formatISODate(
            this.dateService.addDays(endDate, 1),
          ),
          tariff: {
            defaultUnitPricePence: this.fallbackUnitPricePence(),
            rates: this.rates(),
          },
        },
      );
      this.result.set(result);
      this.formControlService.setDateRange(startDate, endDate);
    } catch (error) {
      this.errorService.showError(`Failed to estimate tariff cost: ${error}`);
    } finally {
      this.isCalculating.set(false);
    }
  }
}
