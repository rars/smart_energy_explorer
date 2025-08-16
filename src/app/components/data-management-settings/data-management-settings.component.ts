import { Component } from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { RouterLink } from '@angular/router';

import { invoke } from '@tauri-apps/api/core';
import { confirm } from '@tauri-apps/plugin-dialog';

@Component({
  selector: 'app-data-management-settings',
  imports: [MatButtonModule, MatIconModule, RouterLink],
  templateUrl: './data-management-settings.component.html',
  styleUrl: './data-management-settings.component.scss',
})
export class DataManagementSettingsComponent {
  public async clearAllData(): Promise<void> {
    const confirmation = await confirm(
      'This will delete all local data. Are you sure you want to perform this action?',
      { title: 'Clear local data?', kind: 'warning' },
    );

    if (confirmation) {
      await invoke<void>('clear_all_data', {});
    }
  }

  public async fetchData(): Promise<void> {
    return invoke<void>('fetch_data', {});
  }

  public async reset(): Promise<void> {
    const confirmation = await confirm(
      'This will delete all local data and restore settings to their initial state. Are you sure you want to perform this action?',
      { title: 'Reset?', kind: 'warning' },
    );

    if (confirmation) {
      await invoke<void>('reset', {});
    }
  }
}
