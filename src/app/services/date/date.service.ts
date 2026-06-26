import {
  DateValues,
  addDays,
  addMonths,
  endOfMonth,
  format,
  formatISO,
  parseISO,
  set,
  startOfMonth,
  startOfToday,
} from 'date-fns';

import { Service } from '@angular/core';

@Service()
export class DateService {
  public formatISODate(date: Date): string {
    return formatISO(date, { format: 'extended', representation: 'date' });
  }

  public format(date: Date, formatStr: string): string {
    return format(date, formatStr);
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

  public set(date: Date, values: DateValues) {
    return set(date, values);
  }

  public parseISO(dateStr: string): Date {
    return parseISO(dateStr);
  }
}
