import { CommonModule, JsonPipe } from '@angular/common';
import { Component, OnInit } from '@angular/core';
import { MatTableModule } from '@angular/material/table';
import { invoke } from '@tauri-apps/api/core';

import { from, map } from 'rxjs';

@Component({
  selector: 'app-gas-cost-history',
  standalone: true,
  imports: [CommonModule, MatTableModule],
  templateUrl: './gas-cost-history.component.html',
  styleUrl: './gas-cost-history.component.scss',
})
export class GasCostHistoryComponent {
  public displayedColumns = ['date', 'costPence'];
  public data: any | undefined = undefined;

  public ngOnInit(): void {
    const startDate = '2024-09-01';
    const endDate = '2024-09-10';

    from(
      invoke<{ date: string; costPence: number }[]>('get_gas_cost_history', {
        startDate,
        endDate,
      }),
    )
      .pipe(
        map((x) =>
          x.map(({ date, costPence }) => ({
            date: new Date(date),
            costPence: costPence,
          })),
        ),
      )
      .subscribe((data) => {
        this.data = data;
      });
  }
}
