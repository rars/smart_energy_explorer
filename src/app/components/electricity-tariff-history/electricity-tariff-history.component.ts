import { CommonModule } from '@angular/common';
import { Component, OnInit, ViewChild } from '@angular/core';
import { MatSort, MatSortModule } from '@angular/material/sort';
import { MatTableDataSource, MatTableModule } from '@angular/material/table';

import { from } from 'rxjs';

import { invoke } from '@tauri-apps/api/core';

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
  imports: [CommonModule, MatTableModule, MatSortModule],
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

  public standingChargesDataSource?: MatTableDataSource<StandingCharge> =
    undefined;
  public unitPricesDataSource?: MatTableDataSource<UnitPrice> = undefined;

  @ViewChild('standingChargesSort') set standingChargesSort(sort: MatSort) {
    if (this.standingChargesDataSource) {
      this.standingChargesDataSource.sort = sort;
    }
  }
  @ViewChild('unitPricesSort') set unitPricesSort(sort: MatSort) {
    if (this.unitPricesDataSource) {
      this.unitPricesDataSource.sort = sort;
    }
  }

  public ngOnInit(): void {
    from(
      invoke<{
        standingCharges: { startDate: string; standingChargePence: number }[];
        unitPrices: { priceEffectiveTime: string; unitPricePence: number }[];
      }>('get_electricity_tariff_history', {}),
    ).subscribe((data) => {
      this.standingChargesDataSource = new MatTableDataSource<StandingCharge>(
        data?.standingCharges?.map((x) => {
          return {
            startDate: new Date(x.startDate),
            standingChargePence: x.standingChargePence,
          };
        }) ?? [],
      );

      this.unitPricesDataSource = new MatTableDataSource<UnitPrice>(
        data?.unitPrices?.map((x) => {
          return {
            priceEffectiveTime: new Date(x.priceEffectiveTime),
            unitPricePence: x.unitPricePence,
          };
        }) ?? [],
      );
    });
  }
}
