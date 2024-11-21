import Chart from 'chart.js/auto';

import { Component, Input, OnDestroy, SimpleChanges } from '@angular/core';

@Component({
    selector: 'app-chart',
    imports: [],
    templateUrl: './chart.component.html',
    styleUrl: './chart.component.scss'
})
export class ChartComponent implements OnDestroy {
  @Input()
  public chartConfiguration: unknown;

  public chart?: Chart;

  public constructor() {}

  public ngOnChanges(simpleChanges: SimpleChanges): void {
    if (simpleChanges['chartConfiguration']) {
      this.chart?.destroy();
      this.chart = new Chart(
        'canvas',
        simpleChanges['chartConfiguration'].currentValue,
      );
    }
  }

  public ngOnDestroy(): void {
    this.chart?.destroy();
  }
}
