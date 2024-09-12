import { Injectable } from '@angular/core';
import { addDays, formatISO, startOfToday } from 'date-fns';

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

  public startOfToday(): Date {
    return startOfToday();
  }
}
