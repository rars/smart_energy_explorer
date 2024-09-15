import { Injectable } from '@angular/core';
import { addDays, startOfToday } from 'date-fns';
import { BehaviorSubject, Observable } from 'rxjs';

@Injectable({
  providedIn: 'root',
})
export class FormControlService {
  private readonly dateRange: BehaviorSubject<[Date, Date]>;

  public constructor() {
    this.dateRange = new BehaviorSubject<[Date, Date]>([
      addDays(startOfToday(), -7),
      startOfToday(),
    ]);
  }

  public setDateRange(startDate: Date, endDate: Date): void {
    this.dateRange.next([startDate, endDate]);
  }

  public getDateRange(): Observable<[Date, Date]> {
    return this.dateRange.asObservable();
  }
}
