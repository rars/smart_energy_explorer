import { Injectable } from '@angular/core';
import {
  formatISO,
  addDays,
  addMonths,
  endOfMonth,
  startOfMonth,
  startOfToday,
} from 'date-fns';

@Injectable({
  providedIn: 'root',
})
export class DateService {
  public formatISODate(date: Date): string {
    return formatISO(date, { format: 'extended', representation: 'date' });
  }

  public addDays(date: Date, n: number): Date {
    return addDays(date, n);
  }

  public addMonths(date: Date, n: number): Date {
    return addMonths(date, n);
  }

  public startOfToday(): Date {
    return startOfToday();
  }

  public startOfMonth(date: Date): Date {
    return startOfMonth(date);
  }

  public endOfMonth(date: Date): Date {
    return endOfMonth(date);
  }
}
