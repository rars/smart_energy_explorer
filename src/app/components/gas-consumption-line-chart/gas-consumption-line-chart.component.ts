import { Component, OnInit } from '@angular/core';
import { GasConsumptionDto, GasService } from '../../core/modules/n3rgyapi';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatDatepickerModule } from '@angular/material/datepicker';
import { ChartComponent } from '../chart/chart.component';
import { FormControl, ReactiveFormsModule } from '@angular/forms';
import { MatProgressBarModule } from '@angular/material/progress-bar';
import {
  combineLatest,
  filter,
  forkJoin,
  map,
  Observable,
  of,
  startWith,
  switchMap,
} from 'rxjs';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { DateService } from '../../services/date/date.service';
import { addDays, startOfToday } from 'date-fns';
import { MatSelectModule } from '@angular/material/select';

type Aggregation = 'raw' | 'daily' | 'monthly';

const nonNullOrUndefined = <T>(x: T | null | undefined): x is T => !!x;

const getValueStream = <T>(x: FormControl<T | null>) =>
  x.valueChanges.pipe(startWith(x.value), filter(nonNullOrUndefined));

@Component({
  selector: 'app-gas-consumption-line-chart',
  standalone: true,
  imports: [
    ReactiveFormsModule,
    MatFormFieldModule,
    MatDatepickerModule,
    MatSelectModule,
    MatProgressBarModule,
    ChartComponent,
  ],
  templateUrl: './gas-consumption-line-chart.component.html',
  styleUrl: './gas-consumption-line-chart.component.scss',
})
export class GasConsumptionLineChartComponent implements OnInit {
  public values?: any[];
  public chartConfiguration: any;
  public startDateControl = new FormControl<Date>(addDays(startOfToday(), -7));
  public endDateControl = new FormControl<Date>(startOfToday());
  public aggregationControl = new FormControl<Aggregation>('raw');
  public loading = false;

  public constructor(
    private readonly dateService: DateService,
    private readonly gasService: GasService
  ) {
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

          let data: Observable<GasConsumptionDto[]> = of([]);

          switch (aggregation) {
            case 'monthly': {
              data = this.gasService.apiGasConsumptionMonthlyGet(
                startDate,
                endDate
              );
              break;
            }
            case 'daily': {
              data = this.gasService.apiGasConsumptionDailyGet(
                startDate,
                endDate
              );
              break;
            }
            case 'raw':
            default: {
              data = this.gasService.apiGasConsumptionRawGet(
                startDate,
                endDate
              );
            }
          }

          return forkJoin([data, of(aggregation)]);
        }),
        takeUntilDestroyed()
      )
      .subscribe(([values, aggregation]) => {
        this.loading = false;
        this.values = values;

        const unit =
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
                label: 'Energy Consumption (M3)',
                data: values.map((x) => ({
                  x: x.timestamp
                    ? new Date(Date.parse(x.timestamp))
                    : undefined,
                  y: x.energyConsumptionM3,
                })),
              },
            ],
          },
          options: {
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
                  text: 'Energy Consumption (M3)',
                },
              },
            },
          },
        };
      });
  }

  public ngOnInit() {}
}
