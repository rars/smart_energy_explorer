import { Component, OnDestroy, OnInit } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { FormControl, ReactiveFormsModule } from '@angular/forms';
import { MatButtonModule } from '@angular/material/button';
import { MatDatepickerModule } from '@angular/material/datepicker';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatProgressBarModule } from '@angular/material/progress-bar';
import { MatSelectModule } from '@angular/material/select';

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
  take,
} from 'rxjs';

// When using the Tauri API npm package:
import { invoke } from '@tauri-apps/api/core';

import { DateService } from '../../services/date/date.service';
import { FormControlService } from '../../services/form-control/form-control.service';
import { ChartComponent } from '../chart/chart.component';

type Aggregation = 'raw' | 'daily' | 'monthly';

const nonNullOrUndefined = <T>(x: T | null | undefined): x is T => !!x;

const getValueStream = <T>(x: FormControl<T | null>) =>
  x.valueChanges.pipe(startWith(x.value), filter(nonNullOrUndefined));

@Component({
  selector: 'app-electricity-consumption-line-chart',
  standalone: true,
  imports: [
    ReactiveFormsModule,
    MatButtonModule,
    MatFormFieldModule,
    MatDatepickerModule,
    MatProgressBarModule,
    MatSelectModule,
    ChartComponent,
  ],
  templateUrl: './electricity-consumption-line-chart.component.html',
  styleUrl: './electricity-consumption-line-chart.component.scss',
})
export class ElectricityConsumptionLineChartComponent
  implements OnInit, OnDestroy
{
  public readonly startDateControl: FormControl<Date | null>;
  public readonly endDateControl: FormControl<Date | null>;
  public readonly aggregationControl = new FormControl<Aggregation>('raw');

  public values?: any[];
  public chartConfiguration: any;
  public loading = false;

  public constructor(
    private readonly dateService: DateService,
    private readonly formControlService: FormControlService,
  ) {
    this.startDateControl = new FormControl<Date>(
      this.dateService.addDays(this.dateService.startOfToday(), -7),
    );
    this.endDateControl = new FormControl<Date>(
      this.dateService.startOfToday(),
    );

    this.formControlService
      .getDateRange()
      .pipe(take(1))
      .subscribe(([startDate, endDate]) => {
        this.startDateControl.setValue(startDate);
        this.endDateControl.setValue(endDate);
      });

    combineLatest([
      getValueStream(this.startDateControl),
      getValueStream(this.endDateControl),
      getValueStream(this.aggregationControl),
    ])
      .pipe(
        map(([startDate, endDate, aggregation]) => [
          this.dateService.formatISODate(startDate),
          this.dateService.formatISODate(this.dateService.addDays(endDate, 1)),
          aggregation,
        ]),
        switchMap(([startDate, endDate, aggregation]) => {
          this.loading = true;

          let data: Observable<
            { timestamp: string; energyConsumptionKwh: number }[]
          > = of([]);

          switch (aggregation) {
            case 'daily': {
              data = from(
                invoke<{ timestamp: string; value: number }[]>(
                  'get_daily_electricity_consumption',
                  { startDate, endDate },
                ),
              ).pipe(
                map((x) =>
                  x.map(({ timestamp, value }) => ({
                    timestamp,
                    energyConsumptionKwh: value,
                  })),
                ),
              );

              break;
            }
            case 'monthly': {
              data = from(
                invoke<{ timestamp: string; value: number }[]>(
                  'get_monthly_electricity_consumption',
                  { startDate, endDate },
                ),
              ).pipe(
                map((x) =>
                  x.map(({ timestamp, value }) => ({
                    timestamp,
                    energyConsumptionKwh: value,
                  })),
                ),
              );
              break;
            }
            case 'raw':
            default: {
              data = from(
                invoke<{ timestamp: string; value: number }[]>(
                  'get_raw_electricity_consumption',
                  { startDate, endDate },
                ),
              ).pipe(
                map((x) =>
                  x.map(({ timestamp, value }) => ({
                    timestamp,
                    energyConsumptionKwh: value,
                  })),
                ),
              );
            }
          }

          return forkJoin([data, of(aggregation)]);
        }),
        takeUntilDestroyed(),
        catchError((err) => {
          window.alert(`Error: ${err}`);
          return of([undefined, undefined]);
        }),
      )
      .subscribe(([values, aggregation]) => {
        this.loading = false;

        if (values === undefined || aggregation === undefined) {
          return;
        }

        this.values = values;

        let unit =
          aggregation === 'raw'
            ? 'minute'
            : aggregation === 'daily'
              ? 'day'
              : 'month';

        this.chartConfiguration = {
          type: 'line',
          data: {
            datasets: [
              {
                label: 'Electricity',
                data: values.map((x) => ({
                  x: x.timestamp
                    ? new Date(Date.parse(x.timestamp))
                    : undefined,
                  y: x.energyConsumptionKwh,
                })),
              },
            ],
          },
          options: {
            responsive: true,
            maintainAspectRatio: false,
            scales: {
              x: {
                type: 'time',
                time: {
                  unit,
                  displayFormats: {
                    minute: 'dd MMM hh:mm',
                  },
                  tooltipFormat: 'hh:mm:ss dd MMM yyyy',
                },
                title: {
                  display: true,
                  text: 'Date',
                },
              },
              y: {
                title: {
                  display: true,
                  text: 'Energy Consumption (kWh)',
                },
              },
            },
          },
        };
      });
  }

  public ngOnInit(): void {}

  public ngOnDestroy(): void {
    if (this.startDateControl.value && this.endDateControl.value) {
      this.formControlService.setDateRange(
        this.startDateControl.value,
        this.endDateControl.value,
      );
    }
  }

  public showLastSevenDays(): void {
    const today = this.dateService.startOfToday();

    this.setDateRange(this.dateService.addDays(today, -6), today);
  }

  public showThisMonth(): void {
    const startOfThisMonth = this.dateService.startOfMonth(
      this.dateService.startOfToday(),
    );
    const endDate = this.dateService.startOfToday();

    this.setDateRange(startOfThisMonth, endDate);
  }

  public showPreviousMonth(): void {
    const startOfLastMonth = this.dateService.addMonths(
      this.dateService.startOfMonth(this.dateService.startOfToday()),
      -1,
    );
    const endDate = this.dateService.endOfMonth(startOfLastMonth);

    this.setDateRange(startOfLastMonth, endDate);
  }

  private setDateRange(startDate: Date, endDate: Date): void {
    this.startDateControl.setValue(startDate);
    this.endDateControl.setValue(endDate);
  }
}
