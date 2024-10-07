import { CommonModule } from '@angular/common';
import {
  AfterViewInit,
  Component,
  Input,
  OnDestroy,
  ViewChild,
} from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { FormControl, ReactiveFormsModule } from '@angular/forms';
import { MatButtonModule } from '@angular/material/button';
import { MatDatepickerModule } from '@angular/material/datepicker';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatSort, MatSortModule } from '@angular/material/sort';
import { MatTableDataSource, MatTableModule } from '@angular/material/table';

import {
  ReplaySubject,
  combineLatest,
  filter,
  from,
  map,
  startWith,
  switchMap,
  take,
} from 'rxjs';

import { invoke } from '@tauri-apps/api/core';

import { DateService } from '../../services/date/date.service';
import { FormControlService } from '../../services/form-control/form-control.service';

const nonNullOrUndefined = <T>(x: T | null | undefined): x is T => !!x;

const getValueStream = <T>(x: FormControl<T | null>) =>
  x.valueChanges.pipe(startWith(x.value), filter(nonNullOrUndefined));

@Component({
  selector: 'app-energy-cost-history',
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
  templateUrl: './energy-cost-history.component.html',
  styleUrl: './energy-cost-history.component.scss',
})
export class EnergyCostHistoryComponent implements AfterViewInit, OnDestroy {
  @Input() public title?: string;
  @Input() public set command(value: string) {
    this.commandSubject.next(value);
  }

  public readonly startDateControl: FormControl<Date | null>;
  public readonly endDateControl: FormControl<Date | null>;

  public readonly displayedColumns = ['date', 'costPence'];
  public readonly dataSource: any = new MatTableDataSource([]);

  private readonly commandSubject = new ReplaySubject<string>(1);

  @ViewChild(MatSort) public sort?: MatSort;

  public constructor(
    private readonly formControlService: FormControlService,
    private readonly dateService: DateService,
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
      this.commandSubject.asObservable(),
      getValueStream(this.startDateControl),
      getValueStream(this.endDateControl),
    ])
      .pipe(
        map(([command, startDate, endDate]) => [
          command,
          this.dateService.formatISODate(startDate),
          this.dateService.formatISODate(this.dateService.addDays(endDate, 1)),
        ]),
        switchMap(([command, startDate, endDate]) => {
          return from(
            invoke<{ date: string; costPence: number }[]>(command, {
              startDate,
              endDate,
            }),
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

  public ngOnDestroy(): void {
    if (this.startDateControl.value && this.endDateControl.value) {
      this.formControlService.setDateRange(
        this.startDateControl.value,
        this.endDateControl.value,
      );
    }
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
