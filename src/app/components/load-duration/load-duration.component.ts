import { CurrencyPipe, DecimalPipe } from '@angular/common';
import { Component, computed, inject, resource, signal } from '@angular/core';
import { FormField, form } from '@angular/forms/signals';
import { MatButtonModule } from '@angular/material/button';
import { MatCardModule } from '@angular/material/card';
import { MatDatepickerModule } from '@angular/material/datepicker';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatIconModule } from '@angular/material/icon';
import { MatProgressBarModule } from '@angular/material/progress-bar';
import { MatSelectModule } from '@angular/material/select';
import { MatSliderModule } from '@angular/material/slider';

import { invoke } from '@tauri-apps/api/core';

import { DateService } from '../../services/date/date.service';
import { ErrorService } from '../../services/error/error.service';
import { ChartComponent } from '../chart/chart.component';

const nonNullOrUndefined = <T>(x: T | null | undefined): x is T => !!x;

interface DateRange {
  startDate: Date | null;
  endDate: Date | null;
}

@Component({
  selector: 'app-load-duration',
  imports: [
    CurrencyPipe,
    DecimalPipe,
    FormField,
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
  private readonly dateService = inject(DateService);
  private readonly errorService = inject(ErrorService);

  private readonly dateRangeModel = signal<DateRange>({
    startDate: this.dateService.addDays(this.dateService.startOfToday(), -7),
    endDate: this.dateService.startOfToday(),
  });

  protected readonly dateRangeForm = form(this.dateRangeModel);

  protected readonly electricityConsumptionData = resource({
    params: () => {
      const { startDate, endDate } = this.dateRangeModel();

      if (!(nonNullOrUndefined(startDate) && nonNullOrUndefined(endDate))) {
        return undefined;
      }

      return {
        startDate: this.dateService.formatISODate(startDate),
        endDate: this.dateService.formatISODate(
          this.dateService.addDays(endDate, 1),
        ),
      };
    },
    loader: async ({ params }) => {
      const { startDate, endDate } = params;

      try {
        const values = (
          await invoke<{ timestamp: string; value: number }[]>(
            'get_raw_electricity_consumption',
            { startDate, endDate },
          )
        ).map(({ timestamp, value }) => ({
          timestamp,
          energyConsumptionKwh: value / 1000.0,
        }));

        values.sort((a, b) => b.energyConsumptionKwh - a.energyConsumptionKwh);

        const N = values.length;
        return values.map((v, i) => ({
          percentile: ((i + 1.0) / N) * 100.0,
          energyConsumptionKwh: v.energyConsumptionKwh,
          powerConsumptionW: v.energyConsumptionKwh * 2 * 1000,
        }));
      } catch (error) {
        this.errorService.showError(
          `Failed to load electricity consumption data: ${error}`,
        );
        return [];
      }
    },
  });

  protected readonly baseLoadEstimate = computed(() => {
    const newValues = this.electricityConsumptionData.value();

    if (newValues === undefined) {
      return undefined;
    }

    return newValues.find((v) => v.percentile > 95)?.powerConsumptionW;
  });

  protected readonly chartConfiguration = computed(() => {
    const newValues = this.electricityConsumptionData.value();

    if (newValues === undefined) {
      return undefined;
    }

    return {
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
    };
  });

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
}
