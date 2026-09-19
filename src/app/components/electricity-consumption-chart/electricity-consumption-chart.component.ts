import {
  ChangeDetectionStrategy,
  Component,
  OnDestroy,
  computed,
  inject,
  resource,
  signal,
} from '@angular/core';
import { FormField, form } from '@angular/forms/signals';
import { MatButtonModule } from '@angular/material/button';
import { MatDatepickerModule } from '@angular/material/datepicker';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatIconModule } from '@angular/material/icon';
import { MatProgressBarModule } from '@angular/material/progress-bar';
import { MatSelectModule } from '@angular/material/select';

import { combineLatest, take } from 'rxjs';

import { invoke } from '@tauri-apps/api/core';

import { Aggregation } from '../../common/settings';
import { CsvExportService } from '../../services/csv-export/csv-export.service';
import { DateService } from '../../services/date/date.service';
import { ErrorService } from '../../services/error/error.service';
import { FormControlService } from '../../services/form-control/form-control.service';
import { ChartComponent } from '../chart/chart.component';

const nonNullOrUndefined = <T>(x: T | null | undefined): x is T => !!x;

interface InputParams {
  startDate: Date;
  endDate: Date;
  aggregation: Aggregation;
}

const getFunctionForAggregation = (aggregation: Aggregation): string => {
  switch (aggregation) {
    case 'daily':
      return 'get_daily_electricity_consumption';
    case 'monthly':
      return 'get_monthly_electricity_consumption';
    case 'raw':
    default:
      return 'get_raw_electricity_consumption';
  }
};

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
export class ElectricityConsumptionChartComponent implements OnDestroy {
  private readonly dateService = inject(DateService);
  private readonly formControlService = inject(FormControlService);
  private readonly csvExportService = inject(CsvExportService);
  private readonly errorService = inject(ErrorService);

  protected readonly inputParams = signal<InputParams>({
    startDate: this.dateService.addDays(this.dateService.startOfToday(), -7),
    endDate: this.dateService.startOfToday(),
    aggregation: 'raw',
  });

  protected readonly inputParamsForm = form(this.inputParams);

  protected readonly consumptionData = resource({
    params: () => {
      const { startDate, endDate, aggregation } = this.inputParams();

      if (!(
        nonNullOrUndefined(startDate) &&
        nonNullOrUndefined(endDate) &&
        nonNullOrUndefined(aggregation)
      )) {
        return undefined;
      }

      return {
        startDate: this.dateService.formatISODate(startDate),
        endDate: this.dateService.formatISODate(
          this.dateService.addDays(endDate, 1),
        ),
        aggregation,
      };
    },
    loader: async ({ params }) => {
      const { startDate, endDate, aggregation } = params;

      try {
        const result = await invoke<{ timestamp: string; value: number }[]>(
          getFunctionForAggregation(aggregation),
          { startDate, endDate },
        );

        return {
          values: result.map(({ timestamp, value }) => ({
            timestamp,
            energyConsumptionKwh: value / 1000.0,
          })),
          aggregation,
        };
      } catch (error) {
        this.errorService.showError(
          `Failed to load electricity consumption data: ${error}`,
        );
        return {
          values: [],
          aggregation,
        };
      }
    },
  });

  public chartConfiguration = computed(() => {
    const consumptionDataValue = this.consumptionData.value();

    if (consumptionDataValue === undefined) {
      return;
    }

    const { values, aggregation } = consumptionDataValue;

    if (values === undefined || aggregation === undefined) {
      return;
    }

    let unit =
      aggregation === 'raw'
        ? 'minute'
        : aggregation === 'daily'
          ? 'day'
          : 'month';

    return {
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
    };
  });

  public constructor() {
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
  }

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
    const values = this.consumptionData.value()?.values;
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
