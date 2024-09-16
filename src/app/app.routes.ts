import { Routes } from '@angular/router';
import { ElectricityConsumptionLineChartComponent } from './components/electricity-consumption-line-chart/electricity-consumption-line-chart.component';
import { GasConsumptionLineChartComponent } from './components/gas-consumption-line-chart/gas-consumption-line-chart.component';
import { SettingsComponent } from './components/settings/settings.component';
import { unsavedChangesGuard } from './unsaved-changes.guard';
import { ElectricityTariffHistoryComponent } from './components/electricity-tariff-history/electricity-tariff-history.component';
import { GasTariffHistoryComponent } from './components/gas-tariff-history/gas-tariff-history.component';
import { EnergyCostHistoryComponent } from './components/energy-cost-history/energy-cost-history.component';
import { WelcomeComponent } from './components/welcome/welcome.component';
import { AboutComponent } from './components/about/about.component';

export const routes: Routes = [
  { path: 'about', component: AboutComponent },
  { path: 'welcome', component: WelcomeComponent },
  { path: 'electricity', component: ElectricityConsumptionLineChartComponent },
  { path: 'gas', component: GasConsumptionLineChartComponent },
  { path: 'electricity/tariff', component: ElectricityTariffHistoryComponent },
  {
    path: 'electricity/costs',
    data: {
      title: 'Electricity Cost History',
      command: 'get_electricity_cost_history',
    },
    component: EnergyCostHistoryComponent,
  },
  { path: 'gas/tariff', component: GasTariffHistoryComponent },
  {
    path: 'gas/costs',
    data: {
      title: 'Gas Cost History',
      command: 'get_gas_cost_history',
    },
    component: EnergyCostHistoryComponent,
  },
  {
    path: 'settings',
    component: SettingsComponent,
    canDeactivate: [unsavedChangesGuard],
  },
];
