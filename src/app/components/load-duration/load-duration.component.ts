import { CurrencyPipe, DecimalPipe } from '@angular/common';
import { Component, computed, signal } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { FormControl, ReactiveFormsModule } from '@angular/forms';
import { MatButtonModule } from '@angular/material/button';
import { MatCardModule } from '@angular/material/card';
import { MatDatepickerModule } from '@angular/material/datepicker';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatIconModule } from '@angular/material/icon';
import { MatProgressBarModule } from '@angular/material/progress-bar';
import { MatSelectModule } from '@angular/material/select';
import { MatSliderModule } from '@angular/material/slider';

import {
  Observable,
  catchError,
  combineLatest,
  filter,
  forkJoin,
  from,
  map,
  of,
  startWith,
  switchMap,
} from 'rxjs';

import { invoke } from '@tauri-apps/api/core';

import { DateService } from '../../services/date/date.service';
import { ChartComponent } from '../chart/chart.component';

const nonNullOrUndefined = <T>(x: T | null | undefined): x is T => !!x;

const getValueStream = <T>(x: FormControl<T | null>) =>
  x.valueChanges.pipe(startWith(x.value), filter(nonNullOrUndefined));

@Component({
  selector: 'app-load-duration',
  imports: [
    CurrencyPipe,
    DecimalPipe,
    ReactiveFormsModule,
    MatButtonModule,
    MatCardModule,
    MatFormFieldModule,
    MatDatepickerModule,
    MatProgressBarModule,
    MatSelectModule,
    MatSliderModule,
    MatIconModule,
    ChartComponent,
  ],
  templateUrl: './load-duration.component.html',
  styleUrl: './load-duration.component.scss',
})
export class LoadDurationComponent {
  public readonly startDateControl: FormControl<Date | null>;
  public readonly endDateControl: FormControl<Date | null>;
  public readonly values = signal<any[] | undefined>(undefined);
  public readonly chartConfiguration = signal<any>(undefined);
  public readonly baseLoadEstimate = signal<number | undefined>(undefined);
  public readonly annualBaseloadConsumptionEstimate = computed(() => {
    const baseLoad = this.baseLoadEstimate();
    if (baseLoad) {
      return (baseLoad * 24 * 365) / 1000;
    }
    return undefined;
  });
  public readonly annualBaseloadCostEstimate = computed(() => {
    const annualBaseLoadConsumption = this.annualBaseloadConsumptionEstimate();
    if (annualBaseLoadConsumption) {
      return annualBaseLoadConsumption * 0.25;
    }
    return undefined;
  });
  public readonly loading = signal(false);
  public readonly minPower = computed(() =>
    this.values()?.reduce((minSoFar, x) => {
      minSoFar === undefined
        ? x.powerConsumptionW
        : x.powerConsumptionW < minSoFar
          ? x.powerConsumptionW
          : minSoFar;
    }, undefined),
  );
  public readonly maxPower = computed(() =>
    this.values()?.reduce((minSoFar, x) => {
      minSoFar === undefined
        ? x.powerConsumptionW
        : x.powerConsumptionW < minSoFar
          ? x.powerConsumptionW
          : minSoFar;
    }, undefined),
  );

  public constructor(private readonly dateService: DateService) {
    this.startDateControl = new FormControl<Date>(
      this.dateService.addDays(this.dateService.startOfToday(), -7),
    );
    this.endDateControl = new FormControl<Date>(
      this.dateService.startOfToday(),
    );

    combineLatest([
      getValueStream(this.startDateControl),
      getValueStream(this.endDateControl),
    ])
      .pipe(
        map(([startDate, endDate]) => [
          this.dateService.formatISODate(startDate),
          this.dateService.formatISODate(this.dateService.addDays(endDate, 1)),
        ]),
        switchMap(([startDate, endDate]) => {
          this.loading.set(true);

          let data: Observable<
            { timestamp: string; energyConsumptionKwh: number }[]
          > = of([]);

          data = from(
            invoke<{ timestamp: string; value: number }[]>(
              'get_raw_electricity_consumption',
              { startDate, endDate },
            ),
          ).pipe(
            map((x) =>
              x.map(({ timestamp, value }) => ({
                timestamp,
                energyConsumptionKwh: value / 1000.0,
              })),
            ),
          );

          return forkJoin([data]);
        }),
        takeUntilDestroyed(),
        catchError((err) => {
          window.alert(`Error: ${err}`);
          return of([undefined, undefined]);
        }),
      )
      .subscribe(([values]) => {
        this.loading.set(false);

        if (values === undefined) {
          return;
        }

        values.sort((a, b) => b.energyConsumptionKwh - a.energyConsumptionKwh);

        const N = values.length;
        const newValues = values.map((v, i) => ({
          percentile: ((i + 1.0) / N) * 100.0,
          energyConsumptionKwh: v.energyConsumptionKwh,
          powerConsumptionW: v.energyConsumptionKwh * 2 * 1000,
        }));

        this.values.set(newValues);

        const baseLoad = newValues.find(
          (v) => v.percentile > 95,
        )?.powerConsumptionW;

        this.baseLoadEstimate.set(baseLoad);

        this.chartConfiguration.set({
          type: 'line',
          data: {
            datasets: [
              {
                label: 'Electricity',
                data: newValues.map((x) => ({
                  x: x.percentile,
                  y: x.powerConsumptionW,
                })),
              },
            ],
          },
          options: {
            responsive: true,
            maintainAspectRatio: false,
            plugins: {
              subtitle: {
                display: true,
                text: 'Based on 30-min intervals. Instantaneous peaks may be significantly higher.',
                font: {
                  size: 11,
                  style: 'italic',
                },
                padding: {
                  bottom: 10,
                },
              },
              tooltip: {
                callbacks: {
                  title: (context: any) => {
                    const percentile = context[0].parsed.x;
                    return `${percentile.toFixed(1)}% of the selected period`;
                  },
                  label: (context: any) => {
                    const formattedPower = context.parsed.y.toLocaleString(
                      undefined,
                      {
                        minimumFractionDigits: 0,
                        maximumFractionDigits: 0,
                      },
                    );
                    return `30-min average power is at least ${formattedPower}W`;
                  },
                  footer: (_context: any) => {
                    return 'Calculated from 30-min average power smart meter data';
                  },
                },
              },
            },
            scales: {
              x: {
                type: 'linear',
                title: {
                  display: true,
                  text: 'Percentage of Period (%)',
                },
              },
              y: {
                title: {
                  display: true,
                  text: '30-Min Average Power (W)',
                },
              },
            },
          },
        });
      });
  }
}
