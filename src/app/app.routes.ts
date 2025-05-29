import { Routes } from '@angular/router';

import { AboutComponent } from './components/about/about.component';
import { AssistantComponent } from './components/assistant/assistant.component';
import { ElectricityConsumptionLineChartComponent } from './components/electricity-consumption-line-chart/electricity-consumption-line-chart.component';
import { ElectricityTariffHistoryComponent } from './components/electricity-tariff-history/electricity-tariff-history.component';
import { EnergyCostHistoryComponent } from './components/energy-cost-history/energy-cost-history.component';
import { GasConsumptionLineChartComponent } from './components/gas-consumption-line-chart/gas-consumption-line-chart.component';
import { GasTariffHistoryComponent } from './components/gas-tariff-history/gas-tariff-history.component';
import { SettingsComponent } from './components/settings/settings.component';
import { WelcomeComponent } from './components/welcome/welcome.component';
import { unsavedChangesGuard } from './unsaved-changes.guard';

export const routes: Routes = [
  { path: 'assistant', component: AssistantComponent },
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
  { path: '', redirectTo: '/electricity', pathMatch: 'full' },
];
