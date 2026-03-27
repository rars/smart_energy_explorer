import { CommonModule } from '@angular/common';
import { ChangeDetectionStrategy, Component, OnInit, effect, signal, viewChild } from '@angular/core';
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
  selector: 'app-gas-tariff-history',
  imports: [CommonModule, MatTableModule, MatSortModule],
  templateUrl: './gas-tariff-history.component.html',
  styleUrl: './gas-tariff-history.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class GasTariffHistoryComponent implements OnInit {
  public readonly displayedStandingChargeColumns = [
    'startDate',
    'standingChargePence',
  ];
  public readonly displayedUnitPriceColumns = [
    'priceEffectiveTime',
    'unitPricePence',
  ];

  public standingChargesDataSource = signal<MatTableDataSource<StandingCharge> | undefined>(undefined);
  public unitPricesDataSource = signal<MatTableDataSource<UnitPrice> | undefined>(undefined);

  public standingChargesSort = viewChild<MatSort>('standingChargesSort');
  public unitPricesSort = viewChild<MatSort>('unitPricesSort');

  public constructor() {
    effect(() => {
      const sort = this.standingChargesSort();
      const ds = this.standingChargesDataSource();
      if (ds && sort) {
        ds.sort = sort;
      }
    });

    effect(() => {
      const sort = this.unitPricesSort();
      const ds = this.unitPricesDataSource();
      if (ds && sort) {
        ds.sort = sort;
      }
    });
  }

  public ngOnInit(): void {
    from(
      invoke<{
        standingCharges: { startDate: string; standingChargePence: number }[];
        unitPrices: { priceEffectiveTime: string; unitPricePence: number }[];
      }>('get_gas_tariff_history', {}),
    ).subscribe((data) => {
      this.standingChargesDataSource.set(
        new MatTableDataSource<StandingCharge>(
          data?.standingCharges?.map((x) => {
            return {
              startDate: new Date(x.startDate),
              standingChargePence: x.standingChargePence,
            };
          }) ?? [],
        )
      );

      this.unitPricesDataSource.set(
        new MatTableDataSource<UnitPrice>(
          data?.unitPrices?.map((x) => {
            return {
              priceEffectiveTime: new Date(x.priceEffectiveTime),
              unitPricePence: x.unitPricePence,
            };
          }) ?? [],
        )
      );
    });
  }
}
