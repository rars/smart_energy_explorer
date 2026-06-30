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
  combineLatest,
  filter,
  forkJoin,
  from,
  map,
  of,
  switchMap,
  take,
} from 'rxjs';

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
  selector: 'app-gas-consumption-chart',
  imports: [
    ChartComponent,
    FormField,
    MatButtonModule,
    MatDatepickerModule,
    MatFormFieldModule,
    MatIconModule,
    MatProgressBarModule,
    MatSelectModule,
  ],
  templateUrl: './gas-consumption-chart.component.html',
  styleUrl: './gas-consumption-chart.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class GasConsumptionChartComponent implements OnInit, OnDestroy {
  private readonly dateService = inject(DateService);

  protected inputParams = signal<InputParams>({
    startDate: this.dateService.addDays(this.dateService.startOfToday(), -7),
    endDate: this.dateService.startOfToday(),
    aggregation: 'raw',
  });
  protected inputParamsForm = form(this.inputParams);

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
        this.inputParams.set({ startDate, endDate, aggregation });
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
            case 'monthly': {
              data = from(
                invoke<{ timestamp: string; value: number }[]>(
                  'get_monthly_gas_consumption',
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
            case 'daily': {
              data = from(
                invoke<{ timestamp: string; value: number }[]>(
                  'get_daily_gas_consumption',
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
                  'get_raw_gas_consumption',
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
      )
      .subscribe(([values, aggregation]) => {
        this.loading.set(false);
        this.values.set(values);

        const unit =
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
                label: 'Gas',
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

  public ngOnInit() {}

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
