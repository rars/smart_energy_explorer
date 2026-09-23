import { CommonModule } from '@angular/common';
import {
  ChangeDetectionStrategy,
  Component,
  OnDestroy,
  computed,
  effect,
  inject,
  input,
  resource,
  signal,
  viewChild,
} from '@angular/core';
import { FormField, form } from '@angular/forms/signals';
import { MatButtonModule } from '@angular/material/button';
import { MatDatepickerModule } from '@angular/material/datepicker';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatSort, MatSortModule } from '@angular/material/sort';
import { MatTableDataSource, MatTableModule } from '@angular/material/table';

import { take } from 'rxjs';

import { invoke } from '@tauri-apps/api/core';

import { DateService } from '../../services/date/date.service';
import { ErrorService } from '../../services/error/error.service';
import { FormControlService } from '../../services/form-control/form-control.service';

const nonNullOrUndefined = <T>(x: T | null | undefined): x is T => !!x;

type CostHistoryCommand =
  'get_electricity_cost_history' | 'get_gas_cost_history';

interface CostRow {
  date: Date;
  costPence: number;
}

@Component({
  selector: 'app-energy-cost-history',
  imports: [
    CommonModule,
    FormField,
    MatButtonModule,
    MatFormFieldModule,
    MatDatepickerModule,
    MatTableModule,
    MatSortModule,
  ],
  templateUrl: './energy-cost-history.component.html',
  styleUrl: './energy-cost-history.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class EnergyCostHistoryComponent implements OnDestroy {
  private readonly formControlService = inject(FormControlService);
  private readonly dateService = inject(DateService);
  private readonly errorService = inject(ErrorService);

  public readonly title = input<string>();
  public readonly command = input.required<CostHistoryCommand>();

  private readonly formParams = signal({
    startDate: this.dateService.addDays(this.dateService.startOfToday(), -7),
    endDate: this.dateService.startOfToday(),
  });

  protected readonly form = form(this.formParams);

  public readonly displayedColumns = ['date', 'costPence'];
  public readonly dataSource = new MatTableDataSource<CostRow>([]);

  public sort = viewChild(MatSort);

  protected readonly costData = resource({
    params: () => {
      const command = this.command();
      const { startDate, endDate } = this.formParams();

      if (!(nonNullOrUndefined(startDate) && nonNullOrUndefined(endDate))) {
        return undefined;
      }

      return {
        command,
        startDate: this.dateService.formatISODate(startDate),
        endDate: this.dateService.formatISODate(
          this.dateService.addDays(endDate, 1),
        ),
      };
    },
    loader: async ({ params }) => {
      const { command, startDate, endDate } = params;

      try {
        return (
          await invoke<{ date: string; costPence: number }[]>(command, {
            startDate,
            endDate,
          })
        ).map(({ date, costPence }) => ({
          date: new Date(date),
          costPence: costPence,
        }));
      } catch (err) {
        this.errorService.showError(`Failed to load cost history data: ${err}`);
        return [];
      }
    },
  });

  protected readonly totalCost = computed(() => {
    const costData = this.costData.value();
    if (!costData) {
      return 0;
    }

    return costData
      .map((x) => x.costPence)
      .reduce((acc: number, value: number) => acc + value, 0);
  });

  public constructor() {
    effect(() => {
      const sort = this.sort();
      if (sort) {
        this.dataSource.sort = sort;
      }
    });

    this.formControlService
      .getDateRange()
      .pipe(take(1))
      .subscribe(([startDate, endDate]) => {
        this.formParams.set({ startDate, endDate });
      });

    effect(() => {
      this.dataSource.data = this.costData.value() ?? [];
    });
  }

  public ngOnDestroy(): void {
    const { startDate, endDate } = this.formParams();
    if (startDate && endDate) {
      this.formControlService.setDateRange(startDate, endDate);
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
    this.formParams.set({ startDate, endDate });
  }
}
