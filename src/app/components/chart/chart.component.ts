import { Component, Input, OnDestroy, SimpleChanges } from '@angular/core';
import Chart, { ChartItem } from 'chart.js/auto';

@Component({
  selector: 'app-chart',
  standalone: true,
  imports: [],
  templateUrl: './chart.component.html',
  styleUrl: './chart.component.scss',
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
        simpleChanges['chartConfiguration'].currentValue
      );
    }
  }

  public ngOnDestroy(): void {
    this.chart?.destroy();
  }
}
