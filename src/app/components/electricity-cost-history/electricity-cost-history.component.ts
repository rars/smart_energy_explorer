import { CommonModule, JsonPipe } from '@angular/common';
import { Component, OnInit } from '@angular/core';
import { MatTableModule } from '@angular/material/table';
import { invoke } from '@tauri-apps/api/core';

import { from, map } from 'rxjs';

@Component({
  selector: 'app-electricity-cost-history',
  standalone: true,
  imports: [CommonModule, MatTableModule],
  templateUrl: './electricity-cost-history.component.html',
  styleUrl: './electricity-cost-history.component.scss',
})
export class ElectricityCostHistoryComponent implements OnInit {
  public displayedColumns = ['date', 'costPence'];
  public data: any | undefined = undefined;

  public ngOnInit(): void {
    const startDate = '2024-09-01';
    const endDate = '2024-09-10';

    from(
      invoke<{ date: string; costPence: number }[]>(
        'get_electricity_cost_history',
        { startDate, endDate },
      ),
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
