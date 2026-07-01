import {
  ChangeDetectionStrategy,
  Component,
  OnDestroy,
  OnInit,
  inject,
  signal,
} from '@angular/core';
import { takeUntilDestroyed, toObservable } from '@angular/core/rxjs-interop';
import { FormField, form } from '@angular/forms/signals';
import { MatButtonModule } from '@angular/material/button';
import { MatDatepickerModule } from '@angular/material/datepicker';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatIconModule } from '@angular/material/icon';
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
  switchMap,
  take,
} from 'rxjs';

// When using the Tauri API npm package:
import { invoke } from '@tauri-apps/api/core';

import { Aggregation } from '../../common/settings';
import { CsvExportService } from '../../services/csv-export/csv-export.service';
import { DateService } from '../../services/date/date.service';
import { FormControlService } from '../../services/form-control/form-control.service';
import { ChartComponent } from '../chart/chart.component';

const nonNullOrUndefined = <T>(x: T | null | undefined): x is T => !!x;

interface InputParams {
  startDate: Date;
  endDate: Date;
  aggregation: Aggregation;
}

@Component({
  selector: 'app-electricity-consumption-chart',
  imports: [
    FormField,
    MatButtonModule,
    MatFormFieldModule,
    MatDatepickerModule,
    MatProgressBarModule,
    MatSelectModule,
    MatIconModule,
    ChartComponent,
  ],
  templateUrl: './electricity-consumption-chart.component.html',
  styleUrl: './electricity-consumption-chart.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ElectricityConsumptionChartComponent implements OnInit, OnDestroy {
  private readonly dateService = inject(DateService);

  protected readonly inputParams = signal<InputParams>({
    startDate: this.dateService.addDays(this.dateService.startOfToday(), -7),
    endDate: this.dateService.startOfToday(),
    aggregation: 'raw',
  });

  protected readonly inputParamsForm = form(this.inputParams);

  public values = signal<any[] | undefined>(undefined);
  public chartConfiguration = signal<any>(undefined);
  public loading = signal(false);

  public constructor(
    private readonly formControlService: FormControlService,
    private readonly csvExportService: CsvExportService,
  ) {
    combineLatest([
      this.formControlService.getDateRange(),
      this.formControlService.getAggregationLevel(),
    ])
      .pipe(take(1))
      .subscribe(([[startDate, endDate], aggregation]) => {
        this.inputParams.set({
          startDate,
          endDate,
          aggregation,
        });
      });

    toObservable(this.inputParams)
      .pipe(
        filter(
          ({ startDate, endDate, aggregation }) =>
            nonNullOrUndefined(startDate) &&
            nonNullOrUndefined(endDate) &&
            nonNullOrUndefined(aggregation),
        ),
        map(({ startDate, endDate, aggregation }) => [
          this.dateService.formatISODate(startDate),
          this.dateService.formatISODate(this.dateService.addDays(endDate, 1)),
          aggregation,
        ]),
        switchMap(([startDate, endDate, aggregation]) => {
          this.loading.set(true);

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
                    energyConsumptionKwh: value / 1000.0,
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
                    energyConsumptionKwh: value / 1000.0,
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
                    energyConsumptionKwh: value / 1000.0,
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
        this.loading.set(false);

        if (values === undefined || aggregation === undefined) {
          return;
        }

        this.values.set(values);

        let unit =
          aggregation === 'raw'
            ? 'minute'
            : aggregation === 'daily'
              ? 'day'
              : 'month';

        this.chartConfiguration.set({
          type: 'bar',
          data: {
            datasets: [
              {
                label: 'Electricity',
                data: values.map((x) => ({
                  x: x.timestamp
                    ? new Date(this.dateService.parseISO(x.timestamp))
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
                    minute: 'dd MMM HH:mm',
                  },
                  tooltipFormat: 'HH:mm:ss dd MMM yyyy',
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
        });
      });
  }

  public ngOnInit(): void {}

  public ngOnDestroy(): void {
    const { startDate, endDate, aggregation } = this.inputParams();
    if (startDate && endDate) {
      this.formControlService.setDateRange(startDate, endDate);
    }

    if (aggregation) {
      this.formControlService.setAggregationLevel(aggregation);
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

  public exportAsCsv(): void {
    const values = this.values();
    if (values) {
      this.csvExportService.exportToCSV(values, 'data.csv');
    }
  }

  private setDateRange(startDate: Date, endDate: Date): void {
    this.inputParams.update((currentValue) => ({
      ...currentValue,
      startDate,
      endDate,
    }));
  }
}
