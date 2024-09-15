import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';
import { MatTableModule } from '@angular/material/table';
import { invoke } from '@tauri-apps/api/core';
import { from } from 'rxjs';

export interface StandingCharge {
  startDate: Date;
  standingChargePence: number;
}

export interface UnitPrice {
  priceEffectiveTime: Date;
  unitPricePence: number;
}

@Component({
  selector: 'app-gas-tariff-history',
  standalone: true,
  imports: [CommonModule, MatTableModule],
  templateUrl: './gas-tariff-history.component.html',
  styleUrl: './gas-tariff-history.component.scss',
})
export class GasTariffHistoryComponent {
  public readonly displayedStandingChargeColumns = [
    'startDate',
    'standingChargePence',
  ];
  public readonly displayedUnitPriceColumns = [
    'priceEffectiveTime',
    'unitPricePence',
  ];

  public standingChargesDataSource: StandingCharge[] | undefined = undefined;
  public unitPricesDataSource: UnitPrice[] | undefined = undefined;

  public ngOnInit(): void {
    from(
      invoke<{
        standingCharges: { startDate: string; standingChargePence: number }[];
        unitPrices: { priceEffectiveTime: string; unitPricePence: number }[];
      }>('get_gas_tariff_history', {}),
    ).subscribe((data) => {
      console.log(data);
      this.standingChargesDataSource = data?.standingCharges?.map((x) => {
        return {
          startDate: new Date(x.startDate),
          standingChargePence: x.standingChargePence,
        };
      });

      this.unitPricesDataSource = data?.unitPrices?.map((x) => {
        return {
          priceEffectiveTime: new Date(x.priceEffectiveTime),
          unitPricePence: x.unitPricePence,
        };
      });
    });
  }
}
