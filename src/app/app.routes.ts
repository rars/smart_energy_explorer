import { Routes } from '@angular/router';
import { ElectricityConsumptionLineChartComponent } from './components/electricity-consumption-line-chart/electricity-consumption-line-chart.component';
import { GasConsumptionLineChartComponent } from './components/gas-consumption-line-chart/gas-consumption-line-chart.component';
import { SettingsComponent } from './components/settings/settings.component';
import { unsavedChangesGuard } from './unsaved-changes.guard';

export const routes: Routes = [
  { path: 'electricity', component: ElectricityConsumptionLineChartComponent },
  { path: 'gas', component: GasConsumptionLineChartComponent },
  {
    path: 'settings',
    component: SettingsComponent,
    canDeactivate: [unsavedChangesGuard],
  },
];
