import { Routes } from '@angular/router';
import { ElectricityConsumptionLineChartComponent } from './components/electricity-consumption-line-chart/electricity-consumption-line-chart.component';
import { GasConsumptionLineChartComponent } from './components/gas-consumption-line-chart/gas-consumption-line-chart.component';
import { SettingsComponent } from './components/settings/settings.component';
import { unsavedChangesGuard } from './unsaved-changes.guard';
import { ElectricityTariffHistoryComponent } from './components/electricity-tariff-history/electricity-tariff-history.component';
import { GasTariffHistoryComponent } from './components/gas-tariff-history/gas-tariff-history.component';
import { ElectricityCostHistoryComponent } from './components/electricity-cost-history/electricity-cost-history.component';
import { GasCostHistoryComponent } from './components/gas-cost-history/gas-cost-history.component';

export const routes: Routes = [
  { path: 'electricity', component: ElectricityConsumptionLineChartComponent },
  { path: 'gas', component: GasConsumptionLineChartComponent },
  { path: 'electricity/tariff', component: ElectricityTariffHistoryComponent },
  { path: 'electricity/costs', component: ElectricityCostHistoryComponent },
  { path: 'gas/tariff', component: GasTariffHistoryComponent },
  { path: 'gas/costs', component: GasCostHistoryComponent },
  {
    path: 'settings',
    component: SettingsComponent,
    canDeactivate: [unsavedChangesGuard],
  },
];
