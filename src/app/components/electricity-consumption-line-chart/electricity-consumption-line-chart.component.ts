import { Component, OnInit } from '@angular/core';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatDatepickerModule } from '@angular/material/datepicker';
import { ChartComponent } from '../chart/chart.component';
import { FormControl, ReactiveFormsModule } from '@angular/forms';
import {
  catchError,
  combineLatest,
  filter,
  forkJoin,
  from,
  map,
  Observable,
  of,
  startWith,
  switchMap,
} from 'rxjs';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { MatProgressBarModule } from '@angular/material/progress-bar';
import { DateService } from '../../services/date/date.service';
import {
  ElectricityConsumptionDto,
  ElectricityService,
} from '../../core/modules/n3rgyapi';
import { MatSelectModule } from '@angular/material/select';
import { addDays, startOfToday } from 'date-fns';

type Aggregation = 'raw' | 'daily' | 'monthly';

const nonNullOrUndefined = <T>(x: T | null | undefined): x is T => !!x;

const getValueStream = <T>(x: FormControl<T | null>) =>
  x.valueChanges.pipe(startWith(x.value), filter(nonNullOrUndefined));

// When using the Tauri API npm package:
import { invoke } from '@tauri-apps/api/core';

@Component({
  selector: 'app-electricity-consumption-line-chart',
  standalone: true,
  imports: [
    ReactiveFormsModule,
    MatFormFieldModule,
    MatDatepickerModule,
    MatProgressBarModule,
    MatSelectModule,
    ChartComponent,
  ],
  templateUrl: './electricity-consumption-line-chart.component.html',
  styleUrl: './electricity-consumption-line-chart.component.scss',
})
export class ElectricityConsumptionLineChartComponent implements OnInit {
  public values?: any[];
  public chartConfiguration: any;
  public startDateControl = new FormControl<Date>(addDays(startOfToday(), -7));
  public endDateControl = new FormControl<Date>(startOfToday());
  public aggregationControl = new FormControl<Aggregation>('raw');
  public loading = false;

  public constructor(
    private readonly dateService: DateService,
    private readonly electricityService: ElectricityService
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

          let data: Observable<ElectricityConsumptionDto[]> = of([]);

          switch (aggregation) {
            case 'daily': {
               /* data = this.electricityService.apiElectricityConsumptionDailyGet(
                startDate,
                endDate
              );*/

              data = from(invoke<{timestamp: string, value: number}[]>('get_daily_electricity_consumption', {startDate, endDate}))
              .pipe(map(x => x.map(({timestamp, value}) => ({timestamp, energyConsumptionKwh: value}))));

              break;
            }
            case 'monthly': {

              data = from(invoke<{timestamp: string, value: number}[]>('get_monthly_electricity_consumption', {startDate, endDate}))
              .pipe(map(x => x.map(({timestamp, value}) => ({timestamp, energyConsumptionKwh: value}))));

              /*
              data =
                this.electricityService.apiElectricityConsumptionMonthlyGet(
                  startDate,
                  endDate
                );*/
              break;
            }
            case 'raw':
            default: {
              data = from(invoke<{timestamp: string, value: number}[]>('get_raw_electricity_consumption', {startDate, endDate}))
              .pipe(map(x => x.map(({timestamp, value}) => ({timestamp, energyConsumptionKwh: value}))));

              /* data = this.electricityService.apiElectricityConsumptionRawGet(
                startDate,
                endDate
              );*/
            }
          }

          return forkJoin([data, of(aggregation)]);
        }),
        takeUntilDestroyed(),
        catchError(err => {
          window.alert(`Error: ${err}`);
          return of([undefined, undefined]);
        })
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
                label: 'Energy Consumption (kWh)',
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
                  text: 'value',
                },
              },
            },
          },
        };
      });
  }

  public ngOnInit(): void {}
}
