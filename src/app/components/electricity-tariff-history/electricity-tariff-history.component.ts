import { CommonModule } from '@angular/common';
import { Component, OnInit } from '@angular/core';
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
  selector: 'app-electricity-tariff-history',
  standalone: true,
  imports: [CommonModule, MatTableModule],
  templateUrl: './electricity-tariff-history.component.html',
  styleUrl: './electricity-tariff-history.component.scss',
})
export class ElectricityTariffHistoryComponent implements OnInit {
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
      }>('get_electricity_tariff_history', {}),
    ).subscribe((data) => {
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
