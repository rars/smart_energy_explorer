import { Injectable } from '@angular/core';
import { addDays, formatISO } from 'date-fns';

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
}
