import { Service, inject } from '@angular/core';

import { invoke } from '@tauri-apps/api/core';

import { DateService } from '../date/date.service';

export interface ElectricityConsumption {
  timestamp: Date;
  electricityConsumptionKwh: number;
}

interface TimestampedValue {
  timestamp: string;
  value: number;
}

@Service()
export class ElectricityConsumptionService {
  private readonly dateService = inject(DateService);

  public async getRawElectricityConsumption(
    startDate: Date,
    endDate: Date,
  ): Promise<ElectricityConsumption[]> {
    const params = this.formatISODateRange(startDate, endDate);

    const data = await invoke<TimestampedValue[]>(
      'get_raw_electricity_consumption',
      {
        startDate: params.startDate,
        endDate: params.endDate,
      },
    );

    return data.map((x) => ({
      timestamp: new Date(this.dateService.parseISO(x.timestamp)),
      electricityConsumptionKwh: x.value,
    }));
  }

  private formatISODateRange(
    startDate: Date,
    endDate: Date,
  ): { startDate: string; endDate: string } {
    return {
      startDate: this.dateService.formatISODate(startDate),
      endDate: this.dateService.formatISODate(
        this.dateService.addDays(endDate, 1),
      ),
    };
  }
}
