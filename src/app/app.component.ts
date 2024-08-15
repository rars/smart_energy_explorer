import { Component, OnInit } from '@angular/core';
import { RouterOutlet } from '@angular/router';
import { ApiModule } from './core/modules/n3rgyapi';
import { CommonModule } from '@angular/common';
import { NavigationBarComponent } from './components/navigation-bar/navigation-bar.component';
import { MatIconRegistry } from '@angular/material/icon';

import Database from '@tauri-apps/plugin-sql';
// when using `"withGlobalTauri": true`, you may use
// const V = window.__TAURI_PLUGIN_SQL__;

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [RouterOutlet, ApiModule, CommonModule, NavigationBarComponent],
  templateUrl: './app.component.html',
  styleUrl: './app.component.scss',
})
export class AppComponent implements OnInit {
  public data: any;
  public message: string = '';

  public constructor(private readonly matIconRegistry: MatIconRegistry) {
    this.matIconRegistry.setDefaultFontSetClass('material-symbols-outlined');
  }

  public ngOnInit(): void {
    this.message += 'Initialising...';
    this.fetchData();
  }

  async fetchData() {
    try {
      this.message += 'Fetching data...';
      const db = await Database.load('sqlite:./my_n3rgy.db');
      const data = await db.select(
        'SELECT timestamp, energy_consumption_kwh FROM electricity_consumption;'
        //`SELECT * FROM sqlite_master WHERE type='table';`
      );
      this.message += 'Fetched data...';
      this.data = data;
      this.message += 'Assigned data...';
    } catch (e) {
      this.message += `Error: ${e}`;
    }
  }
}
