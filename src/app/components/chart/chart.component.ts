import Chart from 'chart.js/auto';

import {
  ChangeDetectionStrategy,
  Component,
  HostListener,
  OnDestroy,
  effect,
  input,
} from '@angular/core';

@Component({
  selector: 'app-chart',
  imports: [],
  templateUrl: './chart.component.html',
  styleUrl: './chart.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ChartComponent implements OnDestroy {
  public chartConfiguration = input<unknown>();

  public chart?: Chart;

  public constructor() {
    effect(() => {
      const config = this.chartConfiguration();
      if (config) {
        this.chart?.destroy();
        this.chart = new Chart('canvas', config as any);
      }
    });
  }

  public ngOnDestroy(): void {
    this.chart?.destroy();
  }

  @HostListener('window:resize', ['$event'])
  public onResize(_event: Event): void {
    // To avoid a created chart from overflowing, recreate it so it fits the parent
    this.chart?.destroy();
    this.chart = new Chart('canvas', this.chartConfiguration() as any);
  }
}
