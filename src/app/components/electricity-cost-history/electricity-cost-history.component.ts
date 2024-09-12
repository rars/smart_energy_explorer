import { CommonModule } from '@angular/common';
import { AfterViewInit, Component, ViewChild } from '@angular/core';
import { FormControl, ReactiveFormsModule } from '@angular/forms';
import { MatDatepickerModule } from '@angular/material/datepicker';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatTableDataSource, MatTableModule } from '@angular/material/table';
import { invoke } from '@tauri-apps/api/core';
import {
  addDays,
  addMonths,
  endOfMonth,
  startOfMonth,
  startOfToday,
} from 'date-fns';

import { combineLatest, filter, from, map, startWith, switchMap } from 'rxjs';
import { DateService } from '../../services/date/date.service';
import { MatSort, MatSortModule } from '@angular/material/sort';
import { MatButtonModule } from '@angular/material/button';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';

const nonNullOrUndefined = <T>(x: T | null | undefined): x is T => !!x;

const getValueStream = <T>(x: FormControl<T | null>) =>
  x.valueChanges.pipe(startWith(x.value), filter(nonNullOrUndefined));

@Component({
  selector: 'app-electricity-cost-history',
  standalone: true,
  imports: [
    CommonModule,
    MatButtonModule,
    MatFormFieldModule,
    MatDatepickerModule,
    ReactiveFormsModule,
    MatTableModule,
    MatSortModule,
  ],
  templateUrl: './electricity-cost-history.component.html',
  styleUrl: './electricity-cost-history.component.scss',
})
export class ElectricityCostHistoryComponent implements AfterViewInit {
  public readonly startDateControl = new FormControl<Date>(
    addDays(startOfToday(), -7),
  );
  public readonly endDateControl = new FormControl<Date>(startOfToday());

  public readonly displayedColumns = ['date', 'costPence'];
  public readonly dataSource: any = new MatTableDataSource([]);

  @ViewChild(MatSort) public sort?: MatSort;

  public constructor(private readonly dateService: DateService) {
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
          return from(
            invoke<{ date: string; costPence: number }[]>(
              'get_electricity_cost_history',
              { startDate, endDate },
            ),
          );
        }),
        map((x) =>
          x.map(({ date, costPence }) => ({
            date: new Date(date),
            costPence: costPence,
          })),
        ),
        takeUntilDestroyed(),
      )
      .subscribe((data) => {
        this.dataSource.data = data;
      });
  }

  public ngAfterViewInit(): void {
    this.dataSource.sort = this.sort;
  }

  public getTotalCost(): number {
    return this.dataSource.data
      .map((x: any) => x.costPence)
      .reduce((acc: number, value: number) => acc + value, 0);
  }

  public showLastSevenDays(): void {
    const today = this.dateService.startOfToday();

    this.setDateRange(this.dateService.addDays(today, -6), today);
  }

  public showThisMonth(): void {
    const startOfThisMonth = startOfMonth(this.dateService.startOfToday());
    const endDate = this.dateService.startOfToday();

    this.setDateRange(startOfThisMonth, endDate);
  }

  public showPreviousMonth(): void {
    const startOfLastMonth = addMonths(
      startOfMonth(this.dateService.startOfToday()),
      -1,
    );
    const endDate = endOfMonth(startOfLastMonth);

    this.setDateRange(startOfLastMonth, endDate);
  }

  private setDateRange(startDate: Date, endDate: Date): void {
    this.startDateControl.setValue(startDate);
    this.endDateControl.setValue(endDate);
  }
}
