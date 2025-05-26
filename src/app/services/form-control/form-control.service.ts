import { Injectable } from '@angular/core';

import { BehaviorSubject, Observable } from 'rxjs';

import { Aggregation } from '../../common/settings';
import { DateService } from '../date/date.service';

@Injectable({
  providedIn: 'root',
})
export class FormControlService {
  private readonly dateRange: BehaviorSubject<[Date, Date]>;
  private readonly aggregationLevel: BehaviorSubject<Aggregation>;

  public constructor(private readonly dateService: DateService) {
    this.dateRange = new BehaviorSubject<[Date, Date]>([
      this.dateService.addDays(this.dateService.startOfToday(), -7),
      this.dateService.startOfToday(),
    ]);

    this.aggregationLevel = new BehaviorSubject<Aggregation>('raw');
  }

  public setDateRange(startDate: Date, endDate: Date): void {
    this.dateRange.next([startDate, endDate]);
  }

  public getDateRange(): Observable<[Date, Date]> {
    return this.dateRange.asObservable();
  }

  public setAggregationLevel(aggregation: Aggregation): void {
    this.aggregationLevel.next(aggregation);
  }

  public getAggregationLevel(): Observable<Aggregation> {
    return this.aggregationLevel.asObservable();
  }
}
