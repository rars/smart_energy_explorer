import { Injectable } from '@angular/core';

import { BehaviorSubject, Observable } from 'rxjs';

import { DateService } from '../date/date.service';

@Injectable({
  providedIn: 'root',
})
export class FormControlService {
  private readonly dateRange: BehaviorSubject<[Date, Date]>;

  public constructor(private readonly dateService: DateService) {
    this.dateRange = new BehaviorSubject<[Date, Date]>([
      this.dateService.addDays(this.dateService.startOfToday(), -7),
      this.dateService.startOfToday(),
    ]);
  }

  public setDateRange(startDate: Date, endDate: Date): void {
    this.dateRange.next([startDate, endDate]);
  }

  public getDateRange(): Observable<[Date, Date]> {
    return this.dateRange.asObservable();
  }
}
