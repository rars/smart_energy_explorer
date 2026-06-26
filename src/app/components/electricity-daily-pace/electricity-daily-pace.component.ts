import { Chart, ChartConfiguration } from 'chart.js';
import { addDays, format, set, startOfToday } from 'date-fns';

import {
  Component,
  OnDestroy,
  Signal,
  afterRenderEffect,
  computed,
  inject,
  resource,
  signal,
} from '@angular/core';
import { FormField, form } from '@angular/forms/signals';
import { MatDatepickerModule } from '@angular/material/datepicker';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';

import { UnlistenFn, listen } from '@tauri-apps/api/event';

import {
  ElectricityConsumption,
  ElectricityConsumptionService,
} from '../../services/electricity-consumption/electricity-consumption.service';
import { ChartComponent } from '../chart/chart.component';

interface InputParams {
  date: Date;
}

interface ChartPoint {
  x: Date;
  y: number;
}

type MyChart = ChartConfiguration<'line', ChartPoint[]>;

@Component({
  selector: 'app-electricity-daily-pace-component',
  imports: [
    ChartComponent,
    MatInputModule,
    MatFormFieldModule,
    MatDatepickerModule,
    FormField,
  ],
  templateUrl: './electricity-daily-pace.component.html',
  styleUrl: './electricity-daily-pace.component.scss',
})
export class ElectricityDailyPaceComponent implements OnDestroy {
  private readonly dateToNormalizeTo = startOfToday();

  private readonly electricityConsumptionService = inject(
    ElectricityConsumptionService,
  );

  private readonly inputParams = signal<InputParams>({
    date: addDays(startOfToday(), -1),
  });

  protected readonly chartInstance = signal<Chart | undefined>(undefined);

  private electricityUpdateUnlistenFn?: UnlistenFn;

  private readonly totalKwhUsedToday = signal<
    { timestamp: Date; electricityConsumptionKwh: number } | undefined
  >(undefined);

  protected readonly inputParamsForm = form(this.inputParams);

  public readonly chartConfiguration: Signal<MyChart | undefined>;

  public constructor() {
    const chartConfigurationResource = resource({
      params: () => ({ date: this.inputParamsForm.date().value() }),
      loader: ({ params }) => this.getChartConfiguration(params.date),
    });

    this.chartConfiguration = computed(() => {
      if (!chartConfigurationResource.hasValue()) {
        return undefined;
      }
      return chartConfigurationResource.value();
    });

    afterRenderEffect({
      write: () => {
        const chartConfig = this.chartConfiguration();
        const liveData = this.totalKwhUsedToday();
        const chartInstance = this.chartInstance();

        if (chartConfig && liveData && chartInstance) {
          const plugins = chartInstance.options?.plugins || {};

          plugins.annotation = {
            annotations: this.getAnnotationsChartConfig(
              liveData.timestamp,
              liveData.electricityConsumptionKwh,
            ) as any,
          };

          chartInstance.update('none');
        }
      },
    });

    this.listenForRealTimeData();
  }

  public ngOnDestroy(): void {
    this.electricityUpdateUnlistenFn?.();
  }

  private listenForRealTimeData() {
    listen<any>('electricityUpdate', (message) => {
      const lastUpdated = new Date(message.payload.electricitymeter.timestamp);
      const energy = message.payload.electricitymeter.energy;

      if (energy.import.units !== 'kWh') {
        console.warn('units are incorrect: ' + energy.import.units);
      }

      this.totalKwhUsedToday.set({
        timestamp: this.normalizeTimestamp(lastUpdated),
        electricityConsumptionKwh: energy.import.day,
      });
    }).then((unlisten) => {
      this.electricityUpdateUnlistenFn = unlisten;
    });
  }

  private async getChartConfiguration(
    date: Date,
  ): Promise<ChartConfiguration<'line', ChartPoint[]>> {
    const datasets = await Promise.all([
      (async () => {
        const data =
          await this.electricityConsumptionService.getRawElectricityConsumption(
            date,
            date,
          );

        const cumulativeSeries = this.normalizeTimestamps(
          this.createCumulativeSeriesFromRawConsumption(data),
        );

        const label = format(date, 'eee d MMM');
        return { label, data: cumulativeSeries };
      })(),
      this.getDatasetForOtherDay(-1, date),
      this.getDatasetForOtherDay(-7, date),
    ]);

    return this.createChartConfiguration(datasets);
  }

  private async getDatasetForOtherDay(dayDelta: number, referenceDate: Date) {
    const otherDate = addDays(referenceDate, dayDelta);

    const label = format(otherDate, 'eee d MMM');

    const nextData =
      await this.electricityConsumptionService.getRawElectricityConsumption(
        otherDate,
        otherDate,
      );

    return {
      label,
      data: this.normalizeTimestamps(
        this.createCumulativeSeriesFromRawConsumption(nextData),
      ),
    };
  }

  private createCumulativeSeriesFromRawConsumption(
    series: ElectricityConsumption[],
  ) {
    series.sort((a, b) => a.timestamp.getTime() - b.timestamp.getTime());

    let result: ChartPoint[] = [];

    const cumulativeSeries = series.reduce(
      (acc, current) => {
        const newTotal = acc.sum + current.electricityConsumptionKwh;
        acc.result.push({
          x: current.timestamp,
          y: newTotal,
        });
        return { result, sum: newTotal };
      },
      { result, sum: 0 },
    );

    return cumulativeSeries.result;
  }

  private createChartConfiguration(
    datasets: { label: string; data: ChartPoint[] }[],
  ): ChartConfiguration<'line', ChartPoint[]> {
    return {
      type: 'line',
      data: {
        datasets,
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        scales: {
          x: {
            type: 'time',
            time: {
              unit: 'minute',
              displayFormats: {
                minute: 'HH:mm',
              },
              tooltipFormat: 'HH:mm',
            },
            title: {
              display: true,
              text: 'Time',
            },
          },
          y: {
            beginAtZero: true,
            title: {
              display: true,
              text: 'Cumulative Energy Consumption (kWh)',
            },
          },
        },
      },
    };
  }

  private getAnnotationsChartConfig(now: Date, totalConsumptionKwh: number) {
    return {
      todayProgressBox: {
        type: 'box',
        xMin: this.normalizeTimestamp(startOfToday()).getTime(),
        xMax: now.getTime(),

        yMin: 0,
        yMax: totalConsumptionKwh,

        backgroundColor: 'rgba(0, 230, 118, 0.15)',
        borderColor: 'rgba(0, 230, 118, 0.8)',
        borderWidth: 2,
        borderDash: [4, 4],
        label: {
          display: true,
          content: `Today: ${totalConsumptionKwh} kWh`,
          position: {
            x: 'end',
            y: 'end',
          },
          font: {
            size: 11,
            weight: 'bold',
          },
        },
      },
    };
  }

  private normalizeTimestamps(series: ChartPoint[]): ChartPoint[] {
    return series.map((p) => ({
      x: this.normalizeTimestamp(p.x),
      y: p.y,
    }));
  }

  private normalizeTimestamp(date: Date): Date {
    return set(this.dateToNormalizeTo, {
      hours: date.getHours(),
      minutes: date.getMinutes(),
      seconds: date.getSeconds(),
      milliseconds: 0,
    });
  }
}
