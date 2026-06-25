import Chart from 'chart.js/auto';

import {
  ChangeDetectionStrategy,
  Component,
  ElementRef,
  HostListener,
  OnDestroy,
  effect,
  input,
  output,
  viewChild,
} from '@angular/core';

@Component({
  selector: 'app-chart',
  imports: [],
  templateUrl: './chart.component.html',
  styleUrl: './chart.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ChartComponent implements OnDestroy {
  public readonly chartConfiguration = input<unknown>();
  public readonly chartInstanceChange = output<Chart>();
  private readonly canvasRef =
    viewChild.required<ElementRef<HTMLCanvasElement>>('canvas');

  private chart?: Chart;

  public constructor() {
    effect(() => {
      const canvasRef = this.canvasRef();
      const config = this.chartConfiguration();

      this.createNewChartInstance(canvasRef, config);
    });
  }

  public ngOnDestroy(): void {
    this.chart?.destroy();
  }

  @HostListener('window:resize')
  public onResize(): void {
    const canvasRef = this.canvasRef();
    if (!canvasRef?.nativeElement?.ownerDocument) {
      return;
    }

    // To avoid a created chart from overflowing, recreate it so it fits the parent
    this.createNewChartInstance(canvasRef, this.chartConfiguration());
  }

  private createNewChartInstance(
    canvasRef: ElementRef<HTMLCanvasElement>,
    config: any,
  ): void {
    if (!canvasRef || !config) {
      return;
    }

    this.chart?.destroy();
    this.chart = new Chart(canvasRef.nativeElement, config);
    this.chartInstanceChange.emit(this.chart);
  }
}
